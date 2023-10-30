use std::cell::Cell;
use std::rc::Rc;
use tracing::trace;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InterruptType {
    Reset = 0x01,
    Nmi = 0x02,
    FrameIrq = 0x04,
    LineIrq = 0x08,
}

#[derive(Clone)]
pub struct Interrupt {
    inner: Rc<Cell<u8>>,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn poll(&self) -> u8 {
        self.inner.get()
    }

    pub fn has(&self, int_type: InterruptType) -> bool {
        self.inner.get() & (int_type as u8) != 0
    }

    pub fn clear(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value &= !(int_type as u8);
        self.inner.set(value);
        trace!("Interrupt Cleared: {:?}", int_type);
    }

    pub fn raise(&mut self, int_type: InterruptType) {
        let mut value = self.inner.get();
        value |= int_type as u8;
        self.inner.set(value);
        trace!("Interrupt Raised: {:?}", int_type);
    }
}
