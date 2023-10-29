use register_set::RegisterSetDefault;
use std::fmt;
use tracing::trace;

mod address_mode;
mod condition;
mod instruction;
mod register_set;

const IRQ_DISABLE: u8 = 0x03;
const IRQ_ENABLE: u8 = 0xff;

pub trait Bus {
    fn idle(&mut self, cycles: u64);
    fn fetch(&mut self, address: u16) -> u8;
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read_port(&mut self, address: u16) -> u8;
    fn write_port(&mut self, address: u16, value: u8);
    fn poll(&self) -> u8;
}

pub struct Flags {
    s: u8,
    z: u8,
    y: u8,
    h: bool,
    x: u8,
    pv: bool,
    n: bool,
    c: bool,
}

pub struct Core<T: Bus> {
    a: u8,
    bc: u16,
    de: u16,
    hl: u16,
    ix: u16,
    iy: u16,
    sp: u16,
    pc: u16,
    flags: Flags,
    af_banked: u16,
    bc_banked: u16,
    de_banked: u16,
    hl_banked: u16,
    // i: u8,
    // r: u8,
    im: u8,
    iff: [u8; 2],
    iff_delayed: [u8; 2],
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            a: 0xff,
            bc: 0,
            de: 0,
            hl: 0,
            ix: 0,
            iy: 0,
            sp: 0xffff,
            pc: 0,
            flags: Flags {
                s: 0xff,
                z: 0,
                y: 0xff,
                h: true,
                x: 0xff,
                pv: true,
                n: true,
                c: true,
            },
            af_banked: 0,
            bc_banked: 0,
            de_banked: 0,
            hl_banked: 0,
            // i: 0,
            // r: 0,
            im: 0,
            iff: [IRQ_DISABLE; 2],
            iff_delayed: [IRQ_DISABLE; 2],
            bus,
        }
    }

    pub fn step(&mut self) {
        if (self.iff[0] & self.bus.poll()) != 0 {
            // TODO: RESET/NMI handling
            trace!("Interrupt: IRQ");
            self.iff = [IRQ_DISABLE; 2];
            self.iff_delayed = [IRQ_DISABLE; 2];

            match self.im {
                1 => instruction::rst(self, 0x38),
                _ => unimplemented!("Interrupt Mode {}", self.im),
            }

            return;
        }

        self.iff = self.iff_delayed;

        instruction::dispatch::<RegisterSetDefault>(self);
    }

    fn idle(&mut self, cycles: u64) {
        trace!("  Idle ({})", cycles);
        self.bus.idle(cycles);
    }

    fn fetch(&mut self) -> u8 {
        let value = self.bus.fetch(self.pc);
        trace!("  {:04X} => {:02X}", self.pc, value);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        trace!("  {:04X} => {:02X}", address, value);
        value
    }

    fn write(&mut self, address: u16, value: u8) {
        trace!("  {:04X} <= {:02X}", address, value);
        self.bus.write(address, value);
    }

    fn read_port(&mut self, address: u16) -> u8 {
        let value = self.bus.read_port(address);
        trace!("  [Port:{:04X}] => {:02X}", address, value);
        value
    }

    fn write_port(&mut self, address: u16, value: u8) {
        trace!("  [Port:{:04X}] <= {:02X}", address, value);
        self.bus.write_port(address, value);
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

    fn set_sz(&mut self, value: u8) {
        self.flags.s = value;
        self.flags.z = value;
        self.flags.y = value;
        self.flags.x = value;
    }
}

impl<T: Bus + fmt::Display> fmt::Display for Core<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "A={:02X} BC={:04X} DE={:04X} HL={:04X} IX={:04X} IY={:04X} SP={:04X} PC={:04X} F={}{}{}{}{}{}{}{} {}",
            self.a,
            self.bc,
            self.de,
            self.hl,
            self.ix,
            self.iy,
            self.sp,
            self.pc,
            if (self.flags.s & 0x80) != 0 { 'S' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if (self.flags.y & 0x20) != 0 { 'Y' } else { '-' },
            if self.flags.h { 'H' } else { '-' },
            if (self.flags.x & 0x08) != 0 { 'X' } else { '-' },
            if self.flags.pv { 'P' } else { '-' },
            if self.flags.n { 'N' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
            self.bus
        )
    }
}
