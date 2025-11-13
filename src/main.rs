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
use sal_core::dbg::Dbg;
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}}, thread_pool::ThreadPool};
use tracing_subscriber::{filter::{LevelFilter, Targets}, layer::SubscriberExt, util::SubscriberInitExt};
use std::{
    error::Error, 
    sync::Arc,
};
use crate::{
    ds::DsServer, fft::FftAnalysis, networking::{UdpClient, UdpClientConf}, presentation::ui_app::UiApp
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    let filter = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_target("winit", LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    let dbg = Dbg::own("main");

    // const N: usize = 32_768;
    // const sampleRate: f32 = 2_048.0000;
    // const PI2f: f64 = (PI2 as f64) * sampleRate;
    // InputSignal::run(inputSignal.clone())?;
    // debug!("[main] InputSignal ready\n");

    log::debug!("[main] creating DsServer...");
    let mut ds_server = DsServer::new();
    log::debug!("[main] DsServer created");
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
    services.run()?;

    log::debug!("[main] configuring UdpClient...");
    let path = "./udp-client.yaml";
    let conf = UdpClientConf::read(&dbg, path);
    let udp_client = Arc::new(UdpClient::new(conf, services.clone(), tp.scheduler()));
    services.insert(udp_client.clone());

    log::debug!("[main] configuring FftAnalysis...");
    let fft_analysis = Arc::new(FftAnalysis::new(
        &dbg,
        320_000.0,
        320_000,
        udp_client.clone(),
        ds_server,
        services.clone(),
    ));
    fft_analysis.run()?;
    services.insert(fft_analysis.clone());

    udp_client.run()?;

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
