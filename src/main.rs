#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod circular_queue;
mod input_signal;
mod dsp_filters;
mod analyze_fft;
mod ui_app;
mod interval;
mod tcp_server;
mod udp_server;
mod ds_point;

use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
    env,
    error::Error, 
    sync::{
        Arc,
        Mutex,
    }, 
    time::Duration, 
};
// use analyze_fft::AnalizeFft;
// use input_signal::InputSignal;
use ui_app::UiApp;
use crate::{
    udp_server::UdpServer,
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();


    // const N: usize = 32_768;
    // const sampleRate: f32 = 2_048.0000;
    // const PI2f: f64 = (PI2 as f64) * sampleRate;
    // InputSignal::run(inputSignal.clone())?;
    debug!("[main] InputSignal ready\n");


    // debug!("[main] creating TcpServer...");
    // let tcpSrv = Arc::new(Mutex::new(
    //     TcpServer::new(
    //         "127.0.0.1:5180",
    //         inputSignal.clone(),
    //     )
    // ));
    // debug!("[main] TcpServer created");
    // TcpServer::run(tcpSrv)?;


    let reconnectDelay = Duration::from_secs(3);
    let localAddr = "192.168.120.172:15180";
    let remoteAddr = "192.168.120.173:15180";
    debug!("[main] creating UdpServer...");
    let udpSrv = Arc::new(Mutex::new(
        UdpServer::new(
            localAddr,
            remoteAddr,
            155_339.0,
            155_339,
            Some(reconnectDelay),
        )
    ));
    debug!("[main] UdpServer created");
    UdpServer::run(udpSrv.clone());


    // let analyzeFft = Arc::new(Mutex::new(
    //     AnalizeFft::new(
    //         inputSignal.clone(),
    //         sampleRate, 
    //         N,
    //     )
    // ));
    // AnalizeFft::run(analyzeFft.clone())?;

    let uiApp = UiApp::new(
        // inputSignal,
        // analyzeFft,
        udpSrv,
        Duration::from_secs_f64(10.0/60.0),
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
