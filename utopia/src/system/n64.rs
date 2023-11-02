use crate::core::mips::{self, Core, InitialState, NullCp2};
use crate::util::memory::{Memory, Reader, Value, Writer};
use crate::{InstanceOptions, JoypadState, MemoryMapper, Size, SystemOptions, WgpuContext};
use audio::AudioInterface;
use interrupt::{CpuInterrupt, RcpInterrupt};
use mips_interface::MipsInterface;
use peripheral::PeripheralInterface;
use rdp::Rdp;
use rdram::Rdram;
use rsp::{DmaType, Rsp};
use serial::SerialInterface;
use std::error::Error;
use std::marker::PhantomData;
use tracing::{debug, warn};
use video::VideoInterface;
use vr4300::{Cp0, Cp1};

mod audio;
mod dma;
mod interrupt;
mod mips_interface;
mod peripheral;
mod rdp;
mod rdram;
mod rsp;
mod serial;
mod video;
mod vr4300;

const IPL3_START_ADDRESS: u32 = 0xA4000040;

// TODO: Actual CPU timing
const CYCLES_PER_STEP: u64 = 2;

pub struct System<T: MemoryMapper + 'static> {
    _phantom: PhantomData<T>,
}

impl<T: MemoryMapper> System<T> {
    pub fn new(_options: SystemOptions<'_, T>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: MemoryMapper> crate::System<T> for System<T> {
    fn default_output_resolution(&self) -> Size {
        // This determines the initial size of the output window
        VideoInterface::DEFAULT_TARGET_SIZE
    }

    fn default_sample_rate(&self) -> Option<u64> {
        None
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        let result = Instance::new(options);

        Ok(Box::new(
            result.map_err(|err| crate::Error(err.to_string()))?,
        ))
    }
}

pub struct Instance {
    core: Core<Bus>,
}

impl Instance {
    pub fn new(options: InstanceOptions) -> Result<Self, Box<dyn Error>> {
        let mut regs = [0; 32];
        regs[20] = 1; // $S4
        regs[22] = 0x3f; // $S6
        regs[29] = 0xffff_ffff_a400_1ff0; // $SP

        let initial_state = InitialState {
            pc: IPL3_START_ADDRESS,
            regs,
        };

        Ok(Self {
            core: Core::new(
                Bus::new(options.wgpu_context, options.rom_data),
                Cp0::new(),
                Cp1::new(),
                NullCp2,
                initial_state,
            ),
        })
    }
}

impl crate::Instance for Instance {
    fn run_frame(&mut self, joypad_state: &JoypadState) {
        self.core.bus_mut().si.pif_mut().update_joypad(joypad_state);
        self.core.bus_mut().vi.reset_frame_complete();

        while !self.core.bus().vi.frame_complete() {
            self.core.step();
        }

        let bus = self.core.bus_mut();
        let rdram = bus.rdram.data_mut();
        bus.rdp.sync(rdram);
        bus.vi.update(rdram).unwrap();
    }

    fn present(&self, canvas: &wgpu::Texture) {
        self.core.bus().vi.render(canvas);
    }
}

struct Bus {
    cpu_int: CpuInterrupt,
    rom: Memory,
    rdram: Rdram,
    rsp: Rsp,
    rdp: Rdp,
    mi: MipsInterface,
    vi: VideoInterface,
    ai: AudioInterface,
    pi: PeripheralInterface,
    si: SerialInterface,
    systest_buffer: Memory,
}

impl Bus {
    pub fn new(ctx: WgpuContext, rom_data: Vec<u8>) -> Self {
        let cpu_int = CpuInterrupt::new();
        let rcp_int = RcpInterrupt::new(cpu_int.clone());

        let rsp = Rsp::new(&rom_data, rcp_int.clone());

        Self {
            cpu_int,
            rom: rom_data.into(),
            rdram: Rdram::new(),
            rsp,
            rdp: Rdp::new(ctx.clone(), rcp_int.clone()),
            mi: MipsInterface::new(rcp_int.clone()),
            vi: VideoInterface::new(ctx, rcp_int.clone()),
            ai: AudioInterface::new(rcp_int.clone()),
            pi: PeripheralInterface::new(rcp_int.clone()),
            si: SerialInterface::new(rcp_int),
            systest_buffer: Memory::new(512),
        }
    }
}

impl mips::Bus for Bus {
    const NAME: &'static str = "VR4300";
    const ENABLE_64_BIT: bool = true;
    const ENABLE_MUL_DIV: bool = true;
    const ENABLE_LIKELY_BRANCH: bool = true;
    const FORCE_MEMORY_ALIGNMENT: bool = true;

    type Cp0 = vr4300::Cp0;
    type Cp1 = vr4300::Cp1;
    type Cp2 = mips::NullCp2;

    fn read_data<T: Value>(&self, address: u32) -> T {
        match address >> 20 {
            0x000..=0x03e => {
                if let Some(value) = self.rdram.data().try_read_be(address as usize) {
                    value
                } else {
                    warn!("Unmapped RDRAM read: {:08X}", address);
                    T::zero()
                }
            }
            0x03f => self.rdram.registers().read_be(address & 0x000f_ffff),
            0x040 => self.rsp.read(address & 0x000f_ffff),
            0x041 => self
                .rdp
                .command(self.rsp.regs())
                .read_be(address & 0x000f_ffff),
            0x043 => self.mi.read_be(address & 0x000f_ffff),
            0x044 => self.vi.read_be(address & 0x000f_ffff),
            0x045 => self.ai.read_be(address & 0x000f_ffff),
            0x046 => self.pi.read_be(address & 0x000f_ffff),
            0x047 => self.rdram.interface().read_be(address & 0x000f_ffff),
            0x048 => self.si.read_be(address & 0x000f_ffff),
            0x100..=0x1fb => {
                if let Some(value) = self.rom.try_read_be(address as usize & 0x0fff_ffff) {
                    value
                } else {
                    debug!("Unmapped ROM read: {:08X}", address);
                    T::zero()
                }
            }
            0x1fc => self.si.pif().read(address & 0x000f_ffff),
            _ => panic!("Unmapped Read: {:08X}", address),
        }
    }

    fn write_data<T: Value>(&mut self, address: u32, value: T) {
        match address >> 20 {
            0x000..=0x03e => {
                if !self.rdram.data_mut().try_write_be(address as usize, value) {
                    warn!("Unmapped RDRAM write: {:08X} <= {:08X}", address, value);
                }
            }
            0x03f => self
                .rdram
                .registers_mut()
                .write_be(address & 0x000f_ffff, value),
            0x040 => {
                if let Some(dma_request) = self.rsp.write(address & 0x000f_ffff, value) {
                    self.rsp_dma_transfer(dma_request);
                }
            }
            0x041 => {
                if let Some(dma_request) = self
                    .rdp
                    .command_mut(self.rsp.regs_mut())
                    .write_be(address & 0x000f_ffff, value)
                {
                    self.rdp_dma_transfer(dma_request);
                }
            }
            0x043 => self.mi.write_be(address & 0x000f_ffff, value),
            0x044 => self.vi.write_be(address & 0x000f_ffff, value),
            0x045 => self.ai.write_be(address & 0x000f_ffff, value),
            0x046 => {
                if let Some(dma_request) = self.pi.write_be(address & 0x000f_ffff, value) {
                    self.pi_dma_transfer(dma_request);
                }
            }
            0x047 => self
                .rdram
                .interface_mut()
                .write_be(address & 0x000f_ffff, value),
            0x048 => {
                if let Some(dma_request) = self.si.write_be(address & 0x000f_ffff, value) {
                    self.si_dma_transfer(dma_request);
                }
            }
            0x1fc => self.si.pif_mut().write(address & 0x000f_ffff, value),
            // N64 SystemTest Output
            0x13f => match address {
                0x13ff_0020..=0x13ff_0220 => {
                    self.systest_buffer
                        .write_be(address as usize - 0x13ff_0020, value);
                }
                0x13ff_0014 => println!(
                    "{}",
                    String::from_utf8_lossy(&self.systest_buffer[0..value.to_usize().unwrap()])
                ),
                _ => (),
            },
            _ => panic!("Unmapped Write: {:08X} <= {:08X}", address, value),
        }
    }

    fn step(&mut self) {
        match self.rsp.step() {
            DmaType::None => (),
            DmaType::Rsp(request) => self.rsp_dma_transfer(request),
            DmaType::Rdp(request) => self.rdp_dma_transfer(request),
        }

        self.vi.step(CYCLES_PER_STEP);
        self.ai.step(CYCLES_PER_STEP);
    }

    fn poll(&self) -> u8 {
        self.cpu_int.status()
    }
}
