use std::fmt;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

pub const STACK_PAGE: u16 = 0x0100;

pub type Interrupt = u32;

pub const INT_RESET: Interrupt = 0x0000_0001;
pub const INT_NMI: Interrupt = 0x0000_0002;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
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
        use address_mode as addr;
        use instruction as instr;
        use operator as op;

        if self.interrupt != 0 {
            self.read(self.pc);

            if (self.interrupt & INT_RESET) != 0 {
                instr::reset(self);
            } else {
                panic!("Interrupt type not yet supported");
            }

            self.interrupt = 0;
            return;
        }

        match self.next_byte() {
            // Page 0: Control Ops

            // +0x18
            0x18 => instr::clc(self),
            0x38 => instr::sec(self),
            0x58 => instr::cli(self),
            0x78 => instr::sei(self),
            //0x98 => instr::tya(self),
            0xb8 => instr::clv(self),
            0xd8 => instr::cld(self),
            0xf8 => instr::sed(self),

            // Page 1: Accumulator Ops

            //0x09 => instr::read::<addr::Immediate, op::Ora>(self),
            //0x29 => instr::read::<addr::Immediate, op::And>(self),
            //0x49 => instr::read::<addr::Immediate, op::Eor>(self),
            //0x69 => instr::read::<addr::Immediate, op::Adc>(self),
            //0x89 => instr::read::<addr::Immediate, op::Nop>(self),
            0xa9 => instr::read::<addr::Immediate, op::Lda>(self),
            //0xc9 => instr::read::<addr::Immediate, op::Cmp>(self),
            //0xe9 => instr::read::<addr::Immediate, op::Sbc>(self),

            //0x0d => instr::read::<addr::Absolute, op::Ora>(self),
            //0x2d => instr::read::<addr::Absolute, op::And>(self),
            //0x4d => instr::read::<addr::Absolute, op::Eor>(self),
            //0x6d => instr::read::<addr::Absolute, op::Adc>(self),
            0x8d => instr::write::<addr::Absolute, op::Sta>(self),
            0xad => instr::read::<addr::Absolute, op::Lda>(self),
            //0xcd => instr::read::<addr::Absolute, op::Cmp>(self),
            //0xed => instr::read::<addr::Absolute, op::Sbc>(self),
            opcode @ _ => panic!("Opcode {:02X} not yet implemented", opcode),
        }
    }

    fn poll(&mut self) {
        // TODO
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:04X} => {:02X}", address, value);
        value
    }

    fn write(&mut self, address: u16, value: u8) {
        debug!("  {:04X} <=> {:02X}", address, value);
        self.bus.write(address, value);
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

    fn set_nz(&mut self, value: u8) {
        self.flags.n = value;
        self.flags.z = value;
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
