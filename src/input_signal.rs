#![allow(non_snake_case)]

use rustfft::{
    num_complex::{
        Complex, 
        ComplexFloat
    }, 
    FftPlanner
};

pub const PI: f32 = std::f32::consts::PI;
pub const PI2: f32 = PI * 2.0;


type BuilderCallback = fn(t: f32, f: f32) -> f32;

pub struct InputSignal {
    pub f: f32,
    pub period: f32,
    builder: BuilderCallback,
    len: usize,
    step: f32,
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
    inputFilter: LowPassFilter,
}
impl InputSignal {
    ///
    pub fn new(f: f32, builder: BuilderCallback, len: usize, step: Option<f32>) -> Self {
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
                None => delta,
            },
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
            inputFilter: LowPassFilter::new(2),
        }
    }
    ///
    pub fn next(&mut self) {
        match self.t.last() {
            Some(tOld) => {
                let t = tOld + self.step;
                self.t.push(t);

                let PI2f = PI2 * self.f;
                self.phi += PI2f * self.step;
                if self.phi > PI2f * self.period {
                    self.phi = 0.0;
                }
                
                self.inputFilter.add((self.builder)(t, self.f));
                let input = self.inputFilter.value();
                self.origin.push(input);
                self.xyPoints.push([t as f64, input as f64]);
        
                let PI2ft = PI2f * t;
                let re0 = (PI2ft).cos();
                let im0 = (PI2ft).sin();
                self.complex0Current = vec![[0.0, 0.0], [re0 as f64, im0 as f64]];
                self.complex0.push(Complex{ re: re0, im: im0 });
        
                let re = input * (PI2ft).cos();
                let im = input * (PI2ft).sin();
                self.complexCurrent = vec![[0.0, 0.0], [re as f64, im as f64]];
                self.complex.push(Complex{ re, im });
                if self.t.len() > self.len {
                    self.t.remove(0);
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
            let x = i as f64;
            let y = (self.fftComplex[i].abs() * factor) as f64;
            points.push([x, 0.0]);
            points.push([x, y]);
        }
        points
    }    
}




struct LowPassFilter {
    len: usize,
    values: Vec<f32>,
}

impl LowPassFilter {
    ///
    pub fn new(len: usize) -> Self {
        Self {
            len,
            values: vec![0.0; len],
        }
    }
    ///
    pub fn add(&mut self, value: f32) {
        self.values.push(value);
        if self.values.len() > self.len {
            self.values.remove(0);
        }
    }
    ///
    pub fn value(&self) -> f32 {
        self.values.iter().sum::<f32>() / (self.len as f32)
    }
}