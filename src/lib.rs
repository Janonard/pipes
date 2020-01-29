use std::ops::*;

pub trait Capacitive<I>: Index<I> + IndexMut<I> {
    fn capacity(&self) -> I;
}

impl<T> Capacitive<usize> for [T] {
    fn capacity(&self) -> usize {
        self.len()
    }
}

pub struct CapacityIter<'a, T> where T: Capacitive<usize> {
    data: &'a T,
    index: usize,
}

impl<'a, T> Iterator for CapacityIter<'a, T> where T: Capacitive<usize> + Index<usize> {
    type Item = &'a T::Output;

    fn next(&mut self) -> Option<&'a T::Output> {
        if self.index < self.data.capacity() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
