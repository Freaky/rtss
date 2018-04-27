# rtss -- timestamp standard input

`rtss` reads lines from stdin, and writes them out to stdout prepended with the
total elapsed time and time since the previous line.  Like so:

```
-% cargo build --release 2>&1 | rtss
   262ms    262ms |    Compiling rtss v0.1.0 (file:///home/freaky/code/rtss)
  1.608s   1.345s |     Finished release [optimized] target(s) in 1.60 secs
```

It's a Rust clone of [`tss`](https://github.com/kevinburke/tss).
