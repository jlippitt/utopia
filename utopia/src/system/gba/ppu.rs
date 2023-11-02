use crate::util::memory::Memory;

const VRAM_SIZE: usize = 98304;

pub struct Ppu {
    vram: Memory,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: Memory::new(VRAM_SIZE),
        }
    }

    pub fn vram(&self) -> &Memory {
        &self.vram
    }

    pub fn vram_mut(&mut self) -> &mut Memory {
        &mut self.vram
    }
}
