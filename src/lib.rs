use std::fmt::Write as FmtWrite;
use std::io::{self, ErrorKind, Write};
use std::time::{Duration, Instant};

extern crate memchr;
use memchr::memchr;

const BUF_SIZE: usize = 1024 * 8;

/// Convert a `time::Duration` to a formatted `String` such as
/// "15h4m5.42s" or "424.2ms", or "" for a zero duration.
pub fn duration_to_human(d: &Duration) -> String {
    let mut ret = String::with_capacity(16);
    duration_to_human_replace(&d, &mut ret);
    ret
}

/// As duration_to_human, but replacing the contents of a user-provided `String`.
pub fn duration_to_human_replace(d: &Duration, buf: &mut String) {
    let mut ts = d.as_secs();
    let ns = d.subsec_nanos();

    buf.clear();

    if ts > 0 {
        let mut cs = (f64::from(ns) / 10_000_000_f64).round() as u8;
        if cs == 100 {
            // round up to the nearest centisecond
            ts += 1;
            cs = 0;
        }

        let mut s = ts;

        if ts >= 86400 {
            write!(buf, "{}d", s / 86400).unwrap();
            s %= 86400;
        }

        if ts >= 3600 {
            write!(buf, "{}h", s / 3600).unwrap();
            s %= 3600;
        }

        if ts >= 60 {
            write!(buf, "{}m", s / 60).unwrap();
            s %= 60
        }

        write!(buf, "{}.{:02}s", s, cs).unwrap();
    } else if ns > 100_000 {
        write!(buf, "{:.1}ms", f64::from(ns) / 1_000_000_f64).unwrap();
    } else if ns > 100 {
        write!(buf, "{:.1}μs", f64::from(ns) / 1_000_f64).unwrap();
    }
}

/// Copy each line from `input` to `output`, prepending the output line with
/// elapsed time since `start` and since the previous line, separated by `separator`
/// until EOF or IO error.
///
/// Returns the number of bytes read from `input` on success.
pub fn line_timing_copy<R: io::Read, W: io::Write>(
    input: &mut R,
    output: &mut W,
    separator: char,
    start: &Instant,
) -> io::Result<u64> {
    let mut output = io::BufWriter::new(output);

    let mut start_duration = String::with_capacity(16);
    let mut line_duration = String::with_capacity(16);

    let mut buf = vec![0_u8; BUF_SIZE];
    let mut last = start.clone();
    let mut total = 0_u64;

    let mut at_eol = true;

    loop {
        match input.read(&mut buf) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                let now = Instant::now();
                duration_to_human_replace(&now.duration_since(*start), &mut start_duration);
                duration_to_human_replace(&now.duration_since(last), &mut line_duration);

                total += n as u64;

                let mut pos: usize = 0;
                let mut saw_eol = false;

                while pos < n {
                    if at_eol {
                        write!(
                            output,
                            "{:>8} {:>8} {} ",
                            start_duration, line_duration, separator
                        )?;
                        line_duration.clear();
                    }

                    if let Some(newline) = memchr(b'\n', &buf[pos..n]) {
                        saw_eol = true;
                        at_eol = true;
                        output.write_all(&buf[pos..(pos + newline + 1)])?;
                        pos += newline + 1;
                    } else {
                        at_eol = false;
                        output.write_all(&buf[pos..n])?;
                        break;
                    }
                }

                output.flush()?;

                if saw_eol {
                    last = now;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}
