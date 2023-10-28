use std::fmt;
use tracing::trace;

mod instruction;

pub trait Bus {
    fn fetch(&mut self, address: u16) -> u8;
}

struct Flags {
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
    // af_banked: u16,
    // bc_banked: u16,
    // de_banked: u16,
    // hl_banked: u16,
    // i: u8,
    // r: u8,
    // im: u8,
    // iff1: bool,
    // iff2: bool,
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
            // af_banked: 0,
            // bc_banked: 0,
            // de_banked: 0,
            // hl_banked: 0,
            // i: 0,
            // r: 0,
            // im: 0,
            // iff1: false,
            // iff2: false,
            bus,
        }
    }

    pub fn step(&mut self) {
        instruction::dispatch(self);
    }

    fn fetch(&mut self) -> u8 {
        let value = self.bus.fetch(self.pc);
        trace!("  {:04X} => {:02X}", self.pc, value);
        self.pc = self.pc.wrapping_add(1);
        value
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
