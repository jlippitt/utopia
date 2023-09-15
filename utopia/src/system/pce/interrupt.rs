use crate::core::huc6280;
use std::cell::Cell;
use std::rc::Rc;
use tracing::{debug, warn};

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
    status: Rc<Cell<huc6280::Interrupt>>,
    mask: Rc<Cell<huc6280::Interrupt>>,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            status: Default::default(),
            mask: Default::default(),
        }
    }

    pub fn poll(&self) -> huc6280::Interrupt {
        self.status.get() & !self.mask.get()
    }

    // pub fn has(&self, int_type: InterruptType) -> bool {
    //     self.inner.get() & (int_type as huc6280::Interrupt) != 0
    // }

    pub fn clear(&mut self, int_type: InterruptType) {
        let mut value = self.status.get();
        value &= !(int_type as huc6280::Interrupt);
        self.status.set(value);
        debug!("Interrupt Cleared: {:?}", int_type);
    }

    pub fn raise(&mut self, int_type: InterruptType) {
        let mut value = self.status.get();
        value |= int_type as huc6280::Interrupt;
        self.status.set(value);
        debug!("Interrupt Raised: {:?}", int_type);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            2 => {
                self.mask.set(value);
                debug!("Interrupt Mask: {:02X}", self.mask.get());
            }
            _ => warn!(
                "Unmapped Interrupt Controller Write: {:04X} <= {:02X}",
                address, value
            ),
        }
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
