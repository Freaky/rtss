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

The core of `rtss` — an `io::Write` implementation with timestamped output, a function
to copy one IO to another using it, and one to pretty-print `Durations` — is exposed
as a library for use in other programs.  Its interface should be considered unstable
until version 1.


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
