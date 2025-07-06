use iced::Element;
use iced::widget::{button, text};

use nix::unistd::write;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::{AsFd, OwnedFd};

use rust_term::*;

fn main() -> iced::Result {
    iced::run("test", Model::update, Model::view)
    /*    let default_shell = "/home/mtgmonkey/.nix-profile/bin/dash".to_string();
        let fd = spawn_pty_with_shell(default_shell);
        let mut write_buffer = "tty\n".as_bytes().to_vec();
        write(fd.as_fd(), &mut write_buffer);
        loop {
            let red = read_from_fd(&fd);
            match red {
                Some(red) => print!("{}", String::from_utf8(red).unwrap()),
                None => {
                    let mut read_buffer = [0; 65536];
                    let mut file = File::from(std::io::stdin().as_fd().try_clone_to_owned().unwrap());
                    file.flush();
                    let file = file.read(&mut read_buffer);
                    println!(
                        "{}",
                        match file {
                            Ok(file) => write(fd.as_fd(), &read_buffer[..file]).unwrap(),
                            Err(_) => 0,
                        }
                    );
                }
            };
        }
    */
}
