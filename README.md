# rtss — Relative TimeStamps for Stuff

`rtss` annotates its output with relative durations between consecutive lines and
since program start.

It can be used as a filter in a pipeline:

```
-% cargo build --release 2>&1 | rtss
 261.7ms  261.6ms |    Compiling rtss v0.2.0 (file:///home/freaky/code/rtss)
   3.02s    2.76s |     Finished release [optimized] target(s) in 3.1 secs
   3.02s    exit code: 0
```

It can also directly run commands, annotating both stdout and stderr with durations.
stdin is passed through to the child process, and its exit code will become `rtss`'
own exit code:

```
-% rtss sh -c "echo foo; echo bar; sleep 1; echo moo >&2; sleep 1; echo baz; exit 64"
   1.7ms    0.8ms | foo
   1.7ms   42.5μs | bar
   1.07s    1.06s # moo
   2.07s    2.07s | baz
   2.07s    exit code: 64
zsh: exit 64    rtss sh -c

-% rtss sh -c "echo foo; echo bar; sleep 1; echo moo >&2; sleep 1; echo baz; exit 64" 2>/dev/null
   1.6ms    1.0ms | foo
   1.6ms   51.1μs | bar
   2.06s    2.06s | baz
   2.06s    exit code: 64
zsh: exit 64    rtss sh -c
```

The core of `rtss`; copying one IO to another with timestamps, and pretty-printing
`Durations`, is exposed as a library for use in other programs.


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
