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

use debugging::session::debug_session::{DebugSession, LogLevel};
// #[cfg(not(feature = "plot"))]
// use eframe::{EventLoopBuilder, UserEvent};
use sal_core::dbg::Dbg;
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}, entity::Name}, thread_pool::ThreadPool};
use std::{error::Error, f64::consts::PI, sync::Arc};
use crate::{
    ds::DsServer, fft::FftAnalysis, networking::{FakeUdpServer, FakeUdpServerConfig, UdpClient, UdpClientConf}, presentation::ui_app::UiApp
};

///
/// 
fn main() -> Result<(), Box<dyn Error>> {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("eframe", LogLevel::Info)
        .module("winit", LogLevel::Info)
        .module("sal_sync::thread_pool", LogLevel::Info)
        .init();
    let dbg = Dbg::own("main");
    //
    // ======================== Configure input signal here ========================
    // Frequency of the test signal, Hz
    let freq = [1024 / 2, 4096, 8192, 16384, 20000, 32769];
    // Amplitude of the test signal
    let amp = [2048.0, 2048.0, 2048.0, 2048.0, 2048.0, 2048.0];
    // Angular frequency of the test signal, rad/s
    let ω_amp: Vec<(f64, f64)> = freq.iter().enumerate().map(|(i, f)| (2.0 * PI * *f as f64, amp[i] * 0.5)).collect();
    // ===================== Configure Sampling & FFT analizer =====================
    let sampl_freq = 524_288;   // 262_144;     131_072;    65_536;    666_624;             // Sampling Frequency of the ADC, Hz
    let fft_buflen = 524_288;   // 262_144;     131_072;    65_536;    666_624;             // FFT calculation window
    // =============================================================================
    log::info!("{dbg} | Test signal:");
    for (i, f) in freq.iter().enumerate() {
        log::info!("{dbg} |             Frequency[{i}]: {} Hz", f);
        log::info!("{dbg} |     Angular frequency[{i}]: {} rad/sec", ω_amp[i].0);
        log::info!("{dbg} |             Amplitude[{i}]: {}", amp[i]);
    }
    log::info!("{dbg} | ------------------------------------");
    log::info!("{dbg} | Sampling and FFT:");
    log::info!("{dbg} |    Sampling Frequency: {} Hz", sampl_freq);
    log::info!("{dbg} |    Buf length: {} values of U16", fft_buflen);
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
            addr: "127.0.0.1:15181".to_owned(),
            channel: 1,
            channels: 2,
            count: 512,
            mtu: 1500,
            sampl_freq,
        },
        services.clone(),
        move |time| {
            let val = ω_amp.iter().fold(0.0, |acc, (ω, amp)| {
                acc + ((ω * time).sin() + 1.0) * amp
            });
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
