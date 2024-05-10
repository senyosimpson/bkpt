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
    Read,
    Write(isize),
    Unknown,
}

struct RegisterCmd {
    op: RegisterOp,
    register: Register,
}

enum BreakpointCmd {
    List,
    Set(Location),
    Unset(u8),
    Unknown,
}

enum BreakpointOp {
    List,
    Set,
    Unset,
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
                let (_, breakpoint_cmd) = parse_bkpt_cmd(args).unwrap();
                match breakpoint_cmd {
                    BreakpointCmd::List => println!("List breakpoints"),
                    BreakpointCmd::Set(_) => println!("Set breakpoint"),
                    BreakpointCmd::Unset(num) => println!("Unset breakpoint: {num}"),
                    BreakpointCmd::Unknown => println!("Unknown breakpoint command"),
                }
                // let loc = {
                //     let a = args.next().unwrap().strip_prefix("0x").unwrap();
                //     let addr = isize::from_str_radix(a, 16).unwrap();
                //     Location::Address(addr)
                // };

                // self.set_breakpoint(loc);
            }
            Command::Register => {
                let (_, register_cmd) = parse_reg_cmd(args).unwrap();
                match register_cmd.op {
                    RegisterOp::Read => {
                        let value = register_cmd.register.read(self.pid);
                        println!("{value:0x}");
                    }
                    RegisterOp::Write(value) => register_cmd.register.write(self.pid, value as u64),
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

impl From<(&str, Option<isize>)> for RegisterOp {
    fn from(op: (&str, Option<isize>)) -> Self {
        match op.0 {
            "r" | "read" => RegisterOp::Read,
            "w" | "Write" => RegisterOp::Write(op.1.unwrap()),
            _ => RegisterOp::Unknown,
        }
    }
}

// ===== BreakpointOp =====

impl From<&str> for BreakpointOp {
    fn from(op: &str) -> Self {
        match op {
            "ls" | "list" => BreakpointOp::List,
            "set" => BreakpointOp::Set,
            "unset" => BreakpointOp::Unset,
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
    let (rem, cmd) = until_whitespace_or_eof(input)?;
    let cmd = Command::from(cmd);

    Ok((rem, cmd))
}

fn parse_reg_cmd(input: &str) -> IResult<&str, RegisterCmd> {
    let (rem, (_, op)) = pair(space1, until_whitespace_or_eof)(input)?;
    // now we have to parse the register. it can be in the format of the register name or a dwarf no.
    // TODO: Implement parsing a dwarf no
    let (rem, (_, reg)) = pair(space1, until_whitespace_or_eof)(rem)?;
    // If we have a write command, rem will have a value
    let mut value = None;
    if !rem.is_empty() {
        let (_, (_, v)) = pair(space1, parse_number)(rem)?;
        value = Some(v);
    }

    let op = match value {
        Some(v) => RegisterOp::from((op, Some(v))),
        None => RegisterOp::from((op, None)),
    };

    let cmd = RegisterCmd {
        op,
        register: Register::from_selector(RegisterSelector::Name(reg)),
    };

    Ok(("", cmd))
}

fn parse_bkpt_cmd(input: &str) -> IResult<&str, BreakpointCmd> {
    let (rem, (_, op)) = pair(space1, until_whitespace_or_eof)(input)?;
    let op = BreakpointOp::from(op);

    match op {
        // Get the location
        BreakpointOp::Set => {
            let (_, (_, addr)) = pair(space1, until_whitespace_or_eof)(rem)?;
            // TODO: Fix address
            Ok(("", BreakpointCmd::Set(Location::Address(0x1234))))
        }
        // Get the breakpoint number
        BreakpointOp::Unset => {
            let (_, (_, num)) = pair(space1, until_whitespace_or_eof)(rem)?;
            let (_, num) = parse_number(num)?;
            Ok(("", BreakpointCmd::Unset(num as u8)))
        }
        BreakpointOp::List => Ok(("", BreakpointCmd::List)),
        BreakpointOp::Unknown => Ok(("", BreakpointCmd::Unknown)),
    }
}

fn until_whitespace_or_eof(input: &str) -> IResult<&str, &str> {
    match until_whitespace(input) {
        Ok(res) => Ok(res),
        Err(Err::Error(e)) if e.code == ErrorKind::TakeUntil => Ok(("", e.input)),
        Err(e) => Err(e),
    }
}

fn until_whitespace(input: &str) -> IResult<&str, &str> {
    take_until(" ")(input)
}

fn parse_number(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>())(input)
}
