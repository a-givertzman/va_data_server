#![allow(non_snake_case)]

use std::{error::Error, thread::JoinHandle, sync::mpsc, env};

use egui::{plot::{Plot, Points, PlotPoints}, TextBuffer};

// use std::{thread, time::Instant};
use rustfft::{FftPlanner, num_complex::{Complex, ComplexFloat}};

pub const PI: f64 = std::f64::consts::PI;
pub const PI2: f64 = PI * 2.0;

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "full");
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1024.0, y: 768.0 }),
        ..Default::default()
    };
    eframe::run_native(
        "Rpi-FFT-App", 
        native_options, 
        Box::new(|_| Box::new(App::new()))
    )?;
    Ok(())
}


/// State per thread.
struct ThreadState {
    thread_nr: usize,
    title: String,
    name: String,
    age: u32,
}

impl ThreadState {
    ///
    fn new(thread_nr: usize) -> Self {
        let title = format!("Background thread {thread_nr}");
        Self {
            thread_nr,
            title,
            name: "Arthur".into(),
            age: 12 + thread_nr as u32 * 10,
        }
    }
    ///
    fn show(&mut self, ctx: &egui::Context) {
        let pos = egui::pos2(16.0, 128.0 * (self.thread_nr as f32 + 1.0));
        egui::Window::new(&self.title)
            .default_pos(pos)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_multiline(&mut self.name)
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    self.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
    }
}

fn new_worker(
    thread_nr: usize,
    on_done_tx: mpsc::SyncSender<String>,
) -> (JoinHandle<()>, mpsc::SyncSender<egui::Context>) {
    let (show_tx, show_rc) = mpsc::sync_channel(0);
    let handle = std::thread::Builder::new()
        .name(format!("EguiPanelWorker {}", thread_nr))
        .spawn(move || {
            let mut state = ThreadState::new(thread_nr);
            while let Ok(ctx) = show_rc.recv() {
                state.show(&ctx);
                println!("EguiPanelWorker {}: state.show done", thread_nr);
                let _ = on_done_tx.send(format!("EguiPanelWorker {}", thread_nr));
            }
        })
        .expect("failed to spawn thread");
    (handle, show_tx)
}

struct App {
    threads: Vec<(JoinHandle<()>, mpsc::SyncSender<egui::Context>)>,
    on_done_tx: mpsc::SyncSender<String>,
    on_done_rc: mpsc::Receiver<String>,
}

impl App {
    fn new() -> Self {
    // fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let threads = Vec::with_capacity(3);
        let (on_done_tx, on_done_rc) = mpsc::sync_channel(0);
        let mut slf = Self {
            threads,
            on_done_tx,
            on_done_rc,
        };
        slf
    }
    fn spawn_thread(&mut self) {
        let threadNr = self.threads.len();
        self.threads.push(
            new_worker(
                threadNr,
                self.on_done_tx.clone()
            )
        );
    }
}

impl std::ops::Drop for App {
    fn drop(&mut self) {
        for (handle, show_tx) in self.threads.drain(..) {
            std::mem::drop(show_tx);
            handle.join().unwrap();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Main thread").show(ctx, |ui| {
            ui.label(format!("Current threads: '{}'", self.threads.len()));
            if ui.button("Spawn another thread").clicked() {
                self.spawn_thread();
            }
        });

        for (_handle, show_tx) in &self.threads {
            let _ = show_tx.send(ctx.clone());
        }

        for _ in 0..self.threads.len() {
            let received = self.on_done_rc.recv();
            println!("received: {:?}", received);
        }
    }
}


