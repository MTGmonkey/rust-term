use iced::Element;
use iced::widget::{column, scrollable, text, text_input};

use nix::pty::{ForkptyResult, forkpty};
use nix::unistd::write;

use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsFd, OwnedFd};
use std::process::Command;

pub fn spawn_pty_with_shell(default_shell: String) -> OwnedFd {
    match unsafe { forkpty(None, None) } {
        Ok(fork_pty_res) => match fork_pty_res {
            ForkptyResult::Parent { master, .. } => {
                set_nonblock(&master);
                master
            }
            ForkptyResult::Child => {
                Command::new(&default_shell).spawn().unwrap();
                std::thread::sleep(std::time::Duration::MAX);
                std::process::exit(0);
            }
        },
        Err(e) => panic!("failed to fork {:?}", e),
    }
}

pub fn read_from_fd(fd: &OwnedFd) -> Option<Vec<u8>> {
    let mut read_buffer = [0; 65536];
    let mut file = File::from(fd.try_clone().unwrap());
    file.flush();
    let file = file.read(&mut read_buffer);
    match file {
        Ok(file) => Some(read_buffer[..file].to_vec()),
        Err(_) => None,
    }
}

fn set_nonblock(fd: &OwnedFd) {
    let flags = nix::fcntl::fcntl(fd, nix::fcntl::FcntlArg::F_GETFL).unwrap();
    let mut flags =
        nix::fcntl::OFlag::from_bits(flags & nix::fcntl::OFlag::O_ACCMODE.bits()).unwrap();
    flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);
    nix::fcntl::fcntl(fd, nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();
}

#[derive(Debug, Clone)]
pub enum Msg {
    HasInput,
    InputChanged(String),
}

pub struct Model {
    screen_buffer: [u8; 65536],
    screen_buffer_index: usize,
    fd: OwnedFd,
    stdin: OwnedFd,
    input: String,
}

impl Model {
    fn new(
        screen_buffer: [u8; 65536],
        screen_buffer_index: usize,
        fd: OwnedFd,
        stdin: OwnedFd,
        input: String,
    ) -> Self {
        Model {
            screen_buffer,
            screen_buffer_index,
            fd,
            stdin,
            input,
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::HasInput => {
                let mut write_buffer = self.input.as_bytes().to_vec();
                write_buffer.push(b'\n');
                write(self.fd.as_fd(), &mut write_buffer);
                self.input = String::new();
                let mut nored = 0;
                while nored <= 2 {
                    let red = read_from_fd(&self.fd);
                    match &red {
                        Some(red) => {
                            nored += 1;
                            self.update_screen_buffer(red);
                        }
                        None => (),
                    };
                }
            }
            Msg::InputChanged(input) => self.input = input,
        }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        scrollable(column![
            text(String::from(
                String::from_utf8(self.screen_buffer.to_vec())
                    .unwrap()
                    .trim_end_matches('\0')
            )),
            text_input("", &self.input)
                .on_input(Msg::InputChanged)
                .on_submit(Msg::HasInput)
        ])
        .into()
    }

    fn update_screen_buffer(&mut self, vec: &Vec<u8>) {
        let offset = self.screen_buffer.iter().position(|&c| c == b'\0').unwrap();
        for (i, chr) in vec.iter().enumerate() {
            self.screen_buffer[i + offset] = chr.clone();
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        let mut me = Self::new(
            [0; 65536],
            0,
            spawn_pty_with_shell("/home/mtgmonkey/.nix-profile/bin/dash".to_string()),
            std::io::stdin().as_fd().try_clone_to_owned().unwrap(),
            String::new(),
        );
        let mut nored = true;
        while nored {
            let red = read_from_fd(&me.fd);
            match red {
                Some(red) => {
                    nored = false;
                    me.update_screen_buffer(&red);
                }
                None => (),
            }
        }
        me
    }
}
