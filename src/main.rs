use std::fs::File;
use std::io::{self, Write};
use std::process::{exit, Command, Stdio};
use std::thread;
use std::time::Instant;

#[cfg(unix)]
use std::os::unix::io::FromRawFd;

extern crate libc;

extern crate rtss;
use rtss::{duration_to_human, line_timing_copy};

const VERSION: &str = "0.5.0";

fn usage() {
    println!(
        "Usage: {} [-h | --help] [-v | --version] | [--tty | --pty] [--] [COMMAND [ARGS ...]]",
        std::env::args().into_iter().next().unwrap()
    );
    println!();
    println!("Prepends output lines with elapsed times since program start and previous line.");
    println!();
    println!("Use either to wrap stdout and stderr of a given command, or as a filter.");
    println!();
    println!("Use --pty/--tty to unbuffer commands like tcpdump when ran under rtss.")
}

#[cfg(unix)]
fn attach_tty(child: &mut Command) -> File {
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

    if pty != 0 {
        panic!("Couldn't open pty");
    }

    child.stdout(unsafe { Stdio::from_raw_fd(slave) });
    unsafe { File::from_raw_fd(master) }
}

#[cfg(not(unix))]
fn attach_tty(_child: &mut Command) -> File {
    unimplemented!();
}

fn main() {
    let mut command = vec![];
    let mut myargs = true;
    let mut use_tty = false;
    for arg in std::env::args_os().into_iter().skip(1) {
        if myargs {
            if &arg == "-h" || &arg == "--help" {
                usage();
                std::process::exit(0);
            } else if &arg == "-v" || &arg == "--version" {
                println!("rtss version {}", VERSION);
                std::process::exit(0);
            } else if cfg!(unix) && (&arg == "--pty" || &arg == "--tty") {
                use_tty = true;
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
        let mut child = Command::new(cmd);
        child
            .args(args)
            .stdin(Stdio::inherit())
            .stderr(Stdio::piped());

        let mut tty: Option<File> = None;

        if use_tty {
            tty = Some(attach_tty(&mut child));
        } else {
            child.stdout(Stdio::piped());
        }

        let mut child = child.spawn().unwrap_or_else(|e| {
            writeln!(stderr, "{}: {}", cmd.to_string_lossy(), e).ok();
            exit(64 + e.raw_os_error().unwrap_or(0));
        });

        let out = if let Some(mut child_stdout) = tty {
            thread::spawn(move || line_timing_copy(&mut child_stdout, &mut stdout, '|', &start))
        } else {
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
