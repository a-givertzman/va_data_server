#![allow(non_snake_case)]

use concurrent_queue::ConcurrentQueue;
use log::{
    info,
    // trace,
    debug,
    // warn,
};
use num::{Complex, complex::ComplexFloat};
use rustfft::{FftPlanner, Fft};
use std::{
    sync::{Arc, Mutex}, 
    thread::{self, JoinHandle},
    collections::BTreeMap, f64::consts::PI, time::Duration,
};
use crate::{
    circular_queue::CircularQueue, 
    dsp_filters::average_filter::AverageFilter, 
    udp_server::udp_server::{
        UdpServer,
        UDP_BUF_SIZE,
    }, ds::ds_server::DsServer
};

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


pub struct FftAnalysis {
    handle: Option<JoinHandle<()>>,
    cancel: bool,
    receiver: Arc<ConcurrentQueue<[u8; UDP_BUF_SIZE]>>,
    udpServer: Arc<Mutex<UdpServer>>,
    dsServer: DsServer,
    pub delta: f64,
    pub f: f32,
    pub samplingPeriod: f64,
    pub t: f64,
    pub complex0: Vec<Complex<f64>>,
    pub complex: CircularQueue<Complex<f64>>,
    pub fftBuflen: usize,
    pub fftComplex: Vec<Complex<f64>>,
    pub xyLen: usize,
    pub xy: Vec<[f64; 2]>,
    fft: Arc<dyn Fft<f64>>,
    pub fftXyLen: usize,
    pub fftXy: Vec<[f64; 2]>,
    pub fftAlarmXy: Vec<[f64; 2]>,
    pub fftXyDif: Vec<[f64; 2]>,
    pub envelopeXy: Vec<[f64; 2]>,
    pub limitationsXy: Vec<[f64; 2]>,
    pub baseFreq: f64,
}

impl FftAnalysis {
    ///
    pub fn new(
        f: f32,
        fftBuflen: usize,
        receiver: Arc<ConcurrentQueue<[u8; UDP_BUF_SIZE]>>,
        udpServer: Arc<Mutex<UdpServer>>,
        dsServer: DsServer,
    ) -> Self {
        let samplingPeriod = 1.0 / (f as f64);
        let delta = samplingPeriod / (fftBuflen as f64);
        let iToNList: Vec<f64> = (0..fftBuflen).into_iter().map(|i| {(i as f64) / (fftBuflen as f64)}).collect();
        let phiList: Vec<f64> = iToNList.into_iter().map(|iToN| {PI * 2.0 * iToN}).collect();        
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
            cancel: false,
            receiver: receiver,
            udpServer: udpServer,
            dsServer: dsServer,
            delta: delta,
            f,
            samplingPeriod,
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
            fftAlarmXy: vec![[0.0, 0.0]; fftXyLen],
            fftXyDif: vec![[0.0, 0.0]; fftBuflen],
            envelopeXy: vec![[0.0, 0.0]; fftBuflen],
            limitationsXy: Self::buildLimitations(fftXyLen, 0),
            baseFreq: 0.0,
        }
    }
    ///
    /// 
    fn buildLimitations(len: usize, offset: usize) -> Vec<[f64; 2]> {
        const logLoc: &str = "[FftAnalysis.buildLimitations]";
        let mut res = vec![]; //vec![[0.0, 0.0]; len];
        const low: f64 = 50.0;
        let linitationsConf: BTreeMap<usize, f64> = BTreeMap::from([                
            (0, low),
            (100 - 10, 300.0),
            (100 + 10, low),
            (381 - 10, 300.0),
            (381 + 10, low),
            (3000 - 100, 300.0),
            (3000 + 100, low),
            (4000 - 100, 300.0),
            (4000 + 100, low),
            (len, low),
        ]);
        let mut prevAmplitude = 0.0;
        for (freq, amplitude) in linitationsConf {
            res.push([freq as f64, prevAmplitude]);
            res.push([freq as f64, amplitude]);
            prevAmplitude = amplitude;
        }
        debug!("{} limitations: {:?}", logLoc, res);
        // for i in 0..len {
        //     res[i] = [0.0, 0.0];
        // }
        res
    }
    ///
    ///
    pub fn restart(&mut self) {
        const logLoc: &str = "[FftAnalysis.restart]";
        debug!("{} started...", logLoc);
        self.udpServer.lock().unwrap().restart();
        debug!("{} done", logLoc);
    }
    ///
    pub fn run(this: Arc<Mutex<Self>>) -> () {
        const logLoc: &str = "[FftAnalysis.run]";
        debug!("{} starting...", logLoc);
        info!("{} enter", logLoc);
        let me = this.clone();
        let me1 = this.clone();
        let receiver = this.clone().lock().unwrap().receiver.clone();

        let queues = this.clone().lock().unwrap().dsServer.queues.clone();
        let handleDsServer = thread::Builder::new().name("FftAnalysis(DsServer) tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", logLoc, thread::current().name().unwrap());
            info!("{} started", logLoc);
            // let receiver = receiver.lock().unwrap();
            while !(me1.clone().lock().unwrap().cancel) {
                // let mut buf = Some(Arc::new([0u8; UDP_BUF_SIZE]));
                for queue in &queues {
                    while !queue.is_empty() {
                        let _point = match queue.pop() {
                            Ok(point) => {
                                if point.name == "Drive.Counter" {
                                    let value = point.valueReal();
                                    me1.clone().lock().unwrap().baseFreq = value as f64;
                                }
                            },
                            Err(_) => {},
                        };
                        // debug!("{} point ({:?}): {:?} {:?}", logLoc, point.dataType, point.name, point.value);
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            info!("{} exit", logLoc);
            // this.lock().unwrap().cancel = false;
        }).unwrap();

        let handle = thread::Builder::new().name("FftAnalysis tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", logLoc, thread::current().name().unwrap());
            info!("{} started", logLoc);
            // let receiver = receiver.lock().unwrap();
            while !(this.clone().lock().unwrap().cancel) {
                // let mut buf = Some(Arc::new([0u8; UDP_BUF_SIZE]));
                while !receiver.is_empty() {
                    match receiver.pop() {
                        Ok(buf) => {
                            // debug!("{} received buf {:?}", logLoc, buf);
                            this.lock().unwrap().enqueue(buf);
                        }
                        Err(err) => {
                            debug!("{} receive error: {:?}", logLoc, err);
                        },
                    }
                }
            }
            info!("{} exit", logLoc);
            // this.lock().unwrap().cancel = false;
        }).unwrap();
        me.lock().unwrap().handle = Some(handle);
        debug!("{} started\n", logLoc);
    }
    ///
    fn enqueue(&mut self, buf: [u8; UDP_BUF_SIZE]) {
        // const logLoc: &str = "[FftAnalysis.enqueue]";
        // debug!("{} started..", logLoc);
        let mut value;
        let mut bytes = [0_u8; 2];
        let offset = 3;
        for i in 0..(UDP_BUF_SIZE - offset) / 2 {
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
            self.xy.push([self.t * 1.0e6, value]);
            if self.xy.len() > self.xyLen {
                self.xy.remove(0);
            }
            self.t += self.delta;
        }
        // debug!("{} done/n", logLoc);
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
        let factor = 1.0 / ((self.fftBuflen / 4) as f64);
        let mut x: f64;
        let mut y: f64;
        self.fftXy.clear();
        self.fftAlarmXy.clear();
        self.fftXy.push([0.0, 0.0]);
        self.fftXy.push([0.0, 0.0]);
        for i in 1..self.fftXyLen {
            x = i as f64;
            y = self.fftComplex[i].abs() * factor;
            // y = ((self.fftComplex[i].re.powi(2) + self.fftComplex[i].im.powi(2)) * factor) as f64;
            if self.fftPointOverflowed(x, y) {
                self.fftAlarmXy.push([x, 0.0]);
                self.fftAlarmXy.push([x, y]);    
            }
            self.fftXy.push([x, 0.0]);
            self.fftXy.push([x, y]);
        }
    }
    ///
    ///
    fn fftPointOverflowed(&self, freq: f64, amplitude: f64) -> bool {
        let mut range = Range {min: 0.0, max: 0.0};
        for [x, amplitudeLimit] in self.limitationsXy.clone() {
            range.max = x;
            if range.contains(freq) {
                return amplitude >= amplitudeLimit;
            }
        }
        false
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
        let mut yDif: f64;
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


struct Range {
    pub min: f64,
    pub max: f64,
}
impl Range {
    fn contains(&self, value: f64) -> bool {
        self.min  <= value && value <= self.max
    }
}