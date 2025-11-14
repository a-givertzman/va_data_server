mod circular_queue;
mod dsp_filters;
mod presentation;
mod interval;
mod networking;
mod fft;
mod s7;
mod ds;
#[cfg(test)]
mod tests;

// #[cfg(not(feature = "plot"))]
// use eframe::{EventLoopBuilder, UserEvent};
use sal_core::dbg::Dbg;
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}, entity::Name}, thread_pool::ThreadPool};
use tracing_subscriber::{filter::{LevelFilter, Targets}, layer::SubscriberExt, util::SubscriberInitExt};
use std::{error::Error, f64::consts::PI, sync::Arc};
use crate::{
    ds::DsServer, fft::FftAnalysis, networking::{FakeUdpServer, FakeUdpServerConfig, UdpClient, UdpClientConf}, presentation::ui_app::UiApp
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
    //
    // ======================== Configure input signal here ========================
    let freq = 4_000;                     // Frequency of the test signal, Hz
    let amp = 2400.0;                       // Amplitude of the test signal
    let ω = 2.0 * PI * freq as f64;   // Angular frequency of the test signal, rad/s
    // ===================== Configure Sampling & FFT analizer =====================
    let sampl_freq = 320_000;               // Sampling Frequency of the ADC, Hz
    let fft_buflen = 320_000;             // FFT calculation window
    let udp_len = 512;                    // Values <u16> in the DATA field of the single UDP message, not bytes
    // =============================================================================
    log::info!("{dbg} | Test signal:");
    log::info!("{dbg} |             Frequency: {} Hz", freq);
    log::info!("{dbg} |     Angular frequency: {} rad/sec", ω);
    log::info!("{dbg} |             Amplitude: {}", amp);
    log::info!("{dbg} | ------------------------------------");
    log::info!("{dbg} | Sampling and FFT:");
    log::info!("{dbg} |    Sampling Frequency: {} Hz", sampl_freq);
    log::info!("{dbg} |    UDP Buf length: {} values of U16", fft_buflen);
    log::info!("{dbg} | ------------------------------------");
    log::debug!("{dbg} | Configuring DsServer...");
    let ds_server = DsServer::new();
    // ds_server.run();

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

    log::debug!("{dbg} configuring UdpClient...");
    let path = "./udp-client.yaml";
    let conf = UdpClientConf::read(&dbg, path);
    let udp_client = Arc::new(UdpClient::new(conf, services.clone(), tp.scheduler()));
    services.insert(udp_client.clone());

    log::debug!("{dbg} configuring FftAnalysis...");
    let fft_analysis = Arc::new(FftAnalysis::new(
        &dbg,
        sampl_freq as f32,
        fft_buflen,
        udp_client.clone(),
        ds_server,
    ));
    fft_analysis.run()?;
    services.insert(fft_analysis.clone());

    udp_client.run()?;

    let fake_udp_server = FakeUdpServer::new(
        FakeUdpServerConfig {
            name: Name::new(dbg, "FakeUdpServer"),
            addr: "127.0.0.1:15180".to_owned(),
            channel: 0,
            count: udp_len,
            mtu: 1500,
            sampl_freq,
        },
        services.clone(),
        move |time| {
            let val = ((ω * time).sin() + 1.1) * amp;
            // log::debug!("main.run | t: {},  val: {}", time, val);
            Some(val.round() as u16)
        }
    );
    fake_udp_server.run()?;

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
