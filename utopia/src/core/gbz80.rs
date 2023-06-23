use std::fmt;
use tracing::debug;
use address_mode::WriteAddress;

mod address_mode;
mod instruction;

pub trait Bus : fmt::Display {
    fn read(&mut self, address: u16) -> u8;
}

struct Flags {
    z: u8,
    n: bool,
    h: bool,
    c: bool,
}

pub struct Core<T: Bus> {
    a: u8,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    flags: Flags,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            a: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0,
            pc: 0,
            flags: Flags {
                z: 0xff,
                n: false,
                h: false,
                c: false,
            },
            bus,
        }
    }

    pub fn step(&mut self) {
        use address_mode as addr;
        use instruction as instr;

        match self.next_byte() {
            // Page 0: Misc Ops

            // +0x01 / +0x09
            0x01 => instr::ld16::<addr::BC>(self),
            0x11 => instr::ld16::<addr::DE>(self),
            0x21 => instr::ld16::<addr::HL>(self),
            0x31 => instr::ld16::<addr::SP>(self),

            // Page 1: 8-bit Loads

            // Page 2: 8-bit Arithmetic & Logic

            // Page 3: Misc Ops 2

            opcode @ _ => panic!("Opcode {:02X} not yet implemented", opcode)
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

    fn next_word(&mut self) -> u16 {
        let low = self.next_byte();
        let high = self.next_byte();
        u16::from_le_bytes([low, high])
    }
}

impl<T: Bus> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A={:02X} BC={:04X} DE={:04X} HL={:04X} SP={:04X} PC={:04X} F={}{}{}{}---- {}",
            self.a,
            self.bc,
            self.de,
            self.hl,
            self.sp,
            self.pc,
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.n { 'N' } else { '-' },
            if self.flags.h { 'H' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            self.bus,
        )
    }
}