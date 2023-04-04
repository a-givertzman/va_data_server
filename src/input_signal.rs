use std::collections::VecDeque;

use rustfft::{
    num_complex::{
        Complex, 
        ComplexFloat
    }, 
    FftPlanner
};

pub const PI: f32 = std::f32::consts::PI;
pub const PI2: f32 = PI * 2.0;


type Callback = fn(t: f32, f: f32) -> f32;

pub struct InputSignal {
    f: f32,
    period: f32,
    builder: Callback,
    len: usize,
    step: f32,
    pub t: VecDeque<f32>,
    pub origin: Vec<f32>,
    pub complex0: Vec<Complex<f32>>,
    pub complex0Current: Vec<[f64; 2]>,
    pub complex: Vec<Complex<f32>>,
    pub fftComplex: Vec<Complex<f32>>,
    pub fftScalar: Vec<f32>,
    pub phi: f32,
    pub xyPoints: Vec<[f64; 2]>,
}
impl InputSignal {
    ///
    pub fn new(f: f32, builder: Callback, len: usize, step: Option<f32>) -> Self {
        let period = 1.0 / f;
        let delta = period / (len as f32);
        println!("f: {:?} Hz", f);
        println!("T: {:?} sec", period);
        println!("N: {:?} poins", len);
        println!("delta: {:?} sec", delta);
        Self { 
            f: f,
            period: 1.0 / f,
            builder,
            len: len,
            step: match step {
                Some(value) => value,
                None => delta, // PI2 / (len as f32),
            },
            t: VecDeque::from([0.0]),
            origin: vec![0.0],
            complex0: vec![Complex{re: 0.0, im: 0.0}],
            complex0Current: vec![[0.0, 0.0], [0.0, 0.0]],
            complex: vec![Complex{re: 0.0, im: 0.0}],
            fftComplex: vec![Complex{re: 0.0, im: 0.0}; len],
            fftScalar: vec![0.0; len],
            phi: 0.0,
            xyPoints: vec![[0.0, 0.0]; len],
        }
    }
    ///
    pub fn next(&mut self) {
        match self.t.back() {
            Some(tOld) => {
                let t = tOld + self.step;

                self.t.push_back(t);
        
                let input = (self.builder)(t, self.f);
                self.origin.push(input);
                self.xyPoints.push([t as f64, input as f64]);
        
                self.phi = PI2 * self.f * t;
                let re0 = (PI2 * self.f * t).cos();
                let im0 = (PI2 * self.f * t).sin();
                self.complex0Current = vec![[0.0, 0.0], [re0 as f64, im0 as f64]];
                self.complex0.push(Complex{ re: re0, im: im0 });
        
                let re = input * (PI2 * self.f * t).cos();
                let im = input * (PI2 * self.f * t).sin();
                let complex = Complex{ re, im };
                self.complex.push(complex);
                if self.t.len() > self.len {
                    self.t.pop_front();
                    self.xyPoints.remove(0);
                    self.origin.remove(0);
                    self.complex.remove(0);
                }
                // println!("complex: {:?}", complex);
            },
            None => {},
        };
    }
    ///
    pub fn fftProcess(&mut self) {
        for i in 0..self.complex.len() {
            self.fftComplex[i] = self.complex[i].clone();
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.len.into());
        fft.process(&mut self.fftComplex);
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
    pub fn fftPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        for i in 0..self.t.len() / 2 {
            let x = i as f64;
            let y = (self.fftComplex[i].abs() / 2048.0) as f64;
            points.push([x, 0.0]);
            points.push([x, y]);
        }
        points
    }    
}
