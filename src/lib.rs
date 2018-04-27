use std::fmt::Write as FmtWrite;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

/// Convert a `time::Duration` to a formatted `String` such as
/// "15h4m5.424s" or "424ms", or "" for a zero duration.
pub fn duration_to_human(d: &Duration) -> String {
    let ts = d.as_secs();
    let ms = f64::from(d.subsec_nanos()) / 1_000_000_f64;

    let mut ret = String::with_capacity(10);

    if ts > 0 {
        let mut s = ts;
        let mut ds = (ms / 10_f64).round() as u64;
        if ds == 100 {
            // round up to the nearest decisecond
            s += 1;
            ds = 0;
        }

        if ts >= 86400 {
            write!(ret, "{}d", s / 86400).unwrap();
            s %= 86400;
        }

        if ts >= 3600 {
            write!(ret, "{}h", s / 3600).unwrap();
            s %= 3600;
        }

        if ts >= 60 {
            write!(ret, "{}m", s / 60).unwrap();
            s %= 60
        }

        write!(ret, "{}.{:02}s", s, ds).unwrap();
    } else if ms > 0_f64 {
        write!(ret, "{:.1}ms", ms).unwrap();
    }

    ret
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

    let mut buf = Vec::with_capacity(256);
    let mut last = Instant::now();
    let mut n = 0_u64;

    while input.read_until(b'\n', &mut buf)? > 0 {
        n += buf.len() as u64;
        let since_last = last.elapsed();
        let since_start = start.elapsed();
        last = Instant::now();

        write!(
            output,
            "{:>8} {:>8} {} ",
            duration_to_human(&since_start),
            duration_to_human(&since_last),
            separator
        )?;
        output.write_all(&buf)?;
        output.flush()?;
        buf.clear();
    }

    Ok(n)
}
