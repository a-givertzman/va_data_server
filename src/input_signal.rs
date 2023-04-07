#![allow(non_snake_case)]

use log::{
    // info,
    // trace,
    debug,
    warn,
};
use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    thread,
    error::Error, 
};
use crate::{
    interval::Interval, 
    circular_queue::CircularQueue
};

pub const PI: f32 = std::f32::consts::PI;
pub const PI2: f32 = PI * 2.0;

// type BuilderCallback = fn(t: f32, f: f32) -> f32;

///
/// Emulates analog signal which form can be conigerwd in the `builder` callback
pub struct InputSignal {
    handle: Option<thread::JoinHandle<()>>,
    cancel: bool,
    pub f: f32,
    pub period: f32,
    builder: fn(f32) -> f32,
    len: usize,
    step: f32,
    pub t: f32,
    pub i: usize,
    /// current base phase angle in radians
    pub phi: f32,
    /// current amplitude of analog value
    pub amplitude: f32,
    pub points: CircularQueue<f32>,
    pub xyPoints: Vec<[f64; 2]>,
}
impl InputSignal {
    ///
    /// Creates new instance
    pub fn new(f: f32, builder: fn(f32) -> f32, len: usize, step: Option<f32>) -> Self {
        let period = 1.0 / f;
        let delta = period / (len as f32);
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
            phi: 0.0,
            amplitude: 0.0,
            points: CircularQueue::with_capacity(len),
            xyPoints: vec![[0.0, 0.0]],
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
        self.i += 1;
        if self.i > self.len {
            self.i = 0;
        }
        self.t = self.t + self.step;
        self.phi = PI2 * ((self.i / self.len) as f32);

        // let PI2f = PI2 * self.f;
        
        // self.inputFilter.add((self.builder)(t, self.f));
        // let input = self.inputFilter.value();
        self.amplitude = (self.builder)(self.t);
        self.points.push(self.amplitude);
        self.xyPoints.push([self.t as f64, self.amplitude as f64]);
        if self.xyPoints.len() > self.len {
            self.xyPoints.remove(0);
        }
    }
    ///
    /// current value [time, amplitude]
    pub fn read(&self) -> [f32; 3] {
        [self.phi, self.t, self.amplitude]
    }
}
