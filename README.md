# rtss — Relative TimeStamps for Stuff

`rtss` annotates its output with relative durations between consecutive lines and
since program start.

It can be used as a filter in a pipeline:

```
-% cargo build --release 2>&1 | rtss
 274.1ms  274.1ms |    Compiling libc v0.2.40
   1.50s    1.22s |    Compiling memchr v2.0.1
   2.28s  780.8ms |    Compiling rtss v0.5.0 (file:///home/freaky/code/rtss)
   5.18s    2.90s |     Finished release [optimized] target(s) in 5.17 secs
   5.18s    exit code: 0
```

It can also directly run commands, annotating both stdout and stderr with durations.
stdin is passed through to the child process, and its exit code will become `rtss`'
own exit code:

```
-% rtss sh -c "echo foo; echo bar; sleep 1; echo moo >&2; sleep 1; echo baz; exit 64"
   1.7ms    1.7ms | foo
   1.7ms          | bar
   1.00s    1.00s # moo
   2.03s    2.03s | baz
   2.03s    exit code: 64
zsh: exit 64    rtss sh -c

-% rtss sh -c "echo foo; echo bar; sleep 1; echo moo >&2; sleep 1; echo baz; exit 64" 2>/dev/null
   1.9ms    1.9ms | foo
   1.9ms          | bar
   2.05s    2.04s | baz
   2.05s    exit code: 64
zsh: exit 64    rtss sh -c  2> /dev/null
```

Blank durations indicate lines were read in a single `read()`.  Line durations are per-descriptor,
so stderr and stdout have their own distinct durations.

Output suitable for piping to `sort -k2` can be requested with `-s` / `--sortable`:

```
-% rtss --sortable sh -c "echo foo; echo bar; sleep 1; echo moo >&2; sleep 1; echo baz; exit 64"
00:00:00.001652 00:00:00.001652 | foo
00:00:00.001652 00:00:00.000000 | bar
00:00:01.007287 00:00:01.007287 # moo
00:00:02.071962 00:00:02.070309 | baz
00:00:02.072185    exit code: 64
```

### PTY mode

For programs that buffer their output or otherwise alter their behaviour when connected
to pipes, the `--pty` (aka `--tty`) option will, on supported platforms, run the command
under a pseudo-terminal.

```
-% rtss zpool status 5
  10.01s   10.01s |   pool: rpool
  10.01s          |  state: ONLINE
  10.01s          |   scan: scrub repaired 0 in 1h7m with 0 errors on Wed May  2 04:00:38 2018

-% rtss --pty zpool status 5
   4.2ms    4.2ms |   pool: rpool
   4.2ms          |  state: ONLINE
   4.5ms    0.3ms |   scan: scrub repaired 0 in 1h7m with 0 errors on Wed May  2 04:00:38 2018
```

## API

The backend of `rtss` is provided as a library for use in other programs.  This includes:

 * `RtssWriter` — an `io::Write` wrapper and implementation, forwarding `write()` calls and annotating newlines.
 * `DurationExt` — extends `Duration` with `write_human()`, `write_sortable()`, `human_string()` and `sortable_string()` methods.
 * `line_timing_copy()` — wraps an `io::Write` in `RtssWriter<BufWriter<W>>` and calls `io::copy()` on it and the provided `io::Read`.

```
use std::io::{self, Write};
use std::time::{Duration, Instant};

extern crate rtss;
use rtss::{RtssWriter, DurationExt};

fn main() -> io::Result<()> {
    let mut writer = RtssWriter::new(io::stdout(), Duration::human_string, '|', Instant::now());
    writer.write(b"Hello!\n")?;
    writer.write(b"World!\n")?;
    Ok(())
}
```

Output:

```
   0.2μs    0.2μs | Hello!
  84.7μs   84.6μs | World!
```

## Installation

If you have Cargo installed you can install the latest release with:

```
cargo install rtss
```

You can also install the latest bleeding-edge version using:

```
cargo install --git https://github.com/Freaky/rtss.git
```

Alternatively you can clone and build manually without installing:

```
git clone https://github.com/Freaky/rtss.git &&
cd rtss &&
cargo build --release &&
target/release/rtss echo It works
```

## Alternatives

`rtss` was inspired by Kevin Burke's [`tss`](https://github.com/kevinburke/tss).

Both are basically trendier versions of `ts` from [moreutils](https://joeyh.name/code/moreutils/).
