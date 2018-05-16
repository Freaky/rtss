use std::fs::File;
use std::io;
use std::process::{exit, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::io::FromRawFd;

extern crate libc;

extern crate rtss;
use rtss::{line_timing_copy, DurationExt, DurationFormatter};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() {
    println!(
        "Usage: {} [-h | --help] [-v | --version] | {}[--] [COMMAND [ARGS ...]]",
        std::env::args().into_iter().next().unwrap(),
        if cfg!(unix) { "[--tty | --pty] " } else { "" }
    );
    println!();
    println!("Prepends output lines with elapsed times since program start and previous line.");
    println!();
    println!("Use either to wrap stdout and stderr of a given command, or as a filter.");
    if cfg!(unix) {
        println!();
        println!("Use --pty/--tty to unbuffer commands like tcpdump when ran under rtss.");
    }
}

#[cfg(unix)]
fn attach_tty(child: &mut Command) -> (File, File) {
    use std::os::unix::process::CommandExt;
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
    child.before_exec(move || {
        drop(unsafe { File::from_raw_fd(master) });
        Ok(())
    });
    unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) }
}

#[cfg(not(unix))]
fn attach_tty(_child: &mut Command) -> (File, File) {
    unimplemented!();
}

fn main() {
    let mut command = vec![];
    let mut myargs = true;
    let mut use_tty = false;
    let mut format_duration: DurationFormatter = Duration::human_string;
    for arg in std::env::args_os().into_iter().skip(1) {
        if myargs {
            if &arg == "-h" || &arg == "--help" {
                usage();
                std::process::exit(0);
            } else if &arg == "-v" || &arg == "--version" {
                println!("rtss version {}", VERSION);
                std::process::exit(0);
            } else if &arg == "-s" || &arg == "--sortable" {
                format_duration = Duration::sortable_string;
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
        if let Err(e) = line_timing_copy(&mut stdin, &mut stdout, format_duration, '|', &start) {
            eprintln!("{:?}", e);
            ex = 64 + e.raw_os_error().unwrap_or(0);
        }
        println!(
            "{:>8}    exit code: {}",
            format_duration(&start.elapsed()),
            ex
        );
        exit(ex);
    } else if let Some((cmd, args)) = command.split_first() {
        let mut child = Command::new(cmd);
        child
            .args(args)
            .stdin(Stdio::inherit())
            .stderr(Stdio::piped());

        let tty: Option<(File, File)> = if use_tty {
            Some(attach_tty(&mut child))
        } else {
            child.stdout(Stdio::piped());
            None
        };

        let mut child = child.spawn().unwrap_or_else(|e| {
            eprintln!("{}: {}", cmd.to_string_lossy(), e);
            exit(64 + e.raw_os_error().unwrap_or(0));
        });

        let out = if let Some((mut master, mut slave)) = tty {
            drop(slave);
            thread::spawn(move || {
                line_timing_copy(&mut master, &mut stdout, format_duration, '|', &start)
            })
        } else {
            let mut child_stdout = child.stdout.take().expect("Failed to attach to stdout");
            thread::spawn(move || {
                line_timing_copy(&mut child_stdout, &mut stdout, format_duration, '|', &start)
            })
        };

        let err = {
            let mut child_stderr = child.stderr.take().expect("Failed to attach to stderr");
            thread::spawn(move || {
                line_timing_copy(&mut child_stderr, &mut stderr, format_duration, '#', &start)
            })
        };

        let status = child.wait().expect("waitpid");

        println!("{:>8}    {}", format_duration(&start.elapsed()), status);

        if let Err(e) = err.join().expect("stderr thread panicked") {
            eprintln!("stderr: {}", e);
        }

        if let Err(e) = out.join().expect("stdout thread panicked") {
            // suppress EIO in pty mode (thrown by Linux on normal exit)
            if !use_tty || e.raw_os_error().unwrap_or(0) != 5 {
                eprintln!("stdout: {}", e);
            }
        }

        exit(status.code().unwrap_or(64));
    }
}
