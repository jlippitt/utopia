pub use cp0::{DmaType, Registers};

use super::dma::DmaRequest;
use super::interrupt::RcpInterrupt;
use crate::core::mips::{self, Core, NullCp1};
use crate::util::memory::{Masked, Memory, Reader, Value, Writer};
use cp0::Cp0;
use cp2::Cp2;
use tracing::{debug_span, trace};

mod cp0;
mod cp2;

pub struct Rsp {
    core: Core<Bus>,
}

impl Rsp {
    pub const DMEM_SIZE: usize = 4096;
    pub const IMEM_SIZE: usize = Self::DMEM_SIZE;
    pub const MEM_SIZE: usize = Self::DMEM_SIZE + Self::IMEM_SIZE;

    pub fn new(initial_dmem: &[u8], rcp_int: RcpInterrupt) -> Self {
        let mut initial_mem = Vec::from(&initial_dmem[0..Self::DMEM_SIZE]);
        initial_mem.resize(Self::MEM_SIZE, 0);

        Self {
            core: Core::new(
                Bus::new(initial_mem.into()),
                Cp0::new(rcp_int),
                NullCp1,
                Cp2::new(),
                Default::default(),
            ),
        }
    }

    pub fn regs(&self) -> &Registers {
        self.core.cp0().regs()
    }

    pub fn regs_mut(&mut self) -> &mut Registers {
        self.core.cp0_mut().regs_mut()
    }

    pub fn mem(&self) -> &Memory {
        &self.core.bus().mem
    }

    pub fn mem_mut(&mut self) -> &mut Memory {
        &mut self.core.bus_mut().mem
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        if address < 0x0004_0000 {
            let mem = &self.core.bus().mem;
            // Mirrored every MEM_SIZE bytes
            mem.read_be(address as usize & (mem.len() - 1))
        } else {
            T::read_register_be(self, address)
        }
    }

    pub fn write<T: Value>(&mut self, address: u32, value: T) -> Option<DmaRequest> {
        if address < 0x0004_0000 {
            // Mirrored every MEM_SIZE bytes
            let mem = &mut self.core.bus_mut().mem;
            mem.write_be(address as usize & (mem.len() - 1), value);
            None
        } else {
            T::write_register_be(self, address, value)
        }
    }

    pub fn step(&mut self) -> DmaType {
        if !self.core.cp0_mut().regs_mut().try_restart() {
            return DmaType::None;
        }

        let _span = debug_span!("rsp").entered();

        if self.core.cp0().regs().is_single_step() {
            self.core.step();
            self.core.cp0_mut().regs_mut().halt();
        } else {
            while self.core.cp0().regs().is_running() {
                self.core.step();
            }
        }

        let regs = self.core.cp0_mut().regs_mut();

        regs.take_dma_type()
    }
}

impl Reader for Rsp {
    fn read_u32(&self, address: u32) -> u32 {
        if address < 0x0004_0020 {
            self.core.cp0().regs().get((address & 0x1f) as usize >> 2)
        } else if address == 0x0008_0000 {
            self.core.pc()
        } else {
            panic!("Unmapped RSP Register Read: {:08X}", address);
        }
    }
}

impl Writer for Rsp {
    type SideEffect = Option<DmaRequest>;

    fn write_u32(&mut self, address: u32, value: Masked<u32>) -> Option<DmaRequest> {
        if address < 0x0004_0020 {
            let regs = self.core.cp0_mut().regs_mut();

            regs.set((address & 0x1f) as usize >> 2, value);

            return match regs.take_dma_type() {
                DmaType::Rsp(request) => Some(request),
                DmaType::Rdp(..) => unreachable!(),
                DmaType::None => None,
            };
        } else if address == 0x0008_0000 {
            self.core.set_pc(value.apply(self.core.pc()) & 0xffc);
            trace!("SP_PC: {:08X}", self.core.pc());
        } else {
            panic!("Unmapped RSP Register Write: {:08X}", address);
        }

        None
    }
}

struct Bus {
    mem: Memory,
}

impl Bus {
    fn new(mem: Memory) -> Self {
        Self { mem }
    }
}

impl mips::Bus for Bus {
    const NAME: &'static str = "RSP";
    const ENABLE_64_BIT: bool = false;
    const ENABLE_MUL_DIV: bool = false;
    const ENABLE_LIKELY_BRANCH: bool = false;
    const FORCE_MEMORY_ALIGNMENT: bool = false;
    const PC_MASK: u32 = 0xfff;

    type Cp0 = Cp0;
    type Cp1 = mips::NullCp1;
    type Cp2 = Cp2;

    fn read_opcode(&self, address: u32) -> u32 {
        self.mem.read_be(Rsp::DMEM_SIZE + address as usize)
    }

    fn read_data<T: Value>(&self, address: u32) -> T {
        self.mem.read_be_unaligned(address as usize, Some(0xfff))
    }

    fn write_data<T: Value>(&mut self, address: u32, value: T) {
        self.mem
            .write_be_unaligned(address as usize, value, Some(0xfff))
    }

    fn poll(&self) -> u8 {
        // No interrupts in RSP
        0
    }

    fn step(&mut self) {
        // No processing required here
    }
}
