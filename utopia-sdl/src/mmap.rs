use memmap2::{MmapMut, MmapOptions};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;

pub struct MemoryMapper;

impl MemoryMapper {
    pub fn new() -> Self {
        Self
    }
}

impl utopia::MemoryMapper for MemoryMapper {
    type Mapped = MmapMut;

    fn open(&self, path: Option<&Path>, len: usize) -> Result<Self::Mapped, Box<dyn Error>> {
        let mapped = if let Some(path) = path {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)?;

            file.set_len(len as u64)?;

            unsafe { MmapOptions::new().map_mut(&file)? }
        } else {
            MmapOptions::new().len(len).map_anon()?
        };

        Ok(mapped)
    }
}
