use bytemuck::Pod;
use num_traits::{AsPrimitive, Bounded, PrimInt};
use std::mem;
use std::ops::{Deref, DerefMut};
use tracing::trace;

pub struct Memory {
    data: Box<[u8]>,
}

impl Memory {
    pub fn new(len: usize) -> Self {
        Self {
            data: vec![0; len].into_boxed_slice(),
        }
    }

    pub fn read_le<T: Value>(&self, address: usize) -> T {
        let index = address >> std::mem::size_of::<T>().ilog2();
        let slice = bytemuck::cast_slice::<u8, T>(&self.data);
        slice[index]
    }

    pub fn read_be<T: Value>(&self, address: usize) -> T {
        self.read_le::<T>(address).swap_bytes()
    }

    pub fn try_read_le<T: Value>(&self, address: usize) -> Option<T> {
        let index = address >> std::mem::size_of::<T>().ilog2();
        let slice = bytemuck::cast_slice::<u8, T>(&self.data);
        slice.get(index).cloned()
    }

    pub fn try_read_be<T: Value>(&self, address: usize) -> Option<T> {
        self.try_read_le::<T>(address)
            .map(|value| value.swap_bytes())
    }

    pub fn read_be_unaligned<T: Value>(&self, address: usize, mirror: Option<usize>) -> T {
        let size = std::mem::size_of::<T>();
        let align_mask = size - 1;

        if (address & align_mask) == 0 {
            return self.read_be(address);
        }

        let mirror_mask = mirror.unwrap_or(usize::MAX);
        let mut value = T::zeroed();

        for index in 0..size {
            let byte_address = address.wrapping_add(index) & mirror_mask;
            let shift = (index ^ align_mask) * 8;
            let byte_value = T::from(self.data[byte_address]).unwrap();
            value = value | (byte_value << shift);
        }

        value
    }

    pub fn write_le<T: Value>(&mut self, address: usize, value: T) {
        let index = address >> std::mem::size_of::<T>().ilog2();
        let slice = bytemuck::cast_slice_mut::<u8, T>(&mut self.data);
        slice[index] = value
    }

    pub fn write_be<T: Value>(&mut self, address: usize, value: T) {
        self.write_le(address, value.swap_bytes())
    }

    pub fn try_write_le<T: Value>(&mut self, address: usize, value: T) -> bool {
        let index = address >> std::mem::size_of::<T>().ilog2();
        let slice = bytemuck::cast_slice_mut::<u8, T>(&mut self.data);

        slice.get_mut(index).is_some_and(|element| {
            *element = value;
            true
        })
    }

    pub fn try_write_be<T: Value>(&mut self, address: usize, value: T) -> bool {
        self.try_write_le(address, value.swap_bytes())
    }

    pub fn write_be_unaligned<T: Value>(
        &mut self,
        address: usize,
        value: T,
        mirror: Option<usize>,
    ) {
        let size = std::mem::size_of::<T>();
        let align_mask = size - 1;

        if (address & align_mask) == 0 {
            return self.write_be(address, value);
        }

        let mirror_mask = mirror.unwrap_or(usize::MAX);

        for index in 0..size {
            let byte_address = address.wrapping_add(index) & mirror_mask;
            let shift = (index ^ align_mask) * 8;
            let byte_value = value >> shift;
            self.data[byte_address] = T::as_(byte_value)
        }
    }
}

impl Deref for Memory {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl DerefMut for Memory {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl From<Vec<u8>> for Memory {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value.into_boxed_slice(),
        }
    }
}

impl From<&[u8]> for Memory {
    fn from(value: &[u8]) -> Self {
        Self {
            data: Vec::from(value).into_boxed_slice(),
        }
    }
}

pub trait Reader {
    type Value: Value;

    fn read_register(&self, address: u32) -> Self::Value;

    fn read_be<T: Value>(&self, address: u32) -> T {
        let self_size = mem::size_of::<Self::Value>();
        let other_size = mem::size_of::<T>();
        debug_assert!((self_size % other_size) == 0);
        let flip_mask = self_size - other_size;
        let shift = (address as usize & flip_mask ^ flip_mask) << 3;
        T::from_truncate(self.read_register(address & !(self_size as u32 - 1)) >> shift)
    }
}

pub trait Writer: Reader {
    type SideEffect;

    fn write_register(&mut self, address: u32, value: Masked<Self::Value>) -> Self::SideEffect;

    fn write_be<T: Value + AsPrimitive<Self::Value>>(
        &mut self,
        address: u32,
        value: T,
    ) -> Self::SideEffect {
        let self_size = mem::size_of::<Self::Value>();
        let other_size = mem::size_of::<T>();
        debug_assert!((self_size % other_size) == 0);
        let flip_mask = self_size - other_size;
        let shift = (address as usize & flip_mask ^ flip_mask) << 3;
        let shifted_value = <T as AsPrimitive<Self::Value>>::as_(value) << shift;
        let value_mask = T::max_value() << shift;
        self.write_register(
            address & !(self_size as u32 - 1),
            Masked::new(shifted_value, value_mask.as_()),
        )
    }
}

pub trait Value:
    PrimInt
    + Bounded
    + AsPrimitive<u8>
    + AsPrimitive<u16>
    + AsPrimitive<u32>
    + Pod
    + std::fmt::UpperHex
    + std::fmt::Display
    + std::fmt::Debug
{
    fn from_truncate<T: Value>(other: T) -> Self {
        unsafe { mem::transmute_copy(&other) }
    }
}

impl Value for u8 {}

impl Value for u16 {}

impl Value for u32 {}

pub struct Masked<T: Value> {
    value: T,
    mask: T,
}

impl<T: Value> Masked<T> {
    pub fn new(value: T, mask: T) -> Self {
        Self { value, mask }
    }

    pub fn get(&self) -> T {
        self.value & self.mask
    }

    pub fn apply(&self, other: T) -> T {
        (other & !self.mask) | (self.value & self.mask)
    }
}

impl<T: Value> From<T> for Masked<T> {
    fn from(value: T) -> Masked<T> {
        Self::new(value, T::max_value())
    }
}

impl Masked<u32> {
    pub fn write_reg<U: Copy + From<u32> + Into<u32> + std::fmt::Debug>(
        &self,
        name: &'static str,
        reg: &mut U,
    ) {
        *reg = self.apply((*reg).into()).into();
        trace!("{}: {:?}", name, *reg);
    }

    pub fn write_reg_hex<U: Copy + From<u32> + Into<u32> + std::fmt::Debug>(
        &self,
        name: &'static str,
        reg: &mut U,
    ) {
        *reg = self.apply((*reg).into()).into();
        trace!("{}: {:#X?}", name, *reg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DATA: &[u8] = &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];

    struct Reg<T: Value> {
        data: Box<[T]>,
    }

    impl<T: Value> Reg<T> {
        fn new(value: &[T]) -> Self {
            Self {
                data: Vec::from(value).into_boxed_slice(),
            }
        }
    }

    impl<T: Value> Reader for Reg<T> {
        type Value = T;

        fn read_register(&self, address: u32) -> T {
            let shift = mem::size_of::<T>().ilog2();
            let index = (address >> shift) as usize;
            println!("{:?} {}", self.data, index);
            self.data[index]
        }
    }

    impl<T: Value> Writer for Reg<T> {
        type SideEffect = ();

        fn write_register(&mut self, address: u32, value: Masked<T>) {
            let shift = mem::size_of::<T>().ilog2();
            let index = (address >> shift) as usize;
            self.data[index] = value.apply(self.data[index])
        }
    }

    #[test]
    fn memory_read_write_le_aligned() {
        let mut memory = Memory::from(DATA);

        assert_eq!(memory.read_le::<u32>(0), 0x33221100);
        assert_eq!(memory.read_le::<u16>(4), 0x5544);
        assert_eq!(memory.read_le::<u8>(6), 0x66);

        memory.write_le::<u32>(4, 0xbbaa9988);
        memory.write_le::<u16>(2, 0xddcc);
        memory.write_le::<u8>(1, 0xee);

        assert_eq!(
            memory.deref(),
            &[0x00, 0xee, 0xcc, 0xdd, 0x88, 0x99, 0xaa, 0xbb]
        );
    }

    #[test]
    fn memory_read_write_be_aligned() {
        let mut memory = Memory::from(DATA);

        assert_eq!(memory.read_be::<u32>(0), 0x00112233);
        assert_eq!(memory.read_be::<u16>(4), 0x4455);
        assert_eq!(memory.read_be::<u8>(6), 0x66);

        memory.write_be::<u32>(4, 0x8899aabb);
        memory.write_be::<u16>(2, 0xccdd);
        memory.write_be::<u8>(1, 0xee);

        assert_eq!(
            memory.deref(),
            &[0x00, 0xee, 0xcc, 0xdd, 0x88, 0x99, 0xaa, 0xbb]
        );
    }

    #[test]
    fn memory_read_write_be_unaligned() {
        let mut memory = Memory::from(DATA);

        assert_eq!(memory.read_be_unaligned::<u32>(1, None), 0x11223344);
        assert_eq!(memory.read_be_unaligned::<u16>(5, None), 0x5566);
        assert_eq!(memory.read_be_unaligned::<u8>(7, None), 0x77);

        memory.write_be_unaligned::<u32>(3, 0x8899aabb, None);
        memory.write_be_unaligned::<u16>(1, 0xccdd, None);
        memory.write_be_unaligned::<u8>(0, 0xee, None);

        assert_eq!(
            memory.deref(),
            &[0xee, 0xcc, 0xdd, 0x88, 0x99, 0xaa, 0xbb, 0x77]
        );
    }

    #[test]
    fn memory_read_write_be_unaligned_mirror() {
        let mut memory = Memory::from(DATA);

        assert_eq!(memory.read_be_unaligned::<u32>(1, Some(3)), 0x11223300);

        memory.write_be_unaligned::<u32>(5, 0x8899aabb, Some(7));

        assert_eq!(
            memory.deref(),
            &[0xbb, 0x11, 0x22, 0x33, 0x44, 0x88, 0x99, 0xaa]
        );
    }

    #[test]
    fn read_register_u32_be() {
        let reg = Reg::<u32>::new(&[0x00112233, 0x44556677]);

        assert_eq!(reg.read_be::<u32>(0), 0x00112233);
        assert_eq!(reg.read_be::<u32>(4), 0x44556677);

        assert_eq!(reg.read_be::<u16>(0), 0x0011);
        assert_eq!(reg.read_be::<u16>(2), 0x2233);
        assert_eq!(reg.read_be::<u16>(4), 0x4455);
        assert_eq!(reg.read_be::<u16>(6), 0x6677);

        assert_eq!(reg.read_be::<u8>(0), 0x00);
        assert_eq!(reg.read_be::<u8>(1), 0x11);
        assert_eq!(reg.read_be::<u8>(2), 0x22);
        assert_eq!(reg.read_be::<u8>(3), 0x33);
        assert_eq!(reg.read_be::<u8>(4), 0x44);
        assert_eq!(reg.read_be::<u8>(5), 0x55);
        assert_eq!(reg.read_be::<u8>(6), 0x66);
        assert_eq!(reg.read_be::<u8>(7), 0x77);
    }

    #[test]
    fn write_register_u32_be() {
        let mut reg = Reg::<u32>::new(&[0x00112233, 0x44556677]);

        reg.write_be::<u32>(4, 0x8899aabb);
        assert_eq!(reg.read_be::<u32>(4), 0x8899aabb);

        reg.write_be::<u16>(6, 0xccdd);
        assert_eq!(reg.read_be::<u32>(4), 0x8899ccdd);

        reg.write_be::<u8>(7, 0xee);
        assert_eq!(reg.read_be::<u32>(4), 0x8899ccee);
    }
}
