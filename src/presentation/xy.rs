use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use sal_core::dbg::Dbg;

use crate::{fft::AtomicFloat, presentation::PlotData};

pub struct Xy {
    pub sampl_freq: f64,
    pub delta: f64,
    pub sampling_period: f64,
    pub t: Arc<AtomicFloat<f64>>,
    xy: PlotData,
    dbg: Dbg,
}
impl Xy {
    ///
    /// - `len` - The length of the input signal display window
    pub fn new(len: usize, sampl_freq: impl Into<f64>, fft_buflen: usize) -> Self {
        let sampl_freq = sampl_freq.into();
        let sampling_period = 1.0 / sampl_freq;
        let delta = sampling_period / (fft_buflen as f64);
        Self {
            sampl_freq,
            delta,
            sampling_period,
            t: Arc::new(AtomicFloat::new(0.0)),
            xy: PlotData::new(len),
            dbg: Dbg::new("", "Xy"),
        }
    }
    ///
    /// 
    pub fn len(&self) -> usize {
        self.xy.len()
    }
    ///
    /// 
    pub fn set_len(&self, len: usize) {
        self.xy.set_len(len);
    }

    ///
    pub fn enqueue(&self, values: &[u16]) {
        // let mut xy = self.xy.write();
        for (i, val) in values.iter().enumerate() {
            // log::debug!("{} value: {:?}", self.dbg, value);
            // let i = complex.len();
            let t = self.t.load();
            self.xy.add(&[t * 1.0e6, *val as f64]);
            self.t.store(t + self.sampling_period);
        }
        log::trace!("{}.enqueue | Done", self.dbg);
    }
    ///
    /// Returns a collection of the stored XY values
    pub fn values(&self) -> Vec<[f64; 2]> {
        self.xy.xy()
    }
}