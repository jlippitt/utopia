use std::fmt;

pub trait Bus : fmt::Display {
    //
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