#![allow(
    clippy::collapsible_else_if,
    clippy::comparison_chain,
    clippy::identity_op,
    clippy::manual_range_contains,
    clippy::match_single_binding,
    clippy::neg_multiply,
    clippy::single_match
)]

pub use system::{
    create, AudioQueue, Instance, InstanceOptions, JoypadState, System, SystemOptions, SystemType,
    WgpuContext,
};

use std::error;
use std::fmt;

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

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl error::Error for Error {}

pub trait BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultBiosLoader;

impl BiosLoader for DefaultBiosLoader {
    fn load(&self, _name: &str) -> Result<Vec<u8>, Error> {
        Err("BIOS loader not available".into())
    }
}

pub trait Mapped: MirrorableMut<Output = u8> {}

impl<T: MirrorableMut<Output = u8>> Mapped for T {}

pub trait MemoryMapper {
    type Mapped: Mapped;
    fn open(&self, len: usize, battery_backed: bool) -> Result<Self::Mapped, Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultMemoryMapper;

impl MemoryMapper for DefaultMemoryMapper {
    type Mapped = Vec<u8>;

    fn open(&self, len: usize, _battery_backed: bool) -> Result<Self::Mapped, Error> {
        Ok(vec![0; len])
    }
}
