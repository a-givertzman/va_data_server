#![allow(non_snake_case)]

mod input_signal;
mod analyze_fft;
mod ui_app;
mod interval;
mod circular_queue;

// use log::{
    // info,
    // trace,
    // debug,
    // warn,
// };
use std::{
    env,
    error::Error, 
    sync::{
        Arc,
        Mutex,
    }, 
};


use analyze_fft::{AnalizeFft, PI2};
// use egui::mutex::Mutex;
use input_signal::InputSignal;
use ui_app::UiApp;

/// 1_024,   // up to 0.500 KHz, mach more noise 
/// 2_048,   // up to 1 KHz
/// 4_096,   // up to 1 KHz
/// 8_192,   // up to 1 KHz
/// 16_384,  // up to 5 KHz
/// 32_768,  // up to 15 KHz
/// 65_536,  // up to 30 KHz
fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    const N: usize = 16_384;
    const fIn: f32 = 1000.0000;
    const PI2f: f32 = PI2 * fIn;
    let inputSignal = Arc::new(Mutex::new(
        InputSignal::new(
            fIn, 
            |t| {
                // debug!("build input signal in thread: {:?}", thread::current().name().unwrap());
                0.7 * (PI2f * t * 100.0).sin()
                // + 10.05 * (PI2f * t * 500.0).sin()
                // + 10.10 * (PI2f * t * 1000.0).sin()
                // + 10.50 * (PI2f * t * 5000.0).sin()
                // + 10.60 * (PI2f * t * 6000.0).sin()
                // + 10.70 * (PI2f * t * 7000.0).sin()
                + 10.80 * (PI2f * t * 8000.0).sin()
                // + 0.90 * (PI2 * f * t * 9000.0).sin()
                // + 1.00 * (PI2 * f * t * 10000.0).sin()
                // + 1.10 * (PI2 * f * t * 11000.0).sin()
                // + 1.20 * (PI2 * f * t * 12000.0).sin()
                // + 1.10 * (PI2 * f * t * 10000.0).sin()
                // + 1.15 * (PI2 * f * t * 15000.0).sin()
                // + 1.20 * (PI2 * f * t * 20000.0).sin()
                // + 1.25 * (PI2 * f * t * 25000.0).sin()
                // + 1.30 * (PI2 * f * t * 30000.0).sin()
                // + 1.35 * (PI2 * f * t * 35000.0).sin()
                // + 1.40 * (PI2 * f * t * 40000.0).sin()
            },
            N,
            None, // Some(0.0001),
            )
    ));
    InputSignal::run(inputSignal.clone())?;


    const fIn1: f32 = 100.0000;
    // const PI2f1: f32 = PI2 * fIn1;

    let analyzeFft = Arc::new(Mutex::new(
        AnalizeFft::new(
            inputSignal.clone(),
            fIn1, 
            16_384,  // up to 5 KHz
        )
    ));
    AnalizeFft::run(analyzeFft.clone())?;

    let uiApp = UiApp::new(
        inputSignal,
        analyzeFft,
    );

    env::set_var("RUST_BACKTRACE", "full");
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1024.0, y: 768.0 }),
        ..Default::default()
    };    
    eframe::run_native(
        "Rpi-FFT-App", 
        native_options, 
        Box::new(|_| Box::new(
            uiApp,
        ))    
    )?;    

    Ok(())
}
