#![allow(non_snake_case)]

use std::sync::{
    Arc, 
    // Mutex
};

use egui::{
    plot::{
        Plot, 
        Points, 
        PlotPoints, Line
    }, 
    Color32, 
    mutex::Mutex,
};

use crate::{
    input_signal::{InputSignal, PI}
};



pub struct UiApp {
    pub inputSig: Arc<Mutex<InputSignal>>,
}

impl UiApp {
    pub fn new(inputSig: InputSignal) -> Self {
        Self {
            inputSig: Arc::new(Mutex::new(inputSig)) ,
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let inputSig = self.inputSig.lock();

        egui::Window::new("complex 0").show(ctx, |ui| {
            // ui.label(format!("complex 0: '{}'", 0));
            ui.label(format!(" f: {:?} Hz   T: {:?} sec", inputSig.f, inputSig.period));
            ui.label(format!(" pfi: {:?}", inputSig.phi * 180.0 / PI));
            ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("complex 0")
            .show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        inputSig.complex0Points(),
                    )).color(Color32::BLUE),
                );
                plotUi.line(
                    Line::new(
                        inputSig.complex0Current.clone(),
                    )
                    .color(Color32::YELLOW),
                )
            });
        });
        egui::Window::new("input").show(ctx, |ui| {
            ui.label(format!(" t: {:?}", inputSig.t.last().unwrap()));
            ui.label(format!("origin length: {}", inputSig.origin.len()));
            ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
            // ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("input").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        inputSig.xyPoints.clone(),
                    )),
                )
            });
        });
        egui::Window::new("fft").show(ctx, |ui| {
            // ui.label(format!("new fft: '{}'", 0));
            let points = inputSig.fftPoints();
            ui.label(format!("fftComplex length: {}", inputSig.fftComplex.len()));
            ui.label(format!("fftPoints length: {}", points.len()));
            if ui.button("just button").clicked() {
            }
            Plot::new("fft").show(ui, |plotUi| {
                plotUi.line(
                    Line::new(PlotPoints::new(
                        points,
                    )).color(Color32::LIGHT_GREEN),
                )
            });
        });
        ctx.request_repaint();
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