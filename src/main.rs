#![allow(non_snake_case)]

mod ui_app;
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
                0.7 * (PI2 * f * t * 10.0).sin()
                // + 1.005 * (PI2 * f * t * 500.0).sin()
                // + 1.01 * (PI2 * f * t * 1000.0).sin()
                // + 1.05 * (PI2 * f * t * 5000.0).sin()
                // + 1.10 * (PI2 * f * t * 10000.0).sin()
                // + 1.15 * (PI2 * f * t * 15000.0).sin()
                // + 1.20 * (PI2 * f * t * 20000.0).sin()
                // + 1.25 * (PI2 * f * t * 25000.0).sin()
                // + 1.30 * (PI2 * f * t * 30000.0).sin()
                // + 1.35 * (PI2 * f * t * 35000.0).sin()
                // + 1.40 * (PI2 * f * t * 40000.0).sin()
            },
            // 1_024,    // up to 0.500 KHz, mach more noise 
            // 2_048,    // up to 1 KHz
            // 4_096,    // up to 1 KHz
            // 8_192,   // up to 1 KHz
            // 16_384,  // up to 5 KHz
            32_768,  // up to 15 KHz
            // 65_536, // up to 30 KHz
            None, // Some(0.0001),
        ),
    );
    let inputSignal = uiApp.inputSig.clone();
    thread::Builder::new().name("ffi process".to_string()).spawn(move || {
        loop {
            inputSignal.lock().next();
            inputSignal.lock().fftProcess();
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
