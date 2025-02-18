#![allow(non_snake_case)]

use concurrent_queue::ConcurrentQueue;
use log::{
    info,
    trace,
    debug,
    // warn,
};
use num::{Complex, complex::ComplexFloat};
use rustfft::{FftPlanner, Fft};
use std::{
    sync::{Arc, Mutex}, 
    thread::{self, JoinHandle},
    f64::consts::PI, time::Duration,
};
use crate::{
    ds::ds_server::DsServer,
    circular_queue::CircularQueue, 
    dsp_filters::average_filter::AverageFilter, 
    networking::udp_server::{
        UdpServer,
        UDP_BUF_SIZE, UDP_HEADER_SIZE,
    }, 
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
    pub xy: PlotData,
    fft: Arc<dyn Fft<f64>>,
    pub fftXyLen: usize,
    pub fftXy: PlotData,
    pub fftAlarmXy: PlotData,
    pub fftXyDif: PlotData,
    pub envelopeXy: PlotData,
    pub limitationsXy: PlotData,
    pub baseFreq: f64,
    pub offsetFreq: f64,
    pub udpIndex: u8,
    pub udpLost: f64,
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
            xy: PlotData::new(xyLen),
            fft: planner.plan_fft_forward(fftBuflen),
            fftXyLen: fftXyLen,
            fftXy: PlotData::new(fftXyLen * 2),
            fftAlarmXy: PlotData::new(fftXyLen * 2),
            fftXyDif: PlotData::new(fftBuflen),
            envelopeXy: PlotData::new(fftBuflen),
            limitationsXy: PlotData::new(fftXyLen), // Self::buildLimitations(fftXyLen * 2, 0.0),
            baseFreq: 0.0,
            offsetFreq: 0.0,
            udpIndex: 0,
            udpLost: 0.0,
        }
    }
    ///
    /// 
    fn buildLimitations(&mut self, len: usize, offset: f64) {
        const logLoc: &str = "[FftAnalysis.buildLimitations]";
        self.limitationsXy.clear();
        const low: f64 = 50.0;
        // let linitationsConf: BTreeMap<f64, f64> = BTreeMap::from([                
        let linitationsConf: Vec<(f64, f64)> = vec![                
            (0.0, low),
            (100.0 - 10.0, 300.0),
            (100.0 + 10.0, low),
            (381.0 - 10.0, 300.0),
            (381.0 + 10.0, low),
            (3000.0 - 100.0, 300.0),
            (3000.0 + 100.0, low),
            (4000.0 - 100.0, 300.0),
            (4000.0 + 100.0, low),
            (len as f64, low),
        ];
        let mut prevAmplitude = low;
        for (freq, amplitude) in linitationsConf {
            let mut freq = freq - offset;
            if freq < 0.0 {
                freq = 0.0;
            }
            self.limitationsXy.push([freq, prevAmplitude]);
            self.limitationsXy.push([freq, amplitude]);
            prevAmplitude = amplitude;
        }
        trace!("{} limitations: {:?}", logLoc, self.limitationsXy.xy());
    }
    ///
    ///
    pub fn restart(&mut self) {
        const logLoc: &str = "[FftAnalysis.restart]";
        debug!("{} started...", logLoc);
        self.udpIndex = 0;
        self.udpLost = 0.0;
        self.udpServer.lock().unwrap().restart();
        debug!("{} done", logLoc);
    }
    ///
    pub fn run(this: Arc<Mutex<Self>>) -> () {
        let dbg: &str = "FftAnalysis.run | ";
        debug!("{} starting...", dbg);
        info!("{} enter", dbg);
        let me = this.clone();
        let me1 = this.clone();
        let receiver = this.clone().lock().unwrap().receiver.clone();

        let queues = this.clone().lock().unwrap().dsServer.queues.clone();
        let fftXyLen = me1.clone().lock().unwrap().fftXyLen;
        let handleDsServer = thread::Builder::new().name("FftAnalysis(DsServer) tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", dbg, thread::current().name().unwrap());
            info!("{} started", dbg);
            // let receiver = receiver.lock().unwrap();
            me1.clone().lock().unwrap().buildLimitations(fftXyLen, 0.0);
            while !(me1.clone().lock().unwrap().cancel) {
                // let mut buf = Some(Arc::new([0u8; UDP_BUF_SIZE]));
                for queue in &queues {
                    while !queue.is_empty() {
                        let _point = match queue.pop() {
                            Ok(point) => {
                                if point.name() == "Drive.Counter" {
                                    let value = point.as_real().value;
                                    me1.clone().lock().unwrap().baseFreq = value as f64;
                                    me1.clone().lock().unwrap().offsetFreq = (value as f64) - 3000.0;
                                    let offset = (value as f64) - 3000.0 / 60.0;
                                    me1.clone().lock().unwrap().buildLimitations(fftXyLen, offset)
                                }
                            },
                            Err(_) => {},
                        };
                        // debug!("{} point ({:?}): {:?} {:?}", logLoc, point.dataType, point.name, point.value);
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            info!("{} exit", dbg);
            // this.lock().unwrap().cancel = false;
        }).unwrap();

        let handle = thread::Builder::new().name("FftAnalysis tread".to_string()).spawn(move || {
            debug!("{} started in {:?}", dbg, thread::current().name().unwrap());
            info!("{} started", dbg);
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
                            debug!("{} receive error: {:?}", dbg, err);
                        },
                    }
                }
            }
            info!("{} exit", dbg);
            // this.lock().unwrap().cancel = false;
        }).unwrap();
        me.lock().unwrap().handle = Some(handle);
        debug!("{} started\n", dbg);
    }
    ///
    fn enqueue(&mut self, buf: [u8; UDP_BUF_SIZE]) {
        const logLoc: &str = "[FftAnalysis.enqueue]";
        // debug!("{} started..", logLoc);
        let mut value;
        let mut bytes = [0_u8; 2];
        let udpIndex = &buf[0..8];
        // debug!("{} udpIndex: {:?}", logLoc, udpIndex);
        let udpIndex = buf[3];
        if (self.udpIndex + 1) != udpIndex {
            self.udpLost += (udpIndex - self.udpIndex - 1) as f64;
            debug!("{} self.udpLost: {:?}", logLoc, self.udpLost);
        }
        self.udpIndex = udpIndex;
        // debug!("{} udpIndex: {:?}", logLoc, udpIndex);
        let offset = UDP_HEADER_SIZE;
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
            self.xy.add([self.t * 1.0e6, value]);
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
        // const logLoc: &str = "[FftAnalysis.buildFftXy]";
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
        for [x, amplitudeLimit] in self.limitationsXy.xy() {
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
        // const logLoc: &str = "[FftAnalysis.buildEnvelope]";
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
            x = self.fftXy.get(i)[0];
            y = self.fftXy.get(i)[1];
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
        // const logLoc: &str = "[FftAnalysis.buildFftXyDif]";
        let filterLen = 512;
        let mut filter: AverageFilter<f64> = AverageFilter::new(filterLen);
        let len = self.fftXyLen;
        let mut yDif: f64;
        let mut y: f64;
        let mut i: usize = 0;
        let mut yPrev: f64 = self.fftXy.get(i)[1];
        self.fftXyDif.clear();
        self.fftXyDif.push([0.0, 0.0]);
        for j in 1..len {
            i = j * 2 - 1;
            y = self.fftXy.get(j)[1];
            // yDif = (y - yPrev).abs();
            filter.add((y - yPrev).abs());
            yDif = filter.value() * 100.0 + 1000000.0;
            yPrev = y;
            self.fftXyDif.push([self.fftXy.get(j)[0] - (filterLen / 2) as f64, yDif]);
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

pub struct PlotData {
    length: usize,
    xy: Vec<[f64; 2]>,
}

impl PlotData {
    ///
    pub fn new(length: usize) -> Self {
        Self {
            length: length, 
            xy: vec![[0.0, 0.0]; length],
        }
    }
    ///
    pub fn push(&mut self, xy: [f64; 2]) {
        self.add(xy)
    }
    ///
    pub fn add(&mut self, xy: [f64; 2]) {
        self.xy.push(xy);
        while self.xy.len() > self.length {
            self.xy.remove(0);
        }
    }
    ///
    pub fn update(&mut self, index: usize, xy: [f64; 2]) {
        self.xy[index] = xy;
    }
    pub fn get(&self, index: usize) -> [f64; 2] {
        if index < self.xy.len() {
            self.xy[index]
        } else {
            panic!("'index out of bounds: the len is {} but the index is {}'", self.xy.len(), index);
        }
    }
    ///
    pub fn xy(&self) -> Vec<[f64; 2]> {
        self.xy.clone()
    }
    /// 
    pub fn len(&self) -> usize {
        self.xy.len()
    }
    ///
    pub fn setLen(&mut self, length: usize) {
        self.length = length;
    }
    ///
    pub fn clear(&mut self) {
        self.xy.clear();
    }
}