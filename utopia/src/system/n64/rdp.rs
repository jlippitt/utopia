pub use registers::Registers;

use crate::util::facade::{DataReader, DataWriter};
use std::cell::RefCell;
use std::rc::Rc;

mod registers;

pub struct Rdp {
    command: CommandInterface,
    span: SpanInterface,
}

impl Rdp {
    pub fn new(regs: Rc<RefCell<Registers>>) -> Self {
        Self {
            command: CommandInterface { regs },
            span: SpanInterface {},
        }
    }

    pub fn command(&self) -> &CommandInterface {
        &self.command
    }

    pub fn command_mut(&mut self) -> &mut CommandInterface {
        &mut self.command
    }

    pub fn span(&self) -> &SpanInterface {
        &self.span
    }

    pub fn span_mut(&mut self) -> &mut SpanInterface {
        &mut self.span
    }
}

pub struct CommandInterface {
    regs: Rc<RefCell<Registers>>,
}

impl DataReader for CommandInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        self.regs.borrow().get((address as usize >> 2) & 7)
    }
}

impl DataWriter for CommandInterface {
    fn write(&mut self, address: u32, value: u32) {
        self.regs
            .borrow_mut()
            .set((address as usize >> 2) & 7, value);
    }
}

pub struct SpanInterface {
    //
}

impl DataReader for SpanInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            _ => unimplemented!("RDP Span Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for SpanInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            _ => unimplemented!("RDP Span Register Write: {:08X} <= {:08X}", address, value),
        }
    }
}
