use std::fmt;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

const ZERO_PAGE: u32 = 0x1f0000;
const STACK_PAGE: u32 = 0x1f0100;

pub type Interrupt = u8;

pub const INT_RESET: Interrupt = 0x01;
pub const INT_NMI: Interrupt = 0x02;
pub const INT_TIMER: Interrupt = 0x04;
pub const INT_IRQ1: Interrupt = 0x08;
pub const INT_IRQ2: Interrupt = 0x10;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
enum IrqDisable {
    Clear = 0xff,
    Set = INT_RESET | INT_NMI,
}

pub trait Bus: fmt::Display {
    fn read(&mut self, address: u32) -> u8;
    fn write(&mut self, address: u32, value: u8);
    fn poll(&mut self) -> Interrupt;
    fn acknowledge(&mut self, interrupt: Interrupt);
    fn set_clock_speed(&mut self, clock_speed_high: bool);
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
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    pc: u16,
    flags: Flags,
    mpr: [u32; 8],
    interrupt: Interrupt,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
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
            mpr: [0; 8],
            interrupt: INT_RESET,
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

        if self.interrupt != 0 {
            self.read(self.pc);

            if (self.interrupt & INT_RESET) != 0 {
                self.bus.acknowledge(INT_RESET);
                instr::reset(self);
            } else if (self.interrupt & INT_NMI) != 0 {
                self.bus.acknowledge(INT_NMI);
                instr::nmi(self);
            } else if (self.interrupt & INT_TIMER) != 0 {
                instr::timer(self);
            } else if (self.interrupt & INT_IRQ1) != 0 {
                instr::irq1(self);
            } else if (self.interrupt & INT_IRQ2) != 0 {
                instr::irq2(self);
            } else {
                panic!("Invalid interrupt mask: {:02X}", self.interrupt);
            }

            self.interrupt = 0;
            return;
        }

        match self.next_byte() {
            // Page 0: Control Ops

            // +0x00
            0x00 => instr::brk(self),
            0x20 => instr::jsr(self),
            0x40 => instr::rti(self),
            0x60 => instr::rts(self),
            0x80 => instr::branch::<op::Bra>(self),
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
            0x04 => instr::modify::<addr::ZeroPage, op::Tsb>(self),
            0x24 => instr::read::<addr::ZeroPage, op::Bit>(self),
            //0x44 => instr::read::<addr::ZeroPage, op::Nop>(self),
            0x64 => instr::write::<addr::ZeroPage, op::Stz>(self),
            0x84 => instr::write::<addr::ZeroPage, op::Sty>(self),
            0xa4 => instr::read::<addr::ZeroPage, op::Ldy>(self),
            0xc4 => instr::read::<addr::ZeroPage, op::Cpy>(self),
            0xe4 => instr::read::<addr::ZeroPage, op::Cpx>(self),

            // +0x14
            0x14 => instr::modify::<addr::ZeroPage, op::Trb>(self),
            0x34 => instr::read::<addr::ZeroPageX, op::Bit>(self),
            0x54 => instr::csl(self),
            0x74 => instr::write::<addr::ZeroPageX, op::Stz>(self),
            0x94 => instr::write::<addr::ZeroPageX, op::Sty>(self),
            0xb4 => instr::read::<addr::ZeroPageX, op::Ldy>(self),
            0xd4 => instr::csh(self),
            //0xf4 => instr::read::<addr::ZeroPageX, op::Nop>(self),

            // +0x08
            0x08 => instr::php(self),
            0x28 => instr::plp(self),
            0x48 => instr::pha(self),
            0x68 => instr::pla(self),
            0x88 => instr::dey(self),
            0xa8 => instr::tay(self),
            0xc8 => instr::iny(self),
            0xe8 => instr::inx(self),

            // // +0x18
            0x18 => instr::clc(self),
            0x38 => instr::sec(self),
            0x58 => instr::cli(self),
            0x78 => instr::sei(self),
            0x98 => instr::tya(self),
            0xb8 => instr::clv(self),
            0xd8 => instr::cld(self),
            0xf8 => instr::sed(self),

            // // +0x0c
            0x0c => instr::modify::<addr::Absolute, op::Tsb>(self),
            0x2c => instr::read::<addr::Absolute, op::Bit>(self),
            0x4c => instr::jmp(self),
            0x6c => instr::jmp_indirect(self),
            0x8c => instr::write::<addr::Absolute, op::Sty>(self),
            0xac => instr::read::<addr::Absolute, op::Ldy>(self),
            0xcc => instr::read::<addr::Absolute, op::Cpy>(self),
            0xec => instr::read::<addr::Absolute, op::Cpx>(self),

            // +0x1c
            0x1c => instr::modify::<addr::Absolute, op::Trb>(self),
            0x3c => instr::read::<addr::AbsoluteX, op::Bit>(self),
            //0x5c => instr::read::<addr::AbsoluteX, op::Nop>(self),
            //0x7c => instr::read::<addr::AbsoluteX, op::Nop>(self),
            0x9c => instr::write::<addr::Absolute, op::Stz>(self),
            0xbc => instr::read::<addr::AbsoluteX, op::Ldy>(self),
            //0xdc => instr::read::<addr::AbsoluteX, op::Nop>(self),
            //0xfc => instr::read::<addr::AbsoluteX, op::Nop>(self),

            // Page 1: Accumulator Ops

            // +0x01
            0x01 => instr::read::<addr::ZeroPageXIndirect, op::Ora>(self),
            0x21 => instr::read::<addr::ZeroPageXIndirect, op::And>(self),
            0x41 => instr::read::<addr::ZeroPageXIndirect, op::Eor>(self),
            0x61 => instr::read::<addr::ZeroPageXIndirect, op::Adc>(self),
            0x81 => instr::write::<addr::ZeroPageXIndirect, op::Sta>(self),
            0xa1 => instr::read::<addr::ZeroPageXIndirect, op::Lda>(self),
            0xc1 => instr::read::<addr::ZeroPageXIndirect, op::Cmp>(self),
            0xe1 => instr::read::<addr::ZeroPageXIndirect, op::Sbc>(self),

            // +0x11
            0x11 => instr::read::<addr::ZeroPageIndirectY, op::Ora>(self),
            0x31 => instr::read::<addr::ZeroPageIndirectY, op::And>(self),
            0x51 => instr::read::<addr::ZeroPageIndirectY, op::Eor>(self),
            0x71 => instr::read::<addr::ZeroPageIndirectY, op::Adc>(self),
            0x91 => instr::write::<addr::ZeroPageIndirectY, op::Sta>(self),
            0xb1 => instr::read::<addr::ZeroPageIndirectY, op::Lda>(self),
            0xd1 => instr::read::<addr::ZeroPageIndirectY, op::Cmp>(self),
            0xf1 => instr::read::<addr::ZeroPageIndirectY, op::Sbc>(self),

            // +0x05
            0x05 => instr::read::<addr::ZeroPage, op::Ora>(self),
            0x25 => instr::read::<addr::ZeroPage, op::And>(self),
            0x45 => instr::read::<addr::ZeroPage, op::Eor>(self),
            0x65 => instr::read::<addr::ZeroPage, op::Adc>(self),
            0x85 => instr::write::<addr::ZeroPage, op::Sta>(self),
            0xa5 => instr::read::<addr::ZeroPage, op::Lda>(self),
            0xc5 => instr::read::<addr::ZeroPage, op::Cmp>(self),
            0xe5 => instr::read::<addr::ZeroPage, op::Sbc>(self),

            // +0x15
            0x15 => instr::read::<addr::ZeroPageX, op::Ora>(self),
            0x35 => instr::read::<addr::ZeroPageX, op::And>(self),
            0x55 => instr::read::<addr::ZeroPageX, op::Eor>(self),
            0x75 => instr::read::<addr::ZeroPageX, op::Adc>(self),
            0x95 => instr::write::<addr::ZeroPageX, op::Sta>(self),
            0xb5 => instr::read::<addr::ZeroPageX, op::Lda>(self),
            0xd5 => instr::read::<addr::ZeroPageX, op::Cmp>(self),
            0xf5 => instr::read::<addr::ZeroPageX, op::Sbc>(self),

            // +0x09
            0x09 => instr::read::<addr::Immediate, op::Ora>(self),
            0x29 => instr::read::<addr::Immediate, op::And>(self),
            0x49 => instr::read::<addr::Immediate, op::Eor>(self),
            0x69 => instr::read::<addr::Immediate, op::Adc>(self),
            0x89 => instr::read::<addr::Immediate, op::BitImmediate>(self),
            0xa9 => instr::read::<addr::Immediate, op::Lda>(self),
            0xc9 => instr::read::<addr::Immediate, op::Cmp>(self),
            0xe9 => instr::read::<addr::Immediate, op::Sbc>(self),

            // +0x19
            0x19 => instr::read::<addr::AbsoluteY, op::Ora>(self),
            0x39 => instr::read::<addr::AbsoluteY, op::And>(self),
            0x59 => instr::read::<addr::AbsoluteY, op::Eor>(self),
            0x79 => instr::read::<addr::AbsoluteY, op::Adc>(self),
            0x99 => instr::write::<addr::AbsoluteY, op::Sta>(self),
            0xb9 => instr::read::<addr::AbsoluteY, op::Lda>(self),
            0xd9 => instr::read::<addr::AbsoluteY, op::Cmp>(self),
            0xf9 => instr::read::<addr::AbsoluteY, op::Sbc>(self),

            // +0x0d
            0x0d => instr::read::<addr::Absolute, op::Ora>(self),
            0x2d => instr::read::<addr::Absolute, op::And>(self),
            0x4d => instr::read::<addr::Absolute, op::Eor>(self),
            0x6d => instr::read::<addr::Absolute, op::Adc>(self),
            0x8d => instr::write::<addr::Absolute, op::Sta>(self),
            0xad => instr::read::<addr::Absolute, op::Lda>(self),
            0xcd => instr::read::<addr::Absolute, op::Cmp>(self),
            0xed => instr::read::<addr::Absolute, op::Sbc>(self),

            // +0x1d
            0x1d => instr::read::<addr::AbsoluteX, op::Ora>(self),
            0x3d => instr::read::<addr::AbsoluteX, op::And>(self),
            0x5d => instr::read::<addr::AbsoluteX, op::Eor>(self),
            0x7d => instr::read::<addr::AbsoluteX, op::Adc>(self),
            0x9d => instr::write::<addr::AbsoluteX, op::Sta>(self),
            0xbd => instr::read::<addr::AbsoluteX, op::Lda>(self),
            0xdd => instr::read::<addr::AbsoluteX, op::Cmp>(self),
            0xfd => instr::read::<addr::AbsoluteX, op::Sbc>(self),

            // Page 2: Read-Modify-Write Ops

            // +0x02
            0x02 => instr::sxy(self),
            0x22 => instr::sax(self),
            0x42 => instr::say(self),
            0x62 => instr::cla(self),
            0x82 => instr::clx(self),
            0xa2 => instr::read::<addr::Immediate, op::Ldx>(self),
            0xc2 => instr::cly(self),

            // +0x12
            0x12 => instr::read::<addr::ZeroPageIndirect, op::Ora>(self),
            0x32 => instr::read::<addr::ZeroPageIndirect, op::And>(self),
            0x52 => instr::read::<addr::ZeroPageIndirect, op::Eor>(self),
            0x72 => instr::read::<addr::ZeroPageIndirect, op::Adc>(self),
            0x92 => instr::write::<addr::ZeroPageIndirect, op::Sta>(self),
            0xb2 => instr::read::<addr::ZeroPageIndirect, op::Lda>(self),
            0xd2 => instr::read::<addr::ZeroPageIndirect, op::Cmp>(self),
            0xf2 => instr::read::<addr::ZeroPageIndirect, op::Sbc>(self),

            // +0x06
            0x06 => instr::modify::<addr::ZeroPage, op::Asl>(self),
            0x26 => instr::modify::<addr::ZeroPage, op::Rol>(self),
            0x46 => instr::modify::<addr::ZeroPage, op::Lsr>(self),
            0x66 => instr::modify::<addr::ZeroPage, op::Ror>(self),
            0x86 => instr::write::<addr::ZeroPage, op::Stx>(self),
            0xa6 => instr::read::<addr::ZeroPage, op::Ldx>(self),
            0xc6 => instr::modify::<addr::ZeroPage, op::Dec>(self),
            0xe6 => instr::modify::<addr::ZeroPage, op::Inc>(self),

            // +0x16
            0x16 => instr::modify::<addr::ZeroPageX, op::Asl>(self),
            0x36 => instr::modify::<addr::ZeroPageX, op::Rol>(self),
            0x56 => instr::modify::<addr::ZeroPageX, op::Lsr>(self),
            0x76 => instr::modify::<addr::ZeroPageX, op::Ror>(self),
            0x96 => instr::write::<addr::ZeroPageY, op::Stx>(self),
            0xb6 => instr::read::<addr::ZeroPageY, op::Ldx>(self),
            0xd6 => instr::modify::<addr::ZeroPageX, op::Dec>(self),
            0xf6 => instr::modify::<addr::ZeroPageX, op::Inc>(self),

            // +0x0a
            0x0a => instr::accumulator::<op::Asl>(self),
            0x2a => instr::accumulator::<op::Rol>(self),
            0x4a => instr::accumulator::<op::Lsr>(self),
            0x6a => instr::accumulator::<op::Ror>(self),
            0x8a => instr::txa(self),
            0xaa => instr::tax(self),
            0xca => instr::dex(self),
            0xea => instr::nop(self),

            // +0x1a
            0x1a => instr::accumulator::<op::Inc>(self),
            0x3a => instr::accumulator::<op::Dec>(self),
            0x5a => instr::phy(self),
            0x7a => instr::ply(self),
            0x9a => instr::txs(self),
            0xba => instr::tsx(self),
            0xda => instr::phx(self),
            0xfa => instr::plx(self),

            // +0x0e
            0x0e => instr::modify::<addr::Absolute, op::Asl>(self),
            0x2e => instr::modify::<addr::Absolute, op::Rol>(self),
            0x4e => instr::modify::<addr::Absolute, op::Lsr>(self),
            0x6e => instr::modify::<addr::Absolute, op::Ror>(self),
            0x8e => instr::write::<addr::Absolute, op::Stx>(self),
            0xae => instr::read::<addr::Absolute, op::Ldx>(self),
            0xce => instr::modify::<addr::Absolute, op::Dec>(self),
            0xee => instr::modify::<addr::Absolute, op::Inc>(self),

            // +0x1e
            0x1e => instr::modify::<addr::AbsoluteX, op::Asl>(self),
            0x3e => instr::modify::<addr::AbsoluteX, op::Rol>(self),
            0x5e => instr::modify::<addr::AbsoluteX, op::Lsr>(self),
            0x7e => instr::modify::<addr::AbsoluteX, op::Ror>(self),
            0x9e => instr::write::<addr::AbsoluteX, op::Stz>(self),
            0xbe => instr::read::<addr::AbsoluteY, op::Ldx>(self),
            0xde => instr::modify::<addr::AbsoluteX, op::Dec>(self),
            0xfe => instr::modify::<addr::AbsoluteX, op::Inc>(self),

            // Page 3: 'New' Ops

            // +0x03
            0x43 => instr::tma(self),

            // +0x13
            0x53 => instr::tam(self),

            // +0x07
            0x07 => instr::modify::<addr::ZeroPage, op::Rmb<0>>(self),
            0x27 => instr::modify::<addr::ZeroPage, op::Rmb<2>>(self),
            0x47 => instr::modify::<addr::ZeroPage, op::Rmb<4>>(self),
            0x67 => instr::modify::<addr::ZeroPage, op::Rmb<6>>(self),
            0x87 => instr::modify::<addr::ZeroPage, op::Smb<0>>(self),
            0xa7 => instr::modify::<addr::ZeroPage, op::Smb<2>>(self),
            0xc7 => instr::modify::<addr::ZeroPage, op::Smb<4>>(self),
            0xe7 => instr::modify::<addr::ZeroPage, op::Smb<6>>(self),

            // +0x17
            0x17 => instr::modify::<addr::ZeroPage, op::Rmb<1>>(self),
            0x37 => instr::modify::<addr::ZeroPage, op::Rmb<3>>(self),
            0x57 => instr::modify::<addr::ZeroPage, op::Rmb<5>>(self),
            0x77 => instr::modify::<addr::ZeroPage, op::Rmb<7>>(self),
            0x97 => instr::modify::<addr::ZeroPage, op::Smb<1>>(self),
            0xb7 => instr::modify::<addr::ZeroPage, op::Smb<3>>(self),
            0xd7 => instr::modify::<addr::ZeroPage, op::Smb<5>>(self),
            0xf7 => instr::modify::<addr::ZeroPage, op::Smb<7>>(self),

            // +0x0f
            0x0f => instr::branch::<op::Bbr<0>>(self),
            0x2f => instr::branch::<op::Bbr<2>>(self),
            0x4f => instr::branch::<op::Bbr<4>>(self),
            0x6f => instr::branch::<op::Bbr<6>>(self),
            0x8f => instr::branch::<op::Bbs<0>>(self),
            0xaf => instr::branch::<op::Bbs<2>>(self),
            0xcf => instr::branch::<op::Bbs<4>>(self),
            0xef => instr::branch::<op::Bbs<6>>(self),

            // +0x1f
            0x1f => instr::branch::<op::Bbr<1>>(self),
            0x3f => instr::branch::<op::Bbr<3>>(self),
            0x5f => instr::branch::<op::Bbr<5>>(self),
            0x7f => instr::branch::<op::Bbr<7>>(self),
            0x9f => instr::branch::<op::Bbs<1>>(self),
            0xbf => instr::branch::<op::Bbs<3>>(self),
            0xdf => instr::branch::<op::Bbs<5>>(self),
            0xff => instr::branch::<op::Bbs<7>>(self),

            opcode => panic!("Opcode {:02X} not yet implemented", opcode),
        }
    }

    fn poll(&mut self) {
        self.interrupt = self.bus.poll() & (self.flags.i as Interrupt);
    }

    fn map(&self, address: u16) -> u32 {
        self.mpr[address as usize >> 13] | (address as u32 & 0x1fff)
    }

    fn read_physical(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:06X} => {:02X}", address, value);
        value
    }

    fn write_physical(&mut self, address: u32, value: u8) {
        debug!("  {:06X} <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn read(&mut self, address: u16) -> u8 {
        self.read_physical(self.map(address))
    }

    fn pull(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read_physical(STACK_PAGE | (self.s as u32))
    }

    fn push(&mut self, value: u8) {
        self.write_physical(STACK_PAGE | (self.s as u32), value);
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

    fn flags_from_u8(&mut self, value: u8) {
        self.flags.n = value;
        self.flags.v = value << 1;
        self.flags.d = (value & 0x08) != 0;
        self.flags.i = if (value & 0x04) != 0 {
            IrqDisable::Set
        } else {
            IrqDisable::Clear
        };
        self.flags.z = !value & 0x02;
        self.flags.c = (value & 0x01) != 0;
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
