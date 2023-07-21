use std::fmt;
use tracing::debug;

mod address_mode;
mod instruction;
mod operator;

const STACK_PAGE: u16 = 0x0100;

pub trait Bus: fmt::Display {
    fn idle(&mut self);
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub struct Flags {
    n: u8,
    v: bool,
    p: u16,
    b: bool,
    h: bool,
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
                v: false,
                p: 0,
                b: false,
                h: false,
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
            // +0x00
            0x00 => instr::nop(self),
            0x20 => instr::clrp(self),
            0x40 => instr::setp(self),
            0x60 => instr::clrc(self),
            0x80 => instr::setc(self),
            0xa0 => instr::ei(self),
            0xc0 => instr::di(self),
            0xe0 => instr::clrv(self),

            // +0x10
            0x10 => instr::branch::<op::Bpl>(self),
            0x30 => instr::branch::<op::Bmi>(self),
            0x50 => instr::branch::<op::Bvc>(self),
            0x70 => instr::branch::<op::Bvs>(self),
            0x90 => instr::branch::<op::Bcc>(self),
            0xb0 => instr::branch::<op::Bcs>(self),
            0xd0 => instr::branch::<op::Bne>(self),
            0xf0 => instr::branch::<op::Beq>(self),

            // +0x02
            0x02 => instr::unary::<addr::Direct, op::Set1<0>>(self),
            0x22 => instr::unary::<addr::Direct, op::Set1<1>>(self),
            0x42 => instr::unary::<addr::Direct, op::Set1<2>>(self),
            0x62 => instr::unary::<addr::Direct, op::Set1<3>>(self),
            0x82 => instr::unary::<addr::Direct, op::Set1<4>>(self),
            0xa2 => instr::unary::<addr::Direct, op::Set1<5>>(self),
            0xc2 => instr::unary::<addr::Direct, op::Set1<6>>(self),
            0xe2 => instr::unary::<addr::Direct, op::Set1<7>>(self),

            // +0x12
            0x12 => instr::unary::<addr::Direct, op::Clr1<0>>(self),
            0x32 => instr::unary::<addr::Direct, op::Clr1<1>>(self),
            0x52 => instr::unary::<addr::Direct, op::Clr1<2>>(self),
            0x72 => instr::unary::<addr::Direct, op::Clr1<3>>(self),
            0x92 => instr::unary::<addr::Direct, op::Clr1<4>>(self),
            0xb2 => instr::unary::<addr::Direct, op::Clr1<5>>(self),
            0xd2 => instr::unary::<addr::Direct, op::Clr1<6>>(self),
            0xf2 => instr::unary::<addr::Direct, op::Clr1<7>>(self),

            // +0x03
            0x03 => instr::branch::<op::Bbs<0>>(self),
            0x23 => instr::branch::<op::Bbs<1>>(self),
            0x43 => instr::branch::<op::Bbs<2>>(self),
            0x63 => instr::branch::<op::Bbs<3>>(self),
            0x83 => instr::branch::<op::Bbs<4>>(self),
            0xa3 => instr::branch::<op::Bbs<5>>(self),
            0xc3 => instr::branch::<op::Bbs<6>>(self),
            0xe3 => instr::branch::<op::Bbs<7>>(self),

            // +0x13
            0x13 => instr::branch::<op::Bbc<0>>(self),
            0x33 => instr::branch::<op::Bbc<1>>(self),
            0x53 => instr::branch::<op::Bbc<2>>(self),
            0x73 => instr::branch::<op::Bbc<3>>(self),
            0x93 => instr::branch::<op::Bbc<4>>(self),
            0xb3 => instr::branch::<op::Bbc<5>>(self),
            0xd3 => instr::branch::<op::Bbc<6>>(self),
            0xf3 => instr::branch::<op::Bbc<7>>(self),

            // +0x04
            0x04 => instr::binary::<addr::A, addr::Direct, op::Or>(self),
            0x24 => instr::binary::<addr::A, addr::Direct, op::And>(self),
            0x44 => instr::binary::<addr::A, addr::Direct, op::Eor>(self),
            0x64 => instr::compare::<addr::A, addr::Direct>(self),
            0x84 => instr::binary::<addr::A, addr::Direct, op::Adc>(self),
            0xa4 => instr::binary::<addr::A, addr::Direct, op::Sbc>(self),
            0xc4 => instr::write::<addr::Direct, addr::A>(self),
            0xe4 => instr::binary::<addr::A, addr::Direct, op::Mov>(self),

            // +0x14
            0x14 => instr::binary::<addr::A, addr::DirectX, op::Or>(self),
            0x34 => instr::binary::<addr::A, addr::DirectX, op::And>(self),
            0x54 => instr::binary::<addr::A, addr::DirectX, op::Eor>(self),
            0x74 => instr::compare::<addr::A, addr::DirectX>(self),
            0x94 => instr::binary::<addr::A, addr::DirectX, op::Adc>(self),
            0xb4 => instr::binary::<addr::A, addr::DirectX, op::Sbc>(self),
            0xd4 => instr::write::<addr::DirectX, addr::A>(self),
            0xf4 => instr::binary::<addr::A, addr::DirectX, op::Mov>(self),

            // +0x05
            0x05 => instr::binary::<addr::A, addr::Absolute, op::Or>(self),
            0x25 => instr::binary::<addr::A, addr::Absolute, op::And>(self),
            0x45 => instr::binary::<addr::A, addr::Absolute, op::Eor>(self),
            0x65 => instr::compare::<addr::A, addr::Absolute>(self),
            0x85 => instr::binary::<addr::A, addr::Absolute, op::Adc>(self),
            0xa5 => instr::binary::<addr::A, addr::Absolute, op::Sbc>(self),
            0xc5 => instr::write::<addr::Absolute, addr::A>(self),
            0xe5 => instr::binary::<addr::A, addr::Absolute, op::Mov>(self),

            // +0x15
            0x15 => instr::binary::<addr::A, addr::AbsoluteX, op::Or>(self),
            0x35 => instr::binary::<addr::A, addr::AbsoluteX, op::And>(self),
            0x55 => instr::binary::<addr::A, addr::AbsoluteX, op::Eor>(self),
            0x75 => instr::compare::<addr::A, addr::AbsoluteX>(self),
            0x95 => instr::binary::<addr::A, addr::AbsoluteX, op::Adc>(self),
            0xb5 => instr::binary::<addr::A, addr::AbsoluteX, op::Sbc>(self),
            0xd5 => instr::write::<addr::AbsoluteX, addr::A>(self),
            0xf5 => instr::binary::<addr::A, addr::AbsoluteX, op::Mov>(self),

            // +0x06
            0x06 => instr::binary::<addr::A, addr::XIndirect, op::Or>(self),
            0x26 => instr::binary::<addr::A, addr::XIndirect, op::And>(self),
            0x46 => instr::binary::<addr::A, addr::XIndirect, op::Eor>(self),
            0x66 => instr::compare::<addr::A, addr::XIndirect>(self),
            0x86 => instr::binary::<addr::A, addr::XIndirect, op::Adc>(self),
            0xa6 => instr::binary::<addr::A, addr::XIndirect, op::Sbc>(self),
            0xc6 => instr::write::<addr::XIndirect, addr::A>(self),
            0xe6 => instr::binary::<addr::A, addr::XIndirect, op::Mov>(self),

            // +0x16
            0x16 => instr::binary::<addr::A, addr::AbsoluteY, op::Or>(self),
            0x36 => instr::binary::<addr::A, addr::AbsoluteY, op::And>(self),
            0x56 => instr::binary::<addr::A, addr::AbsoluteY, op::Eor>(self),
            0x76 => instr::compare::<addr::A, addr::AbsoluteY>(self),
            0x96 => instr::binary::<addr::A, addr::AbsoluteY, op::Adc>(self),
            0xb6 => instr::binary::<addr::A, addr::AbsoluteY, op::Sbc>(self),
            0xd6 => instr::write::<addr::AbsoluteY, addr::A>(self),
            0xf6 => instr::binary::<addr::A, addr::AbsoluteY, op::Mov>(self),

            // +0x07
            0x07 => instr::binary::<addr::A, addr::DirectXIndirect, op::Or>(self),
            0x27 => instr::binary::<addr::A, addr::DirectXIndirect, op::And>(self),
            0x47 => instr::binary::<addr::A, addr::DirectXIndirect, op::Eor>(self),
            0x67 => instr::compare::<addr::A, addr::DirectXIndirect>(self),
            0x87 => instr::binary::<addr::A, addr::DirectXIndirect, op::Adc>(self),
            0xa7 => instr::binary::<addr::A, addr::DirectXIndirect, op::Sbc>(self),
            0xc7 => instr::write::<addr::DirectXIndirect, addr::A>(self),
            0xe7 => instr::binary::<addr::A, addr::DirectXIndirect, op::Mov>(self),

            // +0x17
            0x17 => instr::binary::<addr::A, addr::DirectIndirectY, op::Or>(self),
            0x37 => instr::binary::<addr::A, addr::DirectIndirectY, op::And>(self),
            0x57 => instr::binary::<addr::A, addr::DirectIndirectY, op::Eor>(self),
            0x77 => instr::compare::<addr::A, addr::DirectIndirectY>(self),
            0x97 => instr::binary::<addr::A, addr::DirectIndirectY, op::Adc>(self),
            0xb7 => instr::binary::<addr::A, addr::DirectIndirectY, op::Sbc>(self),
            0xd7 => instr::write::<addr::DirectIndirectY, addr::A>(self),
            0xf7 => instr::binary::<addr::A, addr::DirectIndirectY, op::Mov>(self),

            // +0x08
            0x08 => instr::binary::<addr::A, addr::Immediate, op::Or>(self),
            0x28 => instr::binary::<addr::A, addr::Immediate, op::And>(self),
            0x48 => instr::binary::<addr::A, addr::Immediate, op::Eor>(self),
            0x68 => instr::compare::<addr::A, addr::Immediate>(self),
            0x88 => instr::binary::<addr::A, addr::Immediate, op::Adc>(self),
            0xa8 => instr::binary::<addr::A, addr::Immediate, op::Sbc>(self),
            0xc8 => instr::compare::<addr::X, addr::Immediate>(self),
            0xe8 => instr::binary::<addr::A, addr::Immediate, op::Mov>(self),

            // +0x18
            0x18 => instr::binary::<addr::Direct, addr::Immediate, op::Or>(self),
            0x38 => instr::binary::<addr::Direct, addr::Immediate, op::And>(self),
            0x58 => instr::binary::<addr::Direct, addr::Immediate, op::Eor>(self),
            0x78 => instr::compare::<addr::Direct, addr::Immediate>(self),
            0x98 => instr::binary::<addr::Direct, addr::Immediate, op::Adc>(self),
            0xb8 => instr::binary::<addr::Direct, addr::Immediate, op::Sbc>(self),
            0xd8 => instr::write::<addr::Direct, addr::X>(self),
            0xf8 => instr::binary::<addr::X, addr::Direct, op::Mov>(self),

            // +0x09
            0x09 => instr::binary::<addr::Direct, addr::Direct, op::Or>(self),
            0x29 => instr::binary::<addr::Direct, addr::Direct, op::And>(self),
            0x49 => instr::binary::<addr::Direct, addr::Direct, op::Eor>(self),
            0x69 => instr::compare::<addr::Direct, addr::Direct>(self),
            0x89 => instr::binary::<addr::Direct, addr::Direct, op::Adc>(self),
            0xa9 => instr::binary::<addr::Direct, addr::Direct, op::Sbc>(self),
            0xc9 => instr::write::<addr::Absolute, addr::X>(self),
            0xe9 => instr::binary::<addr::X, addr::Absolute, op::Mov>(self),

            // +0x19
            0x19 => instr::binary::<addr::XIndirect, addr::YIndirect, op::Or>(self),
            0x39 => instr::binary::<addr::XIndirect, addr::YIndirect, op::And>(self),
            0x59 => instr::binary::<addr::XIndirect, addr::YIndirect, op::Eor>(self),
            0x79 => instr::compare::<addr::XIndirect, addr::YIndirect>(self),
            0x99 => instr::binary::<addr::XIndirect, addr::YIndirect, op::Adc>(self),
            0xb9 => instr::binary::<addr::XIndirect, addr::YIndirect, op::Sbc>(self),
            0xd9 => instr::write::<addr::DirectY, addr::X>(self),
            0xf9 => instr::binary::<addr::X, addr::DirectY, op::Mov>(self),

            // +0x4a
            0x0a => instr::or1(self),
            0x2a => instr::or1_not(self),
            0x4a => instr::and1(self),
            0x6a => instr::and1_not(self),

            // +0x1a
            0x1a => instr::decw(self),
            0x3a => instr::incw(self),
            0x5a => instr::cmpw(self),
            0x7a => instr::addw(self),
            0x9a => instr::subw(self),
            0xba => instr::movw_read(self),
            0xda => instr::movw_write(self),

            // +0x0b
            0x0b => instr::unary::<addr::Direct, op::Asl>(self),
            0x2b => instr::unary::<addr::Direct, op::Rol>(self),
            0x4b => instr::unary::<addr::Direct, op::Lsr>(self),
            0x6b => instr::unary::<addr::Direct, op::Ror>(self),
            0x8b => instr::unary::<addr::Direct, op::Dec>(self),
            0xab => instr::unary::<addr::Direct, op::Inc>(self),
            0xcb => instr::write::<addr::Direct, addr::Y>(self),
            0xeb => instr::binary::<addr::Y, addr::Direct, op::Mov>(self),

            // +0x1b
            0x1b => instr::unary::<addr::DirectX, op::Asl>(self),
            0x3b => instr::unary::<addr::DirectX, op::Rol>(self),
            0x5b => instr::unary::<addr::DirectX, op::Lsr>(self),
            0x7b => instr::unary::<addr::DirectX, op::Ror>(self),
            0x9b => instr::unary::<addr::DirectX, op::Dec>(self),
            0xbb => instr::unary::<addr::DirectX, op::Inc>(self),
            0xdb => instr::write::<addr::DirectX, addr::Y>(self),
            0xfb => instr::binary::<addr::Y, addr::DirectX, op::Mov>(self),

            // +0x0c
            0x0c => instr::unary::<addr::Absolute, op::Asl>(self),
            0x2c => instr::unary::<addr::Absolute, op::Rol>(self),
            0x4c => instr::unary::<addr::Absolute, op::Lsr>(self),
            0x6c => instr::unary::<addr::Absolute, op::Ror>(self),
            0x8c => instr::unary::<addr::Absolute, op::Dec>(self),
            0xac => instr::unary::<addr::Absolute, op::Inc>(self),
            0xcc => instr::write::<addr::Absolute, addr::Y>(self),
            0xec => instr::binary::<addr::Y, addr::Absolute, op::Mov>(self),

            // +0x1c
            0x1c => instr::unary::<addr::A, op::Asl>(self),
            0x3c => instr::unary::<addr::A, op::Rol>(self),
            0x5c => instr::unary::<addr::A, op::Lsr>(self),
            0x7c => instr::unary::<addr::A, op::Ror>(self),
            0x9c => instr::unary::<addr::A, op::Dec>(self),
            0xbc => instr::unary::<addr::A, op::Inc>(self),
            0xdc => instr::unary::<addr::Y, op::Dec>(self),
            0xfc => instr::unary::<addr::Y, op::Inc>(self),

            // +0x0d
            0x0d => instr::push::<addr::Psw>(self),
            0x2d => instr::push::<addr::A>(self),
            0x4d => instr::push::<addr::X>(self),
            0x6d => instr::push::<addr::Y>(self),
            0x8d => instr::binary::<addr::Y, addr::Immediate, op::Mov>(self),
            0xad => instr::compare::<addr::Y, addr::Immediate>(self),
            0xcd => instr::binary::<addr::X, addr::Immediate, op::Mov>(self),
            0xed => instr::notc(self),

            // +0x1d
            0x1d => instr::unary::<addr::X, op::Dec>(self),
            0x3d => instr::unary::<addr::X, op::Inc>(self),
            0x5d => instr::binary::<addr::X, addr::A, op::Mov>(self),
            0x7d => instr::binary::<addr::A, addr::X, op::Mov>(self),
            0x9d => instr::binary::<addr::X, addr::SP, op::Mov>(self),
            0xbd => instr::write::<addr::SP, addr::X>(self),
            0xdd => instr::binary::<addr::A, addr::Y, op::Mov>(self),
            0xfd => instr::binary::<addr::Y, addr::A, op::Mov>(self),

            // +0x0e
            0x2e => instr::branch::<op::CbneDirect>(self),
            0x6e => instr::branch::<op::DbnzDirect>(self),
            0x8e => instr::pop::<addr::Psw>(self),
            0xae => instr::pop::<addr::A>(self),
            0xce => instr::pop::<addr::X>(self),
            0xee => instr::pop::<addr::Y>(self),

            // +0x1e
            0x1e => instr::compare::<addr::X, addr::Absolute>(self),
            0x3e => instr::compare::<addr::X, addr::Direct>(self),
            0x5e => instr::compare::<addr::Y, addr::Absolute>(self),
            0x7e => instr::compare::<addr::Y, addr::Direct>(self),
            0x9e => instr::div(self),
            0xde => instr::branch::<op::CbneDirectX>(self),
            0xfe => instr::branch::<op::DbnzY>(self),

            // +0x0f
            0x2f => instr::branch::<op::Bra>(self),
            0x6f => instr::ret(self),
            0x8f => instr::write::<addr::Direct, addr::Immediate>(self),
            0xaf => instr::auto_inc_write(self),
            0xcf => instr::mul(self),

            // +0x1f
            0x1f => instr::jmp_x_indirect(self),
            0x3f => instr::call(self),
            0x5f => instr::jmp(self),
            0x9f => instr::xcn(self),
            0xbf => instr::auto_inc_read(self),

            opcode => todo!("SPC700 opcode {:02X}", opcode),
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

    fn read_direct(&mut self, address: u8) -> u8 {
        self.read(self.flags.p | (address as u16))
    }

    fn write_direct(&mut self, address: u8, value: u8) {
        self.write(self.flags.p | (address as u16), value);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(STACK_PAGE | (self.sp as u16))
    }

    fn push(&mut self, value: u8) {
        self.write(STACK_PAGE | (self.sp as u16), value);
        self.sp = self.sp.wrapping_sub(1);
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
            if self.flags.v { 'V' } else { '-' },
            if self.flags.p != 0 { 'P' } else { '-' },
            if self.flags.b { 'B' } else { '-' },
            if self.flags.h { 'H' } else { '-' },
            if self.flags.i { 'I' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            self.bus,
        )
    }
}
