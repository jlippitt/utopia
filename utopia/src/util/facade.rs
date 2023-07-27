use std::mem;
use std::ops::Deref;

pub trait Primitive: Copy + Clone + Default + Eq + PartialEq {
    fn from_be_slice(slice: &[u8]) -> Self;
}

pub trait ReadFacade {
    fn read_be<T: Primitive>(&self, index: usize) -> T;
}

pub trait WriteFacade: ReadFacade {}

impl<T: Deref<Target = [u8]>> ReadFacade for T {
    fn read_be<U: Primitive>(&self, index: usize) -> U {
        let bytes = &self[index..(index + mem::size_of::<U>())];
        U::from_be_slice(bytes)
    }
}

impl Primitive for u32 {
    fn from_be_slice(slice: &[u8]) -> Self {
        u32::from_be_bytes(slice.try_into().unwrap())
    }
}
