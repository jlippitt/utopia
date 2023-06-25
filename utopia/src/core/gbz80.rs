use address_mode::{ReadAddress, WriteAddress};
use condition::Condition;
use std::fmt;
use tracing::debug;

mod address_mode;
mod condition;
mod instruction;

pub trait Bus: fmt::Display {
    fn idle(&mut self);
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read_high(&mut self, address: u8) -> u8;
    fn write_high(&mut self, address: u8, value: u8);
}

#[derive(Clone, Default)]
pub struct State {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub f: u8,
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
    ime: bool,
    ime_delayed: bool,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T, initial_state: Option<State>) -> Self {
        let state = initial_state.unwrap_or_default();

        Self {
            a: state.a,
            bc: ((state.b as u16) << 8) | state.c as u16,
            de: ((state.d as u16) << 8) | state.e as u16,
            hl: ((state.h as u16) << 8) | state.l as u16,
            sp: state.sp,
            pc: state.pc,
            flags: Flags {
                z: !state.f & 0x80,
                n: (state.f & 0x40) != 0,
                h: (state.f & 0x20) != 0,
                c: (state.f & 0x10) != 0,
            },
            ime: false,
            ime_delayed: false,
            bus,
        }
    }

    pub fn step(&mut self) {
        use address_mode as addr;
        use condition as cond;
        use instruction as instr;

        // TODO: Interrupt checks

        self.ime = self.ime_delayed;

        match self.next_byte() {
            // Page 0: Misc Ops

            // +0x00 / +0x08
            0x00 => instr::nop(self),
            0x18 => instr::jr(self),
            0x20 => instr::jr_conditional::<cond::NZ>(self),
            0x28 => instr::jr_conditional::<cond::Z>(self),
            0x30 => instr::jr_conditional::<cond::NC>(self),
            0x38 => instr::jr_conditional::<cond::C>(self),

            // +0x01 / +0x09
            0x01 => instr::ld16::<addr::BC>(self),
            0x09 => instr::add16::<addr::BC>(self),
            0x11 => instr::ld16::<addr::DE>(self),
            0x19 => instr::add16::<addr::DE>(self),
            0x21 => instr::ld16::<addr::HL>(self),
            0x29 => instr::add16::<addr::HL>(self),
            0x31 => instr::ld16::<addr::SP>(self),
            0x39 => instr::add16::<addr::SP>(self),

            // +0x02 / +0x0a
            0x02 => instr::ld::<addr::BCIndirect, addr::A>(self),
            0x0a => instr::ld::<addr::A, addr::BCIndirect>(self),
            0x12 => instr::ld::<addr::DEIndirect, addr::A>(self),
            0x1a => instr::ld::<addr::A, addr::DEIndirect>(self),
            0x22 => instr::ld::<addr::HLIncrement, addr::A>(self),
            0x2a => instr::ld::<addr::A, addr::HLIncrement>(self),
            0x32 => instr::ld::<addr::HLDecrement, addr::A>(self),
            0x3a => instr::ld::<addr::A, addr::HLDecrement>(self),

            // +0x03 / +0x0b
            0x03 => instr::inc16::<addr::BC>(self),
            0x0b => instr::dec16::<addr::BC>(self),
            0x13 => instr::inc16::<addr::DE>(self),
            0x1b => instr::dec16::<addr::DE>(self),
            0x23 => instr::inc16::<addr::HL>(self),
            0x2b => instr::dec16::<addr::HL>(self),
            0x33 => instr::inc16::<addr::SP>(self),
            0x3b => instr::dec16::<addr::SP>(self),

            // +0x04 / +0x0c
            0x04 => instr::inc::<addr::B>(self),
            0x0c => instr::inc::<addr::C>(self),
            0x14 => instr::inc::<addr::D>(self),
            0x1c => instr::inc::<addr::E>(self),
            0x24 => instr::inc::<addr::H>(self),
            0x2c => instr::inc::<addr::L>(self),
            0x34 => instr::inc::<addr::HLIndirect>(self),
            0x3c => instr::inc::<addr::A>(self),

            // +0x05 / +0x0d
            0x05 => instr::dec::<addr::B>(self),
            0x0d => instr::dec::<addr::C>(self),
            0x15 => instr::dec::<addr::D>(self),
            0x1d => instr::dec::<addr::E>(self),
            0x25 => instr::dec::<addr::H>(self),
            0x2d => instr::dec::<addr::L>(self),
            0x35 => instr::dec::<addr::HLIndirect>(self),
            0x3d => instr::dec::<addr::A>(self),

            // +0x06 / +0x0e
            0x06 => instr::ld::<addr::B, addr::Immediate>(self),
            0x0e => instr::ld::<addr::C, addr::Immediate>(self),
            0x16 => instr::ld::<addr::D, addr::Immediate>(self),
            0x1e => instr::ld::<addr::E, addr::Immediate>(self),
            0x26 => instr::ld::<addr::H, addr::Immediate>(self),
            0x2e => instr::ld::<addr::L, addr::Immediate>(self),
            0x36 => instr::ld::<addr::HLIndirect, addr::Immediate>(self),
            0x3e => instr::ld::<addr::A, addr::Immediate>(self),

            // +0x07 / 0x0f
            0x17 => instr::rla(self),
            0x1f => instr::rra(self),

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

            // 0x80
            0x80 => instr::add::<addr::B>(self),
            0x81 => instr::add::<addr::C>(self),
            0x82 => instr::add::<addr::D>(self),
            0x83 => instr::add::<addr::E>(self),
            0x84 => instr::add::<addr::H>(self),
            0x85 => instr::add::<addr::L>(self),
            0x86 => instr::add::<addr::HLIndirect>(self),
            0x87 => instr::add::<addr::A>(self),

            // 0x88
            0x88 => instr::adc::<addr::B>(self),
            0x89 => instr::adc::<addr::C>(self),
            0x8a => instr::adc::<addr::D>(self),
            0x8b => instr::adc::<addr::E>(self),
            0x8c => instr::adc::<addr::H>(self),
            0x8d => instr::adc::<addr::L>(self),
            0x8e => instr::adc::<addr::HLIndirect>(self),
            0x8f => instr::adc::<addr::A>(self),

            // 0x90
            0x90 => instr::sub::<addr::B>(self),
            0x91 => instr::sub::<addr::C>(self),
            0x92 => instr::sub::<addr::D>(self),
            0x93 => instr::sub::<addr::E>(self),
            0x94 => instr::sub::<addr::H>(self),
            0x95 => instr::sub::<addr::L>(self),
            0x96 => instr::sub::<addr::HLIndirect>(self),
            0x97 => instr::sub::<addr::A>(self),

            // 0x98
            0x98 => instr::sbc::<addr::B>(self),
            0x99 => instr::sbc::<addr::C>(self),
            0x9a => instr::sbc::<addr::D>(self),
            0x9b => instr::sbc::<addr::E>(self),
            0x9c => instr::sbc::<addr::H>(self),
            0x9d => instr::sbc::<addr::L>(self),
            0x9e => instr::sbc::<addr::HLIndirect>(self),
            0x9f => instr::sbc::<addr::A>(self),

            // 0xA0
            0xa0 => instr::and::<addr::B>(self),
            0xa1 => instr::and::<addr::C>(self),
            0xa2 => instr::and::<addr::D>(self),
            0xa3 => instr::and::<addr::E>(self),
            0xa4 => instr::and::<addr::H>(self),
            0xa5 => instr::and::<addr::L>(self),
            0xa6 => instr::and::<addr::HLIndirect>(self),
            0xa7 => instr::and::<addr::A>(self),

            // 0xA8
            0xa8 => instr::xor::<addr::B>(self),
            0xa9 => instr::xor::<addr::C>(self),
            0xaa => instr::xor::<addr::D>(self),
            0xab => instr::xor::<addr::E>(self),
            0xac => instr::xor::<addr::H>(self),
            0xad => instr::xor::<addr::L>(self),
            0xae => instr::xor::<addr::HLIndirect>(self),
            0xaf => instr::xor::<addr::A>(self),

            // 0xB0
            0xb0 => instr::or::<addr::B>(self),
            0xb1 => instr::or::<addr::C>(self),
            0xb2 => instr::or::<addr::D>(self),
            0xb3 => instr::or::<addr::E>(self),
            0xb4 => instr::or::<addr::H>(self),
            0xb5 => instr::or::<addr::L>(self),
            0xb6 => instr::or::<addr::HLIndirect>(self),
            0xb7 => instr::or::<addr::A>(self),

            // 0xB8
            0xb8 => instr::cp::<addr::B>(self),
            0xb9 => instr::cp::<addr::C>(self),
            0xba => instr::cp::<addr::D>(self),
            0xbb => instr::cp::<addr::E>(self),
            0xbc => instr::cp::<addr::H>(self),
            0xbd => instr::cp::<addr::L>(self),
            0xbe => instr::cp::<addr::HLIndirect>(self),
            0xbf => instr::cp::<addr::A>(self),

            // Page 3: Misc Ops 2

            // +0x00 / 0x08
            0xc0 => instr::ret_conditional::<cond::NZ>(self),
            0xc8 => instr::ret_conditional::<cond::Z>(self),
            0xd0 => instr::ret_conditional::<cond::NC>(self),
            0xd8 => instr::ret_conditional::<cond::C>(self),
            0xe0 => instr::ld::<addr::High, addr::A>(self),
            0xf0 => instr::ld::<addr::A, addr::High>(self),

            // +0x01 / 0x09
            0xc1 => instr::pop::<addr::BC>(self),
            0xc9 => instr::ret(self),
            0xd1 => instr::pop::<addr::DE>(self),
            0xe1 => instr::pop::<addr::HL>(self),
            0xe9 => instr::jp_hl(self),
            0xf1 => instr::pop::<addr::AF>(self),

            // +0x02 / 0x0a
            0xe2 => instr::ld::<addr::CIndirect, addr::A>(self),
            0xea => instr::ld::<addr::Absolute, addr::A>(self),
            0xf2 => instr::ld::<addr::A, addr::CIndirect>(self),
            0xfa => instr::ld::<addr::A, addr::Absolute>(self),

            // +0x03 / 0x0b
            0xc3 => instr::jp(self),
            0xcb => self.prefix_cb(),
            0xf3 => instr::di(self),
            0xfb => instr::ei(self),

            // +0x04 / 0x0c
            0xc4 => instr::call_conditional::<cond::NZ>(self),
            0xcc => instr::call_conditional::<cond::Z>(self),
            0xd4 => instr::call_conditional::<cond::NC>(self),
            0xdc => instr::call_conditional::<cond::C>(self),

            // +0x05 / 0x0d
            0xc5 => instr::push::<addr::BC>(self),
            0xcd => instr::call(self),
            0xd5 => instr::push::<addr::DE>(self),
            0xe5 => instr::push::<addr::HL>(self),
            0xf5 => instr::push::<addr::AF>(self),

            // +0x06 / 0x0e
            0xc6 => instr::add::<addr::Immediate>(self),
            0xce => instr::adc::<addr::Immediate>(self),
            0xd6 => instr::sub::<addr::Immediate>(self),
            0xde => instr::sbc::<addr::Immediate>(self),
            0xe6 => instr::and::<addr::Immediate>(self),
            0xee => instr::xor::<addr::Immediate>(self),
            0xf6 => instr::or::<addr::Immediate>(self),
            0xfe => instr::cp::<addr::Immediate>(self),

            opcode => panic!("Opcode {:02X} not yet implemented", opcode),
        }
    }

    fn prefix_cb(&mut self) {
        use address_mode as addr;
        use instruction as instr;

        let opcode = self.next_byte();

        match opcode {
            // Page 0: Shifts and Rotates

            // 0x10
            0x10 => instr::rl::<addr::B>(self),
            0x11 => instr::rl::<addr::C>(self),
            0x12 => instr::rl::<addr::D>(self),
            0x13 => instr::rl::<addr::E>(self),
            0x14 => instr::rl::<addr::H>(self),
            0x15 => instr::rl::<addr::L>(self),
            0x16 => instr::rl::<addr::HLIndirect>(self),
            0x17 => instr::rl::<addr::A>(self),

            // 0x18
            0x18 => instr::rr::<addr::B>(self),
            0x19 => instr::rr::<addr::C>(self),
            0x1a => instr::rr::<addr::D>(self),
            0x1b => instr::rr::<addr::E>(self),
            0x1c => instr::rr::<addr::H>(self),
            0x1d => instr::rr::<addr::L>(self),
            0x1e => instr::rr::<addr::HLIndirect>(self),
            0x1f => instr::rr::<addr::A>(self),

            // 0x38
            0x38 => instr::srl::<addr::B>(self),
            0x39 => instr::srl::<addr::C>(self),
            0x3a => instr::srl::<addr::D>(self),
            0x3b => instr::srl::<addr::E>(self),
            0x3c => instr::srl::<addr::H>(self),
            0x3d => instr::srl::<addr::L>(self),
            0x3e => instr::srl::<addr::HLIndirect>(self),
            0x3f => instr::srl::<addr::A>(self),

            // Page 1: BIT
            0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => {
                instr::bit::<addr::B>(self, opcode)
            }
            0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x71 | 0x79 => {
                instr::bit::<addr::C>(self, opcode)
            }
            0x42 | 0x4a | 0x52 | 0x5a | 0x62 | 0x6a | 0x72 | 0x7a => {
                instr::bit::<addr::D>(self, opcode)
            }
            0x43 | 0x4b | 0x53 | 0x5b | 0x63 | 0x6b | 0x73 | 0x7b => {
                instr::bit::<addr::E>(self, opcode)
            }
            0x44 | 0x4c | 0x54 | 0x5c | 0x64 | 0x6c | 0x74 | 0x7c => {
                instr::bit::<addr::H>(self, opcode)
            }
            0x45 | 0x4d | 0x55 | 0x5d | 0x65 | 0x6d | 0x75 | 0x7d => {
                instr::bit::<addr::L>(self, opcode)
            }
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => {
                instr::bit::<addr::HLIndirect>(self, opcode)
            }
            0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 | 0x7f => {
                instr::bit::<addr::A>(self, opcode)
            }

            // Page 2: RES

            // Page 3: SET
            _ => panic!("Opcode CB{:02X} not yet implemented", opcode),
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

    fn pop(&mut self) -> u16 {
        let low = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
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
            "A={:02X} BC={:04X} DE={:04X} HL={:04X} SP={:04X} PC={:04X} F={}{}{}{}---- IME={} {}",
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
            self.ime as u32,
            self.bus,
        )
    }
}
