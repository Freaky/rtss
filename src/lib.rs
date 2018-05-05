//! rtss - Relative TimeStamps for Stuff.
//!
//! An `io::Write` implementation which prefixes each line with a timestamp since
//! a start time, and the duration since the previous line, if any.
//!
//! Also a couple of utility functions for formatting `Duration`, and copying from
//! one IO to another.

use std::fmt::Write as FmtWrite;
use std::io;
use std::time::{Duration, Instant};

extern crate memchr;
use memchr::memchr;

/// Convert a `time::Duration` to a formatted `String` sortable
/// lexographically like "15:04:05.421224"
pub fn duration_to_sortable(d: &Duration) -> String {
    let mut ret = String::with_capacity(16);
    duration_to_sortable_replace(&d, &mut ret);
    ret
}

/// As `duration_to_sortable`, but replacing the contents of the provided `String`.
pub fn duration_to_sortable_replace(d: &Duration, buf: &mut String) {
    let ts = d.as_secs();
    let us = d.subsec_nanos() / 1000;

    buf.clear();

    write!(
        buf,
        "{:02}:{:02}:{:02}.{:06}",
        ts / 3600,
        (ts % 3600) / 60,
        ts % 60,
        us
    ).unwrap();
}

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

pub type DurationFormatter = fn(&Duration) -> String;

/// A writer that prefixes all lines with relative timestamps.
pub struct RtssWriter<W> {
    inner: W,
    formatter: DurationFormatter,
    separator: char,
    start: Instant,
    last: Instant,
    at_eol: bool,
}

impl<W: io::Write> RtssWriter<W> {
    /// Create a new `RtssWriter`, with a given start time, and delimiter separating
    /// the prefix and content.
    pub fn new(inner: W, formatter: DurationFormatter, separator: char, now: &Instant) -> Self {
        Self {
            inner,
            formatter,
            separator,
            start: *now,
            last: *now,
            at_eol: true,
        }
    }
}

impl<W: io::Write> io::Write for RtssWriter<W> {
    /// Writes the contents of `buf` to the underlying writer, with time annotations
    /// for any new lines.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let now = Instant::now();
        let start_duration = (self.formatter)(&now.duration_since(self.start));
        let line_duration = (self.formatter)(&now.duration_since(self.last));

        let pfx_start = format!(
            "{:>8} {:>8} {} ",
            start_duration, line_duration, self.separator
        );
        let pfx_rest = format!("{:>8} {:>8} {} ", start_duration, "", self.separator);

        let mut pos: usize = 0;
        let mut saw_eol = false;
        let mut first = true;

        let n = buf.len();

        while pos < n {
            if self.at_eol {
                if first {
                    self.inner.write_all(pfx_start.as_bytes())?;
                    first = false;
                } else {
                    self.inner.write_all(pfx_rest.as_bytes())?;
                }
            }

            if let Some(newline) = memchr(b'\n', &buf[pos..n]) {
                saw_eol = true;
                self.at_eol = true;
                self.inner.write_all(&buf[pos..(pos + newline + 1)])?;
                pos += newline + 1;
            } else {
                self.at_eol = false;
                self.inner.write_all(&buf[pos..n])?;
                break;
            }
        }

        self.inner.flush()?;

        if saw_eol {
            self.last = now;
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Copy each line from `input` to `output`, prepending the output line with
/// elapsed time since `start` and since the previous line, separated by `separator`
/// until EOF or IO error.
///
/// Returns the number of bytes read from `input` on success.
pub fn line_timing_copy<R: io::Read, W: io::Write>(
    mut input: &mut R,
    output: &mut W,
    formatter: DurationFormatter,
    separator: char,
    start: &Instant,
) -> io::Result<u64> {
    let output = io::BufWriter::new(output);
    let mut output = RtssWriter::new(output, formatter, separator, start);

    io::copy(&mut input, &mut output)
}
