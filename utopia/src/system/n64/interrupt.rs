use crate::core::mips::Interrupt;
use std::cell::Cell;
use std::rc::Rc;
use tracing::debug;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CpuIntType {
    Rcp = 0x04,
    Pif = 0x10,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RcpIntType {
    SP = 0x01,
    SI = 0x02,
    AI = 0x04,
    VI = 0x08,
    PI = 0x10,
    DP = 0x20,
}

#[derive(Clone)]
pub struct CpuInterrupt {
    status: Rc<Cell<Interrupt>>,
}

impl CpuInterrupt {
    pub fn new() -> Self {
        Self {
            status: Rc::new(Cell::new(0)),
        }
    }

    pub fn poll(&self) -> u8 {
        self.status.get()
    }

    pub fn raise(&mut self, int_type: CpuIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status | int_type as u8);

        if self.status.get() != prev_status {
            debug!("CPU Interrupt Raised: {:?}", int_type);
        }
    }

    pub fn clear(&mut self, int_type: CpuIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status & !(int_type as u8));

        if self.status.get() != prev_status {
            debug!("CPU Interrupt Cleared: {:?}", int_type);
        }
    }
}

#[derive(Clone)]
pub struct RcpInterrupt {
    cpu_interrupt: CpuInterrupt,
    mask: Rc<Cell<u8>>,
    status: Rc<Cell<u8>>,
}

impl RcpInterrupt {
    pub fn new(cpu_interrupt: CpuInterrupt) -> Self {
        Self {
            cpu_interrupt,
            mask: Rc::new(Cell::new(0)),
            status: Rc::new(Cell::new(0)),
        }
    }

    pub fn poll(&self) -> u8 {
        self.status.get()
    }

    pub fn has(&self, int_type: RcpIntType) -> bool {
        (self.status.get() & int_type as u8) != 0
    }

    pub fn set_mask(&mut self, mask: u8) {
        self.mask.set(mask);
        debug!("RCP Interrupt Mask: {:06b}", mask);
        self.update();
    }

    pub fn raise(&mut self, int_type: RcpIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status | int_type as u8);

        if self.status.get() != prev_status {
            debug!("RCP Interrupt Raised: {:?}", int_type);
        }

        self.update();
    }

    pub fn clear(&mut self, int_type: RcpIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status & !(int_type as u8));

        if self.status.get() != prev_status {
            debug!("RCP Interrupt Cleared: {:?}", int_type);
        }

        self.update();
    }

    fn update(&mut self) {
        let active = self.status.get() & self.mask.get();

        if active != 0 {
            self.cpu_interrupt.raise(CpuIntType::Rcp);
        } else {
            self.cpu_interrupt.clear(CpuIntType::Rcp);
        }
    }
}
