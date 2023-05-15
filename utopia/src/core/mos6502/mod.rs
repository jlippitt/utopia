use instruction as instr;
use std::fmt;
use tracing::debug;

mod instruction;

pub const STACK_PAGE: u16 = 0x0100;

pub type Interrupt = u32;

pub const INT_RESET: Interrupt = 0x0000_0001;
pub const INT_NMI: Interrupt = 0x0000_0002;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
enum IrqDisable {
    Clear = 0xffff_ffff,
    Set = INT_RESET | INT_NMI,
}

pub struct Flags {
    n: u8,
    v: u8,
    d: bool,
    i: IrqDisable,
    z: u8,
    c: bool,
}

pub struct Core<T: Bus> {
    bus: T,
    interrupt: Interrupt,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    pc: u16,
    flags: Flags,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            interrupt: INT_RESET,
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            pc: 0,
            flags: Flags {
                n: 0,
                v: 0,
                d: false,
                i: IrqDisable::Clear,
                z: 0xff,
                c: false,
            },
        }
    }

    pub fn step(&mut self) {
        if self.interrupt != 0 {
            if (self.interrupt & INT_RESET) != 0 {
                instr::reset(self);
            } else {
                panic!("Interrupt type not yet supported");
            }

            self.interrupt = 0;
            return;
        }

        panic!("Instructions not yet implemented");
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:04X} => {:02X}", address, value);
        value
    }
}

impl<T: Bus> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A={:02X} X={:02X} Y={:02X} S={:02X} PC={:04X} P={}{}--{}{}{}{}",
            self.a,
            self.x,
            self.y,
            self.s,
            self.pc,
            if (self.flags.n & 0x80) != 0 { 'N' } else { '-' },
            if (self.flags.v & 0x80) != 0 { 'V' } else { '-' },
            if self.flags.d { 'D' } else { '-' },
            if self.flags.i == IrqDisable::Set {
                'I'
            } else {
                '-'
            },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
        )
    }
}
