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
    vec2,
    plot::{
        Plot, 
        Points, 
        // PlotPoints, 
        Line
    }, 
    Color32, Align2, Button, 
    // mutex::Mutex,
};
use crate::{
    // analyze_fft::{
    //     AnalizeFft
    // }, 
    // input_signal::{
    //     InputSignal,
    //     PI,
    // }, 
    udp_server::UdpServer
};



pub struct UiApp {
    // pub inputSignal: Arc<Mutex<InputSignal>>,
    // pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    pub udpSrv: Arc<Mutex<UdpServer>>,
    // renderDelay: Duration,
    realInputMinY: f64,
    realInputMaxY: f64,
    realInputLen: usize,
    // realInputAutoscroll: bool,
    realInputAutoscaleY: bool,
    fftMinY: f64,
    fftMaxY: f64,
    fftAutoscaleY: bool,
    events: Vec<String>,
}

impl UiApp {
    pub fn new(
        // inputSignal: Arc<Mutex<InputSignal>>, 
        // analyzeFft: Arc<Mutex<AnalizeFft>>,
        udpSrv: Arc<Mutex<UdpServer>>,
        // renderDelay: Duration,
    ) -> Self {
        Self {
            // inputSignal: inputSignal, 
            // analyzeFft: analyzeFft,
            udpSrv: udpSrv,
            // renderDelay: renderDelay,
            realInputMinY: -100.0,
            realInputMaxY: 2100.0,
            realInputLen: 16,
            // realInputAutoscroll: true,
            realInputAutoscaleY: false,
            fftMinY: -1000.0,
            fftMaxY: 4000000.0,
            fftAutoscaleY: false,        
            events: vec![],
        }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let wSize = _frame.info().window_info.size;
        let headHight = 34.0;

        egui::Window::new("Events")
            .anchor(Align2::LEFT_BOTTOM, [0.0, 0.0])
            .default_size(vec2(800.0, 0.5 * wSize.y - headHight))
            .show(ctx, |ui| {
                // let btn = Button::image_and_text(
                //     Te
                //     "text"
                // );
                if ui.button("Restart").clicked() {
                    self.events.push("New event".to_string());
                }
                egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (i, event) in self.events.iter().enumerate() {
                        ui.label(format!("{:?}\t| {:?}", i, event));
                    }
                });
            });
        egui::Window::new("real input")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .default_size(vec2(1200.0, 800.0))
            .show(ctx, |ui| {
                // debug!("[UiApp.update] self.udpSrv.lock...");
                match self.udpSrv.lock() {
                    Ok(mut inputSignal) => {
                        // debug!("[UiApp.update] self.udpSrv.lock ready");
                        // ui.label(format!(" i: {:?}", inputSignal.i));
                        ui.label(format!(" t: {:?}", inputSignal.t));
                        ui.end_row();
                        // ui.label(format!(" phi: {:?}", inputSignal.phi));
                        ui.label(format!("length: {}", inputSignal.xy.len()));
                        ui.checkbox(&mut self.realInputAutoscaleY, "Autoscale Y");
                        // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
                        // ui.end_row();
                        if ui.button("Restart").clicked() {
                            inputSignal.restart();
                            // inputSignal.cancel();
                        }
                        let mut min = format!("{}", self.realInputMinY);
                        let mut max = format!("{}", self.realInputMaxY);
                        let mut len = format!("{}", self.realInputLen);
                        if ui.text_edit_singleline(&mut min).changed() {
                            if !self.realInputAutoscaleY {
                                self.realInputMinY = match min.parse() {Ok(value) => {value}, Err(_) => {self.realInputMinY}};
                            }    
                        };
                        if ui.text_edit_singleline(&mut max).changed() {
                            if !self.realInputAutoscaleY {
                                self.realInputMaxY = match max.parse() {Ok(value) => {value}, Err(_) => {self.realInputMaxY}};
                            }
                        };
                        ui.end_row();
                        ui.horizontal(|ui| {
                            let btnSub = ui.add_sized([20., 20.], egui::Button::new("-"));
                            if ui.text_edit_singleline(&mut len).changed() {
                                self.realInputLen = match len.parse() {Ok(value) => {value}, Err(_) => {self.realInputLen}};
                            };
                            let btnAdd = ui.add_sized([20., 20.], egui::Button::new("+"));
                            if btnSub.clicked() {
                                self.realInputLen -= self.realInputLen / 4;
                                if self.realInputLen < 1 {
                                    self.realInputLen = 1;
                                }
                            }
                            if btnAdd.clicked() {
                                self.realInputLen += self.realInputLen / 4;
                                if self.realInputLen > inputSignal.xy.len() {
                                    self.realInputLen = inputSignal.xy.len();
                                }
                            }
                        });
                        let mut plot = Plot::new("real input");
                        if !self.realInputAutoscaleY {
                            plot = plot.include_y(self.realInputMinY);
                            plot = plot.include_y(self.realInputMaxY);
                        }
                        // let mut xy = [[0.0; 2]; self.realInputLen];
                        // if !self.realInputAutoscroll {
                        //     self.realInputLen = match max.parse() {Ok(value) => {value}, Err(_) => {self.realInputLen}};
                        //     // plot = plot.include_y(self.realInputLen);
                        //     let xy = inputSignal.xy.buffer().split_at(self.realInputLen).0.to_vec();
                        //     plot.show(ui, |plotUi| {
                        //         plotUi.points(
                        //             Points::new(
                        //                 xy
                        //             ),
                        //         );
                        //     });
                        // }
                        plot.show(ui, |plotUi| {
                            plotUi.points(
                                Points::new(
                                    ((inputSignal.xy)[0..self.realInputLen]).to_vec()
                                ).color(Color32::LIGHT_GREEN).radius(2.0).filled(true),
                            );
                            plotUi.line(
                                Line::new(
                                    ((inputSignal.xy)[0..self.realInputLen]).to_vec()
                                ).color(Color32::GRAY),
                            );                        
                        });
                    },
                    Err(err) => {
                        debug!("[UiApp.update] self.udpSrv.lock error: {:?}", err);
                    },
                };
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
        egui::Window::new("fft")
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .default_size(vec2(800.0, 0.5 * wSize.y - headHight))
            .show(ctx, |ui| {
                let analyzeFft = self.udpSrv.lock().unwrap();
                // ui.label(format!("new fft: '{}'", 0));
                // let points = analyzeFft.fftXy.clone();
                ui.label(format!("fftComplex length: {}", analyzeFft.fftComplex.len()));
                ui.label(format!("fftPoints length: {}", analyzeFft.fftXy.len()));
                if ui.button("just button").clicked() {
                }
                let mut min = format!("{}", self.fftMinY);
                let mut max = format!("{}", self.fftMaxY);
                if ui.text_edit_singleline(&mut min).changed() {
                    if !self.fftAutoscaleY {
                        self.fftMinY = match min.parse() {Ok(value) => {value}, Err(_) => {self.fftMinY}};
                    }    
                };
                if ui.text_edit_singleline(&mut max).changed() {
                    if !self.fftAutoscaleY {
                        self.fftMaxY = match max.parse() {Ok(value) => {value}, Err(_) => {self.fftMaxY}};
                    }
                };
                let mut plot = Plot::new("fft");
                if !self.fftAutoscaleY {
                    plot = plot.include_y(self.fftMinY);
                    plot = plot.include_y(self.fftMaxY);
                }                
                plot.show(ui, |plotUi| {
                        plotUi.line(
                            Line::new(
                                analyzeFft.fftXy.clone(),
                            ).color(Color32::LIGHT_GREEN),
                        );
                        if false {
                            plotUi.points(
                                Points::new(
                                    analyzeFft.fftXyDif.clone()
                                ).color(Color32::DARK_RED),
                            );
                        }
                    });
            });
        // std::thread::sleep(self.renderDelay);
        ctx.request_repaint();
    }
}







        // CentralPanel::default().show(ctx, add_contents);
        // let phi = self.inputSignal.lock().unwrap().phi;
        // let f = self.inputSignal.lock().unwrap().f;
        // let period = self.inputSignal.lock().unwrap().period;
        
        // egui::Window::new("complex 0").show(ctx, |ui| {
        //     let mut analyzeFft = self.analyzeFft.lock().unwrap();
        //     // ui.label(format!("complex 0: '{}'", 0));
        //     ui.label(format!(" f: {:?} Hz   T: {:?} sec", f, period));
        //     ui.label(format!(" pfi: {:?}", phi * 180.0 / PI));
        //     ui.label(format!(" complex 0 len: {:?}", self.inputSignal.lock().unwrap().complex0.len()));
        //     ui.end_row();
        //     if ui.button("Stop").clicked() {
        //         analyzeFft.cancel();
        //     }
        //     Plot::new("complex 0")
        //     .show(ui, |plotUi| {
        //         let points: Vec<[f64; 2]> = self.inputSignal.lock().unwrap().complex0.iter().map(|complex| {
        //             [complex.re, complex.im]
        //         }).collect();
        //         let i = self.inputSignal.lock().unwrap().i;
        //         plotUi.points(
        //             Points::new(
        //                 points.clone(),
        //             ).color(Color32::BLUE),
        //         );
        //         if points.len() > 2 {
        //             plotUi.line(
        //                 Line::new(
        //                     vec![[0.0; 2], points[i-1]],
        //                 )
        //                 .color(Color32::YELLOW),
        //             );
        //         }
        //     });
        // });
        // egui::Window::new("input complex").show(ctx, |ui| {
        //     // let analyzeFft = self.analyzeFft.lock().unwrap();
        //     // ui.label(format!("complex 0: '{}'", 0));
        //     ui.label(format!(" f: {:?} Hz   T: {:?} sec", f, period));
        //     ui.label(format!(" pfi: {:?}", phi * 180.0 / PI));
        //     ui.end_row();
        //     let textEdit = ui.text_edit_singleline(&mut self.text);
        //     if textEdit.changed() {
        //         debug!("text edited: {:?}", self.text);
        //     };
        //     if textEdit.lost_focus() {
        //         debug!("text editing finished: {:?}", self.text);
        //     };
        //     if ui.button("just button").clicked() {
        //     }
        //     Plot::new("complex")
        //     .show(ui, |plotUi| {
        //         let points: Vec<[f64; 2]> = self.inputSignal.lock().unwrap().complex.buffer().iter().map(|complex| {
        //             [complex.re, complex.im]
        //         }).collect();
        //         plotUi.points(
        //             Points::new(
        //                 points.clone(),
        //             ).color(Color32::BLUE),
        //         );
        //         let i = self.inputSignal.lock().unwrap().i;
        //         plotUi.line(
        //             Line::new(
        //                 vec![[0.0; 2], points[i]],
        //             )
        //             .color(Color32::YELLOW),
        //         );
        //     });
        // });

        // egui::Window::new("input signal")
        //     .show(ctx, |ui| {
        //         let mut inputSignal = self.inputSignal.lock().unwrap();
        //         ui.label(format!(" i: {:?}", inputSignal.i));
        //         ui.label(format!(" t: {:?}", inputSignal.t));
        //         ui.label(format!(" phi: {:?}", inputSignal.phi));
        //         // ui.label(format!(" t: {:?}", inputSignal.t));
        //         ui.label(format!("length: {}", inputSignal.xyPoints.len()));
        //         // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
        //         // ui.end_row();
        //         if ui.button("Stop").clicked() {
        //             inputSignal.cancel();
        //         }
        //         Plot::new("inputsignal").show(ui, |plotUi| {
        //             plotUi.points(
        //                 Points::new(
        //                     inputSignal.xyPoints.buffer().clone()
        //                 ),
        //             )
        //         });
        //     });