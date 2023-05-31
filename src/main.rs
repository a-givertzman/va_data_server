#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod circular_queue;
mod dsp_filters;
mod fft_analysis;
mod ui_app;
mod interval;
mod udp_server;
mod ds_point;
mod ds;
mod s7;

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
use crate::{
    ui_app::UiApp,
    udp_server::udp_server::UdpServer, 
    fft_analysis::FftAnalysis,
    ds::ds_server::DsServer,
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "debug");
    // env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_BACKTRACE", "full");
    env_logger::init();


    // const N: usize = 32_768;
    // const sampleRate: f32 = 2_048.0000;
    // const PI2f: f64 = (PI2 as f64) * sampleRate;
    // InputSignal::run(inputSignal.clone())?;
    // debug!("[main] InputSignal ready\n");


    // debug!("[main] creating TcpServer...");
    // let tcpSrv = Arc::new(Mutex::new(
    //     TcpServer::new(
    //         "127.0.0.1:5180",
    //         inputSignal.clone(),
    //     )
    // ));
    // debug!("[main] TcpServer created");
    // TcpServer::run(tcpSrv)?;



    debug!("[main] creating DsServer...");
    let mut dsServer = DsServer::new();
    debug!("[main] DsServer created");
    dsServer.run();


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


    debug!("[main] creating FftAnalysis...");
    let fftAnalysis = Arc::new(Mutex::new(
        FftAnalysis::new(
            320_000.0,
            320_000,
            udpSrv.clone().lock().unwrap().receiver.clone(),
            udpSrv.clone(),
            dsServer
        )
    ));
    debug!("[main] FftAnalysis created");
    FftAnalysis::run(fftAnalysis.clone());


    // let analyzeFft = Arc::new(Mutex::new(
    //     AnalizeFft::new(
    //         inputSignal.clone(),
    //         sampleRate, 
    //         N,
    //     )
    // ));
    // AnalizeFft::run(analyzeFft.clone())?;

    let uiApp = UiApp::new(
        udpSrv,
        fftAnalysis,
    );

  
    eframe::run_native(
        "Rpi-FFT-App", 
        eframe::NativeOptions {
            // fullscreen: true,
            // maximized: true,
            initial_window_size: Some(egui::Vec2 { x: 1920.0, y: 840.0 }),
            ..Default::default()
        }, 
        Box::new(|_| Box::new(
            uiApp,
        ))    
    )?;    
    Ok(())
}
