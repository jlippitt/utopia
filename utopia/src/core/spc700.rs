use std::fmt;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

pub trait Bus: fmt::Display {
    fn read(&mut self, address: u16) -> u8;
}

pub struct Flags {
    n: u8,
    v: u8,
    p: u16,
    b: bool,
    h: u8,
    i: bool,
    z: u8,
    c: bool,
}

pub struct Core<T: Bus> {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: Flags,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0xffc0,
            flags: Flags {
                n: 0,
                v: 0,
                p: 0,
                b: false,
                h: 0,
                i: false,
                z: 0xff,
                c: false,
            },
            bus,
        }
    }

    pub fn bus(&self) -> &T {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut T {
        &mut self.bus
    }

    pub fn step(&mut self) {
        use address_mode as addr;
        use instruction as instr;
        use operator as op;

        match self.next_byte() {
            // +0x0d
            0xcd => instr::read::<addr::X, addr::Immediate, op::Mov>(self),

            opcode => todo!("SPC700 opcode {:02X}", opcode),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:04X} => {:02X}", address, value);
        value
    }

    fn next_byte(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    pub fn set_nz(&mut self, value: u8) {
        self.flags.n = value;
        self.flags.z = value;
    }
}

impl<T: Bus> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A={:02X} X={:02X} Y={:02X} SP={:02X} PC={:04X} PSW={}{}{}{}{}{}{}{} {}",
            self.a,
            self.x,
            self.y,
            self.sp,
            self.pc,
            if (self.flags.n & 0x80) != 0 { 'N' } else { '-' },
            if (self.flags.v & 0x80) != 0 { 'V' } else { '-' },
            if self.flags.p != 0 { 'P' } else { '-' },
            if self.flags.b { 'B' } else { '-' },
            if (self.flags.h & 0x10) != 0 { 'H' } else { '-' },
            if self.flags.i { 'I' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            self.bus,
        )
    }
}
