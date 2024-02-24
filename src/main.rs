use std::ffi::CString;

use clap::Parser;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult, Pid};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

struct Debugger {
    pid: Pid,
}

enum Command {
    Continue,
    Break,
    Unknown,
}

impl Debugger {
    pub fn new(pid: Pid) -> Debugger {
        Debugger { pid }
    }

    pub fn run(&self) {
        // wait for process to start. we get a signal because of the ptrace.
        // once we get that, we can proceed
        let _ = waitpid(self.pid, None);

        let mut rl = DefaultEditor::new().unwrap();
        loop {
            let readline = rl.readline("bpkt >> ");
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

    pub fn handle_input(&self, line: String) {
        let mut args = line.split(" ");

        let cmd = Command::from(args.next().unwrap());
        match cmd {
            Command::Continue => {
                let _ = ptrace::cont(self.pid, None);
                // wait until signaled
                let _ = waitpid(self.pid, None);
            }
            Command::Break => {
                let addr = args.next().unwrap();
            }
            Command::Unknown => println!("Unknown command"),
        }
    }
}

impl From<&str> for Command {
    fn from(cmd: &str) -> Self {
        match cmd {
            "c" | "cont" | "continue" => Command::Continue,
            "b" | "br" | "break" => Command::Break,
            _ => Command::Unknown,
        }
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

            if let Err(e) = execvp(&cmd[0], &cmd) {
                println!("failed to call program. error: {e}");
                return;
            }
        }
        Ok(ForkResult::Parent { child }) => {
            println!("start debugging proces for pid {child}");
            let dbg = Debugger::new(child);
            dbg.run();
        }
    }
}
