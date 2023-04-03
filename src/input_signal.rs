use rustfft::{
    num_complex::{
        Complex, 
        ComplexFloat
    }, 
    FftPlanner
};

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;


type Callback = fn(t: f64) -> f64;

pub struct InputSignal {
    f: f64,
    builder: Callback,
    len: usize,
    step: f64,
    pub t: Vec<f64>,
    pub origin: Vec<f64>,
    pub complex: Vec<Complex<f64>>,
    pub fftComplex: Vec<Complex<f64>>,
    pub fftScalar: Vec<f64>,
}
impl InputSignal {
    ///
    pub fn new(f: f64, builder: Callback, len: usize, step: Option<f64>) -> Self {
        Self { 
            f: f,
            builder,
            len: len,
            step: match step {
                Some(value) => value,
                None => PI2 / (len as f64),
            },
            t: vec![0.0],
            origin: vec![0.0],
            complex: vec![Complex{re: 0.0, im: 0.0}],
            fftComplex: vec![Complex{re: 0.0, im: 0.0}; len],
            fftScalar: vec![0.0; len],
        }
    }
    ///
    pub fn next(&mut self) {
        let t = match self.t.last() {
            Some(value) => value,
            None => &0.0,
        } + self.step;
        self.t.push(t);

        let input = (self.builder)(t);
        self.origin.push(input);

        let re = input * (PI2 * self.f * t).cos();
        let im = input * (PI2 * self.f * t).sin();
        let complex = Complex{ re, im };
        self.complex.push(complex);
        if self.t.len() > self.len {
            self.t.remove(0);
            self.origin.remove(0);
            self.complex.remove(0);
        }
        // println!("complex: {:?}", complex);
    }
    pub fn fft(&mut self) {
        for i in 0..self.complex.len() {
            self.fftComplex[i] = self.complex[i].clone();
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.len.into());
        fft.process(&mut self.fftComplex);
    }
    ///
    pub fn fftPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        for i in 0..self.t.len() {
            let y = self.fftComplex[i].abs();
            points.push([self.t[i] - self.t[0], y]);
        }
        points
    }    
    ///
    pub fn xyPoints(&self) -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = vec![];
        for i in 0..self.t.len() {
            points.push([self.t[i], self.origin[i]]);
        }
        points
    }    
}
