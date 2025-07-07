use iced::widget::text_input::Id;
use iced::widget::{column, row, scrollable, text, text_input};
use iced::{Element, Font, Task};

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
    Tick,
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

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::HasInput => {
                let mut write_buffer = self.input.as_bytes().to_vec();
                write_buffer.push(b'\n');
                write(self.fd.as_fd(), &mut write_buffer);
                self.input = String::new();
            }
            Msg::InputChanged(input) => self.input = input,
            Msg::Tick => match read_from_fd(&self.fd) {
                Some(red) => self.update_screen_buffer(&red),
                None => (),
            },
        };
        iced::widget::text_input::focus::<Msg>(Id::new("text_input"))
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let (left, right) = match String::from_utf8(self.screen_buffer.to_vec())
            .unwrap()
            .trim_end_matches('\0')
            .rsplit_once('\n')
        {
            Some(tup) => (tup.0.to_string(), tup.1.to_string()),
            None => (
                String::new(),
                String::from_utf8(self.screen_buffer.to_vec())
                    .unwrap()
                    .trim_end_matches('\0')
                    .to_string(),
            ),
        };
        scrollable(column![
            text(left),
            row![
                text(right),
                text_input("", &self.input)
                    .on_input(Msg::InputChanged)
                    .on_submit(Msg::HasInput)
                    .padding(0)
                    .id(Id::new("text_input"))
            ]
        ])
        .into()
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }

    pub fn subscription(&self) -> iced::Subscription<Msg> {
        iced::time::every(iced::time::Duration::new(0, 100)).map(|_| Msg::Tick)
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
