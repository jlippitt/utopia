use crate::core::mos6502;
use std::cell::Cell;
use std::rc::Rc;
use tracing::trace;

#[repr(u32)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InterruptType {
    Reset = 0x01,
    Nmi = 0x02,
    FrameIrq = 0x04,
    DmcIrq = 0x08,
    MapperIrq = 0x10,
}

#[derive(Clone)]
pub struct Interrupt {
    inner: Rc<Cell<mos6502::Interrupt>>,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn poll(&self) -> mos6502::Interrupt {
        self.inner.get()
    }

    pub fn has(&self, int_type: InterruptType) -> bool {
        self.inner.get() & (int_type as mos6502::Interrupt) != 0
    }

    pub fn clear(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value &= !(int_type as mos6502::Interrupt);
        self.inner.set(value);
        trace!("Interrupt Cleared: {:?}", int_type);
    }

    pub fn raise(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value |= int_type as mos6502::Interrupt;
        self.inner.set(value);
        trace!("Interrupt Raised: {:?}", int_type);
    }
}

impl From<mos6502::Interrupt> for InterruptType {
    fn from(value: mos6502::Interrupt) -> InterruptType {
        match value {
            0x01 => Self::Reset,
            0x02 => Self::Nmi,
            0x04 => Self::FrameIrq,
            0x08 => Self::DmcIrq,
            0x10 => Self::MapperIrq,
            _ => unimplemented!("Interrupt Type: {:08X}", value),
        }
    }
}
