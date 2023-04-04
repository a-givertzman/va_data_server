use rustfft::num_complex::Complex;

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;

pub struct SinBuf {
    len: usize,
    phi: f64,
    r: f64,
    step: f64,
    pub content: Vec<Complex<f64>>
}
impl SinBuf {
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





// fn t(ctx: &egui::Context, frame: &mut eframe::Frame) {
//     egui::CentralPanel::default().show(ctx, |ui| {

//         let len: usize = 10000;
//         let mut inBuff = SinBuf::new(len, 0.0, 1.0, Some(PI2 / (0.1 * (len as f64))));
//         let mut outBuff = vec![Complex {re: 0.0, im: 0.0}; len.into()];
    
//         let plot = Plot::new("input buff");
//         let mut pPoints: Vec<[f64; 2]> = vec![];
        
//         let mut x: f64 = 0.0;
//         let mut y: f64 = 0.0;
        
//         for i in 0..len {
            
//             // println!("inBuf: {:?}", &inBuff.content);
            
//             // let mut planner = FftPlanner::new();
//             // let fft = planner.plan_fft_forward(len.into());
            
//             // outBuff = inBuff.content.clone();
//             // fft.process(&mut outBuff);
//             inBuff.next();
//         }
    

//         for i in 0..len {
            
//             x = (i as f64) * 0.01;
//             let item = inBuff.content[i];
//             // y = item.abs();
//             let yVer = item.re;
//             // println!("{:?}: re={:?} im={:?}", i, item.re, item.im);
//             // assert!((y - yVer) < 1e-10);
//             // println!("{:?}: {:?}\tx={:?} y={:?}", i, item, x, yVer);
//             pPoints.push([x, yVer]);
//         }

//         plot.show(ui, |plotUi| {
//             plotUi.points(
//                 Points::new(PlotPoints::new(pPoints)),  
//             );                    
//         });
//         // ui.heading("Hello World!");
//     });

// }