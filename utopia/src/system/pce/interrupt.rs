use crate::core::huc6280;
use std::cell::Cell;
use std::rc::Rc;
use tracing::debug;

#[repr(u32)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InterruptType {
    Reset = 0x01,
    Nmi = 0x02,
    Timer = 0x04,
    Irq1 = 0x08,
    Irq2 = 0x10,
}

#[derive(Clone)]
pub struct Interrupt {
    inner: Rc<Cell<huc6280::Interrupt>>,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn poll(&self) -> huc6280::Interrupt {
        self.inner.get()
    }

    // pub fn has(&self, int_type: InterruptType) -> bool {
    //     self.inner.get() & (int_type as huc6280::Interrupt) != 0
    // }

    pub fn clear(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value &= !(int_type as huc6280::Interrupt);
        self.inner.set(value);
        debug!("Interrupt Cleared: {:?}", int_type);
    }

    pub fn raise(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value |= int_type as huc6280::Interrupt;
        self.inner.set(value);
        debug!("Interrupt Raised: {:?}", int_type);
    }
}

impl From<huc6280::Interrupt> for InterruptType {
    fn from(value: huc6280::Interrupt) -> InterruptType {
        match value {
            huc6280::INT_RESET => Self::Reset,
            huc6280::INT_NMI => Self::Nmi,
            huc6280::INT_TIMER => Self::Timer,
            huc6280::INT_IRQ1 => Self::Irq1,
            huc6280::INT_IRQ2 => Self::Irq2,
            _ => unimplemented!("Interrupt Type: {:08X}", value),
        }
    }
}
