use crate::util::memory::Value;
use bitflags::bitflags;
use size::Size;
use std::mem;
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
    fn write<T: Value>(&mut self, address: u32, value: T);
}

pub struct Flags {
    x: bool,
    n: u8,
    z: u32,
    v: u8,
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

            self.interrupt = Interrupt::empty();
            return;
        }

        instr::dispatch(self);
    }

    fn dreg<U: Size>(&self, index: usize) -> U {
        U::dreg(self, index)
    }

    fn set_dreg<U: Size>(&mut self, index: usize, value: U) {
        U::set_dreg(self, index, value);
    }

    fn areg<U: Size>(&self, index: usize) -> U {
        U::areg(self, index)
    }

    fn set_areg<U: Size>(&mut self, index: usize, value: U) {
        U::set_areg(self, index, value);
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
        trace!("  PC: {:08X}", self.pc);
    }

    fn set_ccr(&mut self, cb: impl Fn(&mut Flags)) {
        cb(&mut self.flags);
        trace!(
            "  CCR: ---{}{}{}{}{}",
            if self.flags.x { 'X' } else { '-' },
            if (self.flags.n & 0x80) != 0 { 'N' } else { '-' },
            if self.flags.z == 0 { 'Z' } else { '-' },
            if (self.flags.v & 0x80) != 0 { 'V' } else { '-' },
            if self.flags.c { 'C' } else { '-' },
        );
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

    fn write<U: Size>(&mut self, address: u32, value: U) {
        U::write(self, address, value);
    }

    fn modify<U: Size>(&mut self, address: u32, cb: impl Fn(&mut Core<T>, U) -> U) {
        let value = self.read(address);
        let result = cb(self, value);
        self.write(address, result);
    }

    fn next<U: Size>(&mut self) -> U {
        let value = U::read(self, self.pc);
        self.pc = self.pc.wrapping_add(mem::size_of::<U>() as u32);
        value
    }
}

impl Flags {
    fn set_nz<T: Size>(&mut self, value: T) {
        self.n = (value >> (mem::size_of::<T>() * 8 - 8)).as_();
        self.z = value.as_();
    }
}
