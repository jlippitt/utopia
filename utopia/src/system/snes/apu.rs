use crate::core::spc700::{Bus, Core};
use crate::util::MirrorVec;
use crate::AudioQueue;
use dsp::Dsp;
use std::fmt;
use timer::Timer;
use tracing::{debug_span, trace};

mod dsp;
mod timer;

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

    pub fn audio_queue(&mut self) -> &mut AudioQueue {
        self.core.bus_mut().dsp.audio_queue()
    }

    pub fn read(&self, address: u8) -> u8 {
        self.core.bus().output_ports[address as usize & 3]
    }

    pub fn write(&mut self, address: u8, value: u8) {
        let index = address as usize & 3;
        self.core.bus_mut().input_ports[index] = value;
        trace!("APU Input Port {}: {:02X}", index, value);
    }

    pub fn run_until(&mut self, cpu_cycles: u64) {
        trace!("[CPU:{} => APU:{}]", cpu_cycles, self.core.bus().cycles);

        let _span = debug_span!("spc700").entered();

        trace!("[CPU:{} => APU:{}]", cpu_cycles, self.core.bus().cycles);

        self.core.bus_mut().time_remaining +=
            (cpu_cycles - self.prev_cpu_cycles) as i64 * APU_CLOCK_RATE;

        self.prev_cpu_cycles = cpu_cycles;

        while self.core.bus().time_remaining > 0 {
            self.core.step();
            trace!("{}", self.core);
        }
    }
}

struct Hardware {
    time_remaining: i64,
    cycles: u64,
    ipl_rom_enabled: bool,
    input_ports: [u8; 4],
    output_ports: [u8; 4],
    timers: [Timer; 3],
    ram: MirrorVec<u8>,
    ipl_rom: MirrorVec<u8>,
    dsp: Dsp,
}

impl Hardware {
    pub fn new(ipl_rom: Vec<u8>) -> Self {
        Self {
            time_remaining: 0,
            cycles: 0,
            ipl_rom_enabled: true,
            input_ports: [0; 4],
            output_ports: [0; 4],
            timers: [Timer::new(0), Timer::new(1), Timer::new(2)],
            ram: MirrorVec::new(RAM_SIZE),
            ipl_rom: ipl_rom.into(),
            dsp: Dsp::new(),
        }
    }

    fn step(&mut self) {
        self.time_remaining -= CPU_CLOCK_RATE;
        self.cycles += 1;

        if (self.cycles & 15) == 0 {
            if (self.cycles & 127) == 0 {
                self.timers[0].step();
                self.timers[1].step();
            }

            self.timers[2].step();

            if (self.cycles & 31) == 0 {
                self.dsp.step(&mut self.ram);
            }
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        self.step();
    }

    fn read(&mut self, address: u16) -> u8 {
        self.step();

        if (address & 0xfff0) == 0x00f0 {
            match address & 0x0f {
                0x01 => self.ram[address as usize],
                0x02 => self.dsp.address(),
                0x03 => self.dsp.read(),
                0x04..=0x07 => self.input_ports[address as usize & 3],
                0x08..=0x0c => self.ram[address as usize],
                0x0d => self.timers[0].get_and_reset_output(),
                0x0e => self.timers[1].get_and_reset_output(),
                0x0f => self.timers[2].get_and_reset_output(),
                _ => todo!("SMP register read {:02X}", address),
            }
        } else if address >= 0xffc0 && self.ipl_rom_enabled {
            self.ipl_rom[address as usize]
        } else {
            self.ram[address as usize]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step();

        if (address & 0xfff0) == 0x00f0 {
            match address & 0x0f {
                0x01 => {
                    self.timers[0].set_enabled((value & 0x01) != 0);
                    self.timers[1].set_enabled((value & 0x02) != 0);
                    self.timers[2].set_enabled((value & 0x04) != 0);

                    if (value & 0x10) != 0 {
                        self.input_ports[0] = 0;
                        self.input_ports[1] = 0;
                        trace!("Input Ports 0 & 1 Reset");
                    }

                    if (value & 0x20) != 0 {
                        self.input_ports[2] = 0;
                        self.input_ports[3] = 0;
                        trace!("Input Ports 2 & 3 Reset");
                    }

                    self.ipl_rom_enabled = (value & 0x80) != 0;
                    trace!("IPL ROM Enabled: {}", self.ipl_rom_enabled);
                }
                0x02 => self.dsp.set_address(value),
                0x03 => self.dsp.write(value),
                0x04..=0x07 => {
                    let index = address as usize & 3;
                    self.output_ports[index] = value;
                    trace!("APU Output Port {}: {:02X}", index, value);
                }
                0x0a => self.timers[0].set_divider(value),
                0x0b => self.timers[1].set_divider(value),
                0x0c => self.timers[2].set_divider(value),
                _ => todo!("SMP register write {:02X} <= {:02X}", address, value),
            }
        } else {
            self.ram[address as usize] = value;
        }
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={}", self.cycles)
    }
}
