use std::fmt;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

pub const STACK_PAGE: u16 = 0x0100;

pub type Interrupt = u32;

pub const INT_RESET: Interrupt = 0x0000_0001;
pub const INT_NMI: Interrupt = 0x0000_0002;

pub trait Bus: fmt::Display {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn poll(&mut self) -> Interrupt;
    fn acknowledge(&mut self, interrupt: Interrupt);
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
                self.bus.acknowledge(INT_RESET);
                instr::reset(self);
            } else if (self.interrupt & INT_NMI) != 0 {
                self.bus.acknowledge(INT_NMI);
                instr::nmi(self);
            } else {
                panic!("Interrupt type not yet supported");
            }

            self.interrupt = 0;
            return;
        }

        match self.next_byte() {
            // Page 0: Control Ops

            // +0x00
            0x20 => instr::jsr(self),
            0x60 => instr::rts(self),
            //0x80 => instr::read::<addr::Immediate, op::Nop>(self),
            0xa0 => instr::read::<addr::Immediate, op::Ldy>(self),
            0xc0 => instr::read::<addr::Immediate, op::Cpy>(self),
            0xe0 => instr::read::<addr::Immediate, op::Cpx>(self),

            // +0x10
            0x10 => instr::branch::<op::Bpl>(self),
            0x30 => instr::branch::<op::Bmi>(self),
            0x50 => instr::branch::<op::Bvc>(self),
            0x70 => instr::branch::<op::Bvs>(self),
            0x90 => instr::branch::<op::Bcc>(self),
            0xb0 => instr::branch::<op::Bcs>(self),
            0xd0 => instr::branch::<op::Bne>(self),
            0xf0 => instr::branch::<op::Beq>(self),

            // +0x04
            //0x04 => instr::read::<addr::ZeroPage, op::Nop>(self),
            0x24 => instr::read::<addr::ZeroPage, op::Bit>(self),
            //0x44 => instr::read::<addr::ZeroPage, op::Nop>(self),
            //0x64 => instr::read::<addr::ZeroPage, op::Nop>(self),
            0x84 => instr::write::<addr::ZeroPage, op::Sty>(self),
            0xa4 => instr::read::<addr::ZeroPage, op::Ldy>(self),
            0xc4 => instr::read::<addr::ZeroPage, op::Cpy>(self),
            0xe4 => instr::read::<addr::ZeroPage, op::Cpx>(self),

            // +0x08
            //0x08 => instr::php(self),
            //0x28 => instr::plp(self),
            0x48 => instr::pha(self),
            //0x68 => instr::pla(self),
            0x88 => instr::dey(self),
            0xa8 => instr::tay(self),
            0xc8 => instr::iny(self),
            0xe8 => instr::inx(self),

            // +0x18
            0x18 => instr::clc(self),
            0x38 => instr::sec(self),
            0x58 => instr::cli(self),
            0x78 => instr::sei(self),
            0x98 => instr::tya(self),
            0xb8 => instr::clv(self),
            0xd8 => instr::cld(self),
            0xf8 => instr::sed(self),

            // +0x0c
            //0x0c => instr::read::<addr::Absolute, op::Nop>(self),
            0x2c => instr::read::<addr::Absolute, op::Bit>(self),
            0x4c => instr::jmp(self),
            //0x6c => instr::jmp_indirect(self),
            0x8c => instr::write::<addr::Absolute, op::Sty>(self),
            0xac => instr::read::<addr::Absolute, op::Ldy>(self),
            0xcc => instr::read::<addr::Absolute, op::Cpy>(self),
            0xec => instr::read::<addr::Absolute, op::Cpx>(self),

            // Page 1: Accumulator Ops

            // +0x11
            0x11 => instr::read::<addr::ZeroPageIndirectY, op::Ora>(self),
            0x31 => instr::read::<addr::ZeroPageIndirectY, op::And>(self),
            0x51 => instr::read::<addr::ZeroPageIndirectY, op::Eor>(self),
            //0x71 => instr::read::<addr::ZeroPageIndirectY, op::Adc>(self),
            0x91 => instr::write::<addr::ZeroPageIndirectY, op::Sta>(self),
            0xb1 => instr::read::<addr::ZeroPageIndirectY, op::Lda>(self),
            0xd1 => instr::read::<addr::ZeroPageIndirectY, op::Cmp>(self),
            //0xf1 => instr::read::<addr::ZeroPageIndirectY, op::Sbc>(self),

            // +0x05
            0x05 => instr::read::<addr::ZeroPage, op::Ora>(self),
            0x25 => instr::read::<addr::ZeroPage, op::And>(self),
            0x45 => instr::read::<addr::ZeroPage, op::Eor>(self),
            //0x65 => instr::read::<addr::ZeroPage, op::Adc>(self),
            0x85 => instr::write::<addr::ZeroPage, op::Sta>(self),
            0xa5 => instr::read::<addr::ZeroPage, op::Lda>(self),
            0xc5 => instr::read::<addr::ZeroPage, op::Cmp>(self),
            //0xe5 => instr::read::<addr::ZeroPage, op::Sbc>(self),

            // +0x09
            0x09 => instr::read::<addr::Immediate, op::Ora>(self),
            0x29 => instr::read::<addr::Immediate, op::And>(self),
            0x49 => instr::read::<addr::Immediate, op::Eor>(self),
            //0x69 => instr::read::<addr::Immediate, op::Adc>(self),
            //0x89 => instr::read::<addr::Immediate, op::Nop>(self),
            0xa9 => instr::read::<addr::Immediate, op::Lda>(self),
            0xc9 => instr::read::<addr::Immediate, op::Cmp>(self),
            //0xe9 => instr::read::<addr::Immediate, op::Sbc>(self),

            // +0x19
            0x19 => instr::read::<addr::AbsoluteY, op::Ora>(self),
            0x39 => instr::read::<addr::AbsoluteY, op::And>(self),
            0x59 => instr::read::<addr::AbsoluteY, op::Eor>(self),
            //0x79 => instr::read::<addr::AbsoluteY, op::Adc>(self),
            0x99 => instr::write::<addr::AbsoluteY, op::Sta>(self),
            0xb9 => instr::read::<addr::AbsoluteY, op::Lda>(self),
            0xd9 => instr::read::<addr::AbsoluteY, op::Cmp>(self),
            //0xf9 => instr::read::<addr::AbsoluteY, op::Sbc>(self),

            // +0x0d
            0x0d => instr::read::<addr::Absolute, op::Ora>(self),
            0x2d => instr::read::<addr::Absolute, op::And>(self),
            0x4d => instr::read::<addr::Absolute, op::Eor>(self),
            //0x6d => instr::read::<addr::Absolute, op::Adc>(self),
            0x8d => instr::write::<addr::Absolute, op::Sta>(self),
            0xad => instr::read::<addr::Absolute, op::Lda>(self),
            0xcd => instr::read::<addr::Absolute, op::Cmp>(self),
            //0xed => instr::read::<addr::Absolute, op::Sbc>(self),

            // +0x1d
            0x1d => instr::read::<addr::AbsoluteX, op::Ora>(self),
            0x3d => instr::read::<addr::AbsoluteX, op::And>(self),
            0x5d => instr::read::<addr::AbsoluteX, op::Eor>(self),
            //0x7d => instr::read::<addr::AbsoluteX, op::Adc>(self),
            0x9d => instr::write::<addr::AbsoluteX, op::Sta>(self),
            0xbd => instr::read::<addr::AbsoluteX, op::Lda>(self),
            0xdd => instr::read::<addr::AbsoluteX, op::Cmp>(self),
            //0xfd => instr::read::<addr::AbsoluteX, op::Sbc>(self),

            // Page 2: Read-Modify-Write Ops

            // +0x02
            0xa2 => instr::read::<addr::Immediate, op::Ldx>(self),

            // +0x06
            //0x06 => instr::modify::<addr::ZeroPage, op::Asl>(self),
            //0x26 => instr::modify::<addr::ZeroPage, op::Rol>(self),
            0x46 => instr::modify::<addr::ZeroPage, op::Lsr>(self),
            //0x66 => instr::modify::<addr::ZeroPage, op::Ror>(self),
            0x86 => instr::write::<addr::ZeroPage, op::Stx>(self),
            0xa6 => instr::read::<addr::ZeroPage, op::Ldx>(self),
            0xc6 => instr::modify::<addr::ZeroPage, op::Dec>(self),
            0xe6 => instr::modify::<addr::ZeroPage, op::Inc>(self),

            // +0x0a
            //0x0a => instr::accumulator::<op::Asl>(self),
            //0x2a => instr::accumulator::<op::Rol>(self),
            0x4a => instr::accumulator::<op::Lsr>(self),
            //0x6a => instr::accumulator::<op::Ror>(self),
            0x8a => instr::txa(self),
            0xaa => instr::tax(self),
            0xca => instr::dex(self),

            // +0x1a
            0x9a => instr::txs(self),
            0xba => instr::tsx(self),

            // +0x0e
            //0x0e => instr::modify::<addr::Absolute, op::Asl>(self),
            //0x2e => instr::modify::<addr::Absolute, op::Rol>(self),
            0x4e => instr::modify::<addr::Absolute, op::Lsr>(self),
            //0x6e => instr::modify::<addr::Absolute, op::Ror>(self),
            0x8e => instr::write::<addr::Absolute, op::Stx>(self),
            0xae => instr::read::<addr::Absolute, op::Ldx>(self),
            0xce => instr::modify::<addr::Absolute, op::Dec>(self),
            0xee => instr::modify::<addr::Absolute, op::Inc>(self),

            // +0x1e
            //0x1e => instr::modify::<addr::AbsoluteX, op::Asl>(self),
            //0x3e => instr::modify::<addr::AbsoluteX, op::Rol>(self),
            0x5e => instr::modify::<addr::AbsoluteX, op::Lsr>(self),
            //0x7e => instr::modify::<addr::AbsoluteX, op::Ror>(self),
            //0x9e => instr::write::<addr::AbsoluteY, op::Shx>(self),
            0xbe => instr::read::<addr::AbsoluteY, op::Ldx>(self),
            0xde => instr::modify::<addr::AbsoluteX, op::Dec>(self),
            0xfe => instr::modify::<addr::AbsoluteX, op::Inc>(self),

            opcode @ _ => panic!("Opcode {:02X} not yet implemented", opcode),
        }
    }

    fn poll(&mut self) {
        self.interrupt = self.bus.poll() & (self.flags.i as Interrupt);
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

    fn pull(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read(STACK_PAGE | (self.s as u16))
    }

    fn push(&mut self, value: u8) {
        self.write(STACK_PAGE | (self.s as u16), value);
        self.s = self.s.wrapping_sub(1);
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

    fn flags_to_u8(&self, break_flag: bool) -> u8 {
        let mut result = 0x20;
        result |= self.flags.n & 0x80;
        result |= (self.flags.v & 0x80) >> 1;
        result |= if break_flag { 0x10 } else { 0 };
        result |= if self.flags.d { 0x08 } else { 0 };
        result |= if self.flags.i == IrqDisable::Set {
            0x04
        } else {
            0
        };
        result |= if self.flags.z == 0 { 0x02 } else { 0 };
        result |= self.flags.c as u8;
        result
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
            "A={:02X} X={:02X} Y={:02X} S={:02X} PC={:04X} P={}{}--{}{}{}{} {}",
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
            self.bus,
        )
    }
}
