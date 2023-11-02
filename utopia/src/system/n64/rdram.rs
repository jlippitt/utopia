use crate::util::memory::{Masked, Memory, Reader, Writer};
use tracing::{trace, warn};

const RDRAM_SIZE: usize = 8 * 1024 * 1024;

const RDRAM_REGS: [&str; 10] = [
    "RDRAM_CONFIG",
    "RDRAM_DEVICE_ID",
    "RDRAM_DELAY",
    "RDRAM_MODE",
    "RDRAM_REF_INTERVAL",
    "RDRAM_REF_ROW",
    "RDRAM_RAS_INTERVAL",
    "RDRAM_MIN_INTERVAL",
    "RDRAM_ADDR_SELECT",
    "RDRAM_DEVICE_MANUF",
];

const RI_REGS: [&str; 8] = [
    "RI_MODE",
    "RI_CONFIG",
    "RI_CURRENT_LOAD",
    "RI_SELECT",
    "RI_REFRESH",
    "RI_LATENCY",
    "RI_RERROR",
    "RI_WERROR",
];

pub struct Rdram {
    data: Memory,
    registers: Registers,
    interface: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        let mut data = Memory::new(RDRAM_SIZE);

        data.write_be(0x318, 0x0080_0000u32);

        Self {
            data,
            registers: Registers::new(),
            interface: Interface::new(),
        }
    }

    pub fn data(&self) -> &Memory {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Memory {
        &mut self.data
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }

    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.interface
    }
}

pub struct Registers {
    regs: [u32; 8],
}

impl Registers {
    fn new() -> Self {
        Self { regs: [0; 8] }
    }
}

impl Reader for Registers {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        if address >= 0x0008_0000 {
            // Broadcast area is read-only
            return 0;
        }

        let address = address & 0x0000_03ff;

        let mut value = self.regs[(address as usize) >> 2];

        if address == 0x000c {
            // Some bits in RDRAM_MODE are inverted when read back
            value ^= 0xc0c0_c0c0
        }

        warn!(
            "RDRAM Read: {}: {:08X}",
            RDRAM_REGS[(address as usize) >> 2],
            value
        );

        value
    }
}

impl Writer for Registers {
    type SideEffect = ();

    fn write_register(&mut self, address: u32, value: Masked<u32>) {
        let address = address & 0x0000_03ff;
        let index = (address as usize) >> 2;

        self.regs[index] = value.apply(self.regs[index]);
        trace!("{}: {:08X}", RDRAM_REGS[index], self.regs[index]);
    }
}

pub struct Interface {
    regs: [u32; 8],
}

impl Interface {
    fn new() -> Self {
        let mut regs = [0; 8];

        regs[3] = 0x14;

        Self { regs }
    }
}

impl Reader for Interface {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        self.regs[(address as usize) >> 2]
    }
}

impl Writer for Interface {
    type SideEffect = ();

    fn write_register(&mut self, address: u32, value: Masked<u32>) {
        let index = (address as usize) >> 2;
        self.regs[index] = value.apply(self.regs[index]);
        trace!("{}: {:08X}", RI_REGS[index], self.regs[index]);
    }
}
