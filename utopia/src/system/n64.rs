use super::{InstanceOptions, JoypadState, MemoryMapper, SystemOptions};
use crate::core::mips::{Bus, Core, Cp0, Interrupt, State};
use crate::util::facade::{ReadFacade, Value, WriteFacade};
use crate::WgpuContext;
use audio::AudioInterface;
use interrupt::{CpuInterrupt, RcpInterrupt};
use mips::MipsInterface;
use peripheral::PeripheralInterface;
use rdp::Rdp;
use rdram::Rdram;
use rsp::{DmaType, Registers, Rsp, DMEM_SIZE};
use serial::SerialInterface;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use tracing::info;
use video::VideoInterface;

mod audio;
mod dma;
mod header;
mod interrupt;
mod mips;
mod peripheral;
mod rdp;
mod rdram;
mod rsp;
mod serial;
mod video;

const CYCLES_PER_STEP: u64 = 2;

const IPL3_START_ADDRESS: u32 = 0xA4000040;

pub struct System<T: MemoryMapper + 'static> {
    _phantom: PhantomData<T>,
}

impl<T: MemoryMapper> System<T> {
    pub fn new(_options: SystemOptions<T>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: MemoryMapper> crate::System<T> for System<T> {
    fn default_resolution(&self) -> (u32, u32) {
        (video::DEFAULT_WIDTH * 2, video::DEFAULT_HEIGHT)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        None
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        Ok(Box::new(Instance::new(options)?))
    }
}

pub struct Instance {
    core: Core<Hardware>,
}

impl Instance {
    pub fn new(options: InstanceOptions) -> Result<Self, crate::Error> {
        let wgpu_context = options
            .wgpu_context
            .ok_or("This system requires a WGPU context".to_string())?;

        let header = header::parse(&options.rom_data);

        info!("Title: {}", header.title);
        info!("ROM Size: {}", options.rom_data.len());
        info!("Boot Address: {:08X}", header.boot_address);

        let hw = Hardware::new(options.rom_data, wgpu_context);

        let mut regs: [u64; 32] = Default::default();

        regs[19] = 0; // $S3
        regs[20] = 1; // $S4
        regs[21] = 0; // $S5
        regs[22] = 0x3f; // $S6
        regs[23] = 0; // $S7
        regs[29] = 0xa4001ff0u32 as i32 as i64 as u64; // $SP

        let core = Core::new(
            hw,
            Cp0::new(),
            (),
            State {
                pc: IPL3_START_ADDRESS,
                regs,
                ..Default::default()
            },
        );

        Ok(Instance { core })
    }
}

impl crate::Instance for Instance {
    fn resolution(&self) -> (u32, u32) {
        let video = &self.core.bus().video;
        (video.output_width(), video.output_height())
    }

    fn pixels(&self) -> &[u8] {
        unimplemented!("Raw pixel output for N64");
    }

    fn run_frame(&mut self, joypad_state: &JoypadState) {
        let core = &mut self.core;

        core.bus_mut().serial.update_joypad(joypad_state);
        core.bus_mut().video.start_frame();

        while !core.bus().video.ready() {
            core.step();
        }

        let bus = core.bus_mut();

        bus.video
            .update_pixel_buffer(&bus.wgpu_context, bus.rdram.data());
    }
}

struct Hardware {
    cycles: u64,
    interrupt: CpuInterrupt,
    wgpu_context: WgpuContext,
    rdram: Rdram,
    rsp: Rsp,
    rdp: Rdp,
    mips: MipsInterface,
    video: VideoInterface,
    audio: AudioInterface,
    peripheral: PeripheralInterface,
    serial: SerialInterface,
    rom: Vec<u8>,
}

impl Hardware {
    pub fn new(rom: Vec<u8>, wgpu_context: WgpuContext) -> Self {
        let interrupt = CpuInterrupt::new();
        let rcp_interrupt = RcpInterrupt::new(interrupt.clone());
        let shared_regs = Rc::new(RefCell::new(Registers::new(rcp_interrupt.clone())));

        let video = VideoInterface::new(&wgpu_context, rcp_interrupt.clone());

        Self {
            cycles: 0,
            interrupt,
            wgpu_context,
            rdram: Rdram::new(),
            rsp: Rsp::new(&rom[0..DMEM_SIZE], shared_regs.clone()),
            rdp: Rdp::new(shared_regs),
            mips: MipsInterface::new(rcp_interrupt.clone()),
            video,
            audio: AudioInterface::new(rcp_interrupt.clone()),
            peripheral: PeripheralInterface::new(rcp_interrupt.clone()),
            serial: SerialInterface::new(rcp_interrupt),
            rom,
        }
    }

    fn read_physical<T: Value>(&mut self, address: u32) -> T {
        match address >> 20 {
            0x000..=0x03e => self.rdram.read_data(address),
            0x03f => self.rdram.read_register(address & 0x000f_ffff),
            0x040 => {
                let index = address & 0x000f_ffff;

                if index < 0x0004_0000 {
                    self.rsp.read_ram(index)
                } else {
                    self.rsp.read_be(index)
                }
            }
            0x041 => self.rdp.read_be(address & 0x000f_ffff),
            0x042 => self.rdp.span().read_be(address & 0x000f_ffff),
            0x043 => self.mips.read_be(address & 0x000f_ffff),
            0x044 => self.video.read_be(address & 0x000f_ffff),
            0x045 => self.audio.read_be(address & 0x000f_ffff),
            0x046 => self.peripheral.read_be(address & 0x000f_ffff),
            0x047 => self.rdram.read_interface(address & 0x000f_ffff),
            0x048 => self.serial.read_be(address & 0x000f_ffff),
            0x080..=0x0ff => todo!("SRAM Reads"),
            0x100..=0x1fb => {
                let index = address as usize & 0x0fff_ffff;

                if index < self.rom.len() {
                    self.rom.read_be(index)
                } else {
                    // TODO: Open bus behaviour?
                    T::default()
                }
            }
            0x1fc => self.serial.pif().read_be(address & 0x000f_ffff),
            _ => panic!("Read from open bus: {:08X}", address),
        }
    }

    fn write_physical<T: Value>(&mut self, address: u32, value: T) {
        match address >> 20 {
            0x000..=0x03e => self.rdram.write_data(address, value),
            0x03f => self.rdram.write_register(address & 0x000f_ffff, value),
            0x040 => {
                let index = address & 0x000f_ffff;

                if index < 0x0004_0000 {
                    self.rsp.write_ram(index, value);
                } else {
                    self.rsp.write_be(index, value);

                    if let Some(request) = self.rsp.dma_requested() {
                        self.rsp_dma(request);
                    }
                }
            }
            0x041 => {
                self.rdp.write_be(address & 0x000f_ffff, value);

                if let Some(request) = self.rdp.dma_requested() {
                    self.rdp_dma(request);
                }
            }
            0x042 => self.rdp.span_mut().write_be(address & 0x000f_ffff, value),
            0x043 => self.mips.write_be(address & 0x000f_ffff, value),
            0x044 => self.video.write_be(address & 0x000f_ffff, value),
            0x045 => self.audio.write_be(address & 0x000f_ffff, value),
            0x046 => {
                self.peripheral.write_be(address & 0x000f_ffff, value);

                if let Some(request) = self.peripheral.dma_requested() {
                    self.peripheral_dma(request);
                }
            }
            0x047 => self.rdram.write_interface(address & 0x000f_ffff, value),
            0x048 => {
                self.serial.write_be(address & 0x000f_ffff, value);

                if let Some(request) = self.serial.dma_requested() {
                    self.serial_dma(request);
                }
            }
            0x080..=0x0ff => todo!("SRAM Writes"),
            0x100..=0x1fb => panic!("Write to ROM area: {:08X}", address),
            0x1fc => self.serial.pif_mut().write_be(address & 0x000f_ffff, value),
            _ => panic!("Write to open bus: {:08X}", address),
        }
    }
}

impl Bus for Hardware {
    type Cp0 = Cp0;
    type Cp2 = ();

    const CP1: bool = true;
    const MUL_DIV: bool = true;
    const INSTR_64: bool = true;

    fn read<T: Value>(&mut self, address: u32) -> T {
        match address >> 29 {
            4 => self.read_physical(address - 0x8000_0000), // TODO: Caching
            5 => self.read_physical(address - 0xa000_0000),
            _ => todo!("TLB"),
        }
    }

    fn write<T: Value>(&mut self, address: u32, value: T) {
        match address >> 29 {
            4 => self.write_physical(address - 0x8000_0000, value), // TODO: Caching
            5 => self.write_physical(address - 0xa000_0000, value),
            _ => todo!("TLB"),
        }
    }

    fn step(&mut self) {
        self.cycles += CYCLES_PER_STEP;

        match self.rsp.step() {
            DmaType::None => (),
            DmaType::Rsp(dma) => self.rsp_dma(dma),
            DmaType::Rdp(dma) => self.rdp_dma(dma),
        }

        self.video.step(CYCLES_PER_STEP);
        self.audio.step(CYCLES_PER_STEP);
    }

    fn poll(&self) -> Interrupt {
        self.interrupt.poll()
    }
}
