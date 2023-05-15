use instruction as instr;
use tracing::debug;

mod instruction;

pub const STACK_PAGE: u16 = 0x0100;

pub type Interrupt = u32;

pub const INT_RESET: Interrupt = 0x0000_0001;
pub const INT_NMI: Interrupt = 0x0000_0002;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
}

#[repr(u32)]
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
        if self.interrupt != 0 {
            if (self.interrupt & INT_RESET) != 0 {
                instr::reset(self);
            } else {
                panic!("Interrupt type not yet supported");
            }
        }

        panic!("Instructions not yet implemented");
    }

    fn read(&mut self, address: u16) -> u8 {
        let value = self.bus.read(address);
        debug!("  {:04X} => {:02X}", address, value);
        value
    }
}
