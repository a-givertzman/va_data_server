#![allow(non_snake_case)]
// use std::{thread, time::Instant};
use rustfft::{FftPlanner, num_complex::Complex};

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;

fn main() {
    
    let mut inBuf = SinBuf::new(10, 0.0, 1.0, None);
    for i in 0..10 {
        
        println!("inBuf: {:?}", &inBuf.content);
        
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(10);
        
        // let mut buffer = vec![Complex{ re: 0.0f32, im: 0.0f32 }; 1234];
        fft.process(&mut inBuf.content);
        inBuf.next();
    }
}


struct SinBuf {
    // len: u16,
    phi: f64,
    r: f64,
    step: f64,
    content: Vec<Complex<f64>>
}
impl SinBuf {
    ///
    pub fn new(len: u16, phi: f64, r: f64, step: Option<f64>) -> Self {
        Self { 
            // len: len,
            phi: phi, 
            r: r,
            step: match step {
                Some(value) => value,
                None => PI2 / (len as f64),
            },
            // content: vec![0.0; len as usize],
            content: vec![Complex{re: 0.0, im: 0.0}; len as usize],
        }
    }
    ///
    pub fn next(&mut self) {
        let re = self.r * self.phi.cos();
        let im = self.r * self.phi.sin();
        let complex = Complex{ re, im };
        // println!("complex: {:?}", complex);
        self.content.insert(0, complex);
        // self.content.insert(0, self.phi);
        self.content.pop();
        self.addPhi(self.step);
    }
    ///
    fn addPhi(&mut self, step: f64) {
        self.phi += step;
        if self.phi > PI2 {
            self.phi = 0.0;
        }
    }
}
