use std::fmt;
use std::mem;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

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
    fn idle(&mut self);
    fn read(&mut self, address: u32) -> u8;
    fn write(&mut self, address: u32, value: u8);
    fn poll(&self) -> Interrupt;
    fn acknowledge(&mut self, interrupt: Interrupt);
}

pub struct Flags {
    n: bool,
    v: bool,
    d: bool,
    i: IrqDisable,
    z: u16,
    c: bool,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Mode {
    Native11 = 0,
    Native10 = 1,
    Native01 = 2,
    Native00 = 3,
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
                n: false,
                v: false,
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

    pub fn bus(&self) -> &T {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut T {
        &mut self.bus
    }

    pub fn step(&mut self) {
        match self.mode {
            Mode::Native11 => self.dispatch::<false, true, true>(),
            Mode::Native10 => self.dispatch::<false, true, false>(),
            Mode::Native01 => self.dispatch::<false, false, true>(),
            Mode::Native00 => self.dispatch::<false, false, false>(),
            Mode::Emulation => self.dispatch::<true, true, true>(),
        }
    }

    fn dispatch<const E: bool, const M: bool, const X: bool>(&mut self) {
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
                instr::nmi::<E>(self);
            } else {
                panic!("Interrupt not yet implemented");
            }

            self.interrupt = 0;
            return;
        }

        match self.next_byte() {
            // Page 0: Control Ops

            // +0x00
            //0x00 => instr::brk(self),
            0x20 => instr::jsr::<E>(self),
            0x40 => instr::rti::<E>(self),
            0x60 => instr::rts::<E>(self),
            0x80 => instr::branch::<E, op::Bra>(self),
            0xa0 => instr::immediate::<X, op::Ldy>(self),
            0xc0 => instr::immediate::<X, op::Cpy>(self),
            0xe0 => instr::immediate::<X, op::Cpx>(self),

            // +0x10
            0x10 => instr::branch::<E, op::Bpl>(self),
            0x30 => instr::branch::<E, op::Bmi>(self),
            0x50 => instr::branch::<E, op::Bvc>(self),
            0x70 => instr::branch::<E, op::Bvs>(self),
            0x90 => instr::branch::<E, op::Bcc>(self),
            0xb0 => instr::branch::<E, op::Bcs>(self),
            0xd0 => instr::branch::<E, op::Bne>(self),
            0xf0 => instr::branch::<E, op::Beq>(self),

            // +0x04
            0x04 => instr::modify::<M, addr::Direct, op::Tsb>(self),
            0x24 => instr::read::<M, addr::Direct, op::Bit>(self),
            //0x44 => instr::read::<addr::Direct, op::Nop>(self),
            0x64 => instr::write::<M, addr::Direct, op::Stz>(self),
            0x84 => instr::write::<X, addr::Direct, op::Sty>(self),
            0xa4 => instr::read::<X, addr::Direct, op::Ldy>(self),
            0xc4 => instr::read::<X, addr::Direct, op::Cpy>(self),
            0xe4 => instr::read::<X, addr::Direct, op::Cpx>(self),

            // +0x14
            0x14 => instr::modify::<M, addr::Direct, op::Trb>(self),
            0x34 => instr::read::<M, addr::DirectX<E>, op::Bit>(self),
            //0x54 => instr::read::<addr::DirectX<E>, op::Nop>(self),
            0x74 => instr::write::<M, addr::DirectX<E>, op::Stz>(self),
            0x94 => instr::write::<X, addr::DirectX<E>, op::Sty>(self),
            0xb4 => instr::read::<X, addr::DirectX<E>, op::Ldy>(self),
            0xd4 => instr::pei::<E>(self),
            0xf4 => instr::pea::<E>(self),

            // +0x08
            0x08 => instr::php::<E>(self),
            0x28 => instr::plp::<E>(self),
            0x48 => instr::pha::<E, M>(self),
            0x68 => instr::pla::<E, M>(self),
            0x88 => instr::dey::<X>(self),
            0xa8 => instr::tay::<X>(self),
            0xc8 => instr::iny::<X>(self),
            0xe8 => instr::inx::<X>(self),

            // +0x18
            0x18 => instr::clc(self),
            0x38 => instr::sec(self),
            0x58 => instr::cli(self),
            0x78 => instr::sei(self),
            0x98 => instr::tya::<M>(self),
            0xb8 => instr::clv(self),
            0xd8 => instr::cld(self),
            0xf8 => instr::sed(self),

            // +0x0c
            0x0c => instr::modify::<M, addr::Absolute, op::Tsb>(self),
            0x2c => instr::read::<M, addr::Absolute, op::Bit>(self),
            0x4c => instr::jmp(self),
            0x6c => instr::jmp_indirect(self),
            0x8c => instr::write::<X, addr::Absolute, op::Sty>(self),
            0xac => instr::read::<X, addr::Absolute, op::Ldy>(self),
            0xcc => instr::read::<X, addr::Absolute, op::Cpy>(self),
            0xec => instr::read::<X, addr::Absolute, op::Cpx>(self),

            // +0x1c
            0x1c => instr::modify::<M, addr::Absolute, op::Trb>(self),
            0x3c => instr::read::<M, addr::AbsoluteX<X>, op::Bit>(self),
            0x5c => instr::jmp_long(self),
            0x7c => instr::jmp_x_indirect(self),
            0x9c => instr::write::<M, addr::Absolute, op::Stz>(self),
            0xbc => instr::read::<X, addr::AbsoluteX<X>, op::Ldy>(self),
            0xdc => instr::jmp_indirect_long(self),
            0xfc => instr::jsr_x_indirect::<E>(self),

            // Page 1: Accumulator Ops

            // +0x01
            0x01 => instr::read::<M, addr::DirectXIndirect<E>, op::Ora>(self),
            0x21 => instr::read::<M, addr::DirectXIndirect<E>, op::And>(self),
            0x41 => instr::read::<M, addr::DirectXIndirect<E>, op::Eor>(self),
            0x61 => instr::read::<M, addr::DirectXIndirect<E>, op::Adc>(self),
            0x81 => instr::write::<M, addr::DirectXIndirect<E>, op::Sta>(self),
            0xa1 => instr::read::<M, addr::DirectXIndirect<E>, op::Lda>(self),
            0xc1 => instr::read::<M, addr::DirectXIndirect<E>, op::Cmp>(self),
            0xe1 => instr::read::<M, addr::DirectXIndirect<E>, op::Sbc>(self),

            // +0x11
            0x11 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Ora>(self),
            0x31 => instr::read::<M, addr::DirectIndirectY<E, X>, op::And>(self),
            0x51 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Eor>(self),
            0x71 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Adc>(self),
            0x91 => instr::write::<M, addr::DirectIndirectY<E, X>, op::Sta>(self),
            0xb1 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Lda>(self),
            0xd1 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Cmp>(self),
            0xf1 => instr::read::<M, addr::DirectIndirectY<E, X>, op::Sbc>(self),

            // +0x05
            0x05 => instr::read::<M, addr::Direct, op::Ora>(self),
            0x25 => instr::read::<M, addr::Direct, op::And>(self),
            0x45 => instr::read::<M, addr::Direct, op::Eor>(self),
            0x65 => instr::read::<M, addr::Direct, op::Adc>(self),
            0x85 => instr::write::<M, addr::Direct, op::Sta>(self),
            0xa5 => instr::read::<M, addr::Direct, op::Lda>(self),
            0xc5 => instr::read::<M, addr::Direct, op::Cmp>(self),
            0xe5 => instr::read::<M, addr::Direct, op::Sbc>(self),

            // +0x15
            0x15 => instr::read::<M, addr::DirectX<E>, op::Ora>(self),
            0x35 => instr::read::<M, addr::DirectX<E>, op::And>(self),
            0x55 => instr::read::<M, addr::DirectX<E>, op::Eor>(self),
            0x75 => instr::read::<M, addr::DirectX<E>, op::Adc>(self),
            0x95 => instr::write::<M, addr::DirectX<E>, op::Sta>(self),
            0xb5 => instr::read::<M, addr::DirectX<E>, op::Lda>(self),
            0xd5 => instr::read::<M, addr::DirectX<E>, op::Cmp>(self),
            0xf5 => instr::read::<M, addr::DirectX<E>, op::Sbc>(self),

            // +0x09
            0x09 => instr::immediate::<M, op::Ora>(self),
            0x29 => instr::immediate::<M, op::And>(self),
            0x49 => instr::immediate::<M, op::Eor>(self),
            0x69 => instr::immediate::<M, op::Adc>(self),
            0x89 => instr::immediate::<M, op::BitImmediate>(self),
            0xa9 => instr::immediate::<M, op::Lda>(self),
            0xc9 => instr::immediate::<M, op::Cmp>(self),
            0xe9 => instr::immediate::<M, op::Sbc>(self),

            // +0x19
            0x19 => instr::read::<M, addr::AbsoluteY<X>, op::Ora>(self),
            0x39 => instr::read::<M, addr::AbsoluteY<X>, op::And>(self),
            0x59 => instr::read::<M, addr::AbsoluteY<X>, op::Eor>(self),
            0x79 => instr::read::<M, addr::AbsoluteY<X>, op::Adc>(self),
            0x99 => instr::write::<M, addr::AbsoluteY<X>, op::Sta>(self),
            0xb9 => instr::read::<M, addr::AbsoluteY<X>, op::Lda>(self),
            0xd9 => instr::read::<M, addr::AbsoluteY<X>, op::Cmp>(self),
            0xf9 => instr::read::<M, addr::AbsoluteY<X>, op::Sbc>(self),

            // +0x0d
            0x0d => instr::read::<M, addr::Absolute, op::Ora>(self),
            0x2d => instr::read::<M, addr::Absolute, op::And>(self),
            0x4d => instr::read::<M, addr::Absolute, op::Eor>(self),
            0x6d => instr::read::<M, addr::Absolute, op::Adc>(self),
            0x8d => instr::write::<M, addr::Absolute, op::Sta>(self),
            0xad => instr::read::<M, addr::Absolute, op::Lda>(self),
            0xcd => instr::read::<M, addr::Absolute, op::Cmp>(self),
            0xed => instr::read::<M, addr::Absolute, op::Sbc>(self),

            // +0x1d
            0x1d => instr::read::<M, addr::AbsoluteX<X>, op::Ora>(self),
            0x3d => instr::read::<M, addr::AbsoluteX<X>, op::And>(self),
            0x5d => instr::read::<M, addr::AbsoluteX<X>, op::Eor>(self),
            0x7d => instr::read::<M, addr::AbsoluteX<X>, op::Adc>(self),
            0x9d => instr::write::<M, addr::AbsoluteX<X>, op::Sta>(self),
            0xbd => instr::read::<M, addr::AbsoluteX<X>, op::Lda>(self),
            0xdd => instr::read::<M, addr::AbsoluteX<X>, op::Cmp>(self),
            0xfd => instr::read::<M, addr::AbsoluteX<X>, op::Sbc>(self),

            // Page 2: Read-Modify-Write Ops

            // +0x02
            0x22 => instr::jsl::<E>(self),
            0x62 => instr::per::<E>(self),
            0x82 => instr::brl(self),
            0xa2 => instr::immediate::<X, op::Ldx>(self),
            0xc2 => instr::rep::<E>(self),
            0xe2 => instr::sep::<E>(self),

            // +0x12
            0x12 => instr::read::<M, addr::DirectIndirect<E>, op::Ora>(self),
            0x32 => instr::read::<M, addr::DirectIndirect<E>, op::And>(self),
            0x52 => instr::read::<M, addr::DirectIndirect<E>, op::Eor>(self),
            0x72 => instr::read::<M, addr::DirectIndirect<E>, op::Adc>(self),
            0x92 => instr::write::<M, addr::DirectIndirect<E>, op::Sta>(self),
            0xb2 => instr::read::<M, addr::DirectIndirect<E>, op::Lda>(self),
            0xd2 => instr::read::<M, addr::DirectIndirect<E>, op::Cmp>(self),
            0xf2 => instr::read::<M, addr::DirectIndirect<E>, op::Sbc>(self),

            // +0x06
            0x06 => instr::modify::<M, addr::Direct, op::Asl>(self),
            0x26 => instr::modify::<M, addr::Direct, op::Rol>(self),
            0x46 => instr::modify::<M, addr::Direct, op::Lsr>(self),
            0x66 => instr::modify::<M, addr::Direct, op::Ror>(self),
            0x86 => instr::write::<X, addr::Direct, op::Stx>(self),
            0xa6 => instr::read::<X, addr::Direct, op::Ldx>(self),
            0xc6 => instr::modify::<M, addr::Direct, op::Dec>(self),
            0xe6 => instr::modify::<M, addr::Direct, op::Inc>(self),

            // +0x16
            0x16 => instr::modify::<M, addr::DirectX<E>, op::Asl>(self),
            0x36 => instr::modify::<M, addr::DirectX<E>, op::Rol>(self),
            0x56 => instr::modify::<M, addr::DirectX<E>, op::Lsr>(self),
            0x76 => instr::modify::<M, addr::DirectX<E>, op::Ror>(self),
            0x96 => instr::write::<X, addr::DirectY<E>, op::Stx>(self),
            0xb6 => instr::read::<X, addr::DirectY<E>, op::Ldx>(self),
            0xd6 => instr::modify::<M, addr::DirectX<E>, op::Dec>(self),
            0xf6 => instr::modify::<M, addr::DirectX<E>, op::Inc>(self),

            // +0x0a
            0x0a => instr::accumulator::<M, op::Asl>(self),
            0x2a => instr::accumulator::<M, op::Rol>(self),
            0x4a => instr::accumulator::<M, op::Lsr>(self),
            0x6a => instr::accumulator::<M, op::Ror>(self),
            0x8a => instr::txa::<M>(self),
            0xaa => instr::tax::<X>(self),
            0xca => instr::dex::<X>(self),
            0xea => instr::nop(self),

            // +0x1a
            0x1a => instr::accumulator::<M, op::Inc>(self),
            0x3a => instr::accumulator::<M, op::Dec>(self),
            0x5a => instr::phy::<E, X>(self),
            0x7a => instr::ply::<E, X>(self),
            0x9a => instr::txs::<E>(self),
            0xba => instr::tsx::<X>(self),
            0xda => instr::phx::<E, X>(self),
            0xfa => instr::plx::<E, X>(self),

            // +0x0e
            0x0e => instr::modify::<M, addr::Absolute, op::Asl>(self),
            0x2e => instr::modify::<M, addr::Absolute, op::Rol>(self),
            0x4e => instr::modify::<M, addr::Absolute, op::Lsr>(self),
            0x6e => instr::modify::<M, addr::Absolute, op::Ror>(self),
            0x8e => instr::write::<X, addr::Absolute, op::Stx>(self),
            0xae => instr::read::<X, addr::Absolute, op::Ldx>(self),
            0xce => instr::modify::<M, addr::Absolute, op::Dec>(self),
            0xee => instr::modify::<M, addr::Absolute, op::Inc>(self),

            // +0x1e
            0x1e => instr::modify::<M, addr::AbsoluteX<X>, op::Asl>(self),
            0x3e => instr::modify::<M, addr::AbsoluteX<X>, op::Rol>(self),
            0x5e => instr::modify::<M, addr::AbsoluteX<X>, op::Lsr>(self),
            0x7e => instr::modify::<M, addr::AbsoluteX<X>, op::Ror>(self),
            0x9e => instr::write::<M, addr::AbsoluteX<X>, op::Stz>(self),
            0xbe => instr::read::<X, addr::AbsoluteY<X>, op::Ldx>(self),
            0xde => instr::modify::<M, addr::AbsoluteX<X>, op::Dec>(self),
            0xfe => instr::modify::<M, addr::AbsoluteX<X>, op::Inc>(self),

            // +0x0b
            0x0b => instr::phd::<E>(self),
            0x2b => instr::pld::<E>(self),
            0x4b => instr::phk::<E>(self),
            0x6b => instr::rtl::<E>(self),
            0x8b => instr::phb::<E>(self),
            0xab => instr::plb::<E>(self),
            0xeb => instr::xba(self),

            // +0x1b
            0x1b => instr::tcs::<E>(self),
            0x3b => instr::tsc(self),
            0x5b => instr::tcd(self),
            0x7b => instr::tdc(self),
            0x9b => instr::txy::<X>(self),
            0xbb => instr::tyx::<X>(self),
            0xfb => instr::xce(self),

            // +0x03
            0x03 => instr::read::<M, addr::StackRelative, op::Ora>(self),
            0x23 => instr::read::<M, addr::StackRelative, op::And>(self),
            0x43 => instr::read::<M, addr::StackRelative, op::Eor>(self),
            0x63 => instr::read::<M, addr::StackRelative, op::Adc>(self),
            0x83 => instr::write::<M, addr::StackRelative, op::Sta>(self),
            0xa3 => instr::read::<M, addr::StackRelative, op::Lda>(self),
            0xc3 => instr::read::<M, addr::StackRelative, op::Cmp>(self),
            0xe3 => instr::read::<M, addr::StackRelative, op::Sbc>(self),

            // +0x13
            0x13 => instr::read::<M, addr::StackRelativeIndirectY, op::Ora>(self),
            0x33 => instr::read::<M, addr::StackRelativeIndirectY, op::And>(self),
            0x53 => instr::read::<M, addr::StackRelativeIndirectY, op::Eor>(self),
            0x73 => instr::read::<M, addr::StackRelativeIndirectY, op::Adc>(self),
            0x93 => instr::write::<M, addr::StackRelativeIndirectY, op::Sta>(self),
            0xb3 => instr::read::<M, addr::StackRelativeIndirectY, op::Lda>(self),
            0xd3 => instr::read::<M, addr::StackRelativeIndirectY, op::Cmp>(self),
            0xf3 => instr::read::<M, addr::StackRelativeIndirectY, op::Sbc>(self),

            // +0x07
            0x07 => instr::read::<M, addr::DirectIndirectLong, op::Ora>(self),
            0x27 => instr::read::<M, addr::DirectIndirectLong, op::And>(self),
            0x47 => instr::read::<M, addr::DirectIndirectLong, op::Eor>(self),
            0x67 => instr::read::<M, addr::DirectIndirectLong, op::Adc>(self),
            0x87 => instr::write::<M, addr::DirectIndirectLong, op::Sta>(self),
            0xa7 => instr::read::<M, addr::DirectIndirectLong, op::Lda>(self),
            0xc7 => instr::read::<M, addr::DirectIndirectLong, op::Cmp>(self),
            0xe7 => instr::read::<M, addr::DirectIndirectLong, op::Sbc>(self),

            // +0x17
            0x17 => instr::read::<M, addr::DirectIndirectLongY, op::Ora>(self),
            0x37 => instr::read::<M, addr::DirectIndirectLongY, op::And>(self),
            0x57 => instr::read::<M, addr::DirectIndirectLongY, op::Eor>(self),
            0x77 => instr::read::<M, addr::DirectIndirectLongY, op::Adc>(self),
            0x97 => instr::write::<M, addr::DirectIndirectLongY, op::Sta>(self),
            0xb7 => instr::read::<M, addr::DirectIndirectLongY, op::Lda>(self),
            0xd7 => instr::read::<M, addr::DirectIndirectLongY, op::Cmp>(self),
            0xf7 => instr::read::<M, addr::DirectIndirectLongY, op::Sbc>(self),

            // +0x0f
            0x0f => instr::read::<M, addr::AbsoluteLong, op::Ora>(self),
            0x2f => instr::read::<M, addr::AbsoluteLong, op::And>(self),
            0x4f => instr::read::<M, addr::AbsoluteLong, op::Eor>(self),
            0x6f => instr::read::<M, addr::AbsoluteLong, op::Adc>(self),
            0x8f => instr::write::<M, addr::AbsoluteLong, op::Sta>(self),
            0xaf => instr::read::<M, addr::AbsoluteLong, op::Lda>(self),
            0xcf => instr::read::<M, addr::AbsoluteLong, op::Cmp>(self),
            0xef => instr::read::<M, addr::AbsoluteLong, op::Sbc>(self),

            // +0x1f
            0x1f => instr::read::<M, addr::AbsoluteLongX, op::Ora>(self),
            0x3f => instr::read::<M, addr::AbsoluteLongX, op::And>(self),
            0x5f => instr::read::<M, addr::AbsoluteLongX, op::Eor>(self),
            0x7f => instr::read::<M, addr::AbsoluteLongX, op::Adc>(self),
            0x9f => instr::write::<M, addr::AbsoluteLongX, op::Sta>(self),
            0xbf => instr::read::<M, addr::AbsoluteLongX, op::Lda>(self),
            0xdf => instr::read::<M, addr::AbsoluteLongX, op::Cmp>(self),
            0xff => instr::read::<M, addr::AbsoluteLongX, op::Sbc>(self),

            opcode => todo!("WDC65c816 opcode {:02X}", opcode),
        }
    }

    fn poll(&mut self) {
        self.interrupt = self.bus.poll() & (self.flags.i as Interrupt);
    }

    fn idle(&mut self) {
        debug!("  IO");
        self.bus.idle();
    }

    fn read(&mut self, address: u32) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:06X} => {:02X}", address, value);
        value
    }

    fn write(&mut self, address: u32, value: u8) {
        debug!("  {:06X} <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn pull<const E: bool>(&mut self) -> u8 {
        if E {
            self.s = (self.s & 0xff00) | (self.s.wrapping_add(1) & 0xff);
        } else {
            self.s = self.s.wrapping_add(1);
        }

        self.read(self.s as u32)
    }

    fn push<const E: bool>(&mut self, value: u8) {
        self.write(self.s as u32, value);

        if E {
            self.s = (self.s & 0xff00) | (self.s.wrapping_sub(1) & 0xff);
        } else {
            self.s = self.s.wrapping_sub(1);
        }
    }

    fn next_byte(&mut self) -> u8 {
        let value = self.read(self.pc);
        self.pc = (self.pc & 0xffff0000) | ((self.pc.wrapping_add(1)) & 0xffff);
        value
    }

    fn next_word(&mut self) -> u16 {
        let low = self.next_byte();
        let high = self.next_byte();
        u16::from_le_bytes([low, high])
    }

    fn next_long(&mut self) -> u32 {
        let low = self.next_byte();
        let high = self.next_byte();
        let bank = self.next_byte();
        u32::from_le_bytes([low, high, bank, 0])
    }

    fn flags_to_u8<const E: bool>(&self, break_flag: bool) -> u8 {
        let mut result = 0;
        result |= if self.flags.n { 0x80 } else { 0 };
        result |= if self.flags.v { 0x40 } else { 0 };
        result |= if self.flags.d { 0x08 } else { 0 };
        result |= if self.flags.i == IrqDisable::Set {
            0x04
        } else {
            0
        };
        result |= if self.flags.z == 0 { 0x02 } else { 0 };
        result |= self.flags.c as u8;

        if E {
            result |= if break_flag { 0x30 } else { 0x20 };
        } else {
            result |= !((self.mode as u8) << 4) & 0x30;
        }

        result
    }

    fn flags_from_u8<const E: bool>(&mut self, value: u8) {
        self.flags.n = (value & 0x80) != 0;
        self.flags.v = (value & 0x40) != 0;
        self.flags.d = (value & 0x08) != 0;
        self.flags.i = if (value & 0x04) != 0 {
            IrqDisable::Set
        } else {
            IrqDisable::Clear
        };
        self.flags.z = (!value & 0x02) as u16;
        self.flags.c = (value & 0x01) != 0;

        if !E {
            let mode_value = (!value & 0x30) >> 4;

            unsafe { self.mode = mem::transmute(mode_value) };

            if (value & 0x10) != 0 {
                self.x &= 0xff;
                self.y &= 0xff;
            }
        }
    }

    fn set_nz8(&mut self, value: u8) {
        self.flags.n = (value & 0x80) != 0;
        self.flags.z = value as u16;
    }

    fn set_nz16(&mut self, value: u16) {
        self.flags.n = (value & 0x8000) != 0;
        self.flags.z = value;
    }
}

impl<T: Bus> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A={:04X} X={:04X} Y={:04X} D={:04X} S={:04X} PC={:06X} DBR={:02X} P={}{}{}{}{}{}{}{}{} {}",
            self.a,
            self.x,
            self.y,
            self.d,
            self.s,
            self.pc,
            self.dbr >> 16,
            if self.flags.n { 'N' } else { '-' },
            if self.flags.v { 'V' } else { '-' },
            if (self.mode as u8 & 0x02) == 0 { 'M' } else { '-' },
            if (self.mode as u8 & 0x01) == 0 { 'X' } else { '-' },
            if self.flags.d { 'D' } else { '-' },
            if self.flags.i == IrqDisable::Set { 'I' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            if (self.mode as u8 & 0x04) != 0 { 'E' } else { '-' },
            self.bus,
        )
    }
}
