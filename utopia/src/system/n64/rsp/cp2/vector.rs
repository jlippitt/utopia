use num_traits::{FromBytes, ToBytes};
use std::fmt;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Vector([u16; 8]);

impl Vector {
    pub fn from_le_array(value: [u16; 8]) -> Self {
        Self(value)
    }

    pub fn to_le_array(self) -> [u16; 8] {
        self.0
    }

    pub fn from_be_array(value: [u16; 8]) -> Self {
        Self(std::array::from_fn(|index| value[index ^ 7]))
    }

    pub fn to_be_array(self) -> [u16; 8] {
        std::array::from_fn(|index| self.lane(index))
    }

    pub fn lane(&self, index: usize) -> u16 {
        self.0[index ^ 7]
    }

    pub fn set_lane(&mut self, index: usize, value: u16) {
        self.0[index ^ 7] = value;
    }

    pub fn read<'a, T: FromBytes>(&'a self, index: usize) -> T
    where
        T::Bytes: TryFrom<&'a [u8]>,
        T::Bytes: Default,
        <T::Bytes as std::convert::TryFrom<&'a [u8]>>::Error: fmt::Debug,
    {
        let src_bytes = bytemuck::bytes_of(&self.0);
        let len = std::mem::size_of::<T>();
        let end = 15usize.wrapping_sub(index) & 15;

        let mut dst_bytes = T::Bytes::default();

        for byte in 0..len {
            let element = end.wrapping_sub(byte) & 15;
            dst_bytes.as_mut()[byte] = src_bytes[element];
        }

        T::from_be_bytes(&dst_bytes)
    }

    pub fn write<T: ToBytes>(&mut self, index: usize, value: T)
    where
        T::Bytes: AsRef<[u8]>,
    {
        let bytes = bytemuck::bytes_of_mut(&mut self.0);
        let len = std::mem::size_of::<T>();
        let end = 16_usize.saturating_sub(index);
        let start = end.saturating_sub(len);
        bytes[start..end].copy_from_slice(&value.to_le_bytes().as_ref()[len - (end - start)..])
    }

    pub fn broadcast(&self, element: usize) -> Self {
        Self(match element & 15 {
            0 | 1 => self.0,
            2 => [
                self.lane(6),
                self.lane(6),
                self.lane(4),
                self.lane(4),
                self.lane(2),
                self.lane(2),
                self.lane(0),
                self.lane(0),
            ],
            3 => [
                self.lane(7),
                self.lane(7),
                self.lane(5),
                self.lane(5),
                self.lane(3),
                self.lane(3),
                self.lane(1),
                self.lane(1),
            ],
            4 => [
                self.lane(4),
                self.lane(4),
                self.lane(4),
                self.lane(4),
                self.lane(0),
                self.lane(0),
                self.lane(0),
                self.lane(0),
            ],
            5 => [
                self.lane(5),
                self.lane(5),
                self.lane(5),
                self.lane(5),
                self.lane(1),
                self.lane(1),
                self.lane(1),
                self.lane(1),
            ],
            6 => [
                self.lane(6),
                self.lane(6),
                self.lane(6),
                self.lane(6),
                self.lane(2),
                self.lane(2),
                self.lane(2),
                self.lane(2),
            ],
            7 => [
                self.lane(7),
                self.lane(7),
                self.lane(7),
                self.lane(7),
                self.lane(3),
                self.lane(3),
                self.lane(3),
                self.lane(3),
            ],
            8 => [self.lane(0); 8],
            9 => [self.lane(1); 8],
            10 => [self.lane(2); 8],
            11 => [self.lane(3); 8],
            12 => [self.lane(4); 8],
            13 => [self.lane(5); 8],
            14 => [self.lane(6); 8],
            15 => [self.lane(7); 8],
            _ => unreachable!(),
        })
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            self.0[7], self.0[6], self.0[5], self.0[4], self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

impl From<u128> for Vector {
    fn from(value: u128) -> Self {
        Self(bytemuck::cast(value))
    }
}

impl From<Vector> for u128 {
    fn from(value: Vector) -> u128 {
        bytemuck::cast(value.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut vec = Vector::from(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);

        assert_eq!(vec.read::<u64>(0), 0x0011_2233_4455_6677);
        assert_eq!(vec.read::<u32>(8), 0x8899_aabb);
        assert_eq!(vec.read::<u16>(12), 0xccdd);
        assert_eq!(vec.read::<u8>(14), 0xee);

        vec.write::<u64>(8, 0x0011_2233_4455_6677);
        vec.write::<u32>(4, 0x8899_aabb);
        vec.write::<u16>(2, 0xccdd);
        vec.write::<u8>(1, 0xee);

        assert_eq!(u128::from(vec), 0x00ee_ccdd_8899_aabb_0011_2233_4455_6677);
    }

    #[test]
    fn read_write_end() {
        let mut vec = Vector::from(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);

        assert_eq!(vec.read::<u64>(12), 0xccdd_eeff_0000_0000);

        vec.write::<u64>(12, 0x0011_2233_4455_6677);

        assert_eq!(u128::from(vec), 0x0011_2233_4455_6677_8899_aabb_0011_2233);
    }

    #[test]
    fn broadcast() {
        let vec = Vector::from(0x0000_1111_2222_3333_4444_5555_6666_7777);

        assert_eq!(
            u128::from(vec.broadcast(0)),
            0x0000_1111_2222_3333_4444_5555_6666_7777,
        );

        assert_eq!(
            u128::from(vec.broadcast(1)),
            0x0000_1111_2222_3333_4444_5555_6666_7777,
        );

        assert_eq!(
            u128::from(vec.broadcast(2)),
            0x0000_0000_2222_2222_4444_4444_6666_6666,
        );

        assert_eq!(
            u128::from(vec.broadcast(3)),
            0x1111_1111_3333_3333_5555_5555_7777_7777,
        );

        assert_eq!(
            u128::from(vec.broadcast(4)),
            0x0000_0000_0000_0000_4444_4444_4444_4444,
        );

        assert_eq!(
            u128::from(vec.broadcast(5)),
            0x1111_1111_1111_1111_5555_5555_5555_5555,
        );

        assert_eq!(
            u128::from(vec.broadcast(6)),
            0x2222_2222_2222_2222_6666_6666_6666_6666,
        );

        assert_eq!(
            u128::from(vec.broadcast(7)),
            0x3333_3333_3333_3333_7777_7777_7777_7777,
        );

        assert_eq!(
            u128::from(vec.broadcast(8)),
            0x0000_0000_0000_0000_0000_0000_0000_0000,
        );

        assert_eq!(
            u128::from(vec.broadcast(9)),
            0x1111_1111_1111_1111_1111_1111_1111_1111,
        );

        assert_eq!(
            u128::from(vec.broadcast(10)),
            0x2222_2222_2222_2222_2222_2222_2222_2222,
        );

        assert_eq!(
            u128::from(vec.broadcast(11)),
            0x3333_3333_3333_3333_3333_3333_3333_3333,
        );

        assert_eq!(
            u128::from(vec.broadcast(12)),
            0x4444_4444_4444_4444_4444_4444_4444_4444,
        );

        assert_eq!(
            u128::from(vec.broadcast(13)),
            0x5555_5555_5555_5555_5555_5555_5555_5555,
        );

        assert_eq!(
            u128::from(vec.broadcast(14)),
            0x6666_6666_6666_6666_6666_6666_6666_6666,
        );

        assert_eq!(
            u128::from(vec.broadcast(15)),
            0x7777_7777_7777_7777_7777_7777_7777_7777,
        );
    }
}
