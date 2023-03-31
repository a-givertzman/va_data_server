use egui::plot::{Plot, Points, PlotPoints};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::sin_buf::{SinBuf, PI2};



pub struct UiApp {
    len: usize,
    inBuff: SinBuf,
    freqFactorStr: String,
}

impl UiApp {
    pub fn new(len: usize) -> Self {
        let mut inBuff = SinBuf::new(len, 0.0, 1.0, Some(PI2 / (0.5 * (len as f64))));
        prepareInBuffer(&mut inBuff, len);
        Self {
            len,
            inBuff,
            freqFactorStr: String::from("2.0"),
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Main thread").show(ctx, |ui| {
            ui.label(format!("Current threads: '{}'", 0));
            let response = ui.add(egui::TextEdit::singleline(&mut self.freqFactorStr));
            if response.changed() {
                // …
            }            
            if ui.button("Add freq").clicked() {
                let freqFactor: f64 = self.freqFactorStr.parse().unwrap();
                self.inBuff.addFreq(freqFactor)
            }
            let plot = Plot::new("input");
            plot.show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        inPoints(&self.inBuff.content, self.len),
                )),
                )
            });
        });
        egui::Window::new("output").show(ctx, |ui| {
            ui.label(format!("output: '{}'", 0));
            if ui.button("just button").clicked() {
            }
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(self.len.into());
            let mut outBuff = self.inBuff.content.clone();
            fft.process(&mut outBuff);
            let plotOut = Plot::new("fft Output");
            plotOut.show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        inPoints(&outBuff, self.len),
                )),
                )
            });
        });
    }
}

fn inPoints(content: &Vec<Complex<f64>>, len: usize) -> Vec<[f64; 2]> {
    let mut points: Vec<[f64; 2]> = vec![];
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;    
    for i in 0..len {
        
        x = (i as f64) * 0.01;
        let item = content[i];
        // y = item.abs();
        let yVer = item.re;
        // println!("{:?}: re={:?} im={:?}", i, item.re, item.im);
        // assert!((y - yVer) < 1e-10);
        // println!("{:?}: {:?}\tx={:?} y={:?}", i, item, x, yVer);
        points.push([x, yVer]);
    }    
    points
}

fn prepareInBuffer(inBuff: &mut SinBuf, len: usize) {
    for i in 0..len {
        {
            // println!("inBuf: {:?}", &inBuff.content);
            
            // let mut planner = FftPlanner::new();
            // let fft = planner.plan_fft_forward(len.into());
            
            // outBuff = inBuff.content.clone();
            // fft.process(&mut outBuff);
        }  
        inBuff.next();
    }
}