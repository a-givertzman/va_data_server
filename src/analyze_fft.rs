#![allow(non_snake_case)]

use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
    time,
    thread, 
    sync::{
        Arc, 
        Mutex,
    },
    error::Error, 
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
    }
};


pub const PI: f32 = std::f32::consts::PI;
pub const PI2: f32 = PI * 2.0;

type BuilderCallback = fn(t: f32, f: f32) -> f32;

///
/// 
pub struct AnalizeFft<const N: usize> {
    handle: Option<thread::JoinHandle<()>>,
    cancel: bool,
    pub inputSignal: Arc<Mutex<InputSignal<N>>>,
    pub f: f32,
    pub period: f32,
    len: usize,
    pub t: Vec<f32>,
    pub origin: Vec<f32>,
    pub complex0: Vec<Complex<f32>>,
    pub complex0Current: Vec<[f64; 2]>,
    pub complex: Vec<Complex<f32>>,
    pub complexCurrent: Vec<[f64; 2]>,
    pub fftComplex: Vec<Complex<f32>>,
    pub fftScalar: Vec<f32>,
    pub phi: f32,
    pub xyPoints: Vec<[f64; 2]>,
    fft: Arc<dyn Fft<f32>>,
    PI2f: f32,
    ready: bool,
}
impl<const N: usize> AnalizeFft<N> {
    ///
    pub fn new(inputSignal: Arc<Mutex<InputSignal<N>>>, f: f32, len: usize) -> Self {
        let period = 1.0 / f;
        let delta = period / (len as f32);
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
            period: 1.0 / f,
            len,
            // step: match step {
            //     Some(value) => value,
            //     None => delta,
            // },
            t: Vec::from([0.0]),
            origin: vec![0.0],
            complex0: vec![Complex{re: 0.0, im: 0.0}],
            complex0Current: vec![[0.0, 0.0], [0.0, 0.0]],
            complex: vec![Complex{re: 0.0, im: 0.0}],
            complexCurrent: vec![[0.0, 0.0], [0.0, 0.0]],
            fftComplex: vec![Complex{re: 0.0, im: 0.0}; len],
            fftScalar: vec![0.0; len],
            phi: 0.0,
            xyPoints: vec![[0.0, 0.0]],
            fft,
            PI2f: PI2 * f,
            ready: false,
        }
    }
    ///
    /// Starts in the thread
    pub fn run(this: Arc<Mutex<AnalizeFft<N>>>) -> Result<(), Box<dyn Error>> {
        let cancel = this.lock().unwrap().cancel;
        let me = this.clone();
        let handle = Some(
            thread::Builder::new().name("AnalizeFft tread".to_string()).spawn(move || {
                debug!("[AnalizeFft] started in {:?}", thread::current().name().unwrap());
                while !cancel {
                    // debug!("tread: {:?} cycle started", thread::current().name().unwrap());
                    this.lock().unwrap().next();
                    if this.lock().unwrap().ready {
                        this.lock().unwrap().fftProcess();
                        thread::sleep(time::Duration::from_micros(1000));
                    } else {
                        thread::sleep(time::Duration::from_micros(1000));
                    }
                    // thread::sleep(time::Duration::from_nanos(100000));
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
    pub fn next(&mut self) {
        let current = self.inputSignal.lock().unwrap().read();
        self.phi = current[0];
        let t = current[1];
        let input = current[2];
        
        self.t.push(t);
        self.origin.push(input);
        self.xyPoints.push([t as f64, input as f64]);

        // let PI2ft = self.PI2f * t;
        let re0 = (self.phi).cos();
        let im0 = (self.phi).sin();
        self.complex0Current = vec![[0.0, 0.0], [re0 as f64, im0 as f64]];
        self.complex0.push(Complex{ re: re0, im: im0 });

        let re = input * (self.phi).cos();
        let im = input * (self.phi).sin();
        self.complexCurrent = vec![[0.0, 0.0], [re as f64, im as f64]];
        self.complex.push(Complex{ re, im });
        if self.t.len() > self.len {
            self.t.remove(0);
            self.origin.remove(0);
            self.complex0.remove(0);
            self.complex.remove(0);
            self.xyPoints.remove(0);
            self.ready = true;
        }
    }
    ///
    pub fn fftProcess(&mut self) {
        for i in 0..self.complex.len() {
            self.fftComplex[i] = self.complex[i].clone();
        }
        self.fft.process(&mut self.fftComplex);
        // self.fft.process_with_scratch(&mut self.fftComplex);
    }    
    ///
    pub fn complex0Points(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        for i in 0..self.t.len() {
            let x = self.complex0[i].re as f64;
            let y = self.complex0[i].im as f64;
            points.push([x, y]);
        }
        points
    }    
    ///
    pub fn complexPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        for i in 0..self.t.len() {
            let x = self.complex[i].re as f64;
            let y = self.complex[i].im as f64;
            points.push([x, y]);
        }
        points
    }    
    ///
    pub fn fftPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        let factor = 1.0 / ((self.len / 2) as f32);
        for i in 0..self.t.len() / 2 {
            let x = (i as f32) as f64;
            let y = (self.fftComplex[i].abs() * factor) as f64;
            points.push([x, 0.0]);
            points.push([x, y]);
        }
        points
    }    
}






// pub fn next(&mut self) {
//     match self.t.last() {
//         Some(tOld) => {
//             let t = tOld + self.step;
//             self.t.push(t);

//             let PI2f = PI2 * self.f;
//             self.phi += PI2f * self.step;
//             if self.phi > PI2f * self.period {
//                 self.phi = 0.0;
//             }
            
//             let input = (self.builder)(t, self.f);
//             self.origin.push(input);
//             self.xyPoints.push([t as f64, input as f64]);
    
//             let PI2ft = PI2f * t;
//             let re0 = (PI2ft).cos();
//             let im0 = (PI2ft).sin();
//             self.complex0Current = vec![[0.0, 0.0], [re0 as f64, im0 as f64]];
//             self.complex0.push(Complex{ re: re0, im: im0 });
    
//             let re = input * (PI2ft).cos();
//             let im = input * (PI2ft).sin();
//             self.complexCurrent = vec![[0.0, 0.0], [re as f64, im as f64]];
//             self.complex.push(Complex{ re, im });
//             if self.t.len() > self.len {
//                 self.t.remove(0);
//                 self.xyPoints.remove(0);
//                 self.origin.remove(0);
//                 self.complex.remove(0);
//             }
//             // debug!("complex: {:?}", complex);
//         },
//         None => {},
//     };
// }