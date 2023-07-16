use crate::core::spc700::{Bus, Core};
use std::fmt;
use tracing::debug;

const APU_CLOCK_RATE: i64 = 1024000;
const CPU_CLOCK_RATE: i64 = 21477272;

pub struct Apu {
    core: Core<Hardware>,
    prev_cpu_cycles: u64,
}

impl Apu {
    pub fn new() -> Self {
        let hw = Hardware::new();
        let core = Core::new(hw);

        Self {
            core,
            prev_cpu_cycles: 0,
        }
    }

    pub fn read(&self, _address: u8) -> u8 {
        todo!("APU reads");
    }

    pub fn write(&self, _address: u8, _value: u8) {
        todo!("APU writes");
    }

    pub fn run_until(&mut self, cpu_cycles: u64) {
        debug!("[CPU:{} => SMP:{}]", cpu_cycles, self.core.bus().cycles);

        self.core.bus_mut().time_remaining +=
            (cpu_cycles - self.prev_cpu_cycles) as i64 * APU_CLOCK_RATE;

        self.prev_cpu_cycles = cpu_cycles;

        while self.core.bus().time_remaining > 0 {
            self.core.step();
        }
    }
}

struct Hardware {
    time_remaining: i64,
    cycles: u64,
}

impl Hardware {
    pub fn new() -> Self {
        Self {
            time_remaining: 0,
            cycles: 0,
        }
    }
}

impl Bus for Hardware {
    //
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
