use super::facade::{DataReader, DataWriter, Value};
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub trait Mirrorable {
    type Output;
    fn len(&self) -> usize;
    unsafe fn get_unchecked(&self, index: usize) -> &Self::Output;
}

pub trait MirrorableMut: Mirrorable {
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Output;
}

pub trait Resizable {
    fn from_len(len: usize) -> Self;
    fn resize(self, new_len: usize) -> Self;
}

pub struct Mirror<T: Mirrorable> {
    inner: T,
    mask: usize,
}

pub type MirrorVec<T> = Mirror<Vec<T>>;

fn mask_for(len: usize) -> usize {
    if len.is_power_of_two() {
        len - 1
    } else if len == 0 {
        0
    } else {
        panic!("Mirrored size must be a power of two");
    }
}

impl<T: Mirrorable + Resizable> Mirror<T> {
    pub fn new(len: usize) -> Self {
        Self {
            inner: T::from_len(len),
            mask: mask_for(len),
        }
    }

    pub fn resize(source: T) -> Self {
        let new_len = source.len().next_power_of_two();

        Self {
            inner: source.resize(new_len),
            mask: mask_for(new_len),
        }
    }
}

impl<T: Mirrorable> From<T> for Mirror<T> {
    fn from(source: T) -> Self {
        let len = source.len();

        Self {
            inner: source,
            mask: mask_for(len),
        }
    }
}

impl<T: Mirrorable> Index<usize> for Mirror<T> {
    type Output = T::Output;

    fn index(&self, index: usize) -> &T::Output {
        unsafe { self.inner.get_unchecked(index & self.mask) }
    }
}

impl<T: MirrorableMut> IndexMut<usize> for Mirror<T> {
    fn index_mut(&mut self, index: usize) -> &mut T::Output {
        unsafe { self.inner.get_unchecked_mut(index & self.mask) }
    }
}

impl<T: Mirrorable> DataReader for Mirror<T>
where
    T::Output: Value,
{
    type Address = usize;
    type Value = T::Output;

    fn read(&self, address: usize) -> Self::Value {
        self[address]
    }
}

impl<T: MirrorableMut> DataWriter for Mirror<T>
where
    T::Output: Value,
{
    fn write(&mut self, address: usize, value: Self::Value) {
        self[address] = value;
    }
}

impl<T: Clone, U: Deref<Target = [T]>> Mirrorable for U {
    type Output = T;

    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &Self::Output {
        <[T]>::get_unchecked(self, index)
    }
}

impl<T: Clone, U: DerefMut<Target = [T]>> MirrorableMut for U {
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Output {
        <[T]>::get_unchecked_mut(self, index)
    }
}

impl<T: Clone + Default> Resizable for Vec<T> {
    fn from_len(len: usize) -> Self {
        vec![T::default(); len]
    }

    fn resize(mut self, new_len: usize) -> Self {
        Vec::<T>::resize(&mut self, new_len, T::default());
        self
    }
}
