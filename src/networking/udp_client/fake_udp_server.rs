//!
//! Service implements kind of bihavior
//! Basic configuration parameters:
//! ```yaml
//! service FakeUdpServer Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use std::{net::UdpSocket, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self}, time::Duration};
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{Service, ServiceCycle, Services, entity::{Name, Object, Point}}, sync::{Handles, Mutex, channel::Sender}
};
use crate::networking::{InputType, UdpClient};
///
/// 
#[derive(Clone)]
pub struct FakeUdpServerConfig {
    pub name: Name,
    pub addr: String,
    pub channel: u8,
    pub channels: u8,
    /// `Values <u16>` in the DATA field of the single UDP message, not bytes
    pub count: usize,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
    /// Sampling freq, Hz
    pub sampl_freq: usize,
}
///
/// Do something ...
pub struct FakeUdpServer {
    dbg: Dbg,
    name: Name,
    conf: FakeUdpServerConfig,
    #[allow(unused)]
    services: Arc<Services>,
    /// Calcilations of each value, `f64` argument stores current time
    value: Arc<Mutex<Box<dyn FnMut(f64) -> Option<u16> + Send + Sync + 'static>>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl FakeUdpServer {
    //
    /// Crteates new instance of the FakeUdpServer
    /// - `value` - Calcilations of each value, `f64` argument stores current time
    ///     - if returned `Some(val)` - execition continues
    ///     - if returned `None` - execution finished
    pub fn new(conf: FakeUdpServerConfig, services: Arc<Services>, value: impl FnMut(f64) -> Option<u16> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            value: Arc::new(Mutex::new(Box::new(value))),
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl Object for FakeUdpServer {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for FakeUdpServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FakeUdpServer")
            .field("id", &self.dbg)
            .finish()
    }
}
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    Start,
    Exit,
    UdpBindError,
    UdpRecvError,
    UdpSendError,
    WouldBlock,
    TimedOut,
}
//
// 
impl Service for FakeUdpServer {
    //
    // 
    fn get_link(&self, _name: &str) -> Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.name())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let conf = self.conf.clone();
        let period = 1.0 / conf.sampl_freq as f64;
        let mut cycle = ServiceCycle::new(&dbg, Duration::from_secs_f64(period * conf.count as f64));
        let value = self.value.clone();
        let exit = self.exit.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = thread::Builder::new().name(format!("{}.run", dbg.clone())).spawn(move || {
            let dbg = &dbg;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpSendError,   Box::new(|message| log::error!("{}", message))),
                (State::WouldBlock,     Box::new(|message| log::error!("{}", message))),
                (State::TimedOut,       Box::new(|message| log::error!("{}", message))),
            ]);
            let local_addr =  conf.addr;
            'main: loop {
                match UdpSocket::bind(&local_addr) {
                    Ok(socket) => {
                        let mut buf = vec![0; conf.mtu];
                        let mut error_limit = ErrorLimit::new(3);
                        if let Err(err) = socket.set_read_timeout(Some(Duration::from_millis(100))) {
                            log::error!("{dbg}.run | Socket Set timeout error: {:?}", err);
                        }
                        'read: loop {
                            let mut time = 0.0;
                            match socket.recv_from(&mut buf) {
                                Ok((_, src_addr)) => {
                                    error_limit.reset();
                                    match buf.as_slice() {
                                        // Empty message
                                        &[] => log::debug!("{dbg}.run | {}: Empty message received", src_addr),
                                        // Start of communication
                                        &[UdpClient::SYN, UdpClient::EOT] | &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                            log::debug!("{}.run | {dbg}: Start message received", src_addr);
                                            match socket.send_to(&[UdpClient::SYN, UdpClient::EOT], src_addr) {
                                                Ok(_) => log::debug!("{dbg}.run | {}: Start message ACK sent", src_addr),
                                                Err(err) => {
                                                    log::error!("{dbg}.run | Send ACK to {}: error: {:#?}", src_addr, err);
                                                    // notify.add(State::UdpSendError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err))                                                        
                                                },
                                            }
                                            loop {
                                                cycle.start();
                                                let mut buf = vec![UdpClient::DAT, conf.channels, InputType::U16 as u8];
                                                buf.extend(((conf.count) as u32).to_le_bytes());
                                                for _ in 0..conf.count {
                                                    match (value.lock())(time) {
                                                        Some(val) => {
                                                            for ch in 0..conf.channels {
                                                                if conf.channel == ch {
                                                                    buf.extend(val.to_le_bytes());
                                                                } else {
                                                                    buf.extend(0_u16.to_le_bytes());
                                                                }
                                                            }
                                                            time += period;
                                                        }
                                                        None => break 'main,
                                                    }
                                                }
                                                log::trace!("{dbg}.run | buf: \n\t{:?}", buf);
                                                match socket.send_to(&buf, src_addr) {
                                                    Ok(sent_len) => {
                                                        log::trace!("{dbg}.run | Sent to {}: data ({}): \n\t{:?}", src_addr, sent_len, buf);
                                                    }
                                                    Err(err) => {
                                                        // log::error!("{dbg}.run | Send to {}: error: {:#?}", src_addr, err);
                                                        notify.add(State::UdpSendError, format!("{dbg}.run | Socket send error: {:#?}", err))                                                        
                                                    }
                                                }
                                                if !cycle.interval().is_zero() {
                                                    cycle.wait();
                                                }
                                                if exit.load(Ordering::Acquire) {
                                                    break 'main;
                                                }
                                            }
                                        }
                                        _ => log::warn!("{dbg}.run | {}: Unknown message format: {:?}...", src_addr, &buf[..=10]),
                                    }
                                }
                                Err(err) => {
                                    // notify.add(State::UdpRecvError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err)),
                                    match err.kind() {
                                        std::io::ErrorKind::WouldBlock => {
                                            notify.add(State::WouldBlock, format!("{dbg}.run | Socket read timeout"));
                                        },
                                        std::io::ErrorKind::TimedOut => {
                                            notify.add(State::TimedOut, format!("{dbg}.run | Socket read timeout"));
                                        }
                                        _ => {
                                            log::debug!("{dbg}.run | Read error: {:#?}", err);
                                            if error_limit.add().is_err() {
                                                log::error!("{dbg}.run | Socket read errors limit exceeded, trying to reconnect...");
                                                break 'read;
                                            }
                                        },
                                    }
                                }
                            }
                            if exit.load(Ordering::SeqCst) {
                                break 'main;
                            }
                        }
                    }
                    Err(err) => notify.add(State::UdpBindError, format!("{dbg}.run | UdpSocket::bind error: {:#?}", err)),
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }    
}