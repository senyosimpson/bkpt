use std::ffi::CString;

use clap::Parser;
use nix::sys::personality;
use nix::sys::ptrace;
use nix::unistd::{execvp, fork, ForkResult};

mod debugger;
use debugger::Debugger;

mod breakpoint;

mod register;

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
