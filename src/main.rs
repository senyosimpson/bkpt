use std::collections::HashMap;
use std::ffi::{c_void, CString};

use clap::Parser;
use nix::libc::user_regs_struct;
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

// Found here: /usr/include/x86_64-linux-gnu/sys
enum Registers {
    // General purpose registers
    /// Accumulator register
    Rax,
    /// Base register
    Rbx,
    /// Counter register
    Rcx,
    /// Data regist
    Rdx,
    /// Destination index register
    Rdi,
    /// Source index register
    Rsi,
    /// Stack base pointer register
    Rbp,
    /// Stack pointer register
    Rsp,
    // x86-64 additional registers
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    // Segment registers
    /// Stack segment (pointer to the stack)
    Ss,
    /// Code segment (pointer to the code)
    Cs,
    /// Data segment (pointer to the data)
    Ds,
    /// Extra segment (pointer to extra data)
    Es,
    /// F Segment (pointer to extra data, after Ds)
    Fs,
    /// G segment (pointer to extra data, after Fs)
    Gs,

    // Model-specific registers (MSR)
    /// Base address for F segment
    FsBase,
    /// Base address for G segment
    GsBase,

    /// This is a register needed due to some Linux history
    OrigRax,

    /// ?
    Rip,

    /// FLAGS register
    RFlags,
}

struct Register {
    reg: Registers,
    descriptor: RegisterDescriptor,
}

struct RegisterDescriptor {
    dwarf_no: i64,
    name: String,
}

impl Register {
    fn read(&self, pid: Pid) -> u64 {
        let regs = ptrace::getregs(pid).unwrap();
        match self.reg {
            Registers::Rax => regs.rax,
            Registers::Rbx => regs.rbx,
            Registers::Rcx => regs.rcx,
            Registers::Rdx => regs.rdx,
            Registers::Rdi => regs.rdi,
            Registers::Rsi => regs.rsi,
            Registers::Rbp => regs.rbp,
            Registers::Rsp => regs.rsp,
            Registers::R8 => regs.r8,
            Registers::R9 => regs.r9,
            Registers::R10 => regs.r10,
            Registers::R11 => regs.r11,
            Registers::R12 => regs.r12,
            Registers::R13 => regs.r13,
            Registers::R14 => regs.r14,
            Registers::R15 => regs.r15,
            Registers::Ss => regs.ss,
            Registers::Cs => regs.cs,
            Registers::Ds => regs.ds,
            Registers::Es => regs.es,
            Registers::Fs => regs.fs,
            Registers::Gs => regs.gs,
            Registers::FsBase => regs.fs_base,
            Registers::GsBase => regs.gs_base,
            Registers::OrigRax => regs.orig_rax,
            Registers::Rip => regs.rip,
            Registers::RFlags => regs.eflags,
        }
    }

    fn write(&self, pid: Pid, value: u64) {
        let mut regs = ptrace::getregs(pid).unwrap();
        match self.reg {
            Registers::Rax => regs.rax = value,
            Registers::Rbx => regs.rbx = value,
            Registers::Rcx => regs.rcx = value,
            Registers::Rdx => regs.rdx = value,
            Registers::Rdi => regs.rdi = value,
            Registers::Rsi => regs.rsi = value,
            Registers::Rbp => regs.rbp = value,
            Registers::Rsp => regs.rsp = value,
            Registers::R8 => regs.r8 = value,
            Registers::R9 => regs.r9 = value,
            Registers::R10 => regs.r10 = value,
            Registers::R11 => regs.r11 = value,
            Registers::R12 => regs.r12 = value,
            Registers::R13 => regs.r13 = value,
            Registers::R14 => regs.r14 = value,
            Registers::R15 => regs.r15 = value,
            Registers::Ss => regs.ss = value,
            Registers::Cs => regs.cs = value,
            Registers::Ds => regs.ds = value,
            Registers::Es => regs.es = value,
            Registers::Fs => regs.fs = value,
            Registers::Gs => regs.gs = value,
            Registers::FsBase => regs.fs_base = value,
            Registers::GsBase => regs.gs_base = value,
            Registers::OrigRax => regs.orig_rax = value,
            Registers::Rip => regs.rip = value,
            Registers::RFlags => regs.eflags = value,
        };
        ptrace::setregs(pid, regs).unwrap();
    }

    fn read_from_dwarf(&self) {}

    fn read_from_name(&self) {}

    fn dump(&self) {}

    fn descriptor(&self) -> RegisterDescriptor {
        match self.reg {
            Registers::Rax => RegisterDescriptor {
                dwarf_no: 0,
                name: "rax".into(),
            },
            Registers::Rbx => RegisterDescriptor {
                dwarf_no: 3,
                name: "rbx".into(),
            },
            Registers::Rcx => RegisterDescriptor {
                dwarf_no: 2,
                name: "rcx".into(),
            },
            Registers::Rdx => RegisterDescriptor {
                dwarf_no: 1,
                name: "rdx".into(),
            },
            Registers::Rdi => RegisterDescriptor {
                dwarf_no: 5,
                name: "rdi".into(),
            },
            Registers::Rsi => RegisterDescriptor {
                dwarf_no: 4,
                name: "rsi".into(),
            },
            Registers::Rbp => RegisterDescriptor {
                dwarf_no: 6,
                name: "rbp".into(),
            },
            Registers::Rsp => RegisterDescriptor {
                dwarf_no: 7,
                name: "rsp".into(),
            },
            Registers::R8 => RegisterDescriptor {
                dwarf_no: 8,
                name: "r8".into(),
            },
            Registers::R9 => RegisterDescriptor {
                dwarf_no: 9,
                name: "r9".into(),
            },
            Registers::R10 => RegisterDescriptor {
                dwarf_no: 10,
                name: "r10".into(),
            },
            Registers::R11 => RegisterDescriptor {
                dwarf_no: 11,
                name: "r11".into(),
            },
            Registers::R12 => RegisterDescriptor {
                dwarf_no: 12,
                name: "r12".into(),
            },
            Registers::R13 => RegisterDescriptor {
                dwarf_no: 13,
                name: "r13".into(),
            },
            Registers::R14 => RegisterDescriptor {
                dwarf_no: 14,
                name: "r14".into(),
            },
            Registers::R15 => RegisterDescriptor {
                dwarf_no: 15,
                name: "r15".into(),
            },
            Registers::Ss => RegisterDescriptor {
                dwarf_no: 52,
                name: "ss".into(),
            },
            Registers::Cs => RegisterDescriptor {
                dwarf_no: 51,
                name: "cs".into(),
            },
            Registers::Ds => RegisterDescriptor {
                dwarf_no: 53,
                name: "ds".into(),
            },
            Registers::Es => RegisterDescriptor {
                dwarf_no: 50,
                name: "es".into(),
            },
            Registers::Fs => RegisterDescriptor {
                dwarf_no: 54,
                name: "fs".into(),
            },
            Registers::Gs => RegisterDescriptor {
                dwarf_no: 55,
                name: "gs".into(),
            },
            Registers::FsBase => RegisterDescriptor {
                dwarf_no: 58,
                name: "fs_base".into(),
            },
            Registers::GsBase => RegisterDescriptor {
                dwarf_no: 59,
                name: "gs_base".into(),
            },
            Registers::OrigRax => RegisterDescriptor {
                dwarf_no: -1,
                name: "orig_rax".into(),
            },
            Registers::Rip => RegisterDescriptor {
                dwarf_no: -1,
                name: "rip".into(),
            },
            Registers::RFlags => RegisterDescriptor {
                dwarf_no: 49,
                name: "eflags".into(),
            },
        }
    }
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
