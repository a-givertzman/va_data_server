use rustfft::num_complex::Complex;

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;


type Callback = fn();

pub struct InputSignal {
    builder: Callback,
    len: usize,
    phi: f64,
    r: f64,
    step: f64,
    pub content: Vec<Complex<f64>>
}
impl InputSignal {
    ///
    pub fn new(len: usize, phi: f64, r: f64, step: Option<f64>) -> Self {
        Self { 
            len: len,
            phi: phi, 
            r: r,
            step: match step {
                Some(value) => value,
                None => PI2 / (len as f64),
            },
            content: vec![Complex{re: 0.0, im: 0.0}; len],
        }
    }
    pub fn addFreq(&mut self, freqFactor: f64) {
        for i in 0..self.len {
            self.addPhi(self.step);
            self.content[i].re += self.r * (self.phi * freqFactor).cos();
        }        
    }
    ///
    pub fn next(&mut self) {
        let re = self.r * self.phi.cos() + self.r * 0.2 * (self.phi * 10.0).cos();
        let im = 0.0;//self.r * self.phi.sin();
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
