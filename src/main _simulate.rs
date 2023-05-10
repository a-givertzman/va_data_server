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
use analyze_fft::AnalizeFft;
use input_signal::InputSignal;
use ui_app::UiApp;
use crate::{
    udp_server::UdpServer,
    tcp_server::TcpServer,
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();


    const N: usize = 32_768;
    const sampleRate: f32 = 2_048.0000;
    // const PI2f: f64 = (PI2 as f64) * sampleRate;
    let inputSignal = Arc::new(Mutex::new(
        InputSignal::new(
            sampleRate, 
            |phi| {
                // debug!("build input signal in thread: {:?}", thread::current().name().unwrap());
                10.0 * (phi * 10.0).sin()
                + 10.005 * (phi * 50.0).sin()
                + 10.006 * (phi * 100.0).sin()
                + 10.007 * (phi * 500.0).sin()
                + 10.10 * (phi * 1000.0).sin()
                + 10.50 * (phi * 5000.0).sin()
                + 10.60 * (phi * 6000.0).sin()
                + 10.70 * (phi * 7000.0).sin()
                + 10.80 * (phi * 8000.0).sin()
                + 10.90 * (phi * 9000.0).sin()
                + 11.00 * (phi * 10000.0).sin()
                + 11.00 * (phi * 11000.0).sin()
                + 12.00 * (phi * 12000.0).sin()
                + 13.00 * (phi * 13000.0).sin()
                + 14.00 * (phi * 14000.0).sin()
                + 15.00 * (phi * 15000.0).sin()
                + 16.00 * (phi * 16000.0).sin()
                + 17.00 * (phi * 17000.0).sin()
                + 18.00 * (phi * 18000.0).sin()
                + 19.00 * (phi * 19000.0).sin()
                + 30.00 * (phi * 30000.0).sin()
                + 35.00 * (phi * 35000.0).sin()
                + 40.00 * (phi * 40000.0).sin()
            },
            N,
            None, // Some(0.0001),
            )
    ));
    // InputSignal::run(inputSignal.clone())?;
    debug!("[main] InputSignal ready\n");


    debug!("[main] creating TcpServer...");
    let tcpSrv = Arc::new(Mutex::new(
        TcpServer::new(
            "127.0.0.1:5180",
            inputSignal.clone(),
        )
    ));
    debug!("[main] TcpServer created");
    TcpServer::run(tcpSrv)?;


    let reconnectDelay = Duration::from_secs(3);
    let localAddr = "192.168.120.172:15180";
    let remoteAddr = "192.168.120.173:15180";
    debug!("[main] creating UdpServer...");
    let udpSrv = Arc::new(Mutex::new(
        UdpServer::new(
            localAddr,
            remoteAddr,
            48771.0,
            48771,
            Some(reconnectDelay),
        )
    ));
    debug!("[main] UdpServer created");
    UdpServer::run(udpSrv.clone());


    let analyzeFft = Arc::new(Mutex::new(
        AnalizeFft::new(
            inputSignal.clone(),
            sampleRate, 
            N,
        )
    ));
    // AnalizeFft::run(analyzeFft.clone())?;

    let uiApp = UiApp::new(
        inputSignal,
        analyzeFft,
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
