#![allow(non_snake_case)]

use log::{
    info,
    // trace,
    debug,
    warn,
};
use num::Complex;
use std::{
    net::UdpSocket, 
    time::Duration, 
    sync::{Arc, Mutex}, thread,
};

use crate::{circular_queue::CircularQueue, input_signal::PI2};
// T, uc	QSIZE
// 976.563	1 024
// 488.281	2 048
// 244.141	4 096
// 122.070	8 192
// 61.035	16 384
// 30.518	32 768
// 15.259	65 536
// 7.629	131 072
// 3.815	262 144
// 1.907	524 288

const SYN: u8 = 22;
const EOT: u8 = 4;
const QSIZE: usize = 512;
const QSIZE_DOUBLE: usize = QSIZE * 2;

pub struct UdpServer {
    localAddr: String, //SocketAddr,
    remoteAddr: String, //SocketAddr,
    reconnectDelay: Duration,
    pub isConnected: bool,
    cancel: bool,
    delta: f64,
    pub f: f32,
    pub t: f64,
    pub complex0: Vec<Complex<f64>>,
    pub complex: CircularQueue<Complex<f64>>,
    pub xy: CircularQueue<[f64; 2]>,
}

impl UdpServer {
    ///
    pub fn new(
        localAddr: &str,
        remoteAddr: &str,
        f: f32,
        reconnectDelay: Option<Duration>,
    ) -> Self {
        let period = 1.0 / (f as f64);
        let delta = period / (QSIZE as f64);
        let iToNList: Vec<f64> = (0..QSIZE).into_iter().map(|i| {(i as f64) / (QSIZE as f64)}).collect();
        let phiList: Vec<f64> = iToNList.clone().into_iter().map(|iToN| {PI2 * iToN}).collect();        
        let complex0: Vec<Complex<f64>> = (0..QSIZE).into_iter().map(|i| {
            Complex {
                re: phiList[i].cos(), 
                im: phiList[i].sin()
            }
        }).collect();
        let delta = 1.0 / (f as f64);
        Self {
            localAddr: String::from(localAddr),
            remoteAddr: String::from(remoteAddr),
            reconnectDelay: match reconnectDelay {Some(rd) => rd, None => Duration::from_secs(3)},
            isConnected: false,
            cancel: false,
            delta,
            f,
            t: 0.0,
            complex0,
            complex: CircularQueue::with_capacity_fill(QSIZE, &mut vec![Complex{re: 0.0, im: 0.0}; QSIZE]),
            xy: CircularQueue::with_capacity_fill(QSIZE, &mut vec![[0.0, 0.0]; QSIZE]),
        }
    }
    ///
    pub fn run(this: Arc<Mutex<Self>>) -> () {
        const logLoc: &str = "[UdpServer.run]";
    // pub fn run(this: Arc<Mutex<Self>>) -> Result<(), Box<dyn Error>> {
        debug!("{} starting...", logLoc);
        info!("{} enter", logLoc);
        thread::Builder::new().name("UdpServer tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", logLoc, thread::current().name().unwrap());
            // me.lock().unwrap().listenStream(&mut stream);
            let thisClone = this.clone();
            let mut thisMutax = thisClone.lock().unwrap();
            let cancel = thisMutax.cancel;
            let localAddr = &thisMutax.localAddr.clone();
            let remoteAddr = &thisMutax.remoteAddr.clone();
            let reconnectDelay = thisMutax.reconnectDelay;
            
            info!("{} started", logLoc);
            while !cancel {
                info!("{} try to bind on: {:?}", logLoc, localAddr);
                match UdpSocket::bind(localAddr) {
                    Ok(socket) => {
                        info!("{} ready on: {:?}\n", logLoc, localAddr);
                        thisMutax.isConnected = true;
                        info!("{} isConnected: {:?}\n", logLoc, thisMutax.isConnected);
                        let mut bufDouble = [0; QSIZE_DOUBLE];
                        let mut buf = [0; QSIZE];
                        let handshake = Self::handshake();
                        info!("{} sending handshake({}): {:?}\n", logLoc, handshake.len(), handshake);
                        match socket.send_to(&handshake, remoteAddr) {
                            Ok(_) => {},
                            Err(err) => {
                                warn!("{} send error: {:#?}", logLoc, err);
                            },
                        };
                        loop {
                            match socket.recv_from(&mut bufDouble) {
                                Ok((amt, src)) => {
                                    // debug!("{} receaved bytes({}) from{:?}: {:?}", logLoc, amt, src, buf);
                                    this.lock().unwrap().enqueue(&bufDouble);

                                    debug!("{} receaved bytes({}) from{:?}: {:?}", logLoc, amt, src, buf);
                                    buf.fill(0);
                                    bufDouble.fill(0)
                                },
                                Err(err) => {
                                    warn!("{} read error: {:#?}", logLoc, err);
                                },
                            };
                        }
                    }
                    Err(err) => {
                        thisMutax.isConnected = false;
                        debug!("{} binding error on: {:?}\n\tdetailes: {:?}", logLoc, localAddr, err);
                        std::thread::sleep(reconnectDelay);
                    }
                }
                std::thread::sleep(reconnectDelay);
            }
            info!("{} exit", logLoc);
        }).unwrap();
        debug!("{} started\n", logLoc);
    }
    ///
    fn enqueue(&mut self, buf: &[u8; QSIZE_DOUBLE]) {
                                    
        let mut value;
        let mut bytes = [0_u8; 2];
        for i in 0..QSIZE {
            bytes[0] = buf[i * 2];
            bytes[1] = buf[i * 2 + 1];
            value = u16::from_be_bytes(bytes) as f64;
            // buf[i] = u16::from_be_bytes(bytes);
            self.complex.push(
                Complex {
                    re: value * self.complex0[i].re, 
                    im: value * self.complex0[i].im, 
                },
            );
            self.xy.push([self.t, value]);
            self.t += self.delta;
        }
    }
    ///
    fn handshake() -> [u8; 2] {
        [SYN, EOT]
    }
}