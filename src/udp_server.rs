#![allow(non_snake_case)]

use log::{
    info,
    // trace,
    debug,
    warn,
};
use num::{Complex, complex::ComplexFloat};
use rustfft::{FftPlanner, Fft};
use std::{
    net::UdpSocket, 
    time::Duration, 
    sync::{Arc, Mutex, MutexGuard}, 
    thread::{self, JoinHandle},
};
use crate::{circular_queue::CircularQueue, input_signal::PI2, dsp_filters::average_filter::AverageFilter};

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
const UDP_BUF_SIZE: usize = 1024 + 4;

pub struct UdpServer {
    handle: Option<JoinHandle<()>>,
    localAddr: String, //SocketAddr,
    remoteAddr: String, //SocketAddr,
    reconnectDelay: Duration,
    pub isConnected: bool,
    cancel: bool,
    restart: bool,
    delta: f64,
    pub f: f32,
    pub t: f64,
    pub complex0: Vec<Complex<f64>>,
    pub complex: CircularQueue<Complex<f64>>,
    pub fftBuflen: usize,
    pub fftComplex: Vec<Complex<f64>>,
    pub xyLen: usize,
    pub xy: Vec<[f64; 2]>, //CircularQueue<[f64; 2]>,
    fft: Arc<dyn Fft<f64>>,
    pub fftXyLen: usize,
    pub fftXy: Vec<[f64; 2]>,
    pub fftXyDif: Vec<[f64; 2]>,
    pub envelopeXy: Vec<[f64; 2]>,
}

impl UdpServer {
    ///
    pub fn new(
        localAddr: &str,
        remoteAddr: &str,
        f: f32,
        fftBuflen: usize,
        reconnectDelay: Option<Duration>,
    ) -> Self {
        let period = 1.0 / (f as f64);
        let delta = period / (fftBuflen as f64);
        let iToNList: Vec<f64> = (0..fftBuflen).into_iter().map(|i| {(i as f64) / (fftBuflen as f64)}).collect();
        let phiList: Vec<f64> = iToNList.into_iter().map(|iToN| {PI2 * iToN}).collect();        
        let complex0: Vec<Complex<f64>> = (0..fftBuflen).into_iter().map(|i| {
            Complex {
                re: phiList[i].cos(), 
                im: phiList[i].sin()
            }
        }).collect();
        let xyLen = 2048;
        let fftXyLen = fftBuflen / 10;
        let mut planner = FftPlanner::new();
        Self {
            handle: None,
            localAddr: String::from(localAddr),
            remoteAddr: String::from(remoteAddr),
            reconnectDelay: match reconnectDelay {Some(rd) => rd, None => Duration::from_secs(3)},
            isConnected: false,
            cancel: false,
            restart: false,
            delta: delta,
            f,
            t: 0.0,
            complex0,
            complex: CircularQueue::with_capacity_fill(fftBuflen, &mut vec![Complex{re: 0.0, im: 0.0}; fftBuflen]),
            fftBuflen,
            fftComplex: vec![Complex{re: 0.0, im: 0.0}; fftBuflen],
            xyLen: xyLen,
            xy: vec![[0.0, 0.0]; xyLen], //CircularQueue::with_capacity_fill(QSIZE * 10, &mut vec![[0.0, 0.0]; QSIZE * 10]),
            fft: planner.plan_fft_forward(fftBuflen),
            fftXyLen: fftXyLen,
            fftXy: vec![[0.0, 0.0]; fftXyLen],
            fftXyDif: vec![[0.0, 0.0]; fftBuflen],
            envelopeXy: vec![[0.0, 0.0]; fftBuflen],
    }
    }
    ///
    pub fn restart(&mut self) {
        const logLoc: &str = "[UdpServer.restart]";
        debug!("{} started...", logLoc);
        self.restart = true;
        self.cancel = true;
        debug!("{} done", logLoc);
    }
    ///
    pub fn run(this: Arc<Mutex<Self>>) -> () {
    // pub fn run(this: Arc<Mutex<Self>>) -> Result<(), Box<dyn Error>> {
        const logLoc: &str = "[UdpServer.run]";
        debug!("{} starting...", logLoc);
        info!("{} enter", logLoc);
        let me = this.clone();
        let me1 = this.clone();
        let localAddr = this.lock().unwrap().localAddr.clone();
        let remoteAddr = this.lock().unwrap().remoteAddr.clone();
        let reconnectDelay = this.lock().unwrap().reconnectDelay;
        let handle = thread::Builder::new().name("UdpServer tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", logLoc, thread::current().name().unwrap());
            info!("{} started", logLoc);
            while !(this.lock().unwrap().cancel) {
                info!("{} try to bind on: {:?}", logLoc, localAddr.clone());
                let result = match UdpSocket::bind(localAddr.clone()) {
                    Ok(socket) => {
                        info!("{} ready on: {:?}\n", logLoc, localAddr);
                        this.lock().unwrap().isConnected = true;
                        info!("{} isConnected: {:?}\n", logLoc, this.lock().unwrap().isConnected);
                        let mut udpBuf = [0; UDP_BUF_SIZE];
                        let handshake = Self::handshake();
                        info!("{} sending handshake({}): {:?}", logLoc, handshake.len(), handshake);
                        // match socket.send_to(&handshake, remoteAddr.clone()) {
                        //     Ok(_) => {
                        //         info!("{} handshake done\n", logLoc);
                        //     },
                        //     Err(err) => {
                        //         warn!("{} send error: {:#?}", logLoc, err);
                        //     },
                        // };
                        // socket.set_read_timeout(Some(Duration::from_millis(3000))).unwrap();
                        let mut cancel = match this.try_lock() {
                            Ok(m) => {
                                m.cancel
                            }
                            Err(_) => {
                                false
                            }
                        };
                        while !cancel {
                            // debug!("{} reading from udp socket...", logLoc);
                            match socket.recv_from(&mut udpBuf) {
                                Ok((_amt, _src)) => {
                                    // debug!("{} receaved bytes({}) from{:?}: {:?}", logLoc, _amt, _src, udpBuf);
                                    this.lock().unwrap().enqueue(&udpBuf);
                                    // buf.fill(0);
                                    // bufDouble.fill(0)
                                },
                                Err(err) => {
                                    warn!("{} read error: {:#?}", logLoc, err);
                                },
                            };
                            // debug!("{} udp read done", logLoc);
                            // std::thread::sleep(Duration::from_millis(100));
                            cancel = match this.try_lock() {
                                Ok(m) => {
                                    m.cancel
                                }
                                Err(_) => {
                                    false
                                }
                            };
    
                        }
                        info!("{} exit read cycle", logLoc);
                        Some(socket)
                    }
                    Err(err) => {
                        me1.lock().unwrap().isConnected = false;
                        debug!("{} binding error on: {:?}\n\tdetailes: {:?}", logLoc, localAddr, err);
                        std::thread::sleep(reconnectDelay);
                        None
                    }
                };
                if this.lock().unwrap().restart {
                    info!("{} restart detected", logLoc);
                    match result {
                        Some(socket) => {
                            info!("{} trying to drop socket...", logLoc);
                            drop(socket);
                            info!("{} drop socket - done", logLoc);
                        },
                        None => {},
                    }
                    this.lock().unwrap().cancel = false;
                    this.lock().unwrap().restart = false;
                } else {
                    info!("{} sleep reconnectDelay: {:?}", logLoc, reconnectDelay);
                    std::thread::sleep(reconnectDelay);    
                }
            }
            info!("{} exit", logLoc);
            this.lock().unwrap().cancel = false;
        }).unwrap();
        me.lock().unwrap().handle = Some(handle);
        debug!("{} started\n", logLoc);
    }
    ///
    fn enqueue(&mut self, buf: &[u8; UDP_BUF_SIZE]) {
        // const logLoc: &str = "[UdpServer.enqueue]";
        // debug!("{} started..", logLoc);
        let mut value;
        let mut bytes = [0_u8; 2];
        let offset = 4;
        for i in 0..QSIZE {
            bytes[1] = buf[i * 2 + offset];
            bytes[0] = buf[i * 2 + offset + 1];
            value = u16::from_be_bytes(bytes) as f64;
            // debug!("{} value: {:?}", logLoc, value);
            self.complex.push(
                Complex {
                    re: value * self.complex0[i].re, 
                    im: value * self.complex0[i].im, 
                },
            );
            if self.complex.is_full() {
                self.fftProcess();
                self.complex.clear();
            }
            self.xy.push([self.t, value]);
            if self.xy.len() > self.xyLen {
                self.xy.remove(0);
            }
            self.t += self.delta;
        }
        // debug!("{} done/n", logLoc);
    }
    ///
    fn handshake() -> [u8; 2] {
        [SYN, EOT]
    }
    ///
    /// 
    fn fftProcess(&mut self) {
        self.complex.buffer().clone_into(&mut self.fftComplex);
        self.fft.process(&mut self.fftComplex);
        // self.fft.process_with_scratch(&mut self.fftComplex);
        self.buildFftXy();
        self.buildEnvelope();
        self.buildFftXyDif();
    }    
    ///
    ///
    fn buildFftXy(&mut self) {
        let factor = 1.0 / ((self.fftBuflen / 2) as f64);
        let mut x: f64;
        let mut y: f64;
        self.fftXy.clear();
        self.fftXy.push([0.0, 0.0]);
        self.fftXy.push([0.0, 0.0]);
        for i in 1..self.fftXyLen {
            x = i as f64;
            y = self.fftComplex[i].abs() * factor;
            // y = ((self.fftComplex[i].re.powi(2) + self.fftComplex[i].im.powi(2)) * factor) as f64;
            self.fftXy.push([x, 0.0]);
            self.fftXy.push([x, y]);
        }
    }
    ///
    /// 
    fn buildEnvelope(&mut self) {
        let len = self.fftXyLen;
        // let mut buf: heapless::spsc::Queue<f64, 3> = heapless::spsc::Queue::new();
        let filterLen: usize = 256;
        let mut filterBuf: CircularQueue<f64> = CircularQueue::with_capacity_fill(filterLen, &mut vec![0.0; filterLen]);
        // let factor = 1.0;// / ((self.fftBuflen / 2) as f64);
        let mut x: f64;
        let mut y: f64;
        let mut average: f64;
        self.envelopeXy.clear();
        for i in 0..len {
            x = self.fftXy[i][0];
            y = self.fftXy[i][1];
            filterBuf.push(y);
            average = filterBuf.buffer().iter().sum::<f64>() / (filterLen as f64);
            average = y  + 10.0 * average;
            // average = 100000.0 + 0.1 * y  + (filterBuf.buffer().iter().sum::<f64>() / (filterLen as f64));
            // self.envelopeXy.push([x, 0.0]);
            self.envelopeXy.push([x, average]);
        }
    }
    ///
    /// Производная от FFT
    fn buildFftXyDif(&mut self) {
        let filterLen = 512;
        let mut filter: AverageFilter<f64> = AverageFilter::new(filterLen);
        let len = self.fftXyLen;
        let mut yDif: f64 = 0.0;
        let mut y: f64;
        let mut i: usize = 0;
        let mut yPrev: f64 = self.fftXy[i][1];
        self.fftXyDif.clear();
        self.fftXyDif.push([0.0, 0.0]);
        for j in 1..len {
            i = j * 2 - 1;
            y = self.fftXy[i][1];
            // yDif = (y - yPrev).abs();
            filter.add((y - yPrev).abs());
            yDif = filter.value() * 100.0 + 1000000.0;
            yPrev = y;
            self.fftXyDif.push([self.fftXy[i][0] - (filterLen / 2) as f64, yDif]);
        }
        // for j in 0..(filterLen / 2) {
        //     i = len - (filterLen / 2) + j;
        //     self.fftXyDif.remove(0);
        //     self.fftXyDif.push([self.fftXy[i][0], yDif]);
        // }
    }
}
