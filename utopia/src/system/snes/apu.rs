use crate::core::spc700::{Bus, Core};
use crate::util::MirrorVec;
use std::fmt;
use tracing::{debug, debug_span};

const APU_CLOCK_RATE: i64 = 1024000;
const CPU_CLOCK_RATE: i64 = 21477272;

const RAM_SIZE: usize = 65536;

pub struct Apu {
    core: Core<Hardware>,
    prev_cpu_cycles: u64,
}

impl Apu {
    pub fn new(ipl_rom: Vec<u8>) -> Self {
        let hw = Hardware::new(ipl_rom);
        let core = Core::new(hw);

        Self {
            core,
            prev_cpu_cycles: 0,
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        self.core.bus().output_ports[address as usize & 3]
    }

    pub fn write(&mut self, address: u8, value: u8) {
        self.core.bus_mut().input_ports[address as usize & 3] = value;
    }

    pub fn run_until(&mut self, cpu_cycles: u64) {
        debug!("[CPU:{} => APU:{}]", cpu_cycles, self.core.bus().cycles);

        let _span = debug_span!("spc700").entered();

        debug!("[CPU:{} => APU:{}]", cpu_cycles, self.core.bus().cycles);

        self.core.bus_mut().time_remaining +=
            (cpu_cycles - self.prev_cpu_cycles) as i64 * APU_CLOCK_RATE;

        self.prev_cpu_cycles = cpu_cycles;

        while self.core.bus().time_remaining > 0 {
            self.core.step();
            debug!("{}", self.core);
        }
    }
}

struct Hardware {
    time_remaining: i64,
    cycles: u64,
    input_ports: [u8; 4],
    output_ports: [u8; 4],
    ram: MirrorVec<u8>,
    ipl_rom: MirrorVec<u8>,
}

impl Hardware {
    pub fn new(ipl_rom: Vec<u8>) -> Self {
        Self {
            time_remaining: 0,
            cycles: 0,
            input_ports: [0; 4],
            output_ports: [0; 4],
            ram: MirrorVec::new(RAM_SIZE),
            ipl_rom: ipl_rom.into(),
        }
    }

    fn step(&mut self) {
        self.time_remaining -= CPU_CLOCK_RATE;
        self.cycles += 1;
    }
}

impl Bus for Hardware {
    fn read(&mut self, address: u16) -> u8 {
        self.step();

        if (address & 0xfff0) == 0x00f0 {
            todo!("SMP registers")
        } else if address >= 0xffc0 {
            self.ipl_rom[address as usize]
        } else {
            self.ram[address as usize]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step();

        if (address & 0xfff0) == 0x00f0 {
            todo!("SMP registers")
        } else {
            self.ram[address as usize] = value;
        }
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
