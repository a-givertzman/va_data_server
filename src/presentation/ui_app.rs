use eframe::CreationContext;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, Points};
use sal_sync::sync::channel::Receiver;
use std::{fmt::{Debug, Display}, sync::{Arc, atomic::Ordering}, time::{Duration, Instant}};
use egui::{vec2, Color32, Align2, FontFamily, TextStyle, FontId};
use crate::{fft::FftAnalysis, networking::UdpClient, presentation::Xy};


const UPLAY: &str = "\u{23F5}";
const UPAUSE: &str = "\u{23F8}";

const ZOOM_IN: &str = "\u{e800}";
const ZOOM_OUT: &str = "\u{e801}";
///
/// 
pub struct UiApp {
    // pub inputSignal: Arc<Mutex<InputSignal>>,
    // pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    pub udp_client: Arc<UdpClient>,
    pub fft: Arc<FftAnalysis>,
    pub recv_xy: Receiver<u16>,
    xy: Xy,
    /// Samples per sec received in real
    sps: Sps,
    real_input_min_y: f64,
    real_input_max_y: f64,
    // realInputAutoscroll: bool,
    real_input_autoscale_y: bool,
    fft_min_y: f64,
    fft_max_y: f64,
    fft_autoscale_y: bool,
    events: Vec<String>,
    render_delay: Duration,
}

impl UiApp {
    pub fn new(
        cc: &CreationContext,
        // inputSignal: Arc<Mutex<InputSignal>>, 
        // analyzeFft: Arc<Mutex<AnalizeFft>>,
        udp_client: Arc<UdpClient>,
        fft: Arc<FftAnalysis>,
        render_delay: Duration,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::configure_text_styles(&cc.egui_ctx);
        Self {
            udp_client,
            recv_xy: fft.xy(),
            xy: Xy::new(
                2048,
                fft.f.load(),
                fft.fft_buflen,
            ),
            fft,
            sps: Sps::new(),
            real_input_min_y: -100.0,
            real_input_max_y: 3100.0,
            // realInputAutoscroll: true,
            real_input_autoscale_y: false,
            fft_min_y: -10.0,
            fft_max_y: 400.0,
            fft_autoscale_y: false,        
            events: vec![],
            render_delay,
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
        let head_hight = 34.0;
        self.events.clear();
        let mut even = false;
        for [freq, ampl] in self.fft.fft_alarm_xy.xy() {
            if even {
                if freq > 0.0 {
                    self.events.push(format!("Частота {:.1} Гц,  амплитуда {:.2} ", freq, ampl))
                }
            }
            even = !even;
        }
        let mut values = vec![];
        while let Ok(val) = self.recv_xy.recv_timeout(Duration::from_millis(1)) {
            values.push(val);
            self.sps.add();
        }
        self.xy.enqueue(&values);
        let vp_size = ctx.input(|is| is.content_rect());
        // log::debug!("UiApp.update | ctx.input | vp_size: {:?}", vp_size);
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
                // debug!("UiApp.update | self.udpSrv.lock...");
                // debug!("UiApp.update | self.udpSrv.lock ready");
                // ui.label(format!(" i: {:?}", inputSignal.i));
                ui.horizontal(|ui| {
                    ui.add_sized([64.0, 16.0], egui::Label::new(
                        format!("Channel:"),
                    ),);
                    let mut channel = format!("{}", self.fft.channel.load(Ordering::Acquire));
                    if ui.add_sized([24.0, 16.0], egui::TextEdit::singleline(&mut channel)).changed() {
                        if let Ok(value) = channel.parse() {
                            self.fft.channel.store(value, Ordering::Release);
                        }
                    };                          
                    ui.separator();
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(
                            format!("Sampling:  F: {:?} kHz,  T: {:.2} us", self.fft.f.load() * 1.0e-3, self.fft.sampling_period.load() * 1.0e6),
                        ),
                    );
                    ui.separator();
                    if ui.add_sized([30., 30.], egui::Button::new(ZOOM_OUT)).clicked() {
                        let len = self.xy.len() + self.xy.len() / 4;
                        if len < self.fft.fft_buflen {
                            self.xy.set_len(len);
                        } else {
                            self.xy.set_len(self.fft.fft_buflen);
                        }
                    }
                    ui.add_sized(
                        [100.0, 16.0], 
                        egui::Label::new(format!(" length: {:.4} ns", (self.xy.len() as f64) * self.fft.delta.load() * 1.0e9)),
                    );
                    if ui.add_sized([30., 30.], egui::Button::new(ZOOM_IN)).clicked() {
                        let len = self.xy.len() - self.xy.len() / 4;
                        if len > 10 {
                            self.xy.set_len(len);
                        } else {
                            self.xy.set_len(10);
                        }
                    }
                    ui.separator();
                    // ui.label(format!(" t: {:?}", inputSignal.t));
                    // ui.label(format!(" phi: {:?}", inputSignal.phi));
                    ui.label(format!("max length: {}", self.xy.len()));
                    ui.separator();
                    ui.checkbox(&mut self.real_input_autoscale_y, "Autoscale Y");
                    // ui.label(format!("xyPoints length: {}", inputSig.xyPoints.len()));
                    ui.separator();
                    if ui.button("\u{e802}").clicked() {
                        self.fft.restart();
                    }
                    ui.separator();
                    ui.add_sized(
                        [50.0, 16.0], 
                        egui::Label::new(format!("lost: {}", self.fft.udp_lost)),
                    );
                    if ui.button("\u{e803}").clicked() {
                        self.fft.udp_lost.store(0.0);
                        log::debug!("UiApp.update | real input udpLost clicked");
                    }
                    ui.separator();
                    let pause = self.fft.pause.load(Ordering::Acquire);
                    if ui.button(if pause {UPLAY} else {UPAUSE}).clicked() {
                        self.fft.pause.store(!pause, Ordering::Release);
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(
                            format!("SPS: {}", self.sps),
                        ),
                    );
                    ui.separator();
                });
                ui.separator();
                let mut min = format!("{}", self.real_input_min_y);
                let mut max = format!("{}", self.real_input_max_y);
                // let mut len = format!("{}", self.real_input_len);
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
                            PlotPoints::from_iter(
                                self.xy.values(), //.iter().map(|p| PlotPoint::from(p)),
                            )
                        )
                        .color(Color32::LIGHT_GREEN)
                        // .radius(2.0)
                        .filled(true),
                    );
                    plot_ui.line(
                        Line::new(
                            "",
                            self.xy.values()
                        ).color(Color32::GRAY),
                    );                        
                });
            });
            egui::Window::new("FFT")
                .anchor(Align2::LEFT_TOP, [0.0, 0.0])
                .default_size(vec2(0.6 * vp_size.width(), 1.0 * vp_size.height() - head_hight))
                .show(ctx, |ui| {
                    // ui.label(format!("new fft: '{}'", 0));
                    // let points = analyzeFft.fftXy.clone();
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [200.0, 16.0], 
                            egui::Label::new(format!("fftComplex length: {:?}", self.fft.fft_complex.read().len())),
                        );
                        ui.separator();
                        ui.add_sized(
                            [200.0, 16.0], 
                            egui::Label::new(format!("fftPoints length: {:?}", self.fft.fft_xy.len())),
                        );
                        ui.separator();
                        ui.add_sized(
                            [250.0, 16.0], 
                            egui::Label::new(format!("Drive freq: {:.4} об/мин ({:.2} Гц)", self.fft.base_freq, self.fft.base_freq.load() / 60.0)),
                        );
                        ui.separator();
                        ui.add_sized(
                            [250.0, 16.0], 
                            egui::Label::new(format!("freq offset: {:.4} об/мин ({:.2} Гц)", self.fft.offset_freq, self.fft.offset_freq.load() / 60.0)),
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
                                self.fft.fft_xy.xy(),
                            ).color(Color32::LIGHT_GREEN),
                        );
                        plot_ui.line(
                            Line::new(
                                "limitationsXy",
                                self.fft.limitations_xy.xy(),
                            ).color(Color32::YELLOW),
                        );
                        let mut even = false;
                        let mut series = vec![];
                        for item in self.fft.fft_alarm_xy.xy() {
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
                                    self.fft.fft_xy_dif.xy()
                                ).color(Color32::DARK_RED),
                            );
                        }
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
        std::thread::sleep(self.render_delay);
        ctx.request_repaint();
        // if self.fft.xy.is_changed() {
        // }
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

///
/// Average Samples per sec
pub struct Sps {
    t: Option<Instant>,
    count: usize,
    average: f64,
}
impl Sps {
    pub fn new() -> Self {
        Self {
            t: None,
            count: 0,
            average: 0.0,
        }
    }
    ///
    /// 
    pub fn add(&mut self) {
        let elapsed = match self.t {
            Some(t) => t.elapsed(),
            None => {
                let t = Instant::now();
                let elapsed = t.elapsed();
                self.t = Some(t);
                elapsed
            },
        };
        self.count += 1;
        self.average = self.count as f64 / elapsed.as_secs_f64()
    }
    ///
    /// Returns average SPS value
    pub fn average(&self) -> f64 {
        self.average
    }
}

impl Debug for Sps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.average)
    }
}
impl Display for Sps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.average)
    }
}