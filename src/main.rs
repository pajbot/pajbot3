#![deny(clippy::all)]
#![deny(clippy::cargo)]

use crate::args::Args;

mod args;

fn main() {
    let args = Args::parse();

    dbg!(args);
}
