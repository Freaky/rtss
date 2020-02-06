//! rtss - Relative TimeStamps for Stuff.
//!
//! An `io::Write` implementation which prefixes each line with a timestamp since
//! a start time, and the duration since the previous line, if any.
//!
//! Also a couple of utility functions for formatting `Duration`, and copying from
//! one IO to another.

use std::io::{self, Cursor};
use std::time::{Duration, Instant};

use memchr::memchr;

pub trait DurationExt {
    /// Write a `Duration` to a formatted form for human consumption.
    fn write_human<W: io::Write>(&self, out: &mut W) -> io::Result<()>;

    /// Write a `Duration` to a formatted form sortable lexographically,
    /// like "15:04:05.421224"
    fn write_sortable<W: io::Write>(&self, out: &mut W) -> io::Result<()>;

    /// Return the results of `write_human()` as a new `String`
    fn human_string(&self) -> String {
        let mut v = Cursor::new(Vec::with_capacity(16));
        self.write_human(&mut v).unwrap();
        String::from_utf8(v.into_inner()).unwrap()
    }

    /// Return the results of `write_sortable()` as a new `String`
    fn sortable_string(&self) -> String {
        let mut v = Cursor::new(Vec::with_capacity(16));
        self.write_sortable(&mut v).unwrap();
        String::from_utf8(v.into_inner()).unwrap()
    }
}

impl DurationExt for Duration {
    fn write_human<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        let mut ts = self.as_secs();
        let ns = self.subsec_nanos();

        if ts > 0 {
            let mut cs = (f64::from(ns) / 10_000_000_f64).round() as u8;
            if cs == 100 {
                // round up to the nearest centisecond
                ts += 1;
                cs = 0;
            }

            let mut s = ts;

            if ts >= 86400 {
                write!(out, "{}d", s / 86400)?;
                s %= 86400;
            }

            if ts >= 3600 {
                write!(out, "{}h", s / 3600)?;
                s %= 3600;
            }

            if ts >= 60 {
                write!(out, "{}m", s / 60)?;
                s %= 60
            }

            write!(out, "{}.{:02}s", s, cs)?;
        } else if ns > 100_000 {
            write!(out, "{:.1}ms", f64::from(ns) / 1_000_000_f64)?;
        } else if ns > 100 {
            write!(out, "{:.1}μs", f64::from(ns) / 1_000_f64)?;
        }
        Ok(())
    }

    fn write_sortable<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        let ts = self.as_secs();
        let us = self.subsec_micros();

        write!(
            out,
            "{:02}:{:02}:{:02}.{:06}",
            ts / 3600,
            (ts % 3600) / 60,
            ts % 60,
            us
        )
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
    /// Create a new `RtssWriter`, with a given start time, `Duration` formatter,
    /// and delimiter separating the prefix and content.
    ///
    /// ```
    /// use std::io::{self, Write};
    /// use std::time::{Duration, Instant};
    ///
    /// extern crate rtss;
    /// use rtss::{RtssWriter, DurationExt};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut writer = RtssWriter::new(io::stdout(), Duration::human_string, '|', &Instant::now());
    ///     writer.write(b"Hello!\n")?;
    ///     writer.write(b"World!\n")?;
    ///     Ok(())
    /// }
    ///
    /// // Expected output:
    /// //   0.2μs    0.2μs | Hello!
    /// //  84.7μs   84.6μs | World!
    /// ```
    pub fn new(inner: W, formatter: DurationFormatter, separator: char, now: Instant) -> Self {
        Self {
            inner,
            formatter,
            separator,
            start: now,
            last: now,
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
        let pfx_rest = format!(
            "{:>8} {:>8} {} ",
            start_duration,
            (self.formatter)(&Duration::new(0, 0)),
            self.separator
        );

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
                self.inner.write_all(&buf[pos..=pos + newline])?;
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
    start: Instant,
) -> io::Result<u64> {
    let output = io::BufWriter::new(output);
    let mut output = RtssWriter::new(output, formatter, separator, start);

    io::copy(&mut input, &mut output)
}
