use crate::util::facade::Value;
use tracing::{debug, warn};

mod arm;
mod condition;
mod operator;
mod thumb;

#[rustfmt::skip]
const REGS: [&'static str; 16] = [
    "R0", "R1", "R2", "R3",
    "R4", "R5", "R6", "R7",
    "R8", "R9", "R10", "R11",
    "R12", "SP", "LR", "PC",
];

const SIZES: [&str; 3] = ["B", "H", ""];

pub trait Bus {
    fn read<T: Value>(&mut self, address: u32) -> T;
    fn write<T: Value>(&mut self, address: u32, value: T);
}

#[repr(u8)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
enum Mode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    #[default]
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

#[derive(Clone, Default)]
struct Cpsr {
    n: bool,
    z: bool,
    c: bool,
    v: bool,
    i: bool,
    f: bool,
    t: bool,
    m: Mode,
    reserved: u32,
}

#[derive(Clone, Default)]
struct Spsr {
    fiq: u32,
    svc: u32,
    abt: u32,
    irq: u32,
    und: u32,
}

#[derive(Clone, Default)]
struct Bank {
    usr: [u32; 7],
    fiq: [u32; 7],
    svc: [u32; 2],
    abt: [u32; 2],
    irq: [u32; 2],
    und: [u32; 2],
}

pub struct Core<T: Bus> {
    bus: T,
    pc: u32,
    regs: [u32; 16],
    cpsr: Cpsr,
    spsr: Spsr,
    bank: Bank,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            pc: 0,
            regs: [0; 16],
            cpsr: Cpsr {
                f: true,
                i: true,
                ..Default::default()
            },
            spsr: Default::default(),
            bank: Default::default(),
        }
    }

    pub fn step(&mut self) {
        if self.cpsr.t {
            thumb::dispatch(self);
        } else {
            arm::dispatch(self);
        }
    }

    fn read_byte(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:02X}", address, value);
        value
    }

    fn read_halfword(&mut self, address: u32) -> u16 {
        assert!((address & 1) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:04X}", address, value);
        value
    }

    fn read_word(&mut self, address: u32) -> u32 {
        assert!((address & 3) == 0);
        let value = self.bus.read(address);
        debug!("  [{:08X}] => {:08X}", address, value);
        value
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        debug!("  [{:08X}] <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        assert!((address & 1) == 0);
        debug!("  [{:08X}] <= {:04X}", address, value);
        self.bus.write(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        assert!((address & 3) == 0);
        debug!("  [{:08X}] <= {:08X}", address, value);
        self.bus.write(address, value);
    }

    fn get(&self, reg: usize) -> u32 {
        if reg == 15 {
            self.pc.wrapping_add(4)
        } else {
            self.regs[reg]
        }
    }

    fn set(&mut self, reg: usize, value: u32) {
        if reg == 15 {
            todo!("PC set");
        }

        self.regs[reg] = value;
        debug!("  {}: {:08X}", REGS[reg], value);
    }

    fn cpsr_to_u32(&self) -> u32 {
        let mut value = self.cpsr.reserved | self.cpsr.m as u32;
        value |= if self.cpsr.n { 0x8000_0000 } else { 0 };
        value |= if self.cpsr.z { 0x4000_0000 } else { 0 };
        value |= if self.cpsr.c { 0x2000_0000 } else { 0 };
        value |= if self.cpsr.v { 0x1000_0000 } else { 0 };
        value |= if self.cpsr.i { 0x80 } else { 0 };
        value |= if self.cpsr.f { 0x40 } else { 0 };
        value |= if self.cpsr.t { 0x20 } else { 0 };
        value
    }

    fn cpsr_from_u32(&mut self, value: u32, control: bool) {
        self.cpsr.n = (value & 0x8000_0000) != 0;
        self.cpsr.z = (value & 0x4000_0000) != 0;
        self.cpsr.c = (value & 0x2000_0000) != 0;
        self.cpsr.v = (value & 0x1000_0000) != 0;

        if control && self.cpsr.m != Mode::User {
            self.cpsr.reserved = value & 0x0fff_ff00;
            self.cpsr.i = (value & 0x80) != 0;
            self.cpsr.f = (value & 0x40) != 0;
            self.cpsr.t = (value & 0x20) != 0;

            self.set_mode(match value & 0x1f {
                0b10000 => Mode::User,
                0b10001 => Mode::Fiq,
                0b10010 => Mode::Irq,
                0b10011 => Mode::Supervisor,
                0b10111 => Mode::Abort,
                0b11011 => Mode::Undefined,
                0b11111 => Mode::System,
                mode => panic!("Invalid mode: {:05b}", mode),
            });
        }

        debug!("  CPSR: {:08X}", self.cpsr_to_u32());
    }

    fn spsr_to_u32(&self) -> u32 {
        match self.cpsr.m {
            Mode::Fiq => self.spsr.fiq,
            Mode::Irq => self.spsr.irq,
            Mode::Supervisor => self.spsr.svc,
            Mode::Abort => self.spsr.abt,
            Mode::Undefined => self.spsr.und,
            mode => {
                warn!("No SPSR defined for mode: {:?}", mode);
                0
            }
        }
    }

    fn spsr_from_u32(&mut self, value: u32, control: bool) {
        let spsr = match self.cpsr.m {
            Mode::Fiq => &mut self.spsr.fiq,
            Mode::Irq => &mut self.spsr.irq,
            Mode::Supervisor => &mut self.spsr.svc,
            Mode::Abort => &mut self.spsr.abt,
            Mode::Undefined => &mut self.spsr.und,
            mode => {
                warn!("No SPSR defined for mode: {:?}", mode);
                return;
            }
        };

        if control {
            *spsr = value;
        } else {
            *spsr = (value & 0xf000_0000) | (*spsr & 0x0fff_ffff);
        }

        debug!("  SPSR: {:08X}", spsr);
    }

    fn set_nz(&mut self, value: u32) {
        self.cpsr.n = (value & 0x8000_0000) != 0;
        self.cpsr.z = value == 0;
    }

    fn add_with_carry<const SET_FLAGS: bool>(&mut self, lhs: u32, rhs: u32, carry: bool) -> u32 {
        let result = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
        let carries = lhs ^ rhs ^ result;
        let overflow = (lhs ^ result) & (rhs ^ result);

        if SET_FLAGS {
            self.set_nz(result);
            self.cpsr.c = ((carries ^ overflow) & 0x8000_0000) != 0;
            self.cpsr.v = (overflow & 0x8000_0000) != 0;
        }

        result
    }

    fn set_mode(&mut self, mode: Mode) {
        if mode == self.cpsr.m {
            return;
        }

        debug!("  Mode: {:?}", mode);
        debug!("  Stored: {:X?}", &self.regs[8..=14]);

        match self.cpsr.m {
            Mode::User | Mode::System => self.bank.usr.copy_from_slice(&self.regs[8..=14]),
            Mode::Fiq => self.bank.fiq.copy_from_slice(&self.regs[8..=14]),
            Mode::Irq => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.irq.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Supervisor => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.svc.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Abort => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.abt.copy_from_slice(&self.regs[13..=14]);
            }
            Mode::Undefined => {
                self.bank.usr[0..=4].copy_from_slice(&self.regs[8..=12]);
                self.bank.und.copy_from_slice(&self.regs[13..=14]);
            }
        }

        self.cpsr.m = mode;

        match self.cpsr.m {
            Mode::User | Mode::System => self.regs[8..=14].copy_from_slice(&self.bank.usr),
            Mode::Fiq => self.regs[8..=14].copy_from_slice(&self.bank.fiq),
            Mode::Irq => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.irq);
            }
            Mode::Supervisor => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.svc);
            }
            Mode::Abort => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.abt);
            }
            Mode::Undefined => {
                self.regs[8..=12].copy_from_slice(&self.bank.usr[0..=4]);
                self.regs[13..=14].copy_from_slice(&self.bank.und);
            }
        }

        debug!("  Loaded: {:X?}", &self.regs[8..=14]);
    }
}
