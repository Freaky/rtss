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

Blank durations indicate lines were read in a single `read()`.


### PTY mode

For programs that buffer their output or otherwise alter their behaviour when connected
to pipes, programs can be executed with a pseudo-terminal on supporting platforms:

```
-$ rtss tcpdump
  26.3ms   26.3ms # tcpdump: verbose output suppressed, use -v or -vv for full protocol decode
  26.3ms   68.5μs # listening on igb0, link-type EN10MB (Ethernet), capture size 262144 bytes
   3.17s    3.17s | 00:51:15.288326 ...
   3.17s          | 00:51:15.288426 ...

-$ rtss --pty tcpdump
  30.1ms   30.1ms # tcpdump: verbose output suppressed, use -v or -vv for full protocol decode
  30.1ms   59.1μs # listening on igb0, link-type EN10MB (Ethernet), capture size 262144 bytes
  32.2ms   32.2ms | 00:52:32.893227 ...
  32.2ms   43.9μs | 00:52:32.893329 ...
```


## API

The core of `rtss` — an `io::Write` implementation with timestamped output, a function
to copy one IO to another using it, and one to pretty-print `Durations` — is exposed
as a library for use in other programs.  Its interface should be considered unstable
until version 1.

```
use std::io::{self, Write};
use std::time::Instant;

extern crate rtss;
use rtss::RtssWriter;

fn main() {
    let mut writer = RtssWriter::new(io::stdout(), '|', &Instant::now());
    writer.write(b"Hello!\n").unwrap();
    writer.write(b"World!\n").unwrap();
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
