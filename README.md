# rtss -- timestamp standard input

`rtss` reads lines from stdin, and writes them out to stdout prepended with the
total elapsed time and time since the previous line.  Like so:

```
-% cargo build --release 2>&1 | rtss
 261.7ms  261.6ms |    Compiling rtss v0.2.0 (file:///home/freaky/code/rtss)
   3.02s    2.76s |     Finished release [optimized] target(s) in 3.1 secs
   3.02s    exit code: 0
```

It can also spawn its own child commands.  stdin is redirected to the child,
stdout and stderr are piped into rtss' own stdout and stderr, and the exit code
of the child is the exit code of rtss:

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


## Installation

If you have Cargo installed you can install the latest version using:

```
cargo install --git https://github.com/Freaky/rtss.git
```

A proper release on crates.io will be forthcoming.


## Alternatives

`rtss` was inspired by Kevin Burke's [`tss`](https://github.com/kevinburke/tss).

Both are basically trendier versions of `ts` from [moreutils](https://joeyh.name/code/moreutils/).
