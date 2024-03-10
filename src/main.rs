use std::collections::HashMap;
use std::ffi::{c_void, CString};

use clap::Parser;
use nix::sys::personality;
use nix::sys::ptrace::{self, AddressType};
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult, Pid};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

struct Debugger {
    pid: Pid,
    breakpoints: HashMap<Location, Breakpoint>,
}

struct Breakpoint {
    pid: Pid,
    addr: Location,
    enabled: bool,
    old_instruction: isize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Location {
    Address(isize),
    // TODO: Support these options
    Function(String),
    Line(u64),
}

enum Command {
    Continue,
    Break,
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
        let mut args = line.split(" ");

        let cmd = Command::from(args.next().unwrap());
        match cmd {
            Command::Continue => {
                let _ = ptrace::cont(self.pid, None);
                // wait until signaled
                let _ = waitpid(self.pid, None);
            }
            Command::Break => {
                let loc = {
                    let a = args.next().unwrap().strip_prefix("0x").unwrap();
                    let addr = isize::from_str_radix(a, 16).unwrap();
                    Location::Address(addr)
                };

                self.set_breakpoint(loc);
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

impl From<&str> for Command {
    fn from(cmd: &str) -> Self {
        match cmd {
            "c" | "cont" | "continue" => Command::Continue,
            "b" | "br" | "break" | "bkpt" => Command::Break,
            _ => Command::Unknown,
        }
    }
}

impl Breakpoint {
    const BKPT_OPCODE: isize = 0xcc;
    const OPCODE_BITMASK: isize = 0xff;

    pub fn new(pid: Pid, addr: Location) -> Breakpoint {
        Breakpoint {
            pid,
            addr,
            enabled: false,
            old_instruction: 0,
        }
    }

    pub fn enable(&mut self) {
        let ptr = match self.addr {
            Location::Address(addr) => {
                println!("Got address {:08x}", addr);
                addr as AddressType
            }
            Location::Function(_) => todo!(),
            Location::Line(_) => todo!(),
        };

        let data = ptrace::read(self.pid, ptr).unwrap() as isize;

        let old_int = data & Self::OPCODE_BITMASK;
        let bkpt = (data & !Self::OPCODE_BITMASK) | Self::BKPT_OPCODE;

        unsafe {
            // TODO: Sanity check
            ptrace::write(self.pid, ptr, &bkpt as *const _ as *mut c_void).unwrap();
        }

        self.old_instruction = old_int;
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let ptr = &self.addr as *const _ as AddressType;
        let data = ptrace::read(self.pid, ptr).unwrap() as isize;

        let prev_data = (data & !Self::OPCODE_BITMASK) | self.old_instruction;
        unsafe {
            ptrace::write(self.pid, ptr, &prev_data as *const _ as *mut c_void).unwrap();
        }

        self.old_instruction = 0;
        self.enabled = false;
    }
}

#[derive(Debug, Parser)]
struct Args {
    /// Path to the exectuable to debug
    command: String,
    /// Arguments to the executable
    argv: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    let cmd = args.command;
    let argv = args.argv;

    match unsafe { fork() } {
        Err(_) => return,
        Ok(ForkResult::Child) => {
            // set this process to be traced
            if let Err(e) = ptrace::traceme() {
                println!("traceme call failed. error {e}");
            }

            let mut cmd = vec![CString::new(cmd).expect("CString::new failed")];
            if let Some(argv) = argv {
                let mut argv: Vec<CString> = argv
                    .into_iter()
                    .map(|arg| CString::new(arg).unwrap())
                    .collect();

                cmd.append(&mut argv);
            }

            // Switch off address space layout randomization
            let pers = personality::get().unwrap();
            personality::set(pers | personality::Persona::ADDR_NO_RANDOMIZE).unwrap();

            if let Err(e) = execvp(&cmd[0], &cmd) {
                println!("failed to call program. error: {e}");
                return;
            }
        }
        Ok(ForkResult::Parent { child }) => {
            println!("start debugging proces for pid {child}");
            let mut dbg = Debugger::new(child);
            dbg.run();
        }
    }
}
