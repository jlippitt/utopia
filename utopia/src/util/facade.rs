use num_traits::{FromPrimitive, NumCast, PrimInt, ToPrimitive};
use std::fmt;
use std::ops::Deref;

pub trait Value:
    Copy
    + Clone
    + Default
    + Eq
    + PartialEq
    + Ord
    + PartialOrd
    + PrimInt
    + NumCast
    + FromPrimitive
    + ToPrimitive
    + fmt::Debug
    + fmt::Display
    + fmt::LowerHex
    + fmt::UpperHex
    + fmt::Binary
{
    const BITS: u32;
    const MASK: usize;

    fn from_value<T: Value>(other: T) -> Self {
        Self::from(other).unwrap()
    }

    fn from_be_slice(other: &[u8]) -> Self;
}

pub trait ReadFacade {
    fn read_be<T: Value>(&self, index: usize) -> T;
}

pub trait WriteFacade {}

pub trait Facade: ReadFacade + WriteFacade {}

pub trait DataReader {
    type Address: Value;
    type Value: Value;
    fn read(&self, address: Self::Address) -> Self::Value;
}

pub trait DataWriter: DataReader {
    fn write(&mut self, address: Self::Address, value: Self::Value);
}

impl<T: DataReader> ReadFacade for T {
    fn read_be<U: Value>(&self, index: usize) -> U {
        if U::BITS < T::Value::BITS {
            let address = T::Address::from_value(index);
            let value = self.read(address);
            let mask = ((T::Value::BITS / U::BITS) - 1) as usize;
            let shift = 8 * ((index & mask) ^ mask);
            U::from_value(value >> shift)
        } else if U::BITS > T::Value::BITS {
            let mask = ((U::BITS / T::Value::BITS) - 1) as usize;
            let mut result: U = Default::default();

            for chunk_index in 0..((U::BITS / T::Value::BITS) as usize) {
                let address = index.wrapping_add(chunk_index) & T::Address::MASK;
                let value = self.read(T::Address::from_value(address));
                result = result | (U::from_value(value) << (8 * (chunk_index ^ mask)));
            }

            result
        } else {
            let address = T::Address::from_value(index);
            U::from_value(self.read(address))
        }
    }
}

impl<T: Deref<Target = [u8]>> DataReader for T {
    type Address = usize;
    type Value = u8;

    fn read(&self, address: Self::Address) -> Self::Value {
        self[address]
    }
}

impl Value for u8 {
    const BITS: u32 = u8::BITS;
    const MASK: usize = u8::MAX as usize;

    fn from_be_slice(other: &[u8]) -> Self {
        Self::from_be_bytes(other.try_into().unwrap())
    }
}

impl Value for u32 {
    const BITS: u32 = u32::BITS;
    const MASK: usize = u32::MAX as usize;

    fn from_be_slice(other: &[u8]) -> Self {
        Self::from_be_bytes(other.try_into().unwrap())
    }
}

impl Value for usize {
    const BITS: u32 = usize::BITS;
    const MASK: usize = u32::MAX as usize;

    fn from_be_slice(other: &[u8]) -> Self {
        Self::from_be_bytes(other.try_into().unwrap())
    }
}
