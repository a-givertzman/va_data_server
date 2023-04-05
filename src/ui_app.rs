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
    analyze_fft::{
        AnalizeFft, PI
    }, 
    input_signal::InputSignal
};



pub struct UiApp {
    pub inputSignal: Arc<Mutex<InputSignal>>,
    pub analyzeFft: Arc<Mutex<AnalizeFft>>,
}

impl UiApp {
    pub fn new(inputSignal: Arc<Mutex<InputSignal>>, analyzeFft: AnalizeFft) -> Self {
        Self {
            inputSignal: inputSignal, 
            analyzeFft: Arc::new(Mutex::new(analyzeFft)) ,
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let inputSignal = self.inputSignal.lock();
        let analyzeFft = self.analyzeFft.lock();

        egui::Window::new("complex 0").show(ctx, |ui| {
            // ui.label(format!("complex 0: '{}'", 0));
            ui.label(format!(" f: {:?} Hz   T: {:?} sec", analyzeFft.f, analyzeFft.period));
            ui.label(format!(" pfi: {:?}", analyzeFft.phi * 180.0 / PI));
            ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("complex 0")
            .show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        analyzeFft.complex0Points(),
                    )).color(Color32::BLUE),
                );
                plotUi.line(
                    Line::new(
                        analyzeFft.complex0Current.clone(),
                    )
                    .color(Color32::YELLOW),
                )
            });
        });
        egui::Window::new("input complex").show(ctx, |ui| {
            // ui.label(format!("complex 0: '{}'", 0));
            ui.label(format!(" f: {:?} Hz   T: {:?} sec", analyzeFft.f, analyzeFft.period));
            ui.label(format!(" pfi: {:?}", analyzeFft.phi * 180.0 / PI));
            ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("complex")
            .show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        analyzeFft.complexPoints(),
                    )).color(Color32::BLUE),
                );
                plotUi.line(
                    Line::new(
                        analyzeFft.complexCurrent.clone(),
                    )
                    .color(Color32::YELLOW),
                )
            });
        });

        egui::Window::new("input signal").show(ctx, |ui| {
            ui.label(format!(" t: {:?}", inputSignal.t));
            // ui.label(format!(" t: {:?}", inputSignal.t));
            // ui.label(format!("origin length: {}", inputSig.origin.len()));
            // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
            // ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("inputsignal").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        inputSignal.xyPoints.clone()
                    )),
                )
            });
        });

        egui::Window::new("input").show(ctx, |ui| {
            ui.label(format!(" t: {:?}", analyzeFft.t.last().unwrap()));
            ui.label(format!("origin length: {}", analyzeFft.origin.len()));
            ui.label(format!("xyPoints length: {}", analyzeFft.xyPoints.len()));
            // ui.end_row();
            if ui.button("just button").clicked() {
            }
            Plot::new("input").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(PlotPoints::new(
                        analyzeFft.xyPoints.clone(),
                    )),
                )
            });
        });
        egui::Window::new("fft").show(ctx, |ui| {
            // ui.label(format!("new fft: '{}'", 0));
            let points = analyzeFft.fftPoints();
            ui.label(format!("fftComplex length: {}", analyzeFft.fftComplex.len()));
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