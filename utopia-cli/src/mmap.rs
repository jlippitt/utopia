use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;
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

    fn open(&self, len: usize, battery_backed: bool) -> Result<Self::Mapped, utopia::Error> {
        let result = if battery_backed {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(self.rom_path.with_extension("sav"))
                .and_then(|file| file.set_len(len as u64).map(|_| file))
                .and_then(|file| unsafe { MmapOptions::new().map_mut(&file) })
        } else {
            MmapOptions::new().len(len).map_anon()
        };

        result.map_err(|err| err.to_string().into())
    }
}
