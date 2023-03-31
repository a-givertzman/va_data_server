#![allow(non_snake_case)]

mod ui_app;
mod sin_buf;

use std::{
    env,
    error::Error,

};


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
            UiApp::new(10000),
        ))
    )?;
    Ok(())
}