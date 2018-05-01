use std::io::{self, Write};
use std::process::{exit, Command, Stdio};
use std::thread;
use std::time::Instant;

extern crate libc;

extern crate rtss;
use rtss::{duration_to_human, line_timing_copy};

const VERSION: &str = "0.5.0";

fn usage() {
    println!(
        "Usage: {} [-h | --help] [-v | --version] | [--] [COMMAND [ARGS ...]]",
        std::env::args().into_iter().next().unwrap()
    );
    println!();
    println!("Prepends output lines with elapsed times since program start and previous line.");
    println!();
    println!("Use either to wrap stdout and stderr of a given command, or as a filter.");
}

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
            "{:>8}    exit code: {}",
            duration_to_human(&start.elapsed()),
            ex
        ).ok();
        exit(ex);
    } else if let Some((cmd, args)) = command.split_first() {
        let mut master: libc::c_int = 0;
        let mut slave: libc::c_int = 0;
        let pty = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };

        println!("{:?}", pty);
        if pty != 0 {
            panic!("Couldn't open pty");
        }
        use std::os::unix::io::FromRawFd;

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(unsafe { Stdio::from_raw_fd(slave) })
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                writeln!(stderr, "{}: {}", cmd.to_string_lossy(), e).ok();
                exit(64 + e.raw_os_error().unwrap_or(0));
            });

        let out = {
            // let mut child_stdout = child.stdout.take().expect("Failed to attach to stdout");
            let mut child_stdout = unsafe { std::fs::File::from_raw_fd(master) };
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
