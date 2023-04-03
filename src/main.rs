#![allow(non_snake_case)]

mod ui_app;
mod sin_buf;
mod input_signal;

use std::{
    env,
    error::Error,

};


use input_signal::InputSignal;
use ui_app::UiApp;


fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "full");
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1024.0, y: 768.0 }),
        ..Default::default()
    };
    eframe::run_native(
        "Rpi-FFT-App", 
        native_options, 
        Box::new(|_| Box::new(
            UiApp::new(
                InputSignal::new(
                    100.0, 
                    |t| {
                        t.sin() 
                        + 0.2 * (t * 10.0).sin() 
                        + 0.2 * (t * 5.0).sin()
                        + 0.2 * (t * 500.0).sin()
                    },
                    2000,
                    Some(0.01),
                ),
                10000,
            ),
        ))
    )?;
    Ok(())
}