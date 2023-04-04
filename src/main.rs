#![allow(non_snake_case)]

mod ui_app;
mod sin_buf;
mod input_signal;

use std::{
    env,
    thread,
    error::Error, 
};


use input_signal::{InputSignal, PI2};
use ui_app::UiApp;


fn main() -> Result<(), Box<dyn Error>> {
    let uiApp = UiApp::new(
        InputSignal::new(
            0.1000, 
            |t, f| {
                // println!("build input signal in thread: {:?}", thread::current().name().unwrap());
                1.0 * (PI2 * f * t * 100.0).sin()
                + 0.5 * (PI2 * f * t * 500.0).sin()
                + 0.7 * (PI2 * f * t * 1000.0).sin()
                + 0.10 * (PI2 * f * t * 10000.0).sin()
                + 0.15 * (PI2 * f * t * 15000.0).sin()
                + 0.20 * (PI2 * f * t * 20000.0).sin()
            },
            32768,
            None, // Some(0.0001),
        ),
    );

    let inputSig = uiApp.inputSig.clone();
    thread::Builder::new().name("ffi process".to_string()).spawn(move || {
        loop {
            inputSig.lock().next();
            inputSig.lock().fftProcess();
        }
    })?;

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