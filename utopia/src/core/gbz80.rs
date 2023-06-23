use std::fmt;
use tracing::debug;
use address_mode::{ReadAddress, WriteAddress};
use condition::Condition;

mod address_mode;
mod condition;
mod instruction;

pub trait Bus : fmt::Display {
    fn idle(&mut self);
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read_high(&mut self, address: u8) -> u8;
    fn write_high(&mut self, address: u8, value: u8);
}

pub struct Flags {
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
        use condition as cond;
        use instruction as instr;

        match self.next_byte() {
            // Page 0: Misc Ops

            // +0x00 / +0x08
            0x18 => instr::jr(self),
            0x20 => instr::jr_conditional::<cond::NZ>(self),
            0x28 => instr::jr_conditional::<cond::Z>(self),
            0x30 => instr::jr_conditional::<cond::NC>(self),
            0x38 => instr::jr_conditional::<cond::C>(self),

            // +0x01 / +0x09
            0x01 => instr::ld16::<addr::BC>(self),
            0x11 => instr::ld16::<addr::DE>(self),
            0x21 => instr::ld16::<addr::HL>(self),
            0x31 => instr::ld16::<addr::SP>(self),

            // +0x02 / +0x0a
            0x02 => instr::ld::<addr::BCIndirect, addr::A>(self),
            0x0a => instr::ld::<addr::A, addr::BCIndirect>(self),
            0x12 => instr::ld::<addr::DEIndirect, addr::A>(self),
            0x1a => instr::ld::<addr::A, addr::DEIndirect>(self),
            0x22 => instr::ld::<addr::HLIncrement, addr::A>(self),
            0x2a => instr::ld::<addr::A, addr::HLIncrement>(self),
            0x32 => instr::ld::<addr::HLDecrement, addr::A>(self),
            0x3a => instr::ld::<addr::A, addr::HLDecrement>(self),

            // +0x04 / +0x0c
            0x04 => instr::inc::<addr::B>(self),
            0x0c => instr::inc::<addr::C>(self),
            0x14 => instr::inc::<addr::D>(self),
            0x1c => instr::inc::<addr::E>(self),
            0x24 => instr::inc::<addr::H>(self),
            0x2c => instr::inc::<addr::L>(self),
            0x34 => instr::inc::<addr::HLIndirect>(self),
            0x3c => instr::inc::<addr::A>(self),

            // +0x06 / +0x0e
            0x06 => instr::ld::<addr::B, addr::Immediate>(self),
            0x0e => instr::ld::<addr::C, addr::Immediate>(self),
            0x16 => instr::ld::<addr::D, addr::Immediate>(self),
            0x1e => instr::ld::<addr::E, addr::Immediate>(self),
            0x26 => instr::ld::<addr::H, addr::Immediate>(self),
            0x2e => instr::ld::<addr::L, addr::Immediate>(self),
            0x36 => instr::ld::<addr::HLIndirect, addr::Immediate>(self),
            0x3e => instr::ld::<addr::A, addr::Immediate>(self),

            // Page 1: 8-bit Loads

            // 0x40
            0x40 => instr::ld::<addr::B, addr::B>(self),
            0x41 => instr::ld::<addr::B, addr::C>(self),
            0x42 => instr::ld::<addr::B, addr::D>(self),
            0x43 => instr::ld::<addr::B, addr::E>(self),
            0x44 => instr::ld::<addr::B, addr::H>(self),
            0x45 => instr::ld::<addr::B, addr::L>(self),
            0x46 => instr::ld::<addr::B, addr::HLIndirect>(self),
            0x47 => instr::ld::<addr::B, addr::A>(self),

            // 0x48
            0x48 => instr::ld::<addr::C, addr::B>(self),
            0x49 => instr::ld::<addr::C, addr::C>(self),
            0x4a => instr::ld::<addr::C, addr::D>(self),
            0x4b => instr::ld::<addr::C, addr::E>(self),
            0x4c => instr::ld::<addr::C, addr::H>(self),
            0x4d => instr::ld::<addr::C, addr::L>(self),
            0x4e => instr::ld::<addr::C, addr::HLIndirect>(self),
            0x4f => instr::ld::<addr::C, addr::A>(self),

            // 0x50
            0x50 => instr::ld::<addr::D, addr::B>(self),
            0x51 => instr::ld::<addr::D, addr::C>(self),
            0x52 => instr::ld::<addr::D, addr::D>(self),
            0x53 => instr::ld::<addr::D, addr::E>(self),
            0x54 => instr::ld::<addr::D, addr::H>(self),
            0x55 => instr::ld::<addr::D, addr::L>(self),
            0x56 => instr::ld::<addr::D, addr::HLIndirect>(self),
            0x57 => instr::ld::<addr::D, addr::A>(self),

            // 0x58
            0x58 => instr::ld::<addr::E, addr::B>(self),
            0x59 => instr::ld::<addr::E, addr::C>(self),
            0x5a => instr::ld::<addr::E, addr::D>(self),
            0x5b => instr::ld::<addr::E, addr::E>(self),
            0x5c => instr::ld::<addr::E, addr::H>(self),
            0x5d => instr::ld::<addr::E, addr::L>(self),
            0x5e => instr::ld::<addr::E, addr::HLIndirect>(self),
            0x5f => instr::ld::<addr::E, addr::A>(self),

            // 0x60
            0x60 => instr::ld::<addr::H, addr::B>(self),
            0x61 => instr::ld::<addr::H, addr::C>(self),
            0x62 => instr::ld::<addr::H, addr::D>(self),
            0x63 => instr::ld::<addr::H, addr::E>(self),
            0x64 => instr::ld::<addr::H, addr::H>(self),
            0x65 => instr::ld::<addr::H, addr::L>(self),
            0x66 => instr::ld::<addr::H, addr::HLIndirect>(self),
            0x67 => instr::ld::<addr::H, addr::A>(self),

            // 0x68
            0x68 => instr::ld::<addr::L, addr::B>(self),
            0x69 => instr::ld::<addr::L, addr::C>(self),
            0x6a => instr::ld::<addr::L, addr::D>(self),
            0x6b => instr::ld::<addr::L, addr::E>(self),
            0x6c => instr::ld::<addr::L, addr::H>(self),
            0x6d => instr::ld::<addr::L, addr::L>(self),
            0x6e => instr::ld::<addr::L, addr::HLIndirect>(self),
            0x6f => instr::ld::<addr::L, addr::A>(self),

            // 0x70
            0x70 => instr::ld::<addr::HLIndirect, addr::B>(self),
            0x71 => instr::ld::<addr::HLIndirect, addr::C>(self),
            0x72 => instr::ld::<addr::HLIndirect, addr::D>(self),
            0x73 => instr::ld::<addr::HLIndirect, addr::E>(self),
            0x74 => instr::ld::<addr::HLIndirect, addr::H>(self),
            0x75 => instr::ld::<addr::HLIndirect, addr::L>(self),
            //0x76 => instr::halt(self);
            0x77 => instr::ld::<addr::HLIndirect, addr::A>(self),

            // 0x78
            0x78 => instr::ld::<addr::A, addr::B>(self),
            0x79 => instr::ld::<addr::A, addr::C>(self),
            0x7a => instr::ld::<addr::A, addr::D>(self),
            0x7b => instr::ld::<addr::A, addr::E>(self),
            0x7c => instr::ld::<addr::A, addr::H>(self),
            0x7d => instr::ld::<addr::A, addr::L>(self),
            0x7e => instr::ld::<addr::A, addr::HLIndirect>(self),
            0x7f => instr::ld::<addr::A, addr::A>(self),

            // Page 2: 8-bit Arithmetic & Logic

            // 0xA8
            0xa8 => instr::xor::<addr::B>(self),
            0xa9 => instr::xor::<addr::C>(self),
            0xaa => instr::xor::<addr::D>(self),
            0xab => instr::xor::<addr::E>(self),
            0xac => instr::xor::<addr::H>(self),
            0xad => instr::xor::<addr::L>(self),
            0xae => instr::xor::<addr::HLIndirect>(self),
            0xaf => instr::xor::<addr::A>(self),

            // Page 3: Misc Ops 2

            // +0x00 / 0x08
            0xe0 => instr::ld::<addr::High, addr::A>(self),
            0xf0 => instr::ld::<addr::A, addr::High>(self),

            // +0x02 / 0x0a
            0xe2 => instr::ld::<addr::CIndirect, addr::A>(self),
            0xf2 => instr::ld::<addr::A, addr::CIndirect>(self),

            // +0x03 / 0x0b
            0xcb => self.prefix_cb(),

            // +0x05 / 0x0d
            0xcd => instr::call(self),

            opcode @ _ => panic!("Opcode {:02X} not yet implemented", opcode)
        }
    }

    fn prefix_cb(&mut self) {
        use address_mode as addr;
        use instruction as instr;

        let opcode = self.next_byte();

        match opcode {
            // Page 0: Shifts and Rotates

            // Page 1: BIT
            0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => instr::bit::<addr::B>(self, opcode),
            0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x71 | 0x79 => instr::bit::<addr::C>(self, opcode),
            0x42 | 0x4a | 0x52 | 0x5a | 0x62 | 0x6a | 0x72 | 0x7a => instr::bit::<addr::D>(self, opcode),
            0x43 | 0x4b | 0x53 | 0x5b | 0x63 | 0x6b | 0x73 | 0x7b => instr::bit::<addr::E>(self, opcode),
            0x44 | 0x4c | 0x54 | 0x5c | 0x64 | 0x6c | 0x74 | 0x7c => instr::bit::<addr::H>(self, opcode),
            0x45 | 0x4d | 0x55 | 0x5d | 0x65 | 0x6d | 0x75 | 0x7d => instr::bit::<addr::L>(self, opcode),
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => instr::bit::<addr::HLIndirect>(self, opcode),
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 | 0x7f => instr::bit::<addr::A>(self, opcode),

            // Page 2: RES

            // Page 3: SET

            _ => panic!("Opcode CB{:02X} not yet implemented", opcode)
        }
    }

    fn idle(&mut self) {
        debug!("  IO");
        self.bus.idle();
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:04X} => {:02X}", address, value);
        value
    }

    fn write(&mut self, address: u16, value: u8) {
        debug!("  {:04X} <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn read_high(&mut self, address: u8) -> u8 {
        let value = self.bus.read_high(address);
        debug!("  FF{:02X} => {:02X}", address, value);
        value
    }

    fn write_high(&mut self, address: u8, value: u8) {
        debug!("  FF{:02X} <= {:02X}", address, value);
        self.bus.write_high(address, value);
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

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write(self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.write(self.sp, value as u8);
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