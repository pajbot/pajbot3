# Install prerequisites

## The Rust programming language

For building the bot and the book, you will need to have a working installation of the Rust programming language - you may follow their guide: https://www.rust-lang.org/tools/install.
Afterwards, you should have the `cargo` and `rustup` commands available at the command line, for example:

```none
you@yourmachine:~$ rustup show
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/you/.rustup

stable-x86_64-unknown-linux-gnu (default)
rustc 1.56.1 (59eed8a2a 2021-11-01)
```

or

```none
PS C:\Users\You> rustup show
Default host: x86_64-pc-windows-msvc
rustup home:  C:\Users\You\.rustup

stable-x86_64-pc-windows-msvc (default)
rustc 1.56.1 (59eed8a2a 2021-11-01)
```

## Node.js

For developing or building the web interface, you need an installation of the JavaScript runtime. For Windows and macOS, you can download it from here: https://nodejs.org/en/download/ whilst on Linux and BSD, you should probably use a package manager: https://nodejs.org/en/download/package-manager/.

Afterwards, the `node` and `npm` command-line tools should be available at the command-line.

Note that we currently support Node.js versions 12, 14 and 16 (Try `node --version` if you are unsure)

## mdbook

To build the book, you'll also need the [`mdbook` utility](https://github.com/rust-lang/mdBook) which can be installed using `cargo`:

```none
$ cargo install mdbook --vers "^0.4"
```
