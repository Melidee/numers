#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(slice_split_once)]

use clap::{Arg, Command, Parser};

mod compiler;
mod error;
mod parser;

fn main() {
    let cmd = Command::new("numerus")
        .arg(
            Arg::new("output")
                .short('o')
                .help("path of the file to output program")
                .num_args(1),
        )
        .arg(
            Arg::new("ssa")
                .help("")
                .num_args(0),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .help("compile for a target among:\n\tamd64_sysv (default), amd64_apple, arm64, arm64_apple, rv64")
                .num_args(1)
        );
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path of the file to output to
    #[arg(short, long, default_value = "a.out")]
    output: String,
    /// source code to compile
    #[arg(short, long)]
    /// output in qbe ssa (single static assignment)
    #[arg(long)]
    ssa: bool,
    /// compile for a target among:\n\tamd64_sysv (default), amd64_apple, arm64, arm64_apple, rv64
    #[arg(short, long, default_value = "amd64_sysv")]
    target: String,
}
