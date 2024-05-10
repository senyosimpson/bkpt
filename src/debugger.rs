use std::collections::HashMap;

use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use nom::bytes::complete::take_until;
use nom::character::complete::{digit1, space1};
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::sequence::pair;
use nom::{Err, IResult};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::breakpoint::{Breakpoint, Location};
use crate::register::{Register, RegisterSelector};

pub struct Debugger {
    pub pid: Pid,
    pub breakpoints: HashMap<Location, Breakpoint>,
}

enum Command {
    Continue,
    Break,
    Register,
    Unknown,
}

enum RegisterOp {
    Read { reg: Register },
    Write { reg: Register, value: isize },
    Unknown,
}

enum BreakpointOp {
    List,
    Set(Location),
    Unset(u8),
    Unknown,
}

impl Debugger {
    pub fn new(pid: Pid) -> Debugger {
        Debugger {
            pid,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        // wait for process to start. we get a signal because of the ptrace.
        // once we get that, we can proceed
        let _ = waitpid(self.pid, None);

        let mut rl = DefaultEditor::new().unwrap();
        loop {
            let readline = rl.readline(">>> ");
            match readline {
                Ok(line) => self.handle_input(line),
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(e) => println!("error: {:?}", e),
            }
        }
    }

    pub fn handle_input(&mut self, line: String) {
        let (args, cmd) = parse_cmd(&line).unwrap();

        match cmd {
            Command::Continue => {
                let _ = ptrace::cont(self.pid, None);
                // wait until signaled
                let _ = waitpid(self.pid, None);
            }
            Command::Break => {
                let (_, op) = parse_bkpt_cmd(args).unwrap();
                match op {
                    BreakpointOp::List => println!("List breakpoints"),
                    BreakpointOp::Set(_) => println!("Set breakpoint"),
                    BreakpointOp::Unset(num) => println!("Unset breakpoint: {num}"),
                    BreakpointOp::Unknown => println!("Unknown breakpoint command"),
                }
                // let loc = {
                //     let a = args.next().unwrap().strip_prefix("0x").unwrap();
                //     let addr = isize::from_str_radix(a, 16).unwrap();
                //     Location::Address(addr)
                // };

                // self.set_breakpoint(loc);
            }
            Command::Register => {
                let (_, op) = parse_reg_cmd(args).unwrap();
                match op {
                    RegisterOp::Read { reg } => {
                        let value = reg.read(self.pid);
                        println!("{value:0x}");
                    }
                    RegisterOp::Write { reg, value } => reg.write(self.pid, value as u64),
                    RegisterOp::Unknown => {
                        println!("Unknown register command")
                    }
                }
            }
            Command::Unknown => println!("Unknown command"),
        }
    }

    fn set_breakpoint(&mut self, addr: Location) {
        let mut bp = Breakpoint::new(self.pid, addr.clone());
        bp.enable();
        self.breakpoints.insert(addr.clone(), bp);
        println!("Breakpoint set at {:#?}", addr);
    }
}

// ===== RegisterOp =====

impl RegisterOp {
    fn new(op: &str, reg: &str, write: Option<isize>) -> RegisterOp {
        let reg = Register::from_selector(RegisterSelector::Name(reg));
        match op {
            "r" | "read" => RegisterOp::Read { reg },
            "w" | "write" => RegisterOp::Write {
                reg,
                // TODO: Fix unwrap
                value: write.unwrap(),
            },
            _ => RegisterOp::Unknown,
        }
    }
}

// ===== BreakpointOp =====

impl BreakpointOp {
    fn new(op: &str, bkpt_num: Option<u8>, addr: Option<Location>) -> Self {
        match op {
            "ls" | "list" => BreakpointOp::List,
            "set" => BreakpointOp::Set(addr.unwrap()),
            "unset" => BreakpointOp::Unset(bkpt_num.unwrap()),
            _ => BreakpointOp::Unknown,
        }
    }
}

// ===== Command =====

impl From<&str> for Command {
    fn from(cmd: &str) -> Self {
        match cmd {
            "c" | "cont" | "continue" => Command::Continue,
            "b" | "br" | "break" | "bkpt" => Command::Break,
            "r" | "reg" | "register" => Command::Register,
            _ => Command::Unknown,
        }
    }
}

fn parse_cmd(input: &str) -> IResult<&str, Command> {
    let (rem, cmd) = until_space_or_eof(input)?;
    let cmd = Command::from(cmd);

    Ok((rem, cmd))
}

fn parse_reg_cmd(input: &str) -> IResult<&str, RegisterOp> {
    let (rem, op) = take_space_then_until_space_or_eof(input)?;

    // now we have to parse the register. it can be in the format of the register name or a dwarf no.
    // TODO: Implement parsing a dwarf no
    let (rem, reg) = take_space_then_until_space_or_eof(rem)?;

    // If we have a write command, rem will have a value
    let mut value = None;
    if !rem.is_empty() {
        let (_, (_, v)) = pair(space1, parse_number)(rem)?;
        value = Some(v);
    }

    let op = RegisterOp::new(op, reg, value);
    Ok(("", op))
}

fn parse_bkpt_cmd(input: &str) -> IResult<&str, BreakpointOp> {
    let (rem, op) = take_space_then_until_space_or_eof(input)?;
    let (_, addr) = take_space_then_until_space_or_eof(rem)?;

    let (_, num) = take_space_then_until_space_or_eof(rem)?;
    let (_, num) = parse_number(num)?;

    let op = BreakpointOp::new(op);

    Ok(("", op))
}

fn take_space_then_until_space_or_eof(input: &str) -> IResult<&str, &str> {
    let (rem, (_, op)) = pair(space1, until_space_or_eof)(input)?;
    Ok((rem, op))
}

fn until_space_or_eof(input: &str) -> IResult<&str, &str> {
    match until_space(input) {
        Ok(res) => Ok(res),
        Err(Err::Error(e)) if e.code == ErrorKind::TakeUntil => Ok(("", e.input)),
        Err(e) => Err(e),
    }
}

fn until_space(input: &str) -> IResult<&str, &str> {
    take_until(" ")(input)
}

fn parse_number(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>())(input)
}
