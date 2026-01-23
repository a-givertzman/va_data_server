use eframe::CreationContext;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, Points};
use num::{Complex, complex::ComplexFloat};
use sal_core::dbg::Dbg;
use sal_sync::sync::channel::Receiver;
use std::{fmt::{Debug, Display}, iter::Sum, sync::{Arc, atomic::Ordering}, time::{Duration, Instant}};
use egui::{vec2, Color32, Align2, FontFamily, TextStyle, FontId};
use crate::{circular_queue::CircularQueue, dsp_filters::average_filter::AverageFilter, fft::FftAnalysis, networking::UdpClient, presentation::{PlotData, Xy}};


const UPLAY: &str = "\u{23F5}";
const UPAUSE: &str = "\u{23F8}";

const ZOOM_IN: &str = "\u{e800}";
const ZOOM_OUT: &str = "\u{e801}";
///
/// 
pub struct UiApp {
    // pub inputSignal: Arc<Mutex<InputSignal>>,
    // pub analyzeFft: Arc<Mutex<AnalizeFft>>,
    udp_client: Arc<UdpClient>,
    fft: Arc<FftAnalysis>,
    recv_samples: Receiver<u16>,
    xy: Xy,
    fft_xy: PlotData,
    fft_alarm_xy: PlotData,
    fft_xy_dif: PlotData,
    envelope_xy: PlotData,
    limitations_xy: PlotData,
    /// Samples per sec received in real
    sps: Sps,
    average_amp:  AverageAmp<u16>,
    real_input_min_y: f64,
    real_input_max_y: f64,
    // realInputAutoscroll: bool,
    real_input_autoscale_y: bool,
    fft_min_y: f64,
    fft_max_y: f64,
    fft_autoscale_y: bool,
    events: Vec<String>,
    render_delay: Duration,
    dbg: Dbg,
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
        let fft_xy_len = 35_000;
        Self {
            udp_client,
            recv_samples: fft.samples(),
            xy: Xy::new(
                2048,
                fft.freq.load(),
                fft.buf_len,
            ),
            fft_xy: PlotData::new(fft_xy_len),
            fft_alarm_xy: PlotData::new(fft_xy_len * 2),
            fft_xy_dif: PlotData::new(fft.buf_len),
            envelope_xy: PlotData::new(fft.buf_len),
            limitations_xy: PlotData::new(fft_xy_len), // Self::buildLimitations(fftXyLen * 2, 0.0),
            fft,
            sps: Sps::new(),
            average_amp: AverageAmp::new(),
            real_input_min_y: -100.0,
            real_input_max_y: 3100.0,
            // realInputAutoscroll: true,
            real_input_autoscale_y: false,
            fft_min_y: -10.0,
            fft_max_y: 400.0,
            fft_autoscale_y: false,        
            events: vec![],
            render_delay,
            dbg: Dbg::own("UiApp"),
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
    ///
    ///
    fn build_fft_xy(&self) {
        // let factor = 1.0 / ((sampl_freq / 4) as f64);
        let x_factor = self.fft.freq.load() as f64 / self.fft.buf_len as f64;
        // let coherent_gain = 0.54;  // Hamming window coherent gain
        // let y_factor = coherent_gain * 2.0 / self.fft.buf_len as f64;
        let y_factor = 2.0 / self.fft.buf_len as f64;
        let mut x: f64;
        let mut y: f64;
        self.fft_xy.clear();
        self.fft_alarm_xy.clear();
        self.fft_xy.push(&[0.0, 0.0]);
        self.fft_xy.push(&[0.0, 0.0]);
        let complex = self.fft.fft_complex.read();
        for i in 1..self.fft_xy.len() {
            x = i as f64 * x_factor;
            y = complex.get(i).unwrap_or(&Complex::new(0.0, 0.0)).abs() * y_factor;
            // y = ((self.fftComplex[i].re.powi(2) + self.fftComplex[i].im.powi(2)) * factor) as f64;
            // if Self::fft_point_overflowed(x, y, limitations_xy) {
            //     fft_alarm_xy.push([x, 0.0]);
            //     fft_alarm_xy.push([x, y]);    
            // }
            self.fft_xy.push(&[x, 0.0]);
            self.fft_xy.push(&[x, y]);
        }
    }
    ///
    ///
    fn fft_point_overflowed(freq: f64, amplitude: f64, limitations_xy: &Arc<PlotData>) -> bool {
        let mut range = Range {min: 0.0, max: 0.0};
        for [x, amplitude_limit] in limitations_xy.xy() {
            range.max = x;
            if range.contains(freq) {
                return amplitude >= amplitude_limit;
            }
        }
        false
    }
    ///
    /// 
    fn build_envelope(fft_xy_len: usize, fft_xy: &Arc<PlotData>, envelope_xy: &Arc<PlotData>) {
        let len = fft_xy_len;
        // let mut buf: heapless::spsc::Queue<f64, 3> = heapless::spsc::Queue::new();
        let filter_len: usize = 256;
        let mut filter_buf: CircularQueue<f64> = CircularQueue::with_capacity_fill(filter_len, &mut vec![0.0; filter_len]);
        // let factor = 1.0;// / ((self.fftBuflen / 2) as f64);
        let mut x: f64;
        let mut y: f64;
        let mut average: f64;
        envelope_xy.clear();
        for i in 0..len {
            x = fft_xy.get(i)[0];
            y = fft_xy.get(i)[1];
            filter_buf.push(y);
            average = filter_buf.buffer().iter().sum::<f64>() / (filter_len as f64);
            average = y  + 10.0 * average;
            // average = 100000.0 + 0.1 * y  + (filterBuf.buffer().iter().sum::<f64>() / (filterLen as f64));
            // self.envelopeXy.push([x, 0.0]);
            envelope_xy.push(&[x, average]);
        }
    }
    ///
    /// Производная от FFT
    fn build_fft_xy_dif(fft_xy_len: usize, fft_xy: &Arc<PlotData>, fft_xy_dif: &Arc<PlotData>) {
        let filter_len = 512;
        let mut filter: AverageFilter<f64> = AverageFilter::new(filter_len);
        let len = fft_xy_len;
        let mut y_dif: f64;
        let mut y: f64;
        let mut i: usize = 0;
        let mut y_prev: f64 = fft_xy.get(i)[1];
        fft_xy_dif.clear();
        fft_xy_dif.push(&[0.0, 0.0]);
        for j in 1..len {
            i = j * 2 - 1;
            y = fft_xy.get(j)[1];
            // yDif = (y - yPrev).abs();
            filter.add((y - y_prev).abs());
            y_dif = filter.value() * 100.0 + 1000000.0;
            y_prev = y;
            fft_xy_dif.push(&[fft_xy.get(j)[0] - (filter_len / 2) as f64, y_dif]);
        }
        // for j in 0..(filterLen / 2) {
        //     i = len - (filterLen / 2) + j;
        //     self.fftXyDif.remove(0);
        //     self.fftXyDif.push([self.fftXy[i][0], yDif]);
        // }
    }
}

///
///
impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let head_hight = 34.0;
        self.events.clear();
        let mut even = false;
        for [freq, ampl] in self.fft_alarm_xy.xy() {
            if even {
                if freq > 0.0 {
                    self.events.push(format!("Частота {:.1} Гц,  амплитуда {:.2} ", freq, ampl))
                }
            }
            even = !even;
        }
        let mut values = vec![];
        while let Ok(val) = self.recv_samples.recv_timeout(Duration::from_millis(1)) {
            values.push(val);
            self.average_amp.add(val);
            self.sps.add();
            // Self::build_envelope(fft_xy_len, fft_xy, envelope_xy);
            // Self::build_fft_xy_dif(fft_xy_len, fft_xy, fft_xy_dif);
        }
        self.xy.enqueue(&values);
        if self.recv_samples.len() >= 512 {
            log::warn!("{}.update | recv_samples.len {} > 512", self.dbg, self.recv_samples.len())
        }
        self.build_fft_xy();
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
                            format!("Sampling:  F: {:?} kHz,  T: {:.2} us", self.fft.freq.load() * 1.0e-3, self.fft.sampling_period.load() * 1.0e6),
                        ),
                    );
                    ui.separator();
                    if ui.add_sized([30., 30.], egui::Button::new(ZOOM_OUT)).clicked() {
                        let len = self.xy.len() + self.xy.len() / 4;
                        if len < self.fft.buf_len {
                            self.xy.set_len(len);
                        } else {
                            self.xy.set_len(self.fft.buf_len);
                        }
                    }
                    ui.add_sized(
                        [100.0, 16.0], 
                        egui::Label::new(format!(" length: {:.4} ns", (self.xy.len() as f64) * self.xy.delta * 1.0e9)),
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
                        log::debug!("{}.update | real input udpLost clicked", self.dbg);
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
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(
                            format!("Amp: {}", self.average_amp),
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
                            egui::Label::new(format!("fftPoints length: {:?}", self.fft_xy.len())),
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
                                self.fft_xy.xy(),
                            ).color(Color32::LIGHT_GREEN),
                        );
                        plot_ui.line(
                            Line::new(
                                "limitationsXy",
                                self.limitations_xy.xy(),
                            ).color(Color32::YELLOW),
                        );
                        let mut even = false;
                        let mut series = vec![];
                        for item in self.fft_alarm_xy.xy() {
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
                                    self.fft_xy_dif.xy()
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
/// 
struct Range {
    pub min: f64,
    pub max: f64,
}
impl Range {
    fn contains(&self, value: f64) -> bool {
        self.min  <= value && value <= self.max
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

///
/// Average Amplitude
pub struct AverageAmp<T> {
    values: CircularQueue<T>,
    average: f64,
}
impl<T: Copy + Sum + Into<f64>> AverageAmp<T> {
    pub fn new() -> Self {
        Self {
            values: CircularQueue::with_capacity(512),
            average: 0.0,
        }
    }
    ///
    /// 
    pub fn add(&mut self, val: T) {
        self.values.push(val);
        let sum: f64 = self.values.buffer().iter().map(|v| Into::<f64>::into(*v)).sum();
        self.average =  sum as f64 / self.values.len() as f64;
    }
    ///
    /// Returns average Amp
    pub fn average(&self) -> f64 {
        self.average
    }
}

impl<T> Debug for AverageAmp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.average)
    }
}
impl<T> Display for AverageAmp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.average)
    }
}