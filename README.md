# bkpt

A debugger built to learn about debuggers

## Usage

Starting the debugger

```
bkpt <executable>
```

## Breakpoints

Breakpoints are set with

```
b set <location> 
```

where `location` is either:

  * an address in hexidecimal form
  * a function name
  * a line number

> `b` is aliased to `br`, `break`, `bkpt`

They are unset with

```
b unset <bkpt number>
```

The breakpoint number can be found by listing all breakpoints.

List all breakpoints with

```
b list
```

> `list` is aliased to `ls`

## Resources

- Series on [writing a debugger from scratch in Rust](https://www.timdbg.com/posts/writing-a-debugger-from-scratch-part-1/)
- Series on [writing a linux debugger](https://blog.tartanllama.xyz/writing-a-linux-debugger-setup/)
