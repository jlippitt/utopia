pub use registers::{DmaType, Registers};

use super::dma::Dma;
use crate::core::mips::{Bus, Core, Interrupt};
use crate::util::facade::{DataReader, DataWriter, ReadFacade, Value, WriteFacade};
use crate::util::MirrorVec;
use cp0::Cp0;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, debug_span};
use vector_unit::VectorUnit;

mod cp0;
mod registers;
mod vector_unit;

pub const DMEM_SIZE: usize = 4096;

const IMEM_SIZE: usize = 4096;

pub struct Rsp {
    regs: Rc<RefCell<Registers>>,
    core: Core<Hardware>,
}

impl Rsp {
    pub fn new<T: Into<Vec<u8>>>(dmem: T, regs: Rc<RefCell<Registers>>) -> Self {
        let dmem = dmem.into();

        assert!(dmem.len() == DMEM_SIZE);

        Self {
            regs: regs.clone(),
            core: Core::new(
                Hardware::new(dmem),
                Cp0::new(regs),
                VectorUnit::new(),
                Default::default(),
            ),
        }
    }

    pub fn dmem(&self) -> &[u8] {
        self.core.bus().dmem.inner()
    }

    pub fn read_ram<T: Value>(&self, address: u32) -> T {
        // TODO: Mirroring
        let address = address as usize;

        if address < DMEM_SIZE {
            self.core.bus().dmem.read_be(address)
        } else {
            self.core.bus().imem.read_be(address)
        }
    }

    pub fn write_ram<T: Value>(&mut self, address: u32, value: T) {
        // TODO: Mirroring
        let address = address as usize;

        if address < DMEM_SIZE {
            self.core.bus_mut().dmem.write_be(address, value);
        } else {
            self.core.bus_mut().imem.write_be(address, value);
        }
    }

    pub fn dma_requested(&self) -> Option<Dma> {
        match self.regs.borrow().dma_requested() {
            DmaType::Rsp(dma) => Some(dma),
            _ => None,
        }
    }

    pub fn finish_dma(&mut self) {
        self.regs.borrow_mut().finish_dma()
    }

    pub fn step(&mut self) -> DmaType {
        {
            let regs = self.regs.borrow();

            if regs.halted() {
                return DmaType::None;
            }

            if regs.single_step() {
                todo!("Single step");
            }
        }

        debug!("[CPU => RSP]");

        {
            let _span = debug_span!("rsp").entered();

            debug!("[CPU => RSP]");

            while !self.regs.borrow().is_done() {
                self.core.step();
            }
        }

        self.regs.borrow().dma_requested()
    }
}

impl DataReader for Rsp {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: Self::Address) -> Self::Value {
        match address {
            0x0004_0000..=0x0004_001F => self.regs.borrow().get((address as usize >> 2) & 7),
            0x0008_0000 => self.core.pc(),
            _ => unimplemented!("RSP Register Read: {:08X}", address),
        }
    }
}

impl DataWriter for Rsp {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        match address {
            0x0004_0000..=0x0004_001F => {
                self.regs
                    .borrow_mut()
                    .set((address as usize >> 2) & 7, value);
            }
            0x0008_0000 => {
                self.core.set_pc(value & 0x0ffc);
                debug!("RSP Program Counter: {:04X}", value & 0x0ffc);
            }
            _ => unimplemented!("RSP Register Write: {:08X} <= {:08X}", address, value),
        }
    }
}

struct Hardware {
    dmem: MirrorVec<u8>,
    imem: MirrorVec<u8>,
}

impl Hardware {
    fn new(dmem: Vec<u8>) -> Self {
        Self {
            dmem: dmem.into(),
            imem: MirrorVec::new(IMEM_SIZE),
        }
    }
}

impl Bus for Hardware {
    type Cp0 = Cp0;
    type Cp2 = VectorUnit;

    const CP1: bool = false;
    const MUL_DIV: bool = false;
    const INSTR_64: bool = false;
    const PC_MASK: u32 = 0xfff;

    fn read_opcode<T: Value>(&mut self, address: u32) -> T {
        self.imem.read_be(address as usize)
    }

    fn read<T: Value>(&mut self, address: u32) -> T {
        self.dmem.read_be(address as usize)
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        self.dmem.write_be(address as usize, value);
    }

    fn step(&mut self) {
        // TODO
    }

    fn poll(&self) -> Interrupt {
        // No interrupts in RSP
        0
    }
}
