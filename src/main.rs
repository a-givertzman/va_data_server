mod circular_queue;
mod dsp_filters;
mod presentation;
mod interval;
mod networking;
mod fft;
mod s7;
mod ds;

#[cfg(not(feature = "plot"))]
use eframe::{EventLoopBuilder, UserEvent};
use log::{
    // info,
    // trace,
    debug,
    // warn,
};
use sal_core::dbg::Dbg;
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}}, thread_pool::ThreadPool};
use std::{
    error::Error, 
    sync::Arc,
    time::Duration, 
};
use crate::{
    ds::DsServer, fft::FftAnalysis, networking::{UdpClient, UdpClientConf}, presentation::ui_app::UiApp
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    let dbg = Dbg::own("main");

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

    let tp = ThreadPool::new(&dbg, Some(8));
    let services = Arc::new(Services::new(&dbg, ServicesConf::new(
        &dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ), Some(tp.scheduler())));

    log::debug!("[main] configuring UdpClient...");
    let path = "./udp-client.yaml";
    let conf = UdpClientConf::read(&dbg, path);
    let udp_client = Arc::new(UdpClient::new(conf, services.clone(), tp.scheduler()));
    services.insert(udp_client.clone());

    log::debug!("[main] creating FftAnalysis...");
    let fft_analysis = Arc::new(FftAnalysis::new(
        &dbg,
        320_000.0,
        320_000,
        udp_client.clone(),
        ds_server,
        services.clone(),
    ));
    log::debug!("[main] FftAnalysis created");
    fft_analysis.run()?;
    services.insert(fft_analysis.clone());

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
                udp_client,
                fft_analysis,
            ),
        )))    
    )?;    
    Ok(())
}
