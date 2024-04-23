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

## Registers

To read from a register

```
r read <register>
```

To write from a register

```
r write <register>
```

> `r` is aliased to `reg`, `register`

> `read` is aliased to `r`, `write` is alised to `w`

## Program information

Information about the program can be queried using the `info <type>` command

## Resources

- Series on [writing a debugger from scratch in Rust](https://www.timdbg.com/posts/writing-a-debugger-from-scratch-part-1/)
- Series on [writing a linux debugger](https://blog.tartanllama.xyz/writing-a-linux-debugger-setup/)
