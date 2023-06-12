#![allow(non_snake_case)]

use eframe::CreationContext;
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
};
use egui::{
    vec2,
    plot::{
        Plot, 
        Points, 
        // PlotPoints, 
        Line
    }, 
    Color32, Align2, FontFamily, TextStyle, FontId, 
    // mutex::Mutex,
};
use crate::{
    networking::udp_server::UdpServer, 
    fft::fft_analysis::FftAnalysis,
};



pub struct UiApp {
    // pub inputSignal: Arc<Mutex<InputSignal>>,
    // pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    pub udpSrv: Arc<Mutex<UdpServer>>,
    pub fftAnalysis: Arc<Mutex<FftAnalysis>>,
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
        cc: &CreationContext,
        // inputSignal: Arc<Mutex<InputSignal>>, 
        // analyzeFft: Arc<Mutex<AnalizeFft>>,
        udpSrv: Arc<Mutex<UdpServer>>,
        fftAnalysis: Arc<Mutex<FftAnalysis>>,
        // renderDelay: Duration,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::configure_text_styles(&cc.egui_ctx);
        Self {
            udpSrv: udpSrv,
            fftAnalysis: fftAnalysis,
            realInputMinY: -100.0,
            realInputMaxY: 3100.0,
            realInputLen: 1024,
            // realInputAutoscroll: true,
            realInputAutoscaleY: false,
            fftMinY: -10.0,
            fftMaxY: 400.0,
            fftAutoscaleY: false,        
            events: vec![],
        }
    }
    ///
    fn setup_custom_fonts(ctx: &egui::Context) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "Icons".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/icons.ttf"
            )),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Icons".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("Icons".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }
    ///
    fn configure_text_styles(ctx: &egui::Context) {
        use FontFamily::{Monospace, Proportional};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, Proportional)),
            // (heading2(), FontId::new(22.0, Proportional)),
            // (heading3(), FontId::new(19.0, Proportional)),
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ].into();
        ctx.set_style(style);
    }    
}

///
///
impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let wSize = _frame.info().window_info.size;
        let headHight = 34.0;
        self.events.clear();
        let mut even = false;
        for [freq, ampl] in self.fftAnalysis.lock().unwrap().fftAlarmXy.clone() {
            if even {
                self.events.push(format!("Частота {:.1} Гц,  амплитуда {:.2} ", freq, ampl))
            }
            even = !even;
        }

        egui::Window::new("Events")
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .default_size(vec2(0.4 * wSize.x, 0.5 * wSize.y - headHight))
            .show(ctx, |ui| {
                // let btn = Button::image_and_text(
                //     Te
                //     "text"
                // );
                // if ui.button("Restart").clicked() {
                //     self.events.push("New event".to_string());
                // }
                egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (i, event) in self.events.iter().enumerate() {
                        ui.label(format!("{:?}\t|\t{:?}", i, event));
                        ui.separator();
                    }
                });
            });
        egui::Window::new("real input")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .default_size(vec2(0.4 * wSize.x, 0.45 * wSize.y - headHight))
            .show(ctx, |ui| {
                // debug!("[UiApp.update] self.udpSrv.lock...");
                match self.fftAnalysis.lock() {
                    Ok(mut inputSignal) => {
                        // debug!("[UiApp.update] self.udpSrv.lock ready");
                        // ui.label(format!(" i: {:?}", inputSignal.i));
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [200.0, 16.0], 
                                egui::Label::new(format!("Sampling:  F: {:?} kHz,  T: {:.2} us", inputSignal.f * 1.0e-3, inputSignal.samplingPeriod * 1.0e6)),
                            );
                            ui.separator();
                            if ui.add_sized([30., 30.], egui::Button::new("\u{e800}")).clicked() {
                                self.realInputLen -= self.realInputLen / 4;
                                if self.realInputLen < 10 {
                                    self.realInputLen = 10;
                                }
                            }
                            ui.add_sized(
                                [100.0, 16.0], 
                                egui::Label::new(format!(" length: {:.4} ns", (self.realInputLen as f64) * inputSignal.delta * 1.0e9)),
                            );
                            if ui.add_sized([30., 30.], egui::Button::new("\u{e801}")).clicked() {
                                self.realInputLen += self.realInputLen / 4;
                                if self.realInputLen > inputSignal.xy.len() {
                                    self.realInputLen = inputSignal.xy.len();
                                }
                            }
                            ui.separator();
                            // ui.label(format!(" t: {:?}", inputSignal.t));
                            // ui.label(format!(" phi: {:?}", inputSignal.phi));
                            ui.label(format!("max length: {}", inputSignal.xy.len()));
                            ui.separator();
                            ui.checkbox(&mut self.realInputAutoscaleY, "Autoscale Y");
                            // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
                            ui.separator();
                            if ui.button("\u{e802}").clicked() {
                                inputSignal.restart();
                            }
                            ui.separator();
                            ui.add_sized(
                                [50.0, 16.0], 
                                egui::Label::new(format!("lost: {:?}", inputSignal.udpLost)),
                            );
                            if ui.button("\u{e803}").clicked() {
                                inputSignal.udpLost = 0.0;
                                debug!("[UiApp.update] real input udpLost clicked");
                            }
                        });
                        ui.separator();
                        let mut min = format!("{}", self.realInputMinY);
                        let mut max = format!("{}", self.realInputMaxY);
                        let mut len = format!("{}", self.realInputLen);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [32.0, 16.0 * 2.0 + 6.0], 
                                egui::Label::new(format!("↕")), //⇔⇕   ↔
                            );
                            ui.separator();
                                    ui.vertical(|ui| {
                                if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                                    if !self.realInputAutoscaleY {
                                        self.realInputMinY = match min.parse() {Ok(value) => {value}, Err(_) => {self.realInputMinY}};
                                    }
                                };
                                if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                                    if !self.realInputAutoscaleY {
                                        self.realInputMaxY = match max.parse() {Ok(value) => {value}, Err(_) => {self.realInputMaxY}};
                                    }
                                };                          
                            });        
                        });
                        // ui.horizontal(|ui| {
                        //     if ui.text_edit_singleline(&mut len).changed() {
                        //         self.realInputLen = match len.parse() {Ok(value) => {value}, Err(_) => {self.realInputLen}};
                        //     };
                        // });
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
                                )
                                .color(Color32::LIGHT_GREEN)
                                // .radius(2.0)
                                .filled(true),
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
        egui::Window::new("FFT")
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .default_size(vec2(0.6 * wSize.x, 1.0 * wSize.y - headHight))
            .show(ctx, |ui| {
                let analyzeFft = self.fftAnalysis.lock().unwrap();
                // ui.label(format!("new fft: '{}'", 0));
                // let points = analyzeFft.fftXy.clone();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(format!("fftComplex length: {:?}", analyzeFft.fftComplex.len())),
                    );
                    ui.separator();
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(format!("fftPoints length: {:?}", analyzeFft.fftXy.len())),
                    );
                    ui.separator();
                    ui.add_sized(
                        [250.0, 16.0], 
                        egui::Label::new(format!("Drive freq: {:.4} об/мин ({:.2} Гц)", analyzeFft.baseFreq, analyzeFft.baseFreq / 60.0)),
                    );
                    ui.separator();
                    ui.add_sized(
                        [250.0, 16.0], 
                        egui::Label::new(format!("freq offset: {:.4} об/мин ({:.2} Гц)", analyzeFft.offsetFreq, analyzeFft.offsetFreq / 60.0)),
                    );
                    // ui.separator();
                    ui.separator();
                    // if ui.add_sized([200.0, 16.0], egui::Button::new("just button")).clicked() {
                    // }
                });
                let mut min = format!("{}", self.fftMinY);
                let mut max = format!("{}", self.fftMaxY);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [32.0, 16.0 * 2.0 + 6.0], 
                        egui::Label::new(format!("↕")), //⇔⇕   ↔
                    );
                    ui.separator();
                    ui.vertical(|ui| {
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                            if !self.fftAutoscaleY {
                                self.fftMinY = match min.parse() {Ok(value) => {value}, Err(_) => {self.fftMinY}};
                            }    
                        };                    
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                            if !self.fftAutoscaleY {
                                self.fftMaxY = match max.parse() {Ok(value) => {value}, Err(_) => {self.fftMaxY}};
                            }
                        };                          
                    });
                    // ui.separator();
                    // ui.add_sized(
                    //     [32.0, 16.0 * 2.0 + 6.0], 
                    //     egui::Label::new(format!("↔")), //⇔⇕   ↔
                    // );
                    // ui.separator();
                    // ui.vertical(|ui| {
                    //     if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                    //         if !self.fftAutoscaleY {
                    //             self.fftMinY = match min.parse() {Ok(value) => {value}, Err(_) => {self.fftMinY}};
                    //         }    
                    //     };                    
                    //     if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                    //         if !self.fftAutoscaleY {
                    //             self.fftMaxY = match max.parse() {Ok(value) => {value}, Err(_) => {self.fftMaxY}};
                    //         }
                    //     };                          
                    // });
                });
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
                    plotUi.line(
                        Line::new(
                            analyzeFft.limitationsXy.clone(),
                        ).color(Color32::YELLOW),
                    );
                    let mut even = false;
                    let mut series = vec![];
                    for item in analyzeFft.fftAlarmXy.clone() {
                        series.push(item);
                        if even {
                            plotUi.line(
                                Line::new(
                                    series.clone(),
                                ).color(Color32::RED).width(3.0),
                            );
                            series.clear();
                        }
                        even = !even;
                    }
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


pub trait ExtendedColors {
    const orange: Color32 = Color32::from_rgb(255, 152, 0);
    const orangeAccent: Color32 = Color32::from_rgb(255, 152, 0);
    const lightGreen10: Color32 = Color32::from_rgba_premultiplied(0x90, 0xEE, 0x90, 10);
    fn with_opacity(&self, opacity: u8) -> Self;
}

impl ExtendedColors for Color32 {
    fn with_opacity(&self, opacity: u8) -> Self {
        let [r, g, b, _] = self.to_array();
        Color32::from_rgba_premultiplied(r, g, b, opacity)
    }
}
