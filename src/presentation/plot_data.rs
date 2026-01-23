use std::{collections::VecDeque, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

use sal_sync::sync::RwLock;

///
/// Tread safe collection of xy
pub struct PlotData {
    length: AtomicUsize,
    xy: RwLock<VecDeque<[f64; 2]>>,
    is_changed: AtomicBool,
}
impl PlotData {
    ///
    pub fn new(length: usize) -> Self {
        Self {
            length: AtomicUsize::new(length), 
            xy: RwLock::new(VecDeque::from(vec![[0.0, 0.0]; length])),
            is_changed: AtomicBool::new(true),
        }
    }
    ///
    /// Returns true if internal data was changed since last xy() call
    pub fn is_changed(&self) -> bool {
        self.is_changed.load(Ordering::Acquire)
    }
    ///
    pub fn push(&self, xy: &[f64; 2]) {
        self.add(xy)
    }
    ///
    pub fn add(&self, xy: &[f64; 2]) {
        self.xy.write().push_back(*xy);
        while self.xy.read().len() > self.length.load(Ordering::Acquire) {
            self.xy.write().remove(0);
        }
        self.is_changed.store(true, Ordering::Release);
    }
    ///
    pub fn update(&self, index: usize, xy: [f64; 2]) {
        self.xy.write()[index] = xy;
    }
    pub fn get(&self, index: usize) -> [f64; 2] {
        if index < self.xy.read().len() {
            self.xy.read()[index]
        } else {
            panic!("'index out of bounds: the len is {} but the index is {}'", self.xy.read().len(), index);
        }
    }
    ///
    pub fn xy(&self) -> Vec<[f64; 2]> {
        self.is_changed.store(false, Ordering::Release);
        self.xy.read().to_owned().into()
    }
    /// 
    pub fn len(&self) -> usize {
        self.xy.read().len()
    }
    ///
    pub fn set_len(&self, length: usize) {
        self.length.store(length, Ordering::Release);
    }
    ///
    pub fn clear(&self) {
        self.xy.write().clear();
        self.is_changed.store(true, Ordering::Release);
    }
}
