use std::fmt::Write as FmtWrite;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

/// Convert a `time::Duration` to a formatted `String` such as
/// "15h4m5.424s" or "424ms", or "" for a zero duration.
pub fn duration_to_human(d: &Duration) -> String {
    let mut s = d.as_secs();
    let ms = d.subsec_nanos() / 1_000_000;

    let mut ret = String::new();

    if s >= 86400 {
        write!(ret, "{}d", s / 86400).unwrap();
        s %= 86400;
    }

    if s >= 3600 {
        write!(ret, "{}h", s / 3600).unwrap();
        s %= 3600;
    }

    if s >= 60 {
        write!(ret, "{}m", s / 60).unwrap();
        s %= 60
    }

    if s > 0 {
        write!(ret, "{}.{:03}s", s, ms).unwrap();
    } else if ms > 0 {
        write!(ret, "{}ms", ms).unwrap();
    }

    ret
}

/// Copy each line from `input` to `output`, prepending the output line with
/// elapsed time since `start` and since the previous line, until EOF or IO
/// error.
///
/// Returns the number of bytes read from `input` on success.
pub fn line_timing_copy<R: io::Read, W: io::Write>(
    input: &mut R,
    output: &mut W,
    start: &Instant,
) -> io::Result<u64> {
    let mut input = io::BufReader::new(input);
    let mut output = io::BufWriter::new(output);

    let mut buf = vec![0_u8; 256];
    let mut last = Instant::now();
    let mut n = 0_u64;

    while input.read_until(b'\n', &mut buf)? > 0 {
        n += buf.len() as u64;
        let since_last = last.elapsed();
        let since_start = start.elapsed();
        last = Instant::now();

        write!(
            output,
            "{:>8} {:>8} | ",
            duration_to_human(&since_start),
            duration_to_human(&since_last),
        )?;
        output.write_all(&buf)?;
        output.flush()?;
        buf.clear();
    }

    Ok(n)
}
