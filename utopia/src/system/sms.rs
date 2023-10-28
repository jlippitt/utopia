use super::{InstanceOptions, JoypadState, MemoryMapper, Size, SystemOptions, WgpuContext};
use crate::core::z80::{self, Core};
use std::fmt;
use std::marker::PhantomData;
use tracing::trace;

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
    fn default_output_resolution(&self) -> Size {
        Size::new(256, 192)
    }

    fn default_sample_rate(&self) -> Option<u64> {
        None
    }

    fn create_instance(
        &self,
        options: InstanceOptions,
    ) -> Result<Box<dyn crate::Instance>, crate::Error> {
        Ok(Box::new(Instance::new(options)))
    }
}

pub struct Instance {
    core: Core<Bus>,
    _wgpu_context: WgpuContext,
}

impl Instance {
    pub fn new(options: InstanceOptions) -> Self {
        let bus = Bus::new(options.rom_data);
        let core = Core::new(bus);

        Self {
            core,
            _wgpu_context: options.wgpu_context,
        }
    }
}

impl crate::Instance for Instance {
    fn run_frame(&mut self, _joypad_state: &JoypadState) {
        let core = &mut self.core;

        loop {
            trace!("{}", core);
            core.step();
        }
    }

    fn present(&self, _canvas: &wgpu::Texture) {
        // TODO: Render pixels to WGPU texture
    }
}

pub struct Bus {
    _rom: Box<[u8]>,
}

impl Bus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            _rom: rom_data.into_boxed_slice(),
        }
    }
}

impl z80::Bus for Bus {
    //
}

impl fmt::Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
