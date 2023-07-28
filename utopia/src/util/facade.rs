use num_traits::{FromBytes, FromPrimitive, NumCast, PrimInt, ToBytes, ToPrimitive, WrappingAdd};
use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};

pub trait Address:
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
    + FromBytes
    + ToBytes
    + WrappingAdd
    + fmt::Debug
    + fmt::Display
    + fmt::LowerHex
    + fmt::UpperHex
    + fmt::Binary
{
    const BITS: u32;
    const MASK: usize;

    fn from_address<T: Address>(other: T) -> Self {
        if Self::BITS >= T::BITS {
            Self::from(other).unwrap()
        } else {
            unsafe { mem::transmute_copy::<T, Self>(&other) }
        }
    }

    fn from_value<T: Value>(other: T) -> Self {
        Self::from_address(other)
    }

    fn from_be_slice<'a>(other: &'a [u8]) -> Self
    where
        &'a <Self as FromBytes>::Bytes: 'a,
        &'a <Self as FromBytes>::Bytes: From<&'a [u8]>,
    {
        Self::from_be_bytes(other.try_into().unwrap())
    }

    fn from_le_slice<'a>(other: &'a [u8]) -> Self
    where
        &'a <Self as FromBytes>::Bytes: 'a,
        &'a <Self as FromBytes>::Bytes: From<&'a [u8]>,
    {
        Self::from_le_bytes(other.try_into().unwrap())
    }
}

pub trait Value: Address {}

pub trait ReadFacade {
    type Address: Address;
    fn read_be<T: Value>(&self, address: Self::Address) -> T;
    fn read_le<T: Value>(&self, address: Self::Address) -> T;
}

pub trait WriteFacade: ReadFacade {
    fn write_be<T: Value>(&mut self, address: Self::Address, value: T);
    fn write_le<T: Value>(&mut self, address: Self::Address, value: T);
}

pub trait DataReader {
    type Address: Address;
    type Value: Value;
    fn read(&self, address: Self::Address) -> Self::Value;
}

pub trait DataWriter: DataReader {
    fn write(&mut self, address: Self::Address, value: Self::Value);
}

fn read_data<T: DataReader, U: Value, const BE: bool>(reader: &T, address: T::Address) -> U {
    if U::BITS < T::Value::BITS {
        let value = reader.read(address);
        let mask = T::Address::from_value((T::Value::BITS / U::BITS) - 1);
        let flip = if BE { mask } else { Default::default() };
        let shift = (((address & mask) ^ flip) << 3).to_usize().unwrap();
        U::from_value(value >> shift)
    } else if U::BITS > T::Value::BITS {
        let mask = ((U::BITS / T::Value::BITS) - 1) as usize;
        let flip = if BE { mask } else { Default::default() };
        let mut result: U = Default::default();

        for chunk_index in 0..((U::BITS / T::Value::BITS) as usize) {
            let chunk_address = address.wrapping_add(&Address::from_address(chunk_index));
            let chunk = reader.read(T::Address::from_address(chunk_address));
            result = result | (U::from_value(chunk) << ((chunk_index ^ flip) << 3));
        }

        result
    } else {
        U::from_value(reader.read(address))
    }
}

fn write_data<T: DataWriter, U: Value, const BE: bool>(
    writer: &mut T,
    address: T::Address,
    value: U,
) {
    if U::BITS < T::Value::BITS {
        todo!("Inexact writes");
    } else if U::BITS > T::Value::BITS {
        let mask = ((U::BITS / T::Value::BITS) - 1) as usize;
        let flip = if BE { mask } else { Default::default() };

        for chunk_index in 0..((U::BITS / T::Value::BITS) as usize) {
            let chunk_address = address.wrapping_add(&Address::from_address(chunk_index));
            let chunk = T::Value::from_value(value >> (8 * (chunk_index ^ flip)));
            writer.write(T::Address::from_address(chunk_address), chunk);
        }
    } else {
        writer.write(address, T::Value::from_value(value));
    }
}

impl<T: DataReader> ReadFacade for T {
    type Address = T::Address;

    fn read_be<U: Value>(&self, address: Self::Address) -> U {
        read_data::<T, U, true>(self, address)
    }

    fn read_le<U: Value>(&self, address: Self::Address) -> U {
        read_data::<T, U, false>(self, address)
    }
}

impl<T: DataWriter> WriteFacade for T {
    fn write_be<U: Value>(&mut self, address: Self::Address, value: U) {
        write_data::<T, U, true>(self, address, value);
    }

    fn write_le<U: Value>(&mut self, address: Self::Address, value: U) {
        write_data::<T, U, false>(self, address, value);
    }
}

impl<T: Deref<Target = [u8]>> DataReader for T {
    type Address = usize;
    type Value = u8;

    fn read(&self, address: Self::Address) -> Self::Value {
        self[address]
    }
}

impl<T: DerefMut<Target = [u8]>> DataWriter for T {
    fn write(&mut self, address: Self::Address, value: Self::Value) {
        self[address] = value;
    }
}

impl Address for u8 {
    const BITS: u32 = u8::BITS;
    const MASK: usize = u8::MAX as usize;
}

impl Value for u8 {}

impl Address for u32 {
    const BITS: u32 = u32::BITS;
    const MASK: usize = u32::MAX as usize;
}

impl Value for u32 {}

impl Address for usize {
    const BITS: u32 = usize::BITS;
    const MASK: usize = u32::MAX as usize;
}
