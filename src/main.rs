use std::io::{self, Write};
use std::time::Instant;

extern crate rtss;
use rtss::{duration_to_human, line_timing_copy};

fn ex_usage(ex: i32) {
    println!(
        "Usage: {} [-h | --help] [-v | --version]",
        std::env::args().into_iter().next().unwrap()
    );
    println!();
    println!("Writes stdin to stdout with elapsed times prepended to each line.");

    std::process::exit(ex);
}

const VERSION: &str = "0.1";

fn main() {
    for arg in std::env::args().into_iter().skip(1) {
        match arg.as_ref() {
            "-h" | "--help" => {
                ex_usage(0);
            }
            "-v" | "--version" => {
                println!("rtss version {}", VERSION);
                std::process::exit(0);
            }
            _ => {
                ex_usage(1);
            }
        }
    }

    let start = Instant::now();

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    if let Err(e) = line_timing_copy(&mut stdin, &mut stdout, &start) {
        writeln!(io::stderr(), "{:?}", e).ok();
    }
    println!("Elapsed: {}", duration_to_human(&start.elapsed()));
}
