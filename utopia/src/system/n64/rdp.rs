use super::rsp::{DmaType, Registers};
use crate::util::facade::{DataReader, DataWriter};
use pipeline::Pipeline;
use std::array;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, debug_span};

mod pipeline;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RdpDma {
    pub start: u32,
    pub end: u32,
}

pub struct Rdp {
    regs: Rc<RefCell<Registers>>,
    commands: Vec<u64>,
    pipeline: Pipeline,
    span: SpanInterface,
}

impl Rdp {
    pub fn new(regs: Rc<RefCell<Registers>>) -> Self {
        Self {
            regs,
            commands: Vec::new(),
            pipeline: Pipeline::new(),
            span: SpanInterface {},
        }
    }

    pub fn span(&self) -> &SpanInterface {
        &self.span
    }

    pub fn span_mut(&mut self) -> &mut SpanInterface {
        &mut self.span
    }

    pub fn dma_requested(&self) -> Option<RdpDma> {
        match self.regs.borrow().dma_requested() {
            DmaType::Rdp(dma) => Some(dma),
            _ => None,
        }
    }

    pub fn upload(&mut self, commands: &[u8]) {
        self.commands.clear();

        for chunk in commands.chunks_exact(8) {
            let bytes: [u8; 8] = array::from_fn(|index| chunk[index]);
            self.commands.push(u64::from_be_bytes(bytes));
        }

        self.regs.borrow_mut().finish_dma()
    }

    pub fn run(&mut self, rdram: &mut [u8]) {
        debug!("[CPU => RDP]");

        let _span = debug_span!("rdp").entered();

        debug!("[CPU => RDP]");

        for &command in &self.commands {
            self.pipeline.step(rdram, command);
        }
    }
}

impl DataReader for Rdp {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        self.regs.borrow().get(8 + ((address as usize >> 2) & 7))
    }
}

impl DataWriter for Rdp {
    fn write(&mut self, address: u32, value: u32) {
        self.regs
            .borrow_mut()
            .set(8 + ((address as usize >> 2) & 7), value);
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
