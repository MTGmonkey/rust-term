use nix::pty::{ForkptyResult, forkpty};
use nix::unistd::{read, write};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::process::Command;

fn main() {
    let default_shell = "/home/mtgmonkey/.nix-profile/bin/dash".to_string();
    let fd = spawn_pty_with_shell(default_shell);
    set_nonblock(&fd);
    let mut write_buffer = "tty\n".as_bytes().to_vec();
    write(fd.as_fd(), &mut write_buffer);
    loop {
        let red = read_from_fd(fd.try_clone().unwrap());
        match red {
            Some(red) => print!("{}", String::from_utf8(red).unwrap()),
            None => {
                let mut write_buffer = vec![];
                std::io::stdin().read_to_end(&mut write_buffer);
                write(fd.as_fd(), &mut write_buffer);
            }
        };
    }
}

fn spawn_pty_with_shell(default_shell: String) -> OwnedFd {
    match (unsafe { forkpty(None, None) }) {
        Ok(fork_pty_res) => match fork_pty_res {
            ForkptyResult::Parent { master, .. } => master,
            ForkptyResult::Child => {
                Command::new(&default_shell).spawn().unwrap();
                std::thread::sleep(std::time::Duration::MAX);
                std::process::exit(0);
            }
        },
        Err(e) => panic!("failed to fork {:?}", e),
    }
}

fn read_from_fd(fd: OwnedFd) -> Option<Vec<u8>> {
    let mut read_buffer = [0; 65536];
    let mut file = File::from(fd).read(&mut read_buffer);
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
