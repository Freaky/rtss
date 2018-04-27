# rtss -- timestamp standard input

`rtss` reads lines from stdin, and writes them out to stdout prepended with the
total elapsed time and time since the previous line.  Like so:

```
-% cargo build --release 2>&1 | rtss
   260ms    260ms |    Compiling rtss v0.2.0 (file:///home/freaky/code/rtss)
  2.662s   2.401s |     Finished release [optimized] target(s) in 2.65 secs
Elapsed: 2.663s
```

This also works, though doesn't yet capture stderr:

```
-% rtss sh -c "sleep 1; echo foo; sleep 1; echo bar"
  1.008s   1.008s | foo
  2.011s   1.002s | bar
Exit: 0, Elapsed: 2.011s
```

It's a Rust clone of [`tss`](https://github.com/kevinburke/tss).
