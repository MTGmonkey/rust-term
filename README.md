# this repo's home is at [https://git.mtgmonkey.net/Andromeda/rust-term](https://git.mtgmonkey.net/Andromeda/rust-term)
# please view it there if you are on the github mirror right now

# rust_term

rust_term is a Rust terminal emulator aiming to be akin to [st](https://st.suckless.org) in style and performance.

## installation

use nix to try it out

```bash
nix run git+https://git.mtgmonkey.net/Andromeda/rust-term .# -- --help
```

or compile a binary

```bash
nix build git+https://git.mtgmonkey.net/Andromeda/rust-term
# ./result/bin/rust_term --help
```

note that the default binary is hard-coded to `/home/mtgmonkey/.nix-profile/bin/dash`, so a `--shell={}` argument is needed unless you're me

## usage

```bash
rust_term --help
```

```
cli flags

Usage: rust_term [-S=ARG] [-v] [-v]

Available options:
    -S, --shell=ARG  path to shell
    -v, --verbose    whether to debug log
    -v, --version    whether to display version: TODO
    -h, --help       Prints help information
```

`ARG` should be an absolute path

## contributing

pull requests are welcome on [the github](https://github.com/MTGmonkey/rust-term), as are issues

make sure clippy throws no warnings!

```bash
nix develop
cargo clippy
```

## license

all code is licensed under [WTFPL](https://wtfpl.net)
