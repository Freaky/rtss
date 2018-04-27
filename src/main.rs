use std::io::{self, Write};
use std::process::{exit, Command, Stdio};
use std::thread;
use std::time::Instant;

extern crate rtss;
use rtss::{duration_to_human, line_timing_copy};

fn usage() {
    println!(
        "Usage: {} [-h | --help] [-v | --version] [--] [command [args]]",
        std::env::args().into_iter().next().unwrap()
    );
    println!();
    println!("Writes stdin to stdout with elapsed times prepended to each line.");
    println!();
    println!("Alternatively runs given command, with stdout and stderr filtered through rtss.");
}

const VERSION: &str = "0.2";

fn main() {
    let mut command = vec![];
    let mut myargs = true;
    for arg in std::env::args_os().into_iter().skip(1) {
        if myargs {
            if &arg == "-h" || &arg == "--help" {
                usage();
                std::process::exit(0);
            } else if &arg == "-v" || &arg == "--version" {
                println!("rtss version {}", VERSION);
                std::process::exit(0);
            } else if &arg == "--" {
                myargs = false;
            } else {
                myargs = false;
                command.push(arg);
            }
        } else {
            command.push(arg);
        }
    }

    let start = Instant::now();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    if command.is_empty() {
        let mut stdin = io::stdin();
        let mut ex = 0;
        if let Err(e) = line_timing_copy(&mut stdin, &mut stdout, '|', &start) {
            writeln!(io::stderr(), "{:?}", e).ok();
            ex = 64 + e.raw_os_error().unwrap_or(0);
        }
        writeln!(
            io::stdout(),
            "{}    exit {}",
            duration_to_human(&start.elapsed()),
            ex
        ).ok();
        exit(ex);
    } else if let Some((cmd, args)) = command.split_first() {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                writeln!(stderr, "{}: {}", cmd.to_string_lossy(), e).ok();
                exit(64 + e.raw_os_error().unwrap_or(0));
            });

        let out = {
            let mut child_stdout = child.stdout.take().expect("Failed to attach to stdout");
            thread::spawn(move || line_timing_copy(&mut child_stdout, &mut stdout, '|', &start))
        };

        let err = {
            let mut child_stderr = child.stderr.take().expect("Failed to attach to stderr");
            thread::spawn(move || line_timing_copy(&mut child_stderr, &mut stderr, '#', &start))
        };

        if let Err(e) = err.join().expect("stderr thread panicked") {
            writeln!(io::stderr(), "stderr: {}", e).ok();
        }

        if let Err(e) = out.join().expect("stdout thread panicked") {
            writeln!(io::stderr(), "stdout: {}", e).ok();
        }

        let status = child.wait().expect("waitpid");

        writeln!(
            io::stdout(),
            "{:>8}    {}",
            duration_to_human(&start.elapsed()),
            status
        ).ok();

        exit(status.code().unwrap_or(64));
    }
}
