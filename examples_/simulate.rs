#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[path = "../src/circular_queue.rs"]
mod circular_queue;
#[path = "../src/input_signal.rs"]
mod input_signal;
#[path = "../src/dsp_filters/mod.rs"]
mod dsp_filters;
#[path = "../src/fft/mod.rs"]
mod fft;
#[path = "../src/ui_app.rs"]
mod ui_app;
#[path = "../src/interval.rs"]
mod interval;
#[path = "../src/tcp_server.rs"]
mod tcp_server;
#[path = "../src/udp_server.rs"]
mod udp_server;
// #[path = "../src/ds_point.rs"]
// mod ds_point;
#[path = "../src/ds/mod.rs"]
mod ds;

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
use input_signal::InputSignal;
use ui_app::UiApp;
use crate::{
    udp_server::UdpServer,
    tcp_server::TcpServer, 
    fft_analyze::FftAnalysis, ds::ds_server::DsServer,
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
            Some(reconnectDelay),
        )
    ));
    debug!("[main] UdpServer created");
    UdpServer::run(udpSrv.clone());


    let fftAnalysis = Arc::new(Mutex::new(
        FftAnalysis::new(
            320_000.0,
            320_000,
            udpSrv.clone().lock().unwrap().receiver.clone(),
            udpSrv.clone(), 
            DsServer::new(),
        )
    ));
    // AnalizeFft::run(analyzeFft.clone())?;

    // let uiApp = ;
    env::set_var("RUST_BACKTRACE", "full");
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1024.0, y: 768.0 }),
        ..Default::default()
    };    
    eframe::run_native(
        "Rpi-FFT-App", 
        native_options, 
        Box::new(|cc| Box::new(
            UiApp::new(
                cc,
                // inputSignal,
                udpSrv,
                fftAnalysis,
                // Duration::from_secs_f64(10.0/60.0),
            ),
        ))    
    )?;    
    Ok(())
}
