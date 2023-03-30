#![allow(non_snake_case)]

use std::error::Error;

use egui::plot::{Plot, Points, PlotPoint, PlotPoints};

// use std::{thread, time::Instant};
use rustfft::{FftPlanner, num_complex::Complex};

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;

fn main() -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App", 
        native_options, 
        Box::new(|cc| Box::new(App::new(cc)))
    )?;
    Ok(())
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




#[derive(Default)]
struct App {}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            let len = 100;
            let mut inBuff = SinBuf::new(len, 0.0, 1.0, None);
            let mut outBuff = vec![Complex {re: 0.0, im: 0.0}; len.into()];
        
            for i in 0..len {
                
                println!("inBuf: {:?}", &inBuff.content);
                
                let mut planner = FftPlanner::new();
                let fft = planner.plan_fft_forward(len.into());
                
                outBuff = inBuff.content.clone();
                fft.process(&mut outBuff);
                inBuff.next();
            }
        
        
            let plot = Plot::new("input buff");
            plot.show(ui, |plotUi| {
                let mut pPoints: Vec<[f64; 2]> = vec![];
                for item in inBuff.content {
                    pPoints.push([item.re, item.im]);
                }
                plotUi.points(
                    Points::new(PlotPoints::new(pPoints)),  
                );                    
            });
            // ui.heading("Hello World!");
        });
    }
}