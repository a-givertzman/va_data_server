mod circular_queue;
mod dsp_filters;
mod presentation;
mod interval;
mod networking;
mod fft;
mod profinet;
mod ds;

use egui::viewport;
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
    presentation::ui_app::UiApp,
    networking::udp_server::UdpServer, 
    fft::fft_analysis::FftAnalysis,
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
    // log::debug!("[main] InputSignal ready\n");


    // log::debug!("[main] creating TcpServer...");
    // let tcpSrv = Arc::new(Mutex::new(
    //     TcpServer::new(
    //         "127.0.0.1:5180",
    //         inputSignal.clone(),
    //     )
    // ));
    // log::debug!("[main] TcpServer created");
    // TcpServer::run(tcpSrv)?;



    log::debug!("[main] creating DsServer...");
    let mut ds_server = DsServer::new();
    log::debug!("[main] DsServer created");
    ds_server.run();


    let reconnect_delay = Duration::from_secs(3);
    let local_addr = "192.168.100.172:15180";
    let remote_addr = "192.168.100.173:15180";
    log::debug!("[main] creating UdpServer...");
    let udp_srv = Arc::new(Mutex::new(
        UdpServer::new(
            local_addr,
            remote_addr,
            Some(reconnect_delay),
        )
    ));
    log::debug!("[main] UdpServer created");
    UdpServer::run(udp_srv.clone());


    log::debug!("[main] creating FftAnalysis...");
    let fftAnalysis = Arc::new(Mutex::new(
        FftAnalysis::new(
            320_000.0,
            320_000,
            udp_srv.clone().lock().unwrap().receiver.clone(),
            udp_srv.clone(),
            ds_server
        )
    ));
    log::debug!("[main] FftAnalysis created");
    FftAnalysis::run(fftAnalysis.clone());


    // let analyzeFft = Arc::new(Mutex::new(
    //     AnalizeFft::new(
    //         inputSignal.clone(),
    //         sampleRate, 
    //         N,
    //     )
    // ));
    // AnalizeFft::run(analyzeFft.clone())?;

    eframe::run_native(
        "Rpi-FFT-App", 
        eframe::NativeOptions {
            viewport: viewport::ViewportBuilder::default().with_inner_size([1920.0, 840.0]),
            // fullscreen: true,
            // maximized: true,
            // initial_window_size: Some(egui::Vec2 { x: 1920.0, y: 840.0 }),
            ..Default::default()
        }, 
        Box::new(|cc| Ok(Box::new(
            UiApp::new(
                cc,
                udp_srv,
                fftAnalysis,
            ),
        ))),
    )?;
    Ok(())
}
