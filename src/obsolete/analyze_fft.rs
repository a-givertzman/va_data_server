#![allow(non_snake_case)]

use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
    thread, 
    sync::{
        Arc, 
        Mutex,
    },
    error::Error,
    // time::Duration, 
};
// use egui::mutex::Mutex;
use rustfft::{
    num_complex::{
        Complex, 
        ComplexFloat
    }, 
    FftPlanner, 
    Fft
};

use crate::{
    input_signal::{
        InputSignal
    }, 
    interval::Interval, 
    circular_queue::CircularQueue
};


///
/// 
pub struct AnalizeFft {
    handle: Option<thread::JoinHandle<()>>,
    cancel: bool,
    pub inputSignal: Arc<Mutex<InputSignal>>,
    pub f: f32,
    pub period: f64,
    len: usize,
    pub t: f64,
    pub tList: CircularQueue<f64>,
    pub complex0Current: Vec<[f64; 2]>,
    pub complex: CircularQueue<Complex<f64>>,
    pub complexCurrent: Vec<[f64; 2]>,
    pub fftComplex: Vec<Complex<f64>>,
    pub phi: f64,
    pub xyPoints: CircularQueue<[f64; 2]>,
    fft: Arc<dyn Fft<f64>>,
}
impl AnalizeFft {
    ///
    pub fn new(inputSignal: Arc<Mutex<InputSignal>>, f: f32, len: usize) -> Self {
        let period = 1.0 / (f as f64);
        let delta = period / (len as f64);
        debug!("[AnalizeFft] f: {:?} Hz", f);
        debug!("[AnalizeFft] T: {:?} sec", period);
        debug!("[AnalizeFft] N: {:?} poins", len);
        debug!("[AnalizeFft] delta: {:?} sec", delta);
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);
        Self { 
            handle: None,
            cancel: false,
            inputSignal,
            f,
            period: period,
            len,
            t: 0.0,
            tList: CircularQueue::with_capacity_fill(len, &mut vec![0.0; len]),
            // complex0: CircularQueue::with_capacity_fill(len, &mut vec![Complex{re: 0.0, im: 0.0}; len]),    //vec![Complex{re: 0.0, im: 0.0}],
            complex0Current: vec![[0.0, 0.0], [0.0, 0.0]],
            complex: CircularQueue::with_capacity_fill(len, &mut vec![Complex{re: 0.0, im: 0.0}; len]),     //vec![Complex{re: 0.0, im: 0.0}],
            complexCurrent: vec![[0.0, 0.0], [0.0, 0.0]],
            fftComplex: vec![Complex{re: 0.0, im: 0.0}; len],
            phi: 0.0,
            xyPoints: CircularQueue::with_capacity_fill(len, &mut vec![[0.0, 0.0]; len]),    //vec![[0.0, 0.0]],
            fft,
        }
    }
    ///
    /// Starts in the thread
    pub fn run(this: Arc<Mutex<AnalizeFft>>) -> Result<(), Box<dyn Error>> {
        let cancel = this.lock().unwrap().cancel;
        let period = this.lock().unwrap().period as f64;
        debug!("period: {:?}ms", period * 1000.0);
        let delay = (period) * ((this.lock().unwrap().len / 2) as f64);
        debug!("interval: {:?}ms", delay * 1000.0);
        let mut interval = Interval::new(delay);
            // const delay: Duration = Duration::from_nanos(10);
        let me = this.clone();
        let handle = Some(
            thread::Builder::new().name("AnalizeFft tread".to_string()).spawn(move || {
                debug!("[AnalizeFft] started in {:?}", thread::current().name().unwrap());
                while !cancel {
                    // debug!("tread: {:?} cycle started", thread::current().name().unwrap());
                    // let start = std::time::Instant::now();
                    // this.lock().unwrap().next();
                    this.lock().unwrap().fftProcess();
                    // std::thread::sleep(delay);
                    // debug!("elapsed: {:?}", start.elapsed());
                    interval.wait();
                    // debug!("elapsed: {:?}", start.elapsed());
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
    /// 
    fn fftProcess(&mut self) {
        self.inputSignal.lock().unwrap().complex.buffer().clone_into(&mut self.fftComplex);
        self.fft.process(&mut self.fftComplex);
        // self.fft.process_with_scratch(&mut self.fftComplex);
    }    
    ///
    ///
    pub fn fftPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        let factor = 1.0 / ((self.len / 2) as f64);
        for i in 0..self.tList.len() / 2 {
            let x = i as f64;
            let y = (self.fftComplex[i].abs() * factor) as f64;
            points.push([x, 0.0]);
            points.push([x, y]);
        }
        points
    }    
}
