use nix::{sys::ptrace, unistd::Pid};

pub struct Register {
    kind: RegisterKind,
    descriptor: RegisterDescriptor,
}

struct RegisterDescriptor {
    dwarf_no: i64,
    name: String,
}

// Found here: /usr/include/x86_64-linux-gnu/sys
enum RegisterKind {
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

pub enum RegisterSelector<'a> {
    Dwarf(i64),
    Name(&'a str),
}

impl Register {
    pub fn read(&self, pid: Pid) -> u64 {
        let regs = ptrace::getregs(pid).unwrap();
        match self.kind {
            RegisterKind::Rax => regs.rax,
            RegisterKind::Rbx => regs.rbx,
            RegisterKind::Rcx => regs.rcx,
            RegisterKind::Rdx => regs.rdx,
            RegisterKind::Rdi => regs.rdi,
            RegisterKind::Rsi => regs.rsi,
            RegisterKind::Rbp => regs.rbp,
            RegisterKind::Rsp => regs.rsp,
            RegisterKind::R8 => regs.r8,
            RegisterKind::R9 => regs.r9,
            RegisterKind::R10 => regs.r10,
            RegisterKind::R11 => regs.r11,
            RegisterKind::R12 => regs.r12,
            RegisterKind::R13 => regs.r13,
            RegisterKind::R14 => regs.r14,
            RegisterKind::R15 => regs.r15,
            RegisterKind::Ss => regs.ss,
            RegisterKind::Cs => regs.cs,
            RegisterKind::Ds => regs.ds,
            RegisterKind::Es => regs.es,
            RegisterKind::Fs => regs.fs,
            RegisterKind::Gs => regs.gs,
            RegisterKind::FsBase => regs.fs_base,
            RegisterKind::GsBase => regs.gs_base,
            RegisterKind::OrigRax => regs.orig_rax,
            RegisterKind::Rip => regs.rip,
            RegisterKind::RFlags => regs.eflags,
        }
    }

    pub fn write(&self, pid: Pid, value: u64) {
        let mut regs = ptrace::getregs(pid).unwrap();
        match self.kind {
            RegisterKind::Rax => regs.rax = value,
            RegisterKind::Rbx => regs.rbx = value,
            RegisterKind::Rcx => regs.rcx = value,
            RegisterKind::Rdx => regs.rdx = value,
            RegisterKind::Rdi => regs.rdi = value,
            RegisterKind::Rsi => regs.rsi = value,
            RegisterKind::Rbp => regs.rbp = value,
            RegisterKind::Rsp => regs.rsp = value,
            RegisterKind::R8 => regs.r8 = value,
            RegisterKind::R9 => regs.r9 = value,
            RegisterKind::R10 => regs.r10 = value,
            RegisterKind::R11 => regs.r11 = value,
            RegisterKind::R12 => regs.r12 = value,
            RegisterKind::R13 => regs.r13 = value,
            RegisterKind::R14 => regs.r14 = value,
            RegisterKind::R15 => regs.r15 = value,
            RegisterKind::Ss => regs.ss = value,
            RegisterKind::Cs => regs.cs = value,
            RegisterKind::Ds => regs.ds = value,
            RegisterKind::Es => regs.es = value,
            RegisterKind::Fs => regs.fs = value,
            RegisterKind::Gs => regs.gs = value,
            RegisterKind::FsBase => regs.fs_base = value,
            RegisterKind::GsBase => regs.gs_base = value,
            RegisterKind::OrigRax => regs.orig_rax = value,
            RegisterKind::Rip => regs.rip = value,
            RegisterKind::RFlags => regs.eflags = value,
        };
        ptrace::setregs(pid, regs).unwrap();
    }

    pub fn from_selector(selector: RegisterSelector) -> Register {
        match selector {
            RegisterSelector::Dwarf(-1) | RegisterSelector::Name("orig_rax") => Register {
                kind: RegisterKind::OrigRax,
                descriptor: RegisterDescriptor {
                    dwarf_no: -1,
                    name: "orig_rax".into(),
                },
            },
            RegisterSelector::Dwarf(-1) | RegisterSelector::Name("rip") => Register {
                kind: RegisterKind::Rip,
                descriptor: RegisterDescriptor {
                    dwarf_no: -1,
                    name: "rip".into(),
                },
            },
            RegisterSelector::Dwarf(0) | RegisterSelector::Name("rax") => Register {
                kind: RegisterKind::Rax,
                descriptor: RegisterDescriptor {
                    dwarf_no: 0,
                    name: "rax".into(),
                },
            },
            RegisterSelector::Dwarf(1) | RegisterSelector::Name("rdx") => Register {
                kind: RegisterKind::Rdx,
                descriptor: RegisterDescriptor {
                    dwarf_no: 1,
                    name: "rdx".into(),
                },
            },
            RegisterSelector::Dwarf(2) | RegisterSelector::Name("rcx") => Register {
                kind: RegisterKind::Rcx,
                descriptor: RegisterDescriptor {
                    dwarf_no: 2,
                    name: "rcx".into(),
                },
            },
            RegisterSelector::Dwarf(3) | RegisterSelector::Name("rbx") => Register {
                kind: RegisterKind::Rbx,
                descriptor: RegisterDescriptor {
                    dwarf_no: 3,
                    name: "rbx".into(),
                },
            },
            RegisterSelector::Dwarf(4) | RegisterSelector::Name("rsi") => Register {
                kind: RegisterKind::Rsi,
                descriptor: RegisterDescriptor {
                    dwarf_no: 4,
                    name: "rsi".into(),
                },
            },
            RegisterSelector::Dwarf(5) | RegisterSelector::Name("rdi") => Register {
                kind: RegisterKind::Rdi,
                descriptor: RegisterDescriptor {
                    dwarf_no: 5,
                    name: "rdi".into(),
                },
            },
            RegisterSelector::Dwarf(6) | RegisterSelector::Name("rbp") => Register {
                kind: RegisterKind::Rbp,
                descriptor: RegisterDescriptor {
                    dwarf_no: 6,
                    name: "rbp".into(),
                },
            },
            RegisterSelector::Dwarf(7) | RegisterSelector::Name("rsp") => Register {
                kind: RegisterKind::Rsp,
                descriptor: RegisterDescriptor {
                    dwarf_no: 7,
                    name: "rsp".into(),
                },
            },
            RegisterSelector::Dwarf(8) | RegisterSelector::Name("r8") => Register {
                kind: RegisterKind::R8,
                descriptor: RegisterDescriptor {
                    dwarf_no: 8,
                    name: "r8".into(),
                },
            },
            RegisterSelector::Dwarf(9) | RegisterSelector::Name("r9") => Register {
                kind: RegisterKind::R9,
                descriptor: RegisterDescriptor {
                    dwarf_no: 9,
                    name: "r9".into(),
                },
            },
            RegisterSelector::Dwarf(10) | RegisterSelector::Name("r10") => Register {
                kind: RegisterKind::R10,
                descriptor: RegisterDescriptor {
                    dwarf_no: 10,
                    name: "r10".into(),
                },
            },
            RegisterSelector::Dwarf(11) | RegisterSelector::Name("r11") => Register {
                kind: RegisterKind::R11,
                descriptor: RegisterDescriptor {
                    dwarf_no: 11,
                    name: "r11".into(),
                },
            },
            RegisterSelector::Dwarf(12) | RegisterSelector::Name("r12") => Register {
                kind: RegisterKind::R12,
                descriptor: RegisterDescriptor {
                    dwarf_no: 12,
                    name: "r12".into(),
                },
            },
            RegisterSelector::Dwarf(13) | RegisterSelector::Name("r13") => Register {
                kind: RegisterKind::R13,
                descriptor: RegisterDescriptor {
                    dwarf_no: 13,
                    name: "r13".into(),
                },
            },
            RegisterSelector::Dwarf(14) | RegisterSelector::Name("r14") => Register {
                kind: RegisterKind::R14,
                descriptor: RegisterDescriptor {
                    dwarf_no: 14,
                    name: "r14".into(),
                },
            },
            RegisterSelector::Dwarf(15) | RegisterSelector::Name("r15") => Register {
                kind: RegisterKind::R15,
                descriptor: RegisterDescriptor {
                    dwarf_no: 15,
                    name: "r15".into(),
                },
            },
            RegisterSelector::Dwarf(49) | RegisterSelector::Name("eflags") => Register {
                kind: RegisterKind::RFlags,
                descriptor: RegisterDescriptor {
                    dwarf_no: 49,
                    name: "eflags".into(),
                },
            },
            RegisterSelector::Dwarf(50) | RegisterSelector::Name("es") => Register {
                kind: RegisterKind::Es,
                descriptor: RegisterDescriptor {
                    dwarf_no: 50,
                    name: "es".into(),
                },
            },
            RegisterSelector::Dwarf(51) | RegisterSelector::Name("cs") => Register {
                kind: RegisterKind::Cs,
                descriptor: RegisterDescriptor {
                    dwarf_no: 51,
                    name: "cs".into(),
                },
            },
            RegisterSelector::Dwarf(52) | RegisterSelector::Name("ss") => Register {
                kind: RegisterKind::Ss,
                descriptor: RegisterDescriptor {
                    dwarf_no: 52,
                    name: "ss".into(),
                },
            },
            RegisterSelector::Dwarf(53) | RegisterSelector::Name("ds") => Register {
                kind: RegisterKind::Ds,
                descriptor: RegisterDescriptor {
                    dwarf_no: 53,
                    name: "ds".into(),
                },
            },
            RegisterSelector::Dwarf(54) | RegisterSelector::Name("fs") => Register {
                kind: RegisterKind::Fs,
                descriptor: RegisterDescriptor {
                    dwarf_no: 54,
                    name: "fs".into(),
                },
            },
            RegisterSelector::Dwarf(55) | RegisterSelector::Name("gs") => Register {
                kind: RegisterKind::Gs,
                descriptor: RegisterDescriptor {
                    dwarf_no: 55,
                    name: "gs".into(),
                },
            },
            RegisterSelector::Dwarf(58) | RegisterSelector::Name("fs_base") => Register {
                kind: RegisterKind::FsBase,
                descriptor: RegisterDescriptor {
                    dwarf_no: 58,
                    name: "fs_base".into(),
                },
            },
            RegisterSelector::Dwarf(59) | RegisterSelector::Name("gs_base") => Register {
                kind: RegisterKind::GsBase,
                descriptor: RegisterDescriptor {
                    dwarf_no: 59,
                    name: "gs_base".into(),
                },
            },
            _ => unreachable!(),
        }
    }
}
