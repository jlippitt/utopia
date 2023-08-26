use std::fs;
use std::io;
use std::path::PathBuf;
use tracing::warn;

pub struct BiosLoader {
    base_path: PathBuf,
}

impl BiosLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl utopia::BiosLoader for BiosLoader {
    type Error = io::Error;

    fn load(&self, name: &str) -> Result<Vec<u8>, io::Error> {
        let file_name = format!("{}.bin", name);
        let path = self.base_path.with_file_name(file_name);

        fs::read(&path).map_err(|err| {
            warn!("Failed to load BIOS file '{}': {}", path.display(), err);
            err
        })
    }
}
