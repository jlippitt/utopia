use std::fs;
use std::path::PathBuf;
use tracing::warn;

#[derive(Debug)]
pub struct BiosLoader {
    base_path: PathBuf,
}

impl BiosLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl utopia_winit::BiosLoader for BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, utopia_winit::Error> {
        let file_name = format!("{}.bin", name);
        let path = self.base_path.with_file_name(file_name);

        let result = fs::read(&path).map_err(|err| {
            warn!("Failed to load BIOS file '{}': {}", path.display(), err);
            err
        });

        result.map_err(|err| err.to_string().into())
    }
}
