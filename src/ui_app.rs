use egui::plot::{Plot, Points, PlotPoints};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::{sin_buf::{SinBuf, PI2}, input_signal::InputSignal};



pub struct UiApp {
    inputSig: InputSignal,
    len: usize,
    inBuff: SinBuf,
    freqFactorStr: String,
}

impl UiApp {
    pub fn new(inputSig: InputSignal, len: usize) -> Self {
        let mut inBuff = SinBuf::new(len, 0.0, 1.0, Some(PI2 / (0.5 * (len as f64))));
        prepareInBuffer(&mut inBuff, len);
        Self {
            inputSig,
            len,
            inBuff,
            freqFactorStr: String::from("2.0"),
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        self.inputSig.next();
        egui::Window::new("new input").show(ctx, |ui| {
            ui.label(format!("new input: '{}'", 0));
            if ui.button("just button").clicked() {
            }
            Plot::new("new input").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        self.inputSig.xyPoints(),
                    )),
                )
            });
        });
        self.inputSig.fft();
        egui::Window::new("new fft").show(ctx, |ui| {
            ui.label(format!("new fft: '{}'", 0));
            if ui.button("just button").clicked() {
            }
            Plot::new("new fft").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        self.inputSig.fftPoints(),
                    )),
                )
            });
        });
    }
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


// egui::Window::new("Main thread").show(ctx, |ui| {
//     ui.label(format!("Current threads: '{}'", 0));
//     let response = ui.add(egui::TextEdit::singleline(&mut self.freqFactorStr));
//     if response.changed() {
//         // …
//     }            
//     if ui.button("Add freq").clicked() {
//         let freqFactor: f64 = self.freqFactorStr.parse().unwrap();
//         self.inBuff.addFreq(freqFactor)
//     }
//     let plot = Plot::new("input");
//     plot.show(ui, |plotUi| {
//         plotUi.points(
//             Points::new(PlotPoints::new(
//                 inPoints(&self.inBuff.content, self.len),
//         )),
//         )
//     });
// });
// egui::Window::new("output").show(ctx, |ui| {
//     ui.label(format!("output: '{}'", 0));
//     if ui.button("just button").clicked() {
//     }
//     let mut planner = FftPlanner::new();
//     let fft = planner.plan_fft_forward(self.len.into());
//     let mut outBuff = self.inBuff.content.clone();
//     fft.process(&mut outBuff);
//     let plot = Plot::new("fft Output");
//     plot.show(ui, |plotUi| {
//         plotUi.points(
//             Points::new(PlotPoints::new(
//                 inPoints(&outBuff, self.len),
//         )),
//         )
//     });
// });