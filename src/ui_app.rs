#![allow(non_snake_case)]

use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    time::Duration
};
use egui::{
    plot::{
        Plot, 
        Points, 
        PlotPoints, Line
    }, 
    Color32, 
    // mutex::Mutex,
};
use crate::{
    analyze_fft::{
        AnalizeFft
    }, 
    input_signal::{
        InputSignal,
        PI,
    }
};



pub struct UiApp {
    pub inputSignal: Arc<Mutex<InputSignal>>,
    pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    renderDelay: Duration,
    text: String,
}

impl UiApp {
    pub fn new(
        inputSignal: Arc<Mutex<InputSignal>>, 
        analyzeFft: Arc<Mutex<AnalizeFft>>,
        renderDelay: Duration,
    ) -> Self {
        Self {
            inputSignal: inputSignal, 
            analyzeFft: analyzeFft,
            renderDelay: renderDelay,
            text: String::from(""),
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let i = self.inputSignal.lock().unwrap().i;
        
        egui::Window::new("complex 0").show(ctx, |ui| {
            let mut analyzeFft = self.analyzeFft.lock().unwrap();
            // ui.label(format!("complex 0: '{}'", 0));
            ui.label(format!(" f: {:?} Hz   T: {:?} sec", analyzeFft.f, analyzeFft.period));
            ui.label(format!(" pfi: {:?}", analyzeFft.phi * 180.0 / PI));
            ui.end_row();
            if ui.button("Stop").clicked() {
                analyzeFft.cancel();
            }
            Plot::new("complex 0")
            .show(ui, |plotUi| {
                let points: Vec<[f64; 2]> = analyzeFft.inputSignal.lock().unwrap().complex0.iter().map(|complex| {
                    [complex.re, complex.im]
                }).collect();
                plotUi.points(
                    Points::new(
                        points.clone(),
                    ).color(Color32::BLUE),
                );
                plotUi.line(
                    Line::new(
                        vec![[0.0; 2], points[i-1]],
                    )
                    .color(Color32::YELLOW),
                )
            });
        });
        egui::Window::new("input complex").show(ctx, |ui| {
            let analyzeFft = self.analyzeFft.lock().unwrap();
            // ui.label(format!("complex 0: '{}'", 0));
            ui.label(format!(" f: {:?} Hz   T: {:?} sec", analyzeFft.f, analyzeFft.period));
            ui.label(format!(" pfi: {:?}", analyzeFft.phi * 180.0 / PI));
            ui.end_row();
            let textEdit = ui.text_edit_singleline(&mut self.text);
            if textEdit.changed() {
                debug!("text edited: {:?}", self.text);
            };
            if textEdit.lost_focus() {
                debug!("text editing finished: {:?}", self.text);
            };
            if ui.button("just button").clicked() {
            }
            let points: Vec<[f64; 2]> = analyzeFft.inputSignal.lock().unwrap().complex.iter().map(|complex| {
                [complex.re, complex.im]
            }).collect();
            Plot::new("complex")
            .show(ui, |plotUi| {
                plotUi.points(
                    Points::new(
                        points.clone(),
                    ).color(Color32::BLUE),
                );
                plotUi.line(
                    Line::new(
                        vec![[0.0; 2], points[i-1]],
                    )
                    .color(Color32::YELLOW),
                )
            });
        });

        egui::Window::new("input signal").show(ctx, |ui| {
            let mut inputSignal = self.inputSignal.lock().unwrap();
            ui.label(format!(" i: {:?}", inputSignal.i));
            ui.label(format!(" t: {:?}", inputSignal.t));
            ui.label(format!(" phi: {:?}", inputSignal.phi));
            // ui.label(format!(" t: {:?}", inputSignal.t));
            ui.label(format!("length: {}", inputSignal.xyPoints.len()));
            // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
            // ui.end_row();
            if ui.button("Stop").clicked() {
                inputSignal.cancel();
            }
            Plot::new("inputsignal").show(ui, |plotUi| {
                plotUi.points(
                    Points::new(
                        inputSignal.xyPoints.buffer().clone()
                    ),
                )
            });
        });

        // egui::Window::new("AnalyzeFft input").show(ctx, |ui| {
        //     let analyzeFft = self.analyzeFft.lock().unwrap();
        //     ui.label(format!(" t: {:?}", analyzeFft.t));
        //     ui.label(format!("t length: {}", analyzeFft.tList.len()));
        //     ui.label(format!("xyPoints length: {}", analyzeFft.xyPoints.len()));
        //     // ui.end_row();
        //     if ui.button("just button").clicked() {
        //     }
        //     Plot::new("input").show(ui, |plotUi| {
        //         plotUi.points(
        //             Points::new(
        //                 analyzeFft.xyPoints.buffer().clone(),
        //             ),
        //         )
        //     });
        // });
        egui::Window::new("fft").show(ctx, |ui| {
            let analyzeFft = self.analyzeFft.lock().unwrap();
            // ui.label(format!("new fft: '{}'", 0));
            let points = analyzeFft.fftPoints();
            ui.label(format!("fftComplex length: {}", analyzeFft.fftComplex.len()));
            ui.label(format!("fftPoints length: {}", points.len()));
            if ui.button("just button").clicked() {
            }
            Plot::new("fft").show(ui, |plotUi| {
                plotUi.line(
                    Line::new(
                        points,
                    ).color(Color32::LIGHT_GREEN),
                )
            });
        });
        std::thread::sleep(self.renderDelay);
        ctx.request_repaint();
    }
}
