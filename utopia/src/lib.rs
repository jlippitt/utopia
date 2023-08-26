#![allow(
    clippy::collapsible_else_if,
    clippy::comparison_chain,
    clippy::identity_op,
    clippy::manual_range_contains,
    clippy::match_single_binding,
    clippy::neg_multiply,
    clippy::single_match
)]

pub use system::{AudioQueue, JoypadState, System};

use std::error;
use std::fmt;
use std::path::Path;

use util::mirror::MirrorableMut;

#[cfg(feature = "cpu-tests")]
pub mod core;

#[cfg(not(feature = "cpu-tests"))]
mod core;

mod system;
mod util;

#[derive(Clone, Debug)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {}

pub trait BiosLoader {
    type Error: error::Error + 'static;
    fn load(&self, name: &str) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultBiosLoader;

impl BiosLoader for DefaultBiosLoader {
    type Error = Error;
    fn load(&self, _name: &str) -> Result<Vec<u8>, Self::Error> {
        Err(Error("BIOS loader not available".into()))
    }
}

pub trait Mapped: MirrorableMut<Output = u8> {}

impl<T: MirrorableMut<Output = u8>> Mapped for T {}

pub trait MemoryMapper {
    type Mapped: Mapped;
    type Error: error::Error + 'static;
    fn open(&self, len: usize, battery_backed: bool) -> Result<Self::Mapped, Self::Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultMemoryMapper;

impl MemoryMapper for DefaultMemoryMapper {
    type Mapped = Vec<u8>;
    type Error = Error;

    fn open(&self, len: usize, _battery_backed: bool) -> Result<Self::Mapped, Self::Error> {
        Ok(vec![0; len])
    }
}

#[derive(Clone, Debug)]
pub struct CreateOptions<T: MemoryMapper, U: BiosLoader> {
    pub memory_mapper: T,
    pub bios_loader: U,
    pub skip_boot: bool,
}

pub fn create<T: MemoryMapper + 'static, U: BiosLoader>(
    rom_path: &str,
    rom_data: Vec<u8>,
    options: &CreateOptions<T, U>,
) -> Result<Box<dyn System>, Error> {
    let extension = Path::new(rom_path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or("".to_owned());

    system::create(&extension, rom_data, options).map_err(|err| Error(format!("{}", err)))
}
