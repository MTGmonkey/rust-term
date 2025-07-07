use iced::widget::text_input::Id;
use iced::widget::{column, row, scrollable, text, text_input};
use iced::{Element, Font, Task, keyboard};

use nix::pty::{ForkptyResult, forkpty};
use nix::unistd::write;

use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsFd, OwnedFd};
use std::process::Command;

fn spawn_pty_with_shell(default_shell: String) -> OwnedFd {
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

fn read_from_fd(fd: &OwnedFd) -> Option<Vec<u8>> {
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
    Tick,
    KeyPressed(iced::keyboard::Key),
}

pub struct Model {
    screen_buffer: [u8; 65536],
    screen_buffer_index: usize,
    cursor_index: usize,
    fd: OwnedFd,
    stdin: OwnedFd,
    input: String,
}

impl Model {
    fn new(
        screen_buffer: [u8; 65536],
        screen_buffer_index: usize,
        cursor_index: usize,
        fd: OwnedFd,
        stdin: OwnedFd,
        input: String,
    ) -> Self {
        Model {
            screen_buffer,
            screen_buffer_index,
            cursor_index,
            fd,
            stdin,
            input,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Tick => match read_from_fd(&self.fd) {
                Some(red) => self.update_screen_buffer(&red),
                None => (),
            },
            Msg::KeyPressed(key) => match key {
                keyboard::Key::Character(c) => {
                    self.input_char(c.chars().nth(0).unwrap());
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    self.input.push('\n');
                    let mut write_buffer = self.input.as_bytes().to_vec();
                    write(self.fd.as_fd(), &mut write_buffer);
                    self.input = String::new();
                    self.cursor_index = 0;
                }
                keyboard::Key::Named(keyboard::key::Named::Space) => {
                    self.input_char(' ');
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                    if self.cursor_index <= 0 {
                        self.cursor_index = 0;
                    } else {
                        self.cursor_index -= 1;
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                    if self.cursor_index >= self.input.len() - 1 {
                        self.cursor_index = self.input.len() - 1;
                    } else {
                        self.cursor_index += 1;
                    }
                }
                _ => (),
            },
        };
        iced::Task::<Msg>::none()
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let (left, right) =
            match String::from_utf8(self.screen_buffer[..self.screen_buffer_index].to_vec())
                .unwrap()
                .rsplit_once('\n')
            {
                Some(tup) => (tup.0.to_string(), tup.1.to_string()),
                None => (
                    String::new(),
                    String::from_utf8(self.screen_buffer[..self.screen_buffer_index].to_vec())
                        .unwrap()
                        .to_string(),
                ),
            };
        scrollable(column![text(left), row![text(right), text(&self.input),]]).into()
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }

    pub fn subscription(&self) -> iced::Subscription<Msg> {
        let tick = iced::time::every(iced::time::Duration::new(0, 1)).map(|_| Msg::Tick);
        let key = keyboard::on_key_press(|key, _| Some(Msg::KeyPressed(key)));
        iced::Subscription::batch(vec![tick, key])
    }

    fn update_screen_buffer(&mut self, vec: &Vec<u8>) {
        let offset = self.screen_buffer_index;
        for (i, chr) in vec.iter().enumerate() {
            self.screen_buffer[i + offset] = chr.clone();
            self.screen_buffer_index += 1;
        }
    }

    fn input_char(&mut self, c: char) {
        if self.cursor_index == self.input.len() {
            self.input.push_str(c.to_string().as_str());
        } else {
            self.input.insert(self.cursor_index, c);
        }
        self.cursor_index += 1;
    }
}

impl Default for Model {
    fn default() -> Self {
        let mut me = Self::new(
            [0; 65536],
            0,
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
