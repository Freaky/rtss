use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

extern crate rtss;
use rtss::{duration_to_human, line_timing_copy};

fn ex_usage(ex: i32) {
    println!(
        "Usage: {} [-h | --help] [-v | --version] [--] [command [args]]",
        std::env::args().into_iter().next().unwrap()
    );
    println!();
    println!("Writes stdin to stdout with elapsed times prepended to each line.");
    println!();
    println!("Alternatively runs given command, with stdout and stderr filtered through rtss.");

    std::process::exit(ex);
}

const VERSION: &str = "0.2";

fn main() {
    let mut command = vec![];
    let mut myargs = true;
    for arg in std::env::args().into_iter().skip(1) {
        match arg.as_ref() {
            "-h" | "--help" if myargs => {
                ex_usage(0);
            }
            "-v" | "--version" if myargs => {
                println!("rtss version {}", VERSION);
                std::process::exit(0);
            }
            "--" if myargs => {
                myargs = false;
            }
            arg => {
                myargs = false;
                command.push(arg.to_string());
            }
        }
    }

    let start = Instant::now();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    if command.is_empty() {
        let mut stdin = io::stdin();
        if let Err(e) = line_timing_copy(&mut stdin, &mut stdout, &start) {
            writeln!(io::stderr(), "{:?}", e).ok();
        }
        println!("Elapsed: {}", duration_to_human(&start.elapsed()));
    } else if let Some((cmd, args)) = command.split_first() {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child");

        {
            let out = {
                let mut child_stdout = child.stdout.take().expect("Failed to attach to stdout");
                thread::spawn(move || line_timing_copy(&mut child_stdout, &mut stdout, &start))
            };

            let err = {
                let mut child_stderr = child.stderr.take().expect("Failed to attach to stderr");
                thread::spawn(move || line_timing_copy(&mut child_stderr, &mut stderr, &start))
            };

            if let Err(e) = out.join().expect("stdout thread paniced") {
                writeln!(io::stderr(), "Error on stdout: {:?}", e).ok();
            }

            if let Err(e) = err.join().expect("stderr thread paniced") {
                writeln!(io::stderr(), "Error on stderr: {:?}", e).ok();
            }
        }

        let ex = child.wait().unwrap().code().unwrap_or(-1);
        writeln!(
            io::stderr(),
            "Exit: {}, Elapsed: {}",
            ex,
            duration_to_human(&start.elapsed())
        ).ok();

        std::process::exit(ex);
    }
}
