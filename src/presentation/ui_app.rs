use eframe::CreationContext;
use egui_plot::{Line, Plot, Points};
use std::{sync::{Arc, Mutex}};
use egui::{
    vec2,
    Color32, Align2, FontFamily, TextStyle, FontId, 
};
use crate::{
    networking::udp_server::UdpServer, 
    fft::fft_analysis::FftAnalysis,
};



pub struct UiApp {
    // pub inputSignal: Arc<Mutex<InputSignal>>,
    // pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    pub udp_srv: Arc<Mutex<UdpServer>>,
    pub fft_analysis: Arc<Mutex<FftAnalysis>>,
    // renderDelay: Duration,
    real_input_min_y: f64,
    real_input_max_y: f64,
    real_input_len: usize,
    // realInputAutoscroll: bool,
    real_input_autoscale_y: bool,
    fft_min_y: f64,
    fft_max_y: f64,
    fft_autoscale_y: bool,
    events: Vec<String>,
}

impl UiApp {
    pub fn new(
        cc: &CreationContext,
        // inputSignal: Arc<Mutex<InputSignal>>, 
        // analyzeFft: Arc<Mutex<AnalizeFft>>,
        udp_srv: Arc<Mutex<UdpServer>>,
        fft_analysis: Arc<Mutex<FftAnalysis>>,
        // renderDelay: Duration,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::configure_text_styles(&cc.egui_ctx);
        Self {
            udp_srv,
            fft_analysis,
            real_input_min_y: -100.0,
            real_input_max_y: 3100.0,
            real_input_len: 1024,
            // realInputAutoscroll: true,
            real_input_autoscale_y: false,
            fft_min_y: -10.0,
            fft_max_y: 400.0,
            fft_autoscale_y: false,        
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
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/icons.ttf"
            ))),
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
        let vp_size = ctx.input(|i| i.viewport().inner_rect).unwrap();
        let head_hight = 34.0;
        self.events.clear();
        let mut even = false;
        for [freq, ampl] in self.fft_analysis.lock().unwrap().fftAlarmXy.xy() {
            if even {
                self.events.push(format!("Частота {:.1} Гц,  амплитуда {:.2} ", freq, ampl))
            }
            even = !even;
        }

        egui::Window::new("Events")
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .default_size(vec2(0.4 * vp_size.width(), 0.5 * vp_size.height() - head_hight))
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
            .default_size(vec2(0.4 * vp_size.width(), 0.45 * vp_size.height() - head_hight))
            .show(ctx, |ui| {
                // debug!("[UiApp.update] self.udpSrv.lock...");
                match self.fft_analysis.lock() {
                    Ok(mut input_signal) => {
                        // debug!("[UiApp.update] self.udpSrv.lock ready");
                        // ui.label(format!(" i: {:?}", inputSignal.i));
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [200.0, 16.0], 
                                egui::Label::new(format!("Sampling:  F: {:?} kHz,  T: {:.2} us", input_signal.f * 1.0e-3, input_signal.samplingPeriod * 1.0e6)),
                            );
                            ui.separator();
                            if ui.add_sized([30., 30.], egui::Button::new("\u{e801}")).clicked() {
                                self.real_input_len += self.real_input_len / 4;
                                if self.real_input_len > input_signal.xyLen * 4 {
                                    self.real_input_len = input_signal.xyLen * 4;
                                }
                                input_signal.xy.setLen(self.real_input_len);
                            }
                            ui.add_sized(
                                [100.0, 16.0], 
                                egui::Label::new(format!(" length: {:.4} ns", (self.real_input_len as f64) * input_signal.delta * 1.0e9)),
                            );
                            if ui.add_sized([30., 30.], egui::Button::new("\u{e800}")).clicked() {
                                self.real_input_len -= self.real_input_len / 4;
                                if self.real_input_len < 10 {
                                    self.real_input_len = 10;
                                }
                                input_signal.xy.setLen(self.real_input_len);
                            }
                            ui.separator();
                            // ui.label(format!(" t: {:?}", inputSignal.t));
                            // ui.label(format!(" phi: {:?}", inputSignal.phi));
                            ui.label(format!("max length: {}", input_signal.xy.len()));
                            ui.separator();
                            ui.checkbox(&mut self.real_input_autoscale_y, "Autoscale Y");
                            // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
                            ui.separator();
                            if ui.button("\u{e802}").clicked() {
                                input_signal.restart();
                            }
                            ui.separator();
                            ui.add_sized(
                                [50.0, 16.0], 
                                egui::Label::new(format!("lost: {:?}", input_signal.udpLost)),
                            );
                            if ui.button("\u{e803}").clicked() {
                                input_signal.udpLost = 0.0;
                                log::debug!("[UiApp.update] real input udpLost clicked");
                            }
                        });
                        ui.separator();
                        let mut min = format!("{}", self.real_input_min_y);
                        let mut max = format!("{}", self.real_input_max_y);
                        let mut len = format!("{}", self.real_input_len);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [32.0, 16.0 * 2.0 + 6.0], 
                                egui::Label::new(format!("↕")), //⇔⇕   ↔
                            );
                            ui.separator();
                            ui.vertical(|ui| {
                                if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                                    if !self.real_input_autoscale_y {
                                        self.real_input_max_y = match max.parse() {Ok(value) => {value}, Err(_) => {self.real_input_max_y}};
                                    }
                                };                          
                                if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                                    if !self.real_input_autoscale_y {
                                        self.real_input_min_y = match min.parse() {Ok(value) => {value}, Err(_) => {self.real_input_min_y}};
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
                        if !self.real_input_autoscale_y {
                            plot = plot.include_y(self.real_input_min_y);
                            plot = plot.include_y(self.real_input_max_y);
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
                        plot.show(ui, |plot_ui| {
                            plot_ui.points(
                                Points::new(
                                    "input_signal",
                                    input_signal.xy.xy()
                                    // ((inputSignal.xy.xy())[0..self.realInputLen]).to_vec()
                                )
                                .color(Color32::LIGHT_GREEN)
                                // .radius(2.0)
                                .filled(true),
                            );
                            plot_ui.line(
                                Line::new(
                                    "",
                                    input_signal.xy.xy()
                                    // ((inputSignal.xy.xy())[0..self.realInputLen]).to_vec()
                                ).color(Color32::GRAY),
                            );                        
                        });
                    },
                    Err(err) => {
                        log::warn!("[UiApp.update] self.udpSrv.lock error: {:?}", err);
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
            .default_size(vec2(0.6 * vp_size.width(), 1.0 * vp_size.height() - head_hight))
            .show(ctx, |ui| {
                let analyze_fft = self.fft_analysis.lock().unwrap();
                // ui.label(format!("new fft: '{}'", 0));
                // let points = analyzeFft.fftXy.clone();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(format!("fftComplex length: {:?}", analyze_fft.fftComplex.len())),
                    );
                    ui.separator();
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(format!("fftPoints length: {:?}", analyze_fft.fftXy.len())),
                    );
                    ui.separator();
                    ui.add_sized(
                        [250.0, 16.0], 
                        egui::Label::new(format!("Drive freq: {:.4} об/мин ({:.2} Гц)", analyze_fft.baseFreq, analyze_fft.baseFreq / 60.0)),
                    );
                    ui.separator();
                    ui.add_sized(
                        [250.0, 16.0], 
                        egui::Label::new(format!("freq offset: {:.4} об/мин ({:.2} Гц)", analyze_fft.offsetFreq, analyze_fft.offsetFreq / 60.0)),
                    );
                    // ui.separator();
                    ui.separator();
                    // if ui.add_sized([200.0, 16.0], egui::Button::new("just button")).clicked() {
                    // }
                });
                let mut min = format!("{}", self.fft_min_y);
                let mut max = format!("{}", self.fft_max_y);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [32.0, 16.0 * 2.0 + 6.0], 
                        egui::Label::new(format!("↕")), //⇔⇕   ↔
                    );
                    ui.separator();
                    ui.vertical(|ui| {
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                            if !self.fft_autoscale_y {
                                self.fft_max_y = match max.parse() {Ok(value) => {value}, Err(_) => {self.fft_max_y}};
                            }
                        };                          
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                            if !self.fft_autoscale_y {
                                self.fft_min_y = match min.parse() {Ok(value) => {value}, Err(_) => {self.fft_min_y}};
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
                if !self.fft_autoscale_y {
                    plot = plot.include_y(self.fft_min_y);
                    plot = plot.include_y(self.fft_max_y);
                }                
                plot.show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new(
                            "fftXy",
                            analyze_fft.fftXy.xy(),
                        ).color(Color32::LIGHT_GREEN),
                    );
                    plot_ui.line(
                        Line::new(
                            "limitationsXy",
                            analyze_fft.limitationsXy.xy(),
                        ).color(Color32::YELLOW),
                    );
                    let mut even = false;
                    let mut series = vec![];
                    for item in analyze_fft.fftAlarmXy.xy() {
                        series.push(item);
                        if even {
                            plot_ui.line(
                                Line::new(
                                    "fftAlarmXy",
                                    series.clone(),
                                ).color(Color32::RED).width(3.0),
                            );
                            series.clear();
                        }
                        even = !even;
                    }
                    if false {
                        plot_ui.points(
                            Points::new(
                                "fftXyDif",
                                analyze_fft.fftXyDif.xy()
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
    const ORANGE: Color32 = Color32::from_rgb(255, 152, 0);
    const ORANGE_ACCENT: Color32 = Color32::from_rgb(255, 152, 0);
    const LIGHT_GREEN10: Color32 = Color32::from_rgba_premultiplied(0x90, 0xEE, 0x90, 10);
    fn with_opacity(&self, opacity: u8) -> Self;
}

impl ExtendedColors for Color32 {
    fn with_opacity(&self, opacity: u8) -> Self {
        let [r, g, b, _] = self.to_array();
        Color32::from_rgba_premultiplied(r, g, b, opacity)
    }
}
