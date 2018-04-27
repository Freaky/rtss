use std::fmt::Write as FmtWrite;
use std::io::{self, BufRead, Read, Write};
use std::time::{Duration, Instant};

const MAX_LINE: u64 = 1024 * 8;

/// Convert a `time::Duration` to a formatted `String` such as
/// "15h4m5.42s" or "424.2ms", or "" for a zero duration.
pub fn duration_to_human(d: &Duration) -> String {
    let mut ret = String::with_capacity(16);
    duration_to_human_replace(&d, &mut ret);
    ret
}

/// As duration_to_human, but replacing the contents of a user-provided `String`.
pub fn duration_to_human_replace(d: &Duration, buf: &mut String) {
    let ts = d.as_secs();
    let ns = d.subsec_nanos();

    buf.clear();

    if ts > 0 {
        let mut s = ts;
        let mut cs = (f64::from(ns) / 10_000_000_f64).round() as u64;
        if cs == 100 {
            // round up to the nearest centisecond
            s += 1;
            cs = 0;
        }

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
    let mut input = io::BufReader::new(input);
    let mut output = io::BufWriter::new(output);

    let mut start_duration = String::with_capacity(16);
    let mut line_duration = String::with_capacity(16);

    let mut buf = Vec::with_capacity(MAX_LINE as usize);
    let mut last = Instant::now();
    let mut run_on = false;
    let mut n = 0_u64;

    while input.by_ref().take(MAX_LINE).read_until(b'\n', &mut buf)? > 0 {
        n += buf.len() as u64;

        if !run_on {
            duration_to_human_replace(&start.elapsed(), &mut start_duration);
            duration_to_human_replace(&last.elapsed(), &mut line_duration);
            last = Instant::now();
            write!(
                output,
                "{:>8} {:>8} {} ",
                start_duration, line_duration, separator
            )?;
        }

        run_on = buf.last().expect("buf can't be empty") != &b'\n';

        output.write_all(&buf)?;
        output.flush()?;
        buf.clear();
    }

    Ok(n)
}
