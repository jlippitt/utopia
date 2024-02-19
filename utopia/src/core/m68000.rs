use crate::util::memory::Value;
use bitflags::bitflags;
use size::Size;
use tracing::trace;

mod instruction;
mod size;

bitflags! {
    pub struct Interrupt: u32 {
        const RESET = 0x0100;
    }
}

pub trait Bus {
    fn read<T: Value>(&self, address: u32) -> T;
}

pub struct Flags {
    x: bool,
    n: u32,
    z: u32,
    v: u32,
    c: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Supervisor,
    User,
}

pub struct Core<T: Bus> {
    dreg: [u32; 8],
    areg: [u32; 8],
    pc: u32,
    sp_shadow: u32,
    flags: Flags,
    mode: Mode,
    int_level: u8,
    interrupt: Interrupt,
    bus: T,
}

impl<T: Bus> Core<T> {
    pub fn new(bus: T) -> Self {
        Self {
            dreg: [0; 8],
            areg: [0; 8],
            pc: 0,
            sp_shadow: 0,
            flags: Flags {
                x: false,
                n: 0,
                z: u32::MAX,
                v: 0,
                c: false,
            },
            mode: Mode::Supervisor,
            int_level: 0,
            interrupt: Interrupt::RESET,
            bus,
        }
    }

    pub fn step(&mut self) {
        use instruction as instr;

        if !self.interrupt.is_empty() {
            if self.interrupt.contains(Interrupt::RESET) {
                instr::reset(self);
            } else {
                unimplemented!("Interrupt types other than reset");
            }
        }

        instr::dispatch(self);
    }

    fn set_areg<U: Size>(&mut self, index: usize, value: U) {
        U::set_areg(self, index, value);
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
        trace!("  PC: {:08X}", self.pc);
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        trace!("  Mode: {:?}", self.mode);
    }

    fn set_int_level(&mut self, int_level: u8) {
        self.int_level = int_level;
        trace!("  Interrupt Level: {}", self.int_level);
    }

    fn read<U: Size>(&self, address: u32) -> U {
        U::read(self, address)
    }

    fn next<U: Size>(&mut self) -> U {
        let value = U::read(self, self.pc);
        self.pc = self.pc.wrapping_add(std::mem::size_of::<U>() as u32);
        value
    }
}
