use std::mem;

pub trait Primitive: Copy + Clone + Default + Eq + PartialEq {
    fn from_be_slice(slice: &[u8]) -> Self;
}

pub trait Facade {
    fn read_be<T: Primitive>(&self, index: usize) -> T;
}

impl Facade for [u8] {
    fn read_be<T: Primitive>(&self, index: usize) -> T {
        let bytes = &self[index..(index + mem::size_of::<T>())];
        T::from_be_slice(bytes)
    }
}

impl Primitive for u32 {
    fn from_be_slice(slice: &[u8]) -> Self {
        u32::from_be_bytes(slice.try_into().unwrap())
    }
}
