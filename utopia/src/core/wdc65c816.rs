use std::fmt;
use tracing::debug;

mod instruction;

pub type Interrupt = u32;

pub const INT_RESET: Interrupt = 0x0000_0001;
pub const INT_NMI: Interrupt = 0x0000_0002;

pub const EMULATION_STACK_PAGE: u16 = 0x0100;

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
enum IrqDisable {
    Clear = 0xffff_ffff,
    Set = INT_RESET | INT_NMI,
}

pub trait Bus: fmt::Display {
    fn read(&mut self, address: u32) -> u8;
}

pub struct Flags {
    n: u8,
    v: u8,
    d: bool,
    i: IrqDisable,
    z: u16,
    c: bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    //Native11 = 0,
    //Native10 = 1,
    //Native01 = 2,
    //Native00 = 3,
    Emulation = 4,
}

pub struct Core<T: Bus> {
    a: u16,
    x: u16,
    y: u16,
    d: u16,
    s: u16,
    pc: u32,
    dbr: u32,
    flags: Flags,
    interrupt: Interrupt,
    mode: Mode,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            d: 0,
            s: EMULATION_STACK_PAGE,
            pc: 0,
            dbr: 0,
            flags: Flags {
                n: 0,
                v: 0,
                d: false,
                i: IrqDisable::Clear,
                z: 0xffff,
                c: false,
            },
            interrupt: INT_RESET,
            mode: Mode::Emulation,
            bus,
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;

        if self.interrupt != 0 {
            self.read(self.pc);

            if (self.interrupt & INT_RESET) != 0 {
                instr::reset(self);
            } else {
                panic!("Interrupt not yet implemented");
            }

            self.interrupt = 0;
            return;
        }

        panic!("Opcode dispatch not yet implemented");
    }

    fn read(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:06X} => {:02X}", address, value);
        value
    }
}

impl<T: Bus> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A={:04X} X={:04X} Y={:04X} D={:04X} S={:04X} PC={:06X} DBR={:02X} P={}{}{}{}{}{}{}{}{}",
            self.a,
            self.x,
            self.y,
            self.d,
            self.s,
            self.pc,
            self.dbr >> 16,
            if (self.flags.n & 0x80) != 0 { 'N' } else { '-' },
            if (self.flags.v & 0x80) != 0 { 'V' } else { '-' },
            if (self.mode as u8 & 0x02) == 0 { 'M' } else { '-' },
            if (self.mode as u8 & 0x01) == 0 { 'X' } else { '-' },
            if self.flags.d { 'D' } else { '-' },
            if self.flags.i == IrqDisable::Set { 'I' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            if (self.mode as u8 & 0x04) != 0 { 'E' } else { '-' },
        )
    }
}
