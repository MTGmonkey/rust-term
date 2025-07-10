//! this crate runs a terminal

#![expect(
    clippy::needless_return,
    clippy::shadow_reuse,
    clippy::blanket_clippy_restriction_lints,
    clippy::must_use_candidate,
    clippy::missing_trait_methods,
    clippy::pattern_type_mismatch,
    clippy::std_instead_of_alloc,
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::semicolon_outside_block,
    static_mut_refs,
    unused_doc_comments,
    reason = ""
)]

use crate::enums::*;
use crate::parsers::*;

use bpaf::Bpaf;

use iced::widget::{column, rich_text, row, scrollable, span, text};
use iced::{Element, Task, keyboard, time, window};

use nix::errno::Errno;
use nix::fcntl;
use nix::pty::{ForkptyResult, forkpty};
use nix::unistd::write;

use std::fs::File;
use std::io::{self, Read as _};
use std::os::unix::io::{AsFd as _, OwnedFd};
use std::process::Command;
use std::{error, fmt, thread, time as core_time};

pub mod enums;
pub mod parsers;

/// whether to enable verbose logging; see `Flags::verbose`
static mut VERBOSE: bool = false;

/// whether to enable debug logging; see `Flags::debug`
static mut DEBUG: bool = false;

/// whether to enable vomit logging; see `Flags::vomit`
static mut VOMIT: bool = false;

/// shell path; see `Flags::shell`
static mut SHELL: Option<String> = None;

/// events to be passed to `Model::update`
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Msg {
    Exit,
    KeyPressed(keyboard::Key),
    Tick,
}

/// errors for this program
#[non_exhaustive]
#[derive(Debug)]
enum Error {
    /// out of bounds err while accessing a slice
    IndexOutOfBounds,
    /// io error
    Io(io::Error),
    /// nix crate error
    Nix(NixError),
    /// try to access a `File::from::<OwnedFd>()` without an `OwnedFd`
    NoFileDescriptor,
    /// impossible error
    Unreachable,
}

impl fmt::Display for Error {
    #[expect(clippy::min_ident_chars, reason = "it's in the docs like that")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nix(nix_error) => return write!(f, "{nix_error}"),
            Self::Io(io_error) => return write!(f, "{io_error}"),
            Self::NoFileDescriptor => return write!(f, "no file descriptor specified"),
            Self::IndexOutOfBounds => return write!(f, "index out of bounds"),
            Self::Unreachable => return write!(f, "unreachable error, panic"),
        }
    }
}

impl error::Error for Error {}

/// error wrapper for the `nix` crate
#[non_exhaustive]
#[derive(Debug)]
enum NixError {
    /// an OS error
    Errno(Errno),
    /// the error when `OFlags::from_bits(..)` returns `None`
    UnrecognisedFlag,
}

impl fmt::Display for NixError {
    #[expect(clippy::min_ident_chars, reason = "it's in the docs like that")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnrecognisedFlag => return write!(f, "unrecognised flag"),
            Self::Errno(errno) => return write!(f, "bad fcntl argument. errno: {errno}"),
        }
    }
}

/// cli flags
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Flags {
    /// path to shell
    #[bpaf(short('S'), long)]
    shell: Option<String>,

    /// no logging, NOOP; log level 0
    #[bpaf(short, long)]
    quiet: bool,

    /// whether to error log; log level 1
    #[bpaf(short('v'), long)]
    verbose: bool,

    /// whether to debug log; log level 2
    #[bpaf(long)]
    debug: bool,

    /// whether to vomit log; log level 3
    #[bpaf(long)]
    vomit: bool,

    /// whether to display version, NOOP; TODO
    #[expect(dead_code, reason = "TODO")]
    #[bpaf(short('V'), long)]
    version: bool,
}

/// represents the terminal emulator\
/// example usage:
/// ```rust
/// iced::application("window title", Model::update, Model::view)
///     .theme(Model::theme)
///     .default_font(iced::Font::MONOSPACE)
///     .decorations(false)
///     .subscription(Model::subscription)
///     .run()
/// ```
pub struct Model<'a> {
    /// location of cursor in user input line
    cursor_index: usize,
    /// fd of pty
    fd: Option<OwnedFd>,
    /// user input line
    input: String,
    /// all chars on screen
    screen_buffer: [u8; 0x4000],
    /// length of `screen_buffer`'s filled area
    screen_buffer_index: usize,
    /// path to shell
    shell: String,

    screen: Vec<&'a str>,
    cursor: (usize, usize),
}

impl Model<'_> {
    /// applies needed side effects when taking an input char
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "cursor_index is bound checked"
    )]
    fn input_char(&mut self, chr: char) {
        if self.cursor_index == self.input.len() {
            self.input.push_str(chr.to_string().as_str());
        } else {
            self.input.insert(self.cursor_index, chr);
        }
        self.cursor_index += 1;
    }

    /// subscription logic for model
    #[inline]
    pub fn subscription(&self) -> iced::Subscription<Msg> {
        let tick = time::every(time::Duration::new(0, 1)).map(|_| {
            return Msg::Tick;
        });
        let key = keyboard::on_key_press(|key, _| {
            return Some(Msg::KeyPressed(key));
        });
        return iced::Subscription::batch(vec![tick, key]);
    }

    /// theme logic for model
    #[inline]
    pub const fn theme(&self) -> iced::Theme {
        return iced::Theme::GruvboxDark;
    }
    /// update logic for model
    /// TODO fix pattern type mismatch
    /// TODO add more keys
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::wildcard_enum_match_arm,
        reason = "bounds checked"
    )]
    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Exit => return window::get_latest().and_then(window::close),
            Msg::KeyPressed(key) => {
                match key {
                    keyboard::Key::Character(chr) => match chr.chars().nth(0) {
                        Some(chr) => self.input_char(chr),
                        None => return window::get_latest().and_then(window::close),
                    },
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        self.input.push('\n');
                        let write_buffer = self.input.as_bytes().to_vec();
                        if let Some(fd) = &self.fd {
                            match write(fd.as_fd(), &write_buffer) {
                                Ok(_) => (),
                                Err(error) => print_err(&Error::Nix(NixError::Errno(error))),
                            }
                        }
                        self.input = String::new();
                        self.cursor_index = 0;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Space) => {
                        self.input_char(' ');
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if self.cursor_index == 0 {
                            self.cursor_index = 0;
                        } else {
                            self.cursor_index -= 1;
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if self.cursor_index >= self.input.len() {
                            self.cursor_index = self.input.len();
                        } else {
                            self.cursor_index += 1;
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        self.cursor_index = 0;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        self.cursor_index = self.input.len();
                    }
                    _ => (),
                }
                return iced::Task::none();
            }
            Msg::Tick => {
                let red = read_from_option_fd(self.fd.as_ref());
                match red {
                    Ok(red) => {
                        if let Err(error) = self.update_screen_buffer(&red) {
                            print_err(&error);
                        }
                    }
                    Err(error) => print_vomit(&error.to_string()),
                }
                return iced::Task::none();
            }
        }
    }

    /// reads from the pty and adds it to the buffer
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "all is bound checked"
    )]
    fn update_screen_buffer(&mut self, vec: &[u8]) -> Result<(), Error> {
        for chr in String::from_utf8_lossy(vec).ansi_parse() {
            match chr {
                Token::Text(txt) => {
                    print_debug(&(String::from("[CHR]") + txt));
                    if self.screen_buffer_index < self.screen_buffer.len() {
                        self.screen_buffer[self.screen_buffer_index] =
                            *txt.as_bytes().get(0).unwrap_or(&b'_');
                        self.screen_buffer_index += 1;
                    }
                }
                Token::C0(c0) => print_debug(&(String::from("[C0]") + &format!("{:?}", c0))),
                Token::EscapeSequence(seq) => {
                    print_debug(&(String::from("[SEQ]") + &format!("{:?}", seq)))
                }
            }
        }
        return Ok(());
    }

    /// view logic for model\
    /// TODO add wide char support\
    /// TODO bound check
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::string_slice,
        reason = "TODO"
    )]
    pub fn view(&self) -> Element<'_, Msg> {
        let (left, right) =
            String::from_utf8_lossy(&self.screen_buffer[..self.screen_buffer_index])
                .rsplit_once('\n')
                .map_or_else(
                    || {
                        return (
                            String::new(),
                            String::from_utf8_lossy(
                                &self.screen_buffer[..self.screen_buffer_index],
                            )
                            .to_string(),
                        );
                    },
                    |tup| {
                        return (tup.0.to_owned(), tup.1.to_owned());
                    },
                );
        return scrollable(column![
            text(left),
            row![
                text(right),
                text(&self.input[..(self.cursor_index)]),
                if self.cursor_index < self.input.len() {
                    row![
                        if self.input[self.cursor_index..=self.cursor_index] == *" " {
                            text("_").color(iced::Color::from_rgb(f32::MAX, 0.0, 0.0))
                        } else {
                            text(&self.input[self.cursor_index..=self.cursor_index])
                                .color(iced::Color::from_rgb(f32::MAX, 0.0, 0.0))
                        },
                        text(&self.input[(self.cursor_index + 1)..])
                    ]
                } else {
                    row![
                        text(&self.input[self.cursor_index..]),
                        rich_text![
                            span("_")
                                .color(iced::Color::from_rgb(f32::MAX, 0.0, 0.0))
                                .background(iced::Color::from_rgb(f32::MAX, f32::MAX, 0.0))
                        ]
                    ]
                }
            ]
        ])
        .into();
    }
}

impl Default for Model<'_> {
    #[inline]
    #[expect(clippy::undocumented_unsafe_blocks, reason = "clippy be trippin")]
    fn default() -> Self {
        let mut me = Self {
            screen_buffer: [0; 0x4000],
            screen_buffer_index: 0,
            cursor_index: 0,
            fd: None,
            input: String::new(),
            /// SAFETY call *after* `init()`
            shell: unsafe { SHELL.clone() }.map_or_else(
                || return String::from("/home/mtgmonkey/.nix-profile/bin/dash"),
                |shell| return shell,
            ),
            screen: vec![],
            cursor: (1, 1),
        };
        me.fd = spawn_pty_with_shell(&me.shell).ok();
        let mut nored = true;
        while nored {
            let red = read_from_option_fd(me.fd.as_ref());
            if let Ok(red) = red {
                nored = false;
                if let Err(error) = me.update_screen_buffer(&red) {
                    print_err(&error);
                }
            }
        }
        return me;
    }
}

/// # Safety
/// call *before* creating a `Model` because `Model::default()` relies on `SHELL`
/// call *before* `print_err()` because `print_err()` relies on `VERBOSE`
#[inline]
#[expect(clippy::undocumented_unsafe_blocks, reason = "clippy be trippin")]
pub unsafe fn init(flags: Flags) {
    unsafe {
        DEBUG = flags.debug;
    }
    unsafe {
        VERBOSE = flags.verbose;
    }
    unsafe {
        VOMIT = flags.vomit;
    }
    unsafe {
        SHELL = flags.shell;
    }
}

/// spawns a pty with the specified shell program
#[expect(clippy::single_call_fn, reason = "abstraction")]
fn spawn_pty_with_shell(default_shell: &str) -> Result<OwnedFd, Error> {
    // SAFETY: always safe unless the OS is out of ptys
    // so it is always safe
    match unsafe { forkpty(None, None) } {
        Ok(fork_pty_res) => match fork_pty_res {
            ForkptyResult::Parent { master, .. } => {
                if let Err(error) = set_nonblock(&master) {
                    return Err(error);
                }
                return Ok(master);
            }
            ForkptyResult::Child => {
                if let Err(error) = Command::new(default_shell).spawn() {
                    return Err(Error::Io(error));
                }
                thread::sleep(core_time::Duration::MAX);
                return Err(Error::Unreachable);
            }
        },
        Err(error) => return Err(Error::Nix(NixError::Errno(error))),
    }
}

/// reads from an `&OwnedFd`
/// TODO check bounds
#[expect(
    clippy::single_call_fn,
    clippy::indexing_slicing,
    reason = "abstraction"
)]
fn read_from_fd(fd: &OwnedFd) -> Result<Vec<u8>, Error> {
    let mut read_buffer = [0; 0x4000];
    #[expect(clippy::unwrap_used, reason = "platform-specific but fine")]
    let mut file = File::from(fd.try_clone().unwrap());
    let file = file.read(&mut read_buffer);
    match file {
        Ok(file) => return Ok(read_buffer[..file].to_vec()),
        Err(error) => return Err(Error::Io(error)),
    }
}

/// reads from an `Option<&OwnedFd>` if it's there
fn read_from_option_fd(maybe_fd: Option<&OwnedFd>) -> Result<Vec<u8>, Error> {
    return maybe_fd.map_or(Err(Error::NoFileDescriptor), |fd| {
        return read_from_fd(fd);
    });
}

/// sets a `OwnedFd` as nonblocking.
#[expect(clippy::single_call_fn, reason = "abstraction")]
fn set_nonblock(fd: &OwnedFd) -> Result<(), Error> {
    let flags = match fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL) {
        Ok(flags) => flags,
        Err(errno) => return Err(Error::Nix(NixError::Errno(errno))),
    };
    let flags = fcntl::OFlag::from_bits(flags & fcntl::OFlag::O_ACCMODE.bits());
    match flags {
        Some(mut flags) => {
            flags.set(fcntl::OFlag::O_NONBLOCK, true);
            if let Err(errno) = fcntl::fcntl(fd, fcntl::FcntlArg::F_SETFL(flags)) {
                return Err(Error::Nix(NixError::Errno(errno)));
            }
        }
        None => return Err(Error::Nix(NixError::UnrecognisedFlag)),
    }
    return Ok(());
}

/// if `VERBOSE` is `true`, logs errors
#[inline]
#[expect(
    clippy::print_stdout,
    clippy::undocumented_unsafe_blocks,
    reason = "toggleable with VERBOSE option\n
    clippy be buggin"
)]
fn print_err(error: &Error) {
    /// SAFETY the only time `VERBOSE` is written to should be `init()`
    if unsafe { VERBOSE } {
        println!("[ERROR] {error}");
    }
}

/// if `VOMIT` is `true`, logs vomit
#[inline]
#[expect(
    clippy::print_stdout,
    clippy::undocumented_unsafe_blocks,
    reason = "toggleable with VERBOSE option\n
    clippy be buggin"
)]
fn print_vomit(vomit: &str) {
    /// SAFETY the only time `VOMIT` is written to should be `init()`
    if unsafe { VOMIT } {
        println!("[VOMIT] {:?}", vomit);
    }
}

/// if `DEBUG` is `true`, logs errors
#[inline]
#[expect(
    clippy::print_stdout,
    clippy::undocumented_unsafe_blocks,
    reason = "toggleable with VERBOSE option\n
    clippy be buggin"
)]
fn print_debug(debug: &str) {
    /// SAFETY the only time `DEBUG` is written to should be `init()`
    if unsafe { DEBUG } {
        println!("[DEBUG] {:?}", debug);
    }
}
