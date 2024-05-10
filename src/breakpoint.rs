use std::ffi::c_void;

use nix::sys::ptrace::{self, AddressType};
use nix::unistd::Pid;

pub struct Breakpoint {
    pid: Pid,
    addr: Location,
    enabled: bool,
    old_instruction: isize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Location {
    Address(isize),
    // TODO: Support these options
    Function(String),
    Line(u64),
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
