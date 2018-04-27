# rtss -- timestamp standard input

`rtss` reads lines from stdin, and writes them out to stdout prepended with the
total elapsed time and time since the previous line.  Like so:

```
-% cargo build --release 2>&1 | rtss
   260ms    260ms |    Compiling rtss v0.2.0 (file:///home/freaky/code/rtss)
  2.662s   2.401s |     Finished release [optimized] target(s) in 2.65 secs
Elapsed: 2.663s
```

It can also spawn its own child commands.  stdin is redirected to the child,
stdout and stderr are piped into rtss' own stdout and stderr, and the exit code
of the child is the exit code of rtss:

```
-% rtss sh -c "echo foo; sleep 1; echo bar >&2; sleep 1; echo baz"
     1ms          | foo
  1.056s   1.055s | bar
  2.109s   2.107s | baz
Exit: 0, Elapsed: 2.111s
-% rtss sh -c "echo foo; sleep 1; echo bar >&2; sleep 1; echo baz" 2>/dev/null
     1ms          | foo
  2.023s   2.021s | baz
Exit: 0, Elapsed: 2.025s
```

## Alternatives

`rtss` was inspired by Kevin Burke's [`tss`](https://github.com/kevinburke/tss).

Both are basically trendier versions of `ts` from [moreutils](https://joeyh.name/code/moreutils/).
