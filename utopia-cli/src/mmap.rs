use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

pub struct MemoryMapper {
    rom_path: PathBuf,
}

impl MemoryMapper {
    pub fn new(rom_path: PathBuf) -> Self {
        Self { rom_path }
    }
}

impl utopia::MemoryMapper for MemoryMapper {
    type Mapped = MmapMut;
    type Error = io::Error;

    fn open(&self, len: usize, battery_backed: bool) -> Result<Self::Mapped, io::Error> {
        let mapped = if battery_backed {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(self.rom_path.with_extension("sav"))?;

            file.set_len(len as u64)?;

            unsafe { MmapOptions::new().map_mut(&file)? }
        } else {
            MmapOptions::new().len(len).map_anon()?
        };

        Ok(mapped)
    }
}
