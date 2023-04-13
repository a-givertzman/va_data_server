#![allow(non_snake_case)]

use heapless::spsc::{Producer, Consumer};
use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    thread,
    error::Error, time::{Instant, UNIX_EPOCH}, 
};
use rustfft::num_complex::Complex;
use crate::{
    interval::Interval, 
    circular_queue::CircularQueue,
    dsp_filters::{
        // average_filter::AverageFilter,
        slp_filter::SlpFilter,
    }
};

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;

const QSIZE: usize = 16_384;
// type BuilderCallback = fn(t: f32, f: f32) -> f32;

///
/// Emulates analog signal which form can be conigerwd in the `builder` callback
pub struct InputSignal {
    handle: Option<thread::JoinHandle<()>>,
    cancel: bool,
    pub f: f32,
    pub period: f64,
    builder: fn(f64) -> f64,
    len: usize,
    step: f64,
    pub t: f64,
    pub i: usize,
    pub phiList: Vec<f64>,
    pub phi: f64,
    pub amplitude: f64,
    pub complex0: Vec<Complex<f64>>,
    pub complex: CircularQueue<Complex<f64>>,
    pub xyPoints: CircularQueue<[f64; 2]>,
    // pub test: CircularQueue<[f64; 16_384]>,
    inputFilter: SlpFilter<f64>,
    pub queueTx: Arc<Mutex<Producer<'static, [f64; 2], QSIZE>>>,
    pub queueRx: Arc<Mutex<Consumer<'static, [f64; 2], QSIZE>>>,
}
impl InputSignal {
    ///
    /// Creates new instance
    pub fn new(f: f32, builder: fn(f64) -> f64, len: usize, step: Option<f64>) -> Self {
        static mut RB: heapless::spsc::Queue<[f64; 2], QSIZE> = heapless::spsc::Queue::<[f64; 2], QSIZE>::new();
        let queue = unsafe { &mut RB };
        let (tx, rx) = queue.split();
    
        let period = 1.0 / (f as f64);
        let delta = period / (len as f64);
        let iToNList: Vec<f64> = (0..len).into_iter().map(|i| {(i as f64) / (len as f64)}).collect();
        let phiList: Vec<f64> = iToNList.clone().into_iter().map(|iToN| {PI2 * iToN}).collect();
        let complex0: Vec<Complex<f64>> = (0..len).into_iter().map(|i| {
            Complex {
                re: phiList[i].cos(), 
                im: phiList[i].sin()
            }
        }).collect();
        debug!("[InputSignal] f: {:?} Hz", f);
        debug!("[InputSignal] T: {:?} sec", period);
        debug!("[InputSignal] N: {:?} poins", len);
        debug!("[InputSignal] delta t: {:?} sec", delta);
        Self { 
            handle: None,
            cancel: false,
            f,
            period,
            builder,
            len,
            step: match step {
                Some(value) => value,
                None => delta,
            },
            t: 0.0,
            i: 0,
            phiList,
            phi: 0.0,
            amplitude: 0.0,
            complex0,
            complex: CircularQueue::with_capacity_fill(len, &mut vec![Complex{re: 0.0, im: 0.0}; len]),
            // test: CircularQueue::with_capacity(16_384),
            xyPoints: CircularQueue::with_capacity_fill(len, &mut vec![[0.0, 0.0]; len]),
            inputFilter: SlpFilter::new(4),
            queueTx: Arc::new(Mutex::new(tx)),
            queueRx: Arc::new(Mutex::new(rx)),
        }
    }
    ///
    /// Starts in the thread
    pub fn run(this: Arc<Mutex<Self>>) -> Result<(), Box<dyn Error>> {
        let cancel = this.lock().unwrap().cancel;
        let me = this.clone();
        let mut interval = Interval::new(this.lock().unwrap().period as f64);
        let handle = Some(
            thread::Builder::new().name("InputSignal tread".to_string()).spawn(move || {
                debug!("[InputSignal] started in {:?}", thread::current().name().unwrap());
                while !cancel {
                    // debug!("tread: {:?} cycle started", thread::current().name().unwrap());
                    this.lock().unwrap().next();
                    interval.wait();
                    // thread::sleep(time::Duration::from_micros(1000));
                }
            })?
        );
        me.lock().unwrap().handle = handle;
        Ok(())
    }
    ///
    /// Stops thread
    pub fn cancel(&mut self) {
        self.cancel = true;
    }
    /// 
    /// Calculates all new values with new time
    fn next(&mut self) {
        self.t = self.t + self.step;
        self.phi = self.phiList[self.i];
        // debug!("i: {:?}   phi: {:?}", self.i, self.phi);
        
        self.inputFilter.add(
            (self.builder)(self.phi),
        );
        self.amplitude = self.inputFilter.value();
        // self.amplitude = (self.builder)(self.phi);

        // self.points.push(self.amplitude);
        self.complex.push(
            Complex {
                re: self.amplitude * self.complex0[self.i].re, 
                im: self.amplitude * self.complex0[self.i].im, 
            },
        );
        self.xyPoints.push([self.t as f64, self.amplitude as f64]);
        let t = Instant::now().duration_since(UNIX_EPOCH).as_secs_f64();
        self.queueTx.lock().unwrap().enqueue([self.amplitude as f64, t]);
        self.increment();
    }
    ///
    /// add current self.i up to self.len, then return it to 0
    fn increment(&mut self) {
        self.i = (self.i + 1) % self.len;
    }
    ///
    /// current value [time, amplitude]
    pub fn read(&self) -> [f64; 3] {
        [self.phi, self.t, self.amplitude]
    }  
}
