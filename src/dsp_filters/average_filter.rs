#![allow(non_snake_case)]

use std::convert::From;

use crate::circular_queue::CircularQueue;


///
/// 
pub struct AverageFilter<T> 
where 
    T: Clone + From<u32> + From<f64>,
    T: std::ops::Add<Output = T>,
    T: std::iter::Sum + std::ops::Div<Output = T>,
{
    len: usize,
    values: CircularQueue<T>,
}

impl<T> AverageFilter<T> 
where 
    T: Clone + From<u32> + From<f64>,
    T: std::ops::Add<Output = T>,
    T: std::iter::Sum + std::ops::Div<Output = T>,
{
    ///
    pub fn new(len: usize) -> Self {
        Self {
            len,
            values: CircularQueue::with_capacity_fill(len, &mut vec![T::from(0.0); len]),
        }
    }
    ///
    pub fn add(&mut self, value: T) {
        self.values.push(value);
    }
    ///
    pub fn value(&self) -> T {
        let iter = self.values.buffer().to_owned().into_iter();
        let value = iter.sum::<T>();
        let k = T::from(self.len as u32);
        value / k
    }
}
