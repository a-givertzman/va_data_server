mod circular_queue;
mod dsp_filters;
mod presentation;
mod interval;
mod networking;
mod fft;
mod s7;
mod ds;

use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use std::{
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
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();


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
    let mut ds_server = DsServer::new();
    debug!("[main] DsServer created");
    ds_server.run();


    let reconnect_delay = Duration::from_secs(3);
    let local_addr = "192.168.100.172:15180";
    let remote_addr = "192.168.100.173:15180";
    debug!("[main] creating UdpServer...");
    let udp_srv = Arc::new(Mutex::new(
        UdpServer::new(
            local_addr,
            remote_addr,
            Some(reconnect_delay),
        )
    ));
    debug!("[main] UdpServer created");
    UdpServer::run(udp_srv.clone());


    debug!("[main] creating FftAnalysis...");
    let fft_analysis = Arc::new(Mutex::new(
        FftAnalysis::new(
            320_000.0,
            320_000,
            udp_srv.clone().lock().unwrap().receiver.clone(),
            udp_srv.clone(),
            ds_server
        )
    ));
    debug!("[main] FftAnalysis created");
    FftAnalysis::run(fft_analysis.clone());


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
            // fullscreen: true,
            // maximized: true,
            viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 840.0]),
            ..Default::default()
        }, 
        Box::new(|cc| Ok(Box::new(
            UiApp::new(
                cc,
                udp_srv,
                fft_analysis,
            ),
        )))    
    )?;    
    Ok(())
}
