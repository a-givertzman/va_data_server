use egui_plot::{PlotPoint, PlotPoints};
use num::{Complex, complex::ComplexFloat};
use rustfft::{FftPlanner, Fft};
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{services::{RECV_TIMEOUT, Service, entity::{Name, Object, Point}}, sync::{Handles, Owner, RwLock, channel::{self, Receiver, RecvTimeoutError, Sender}}};
use std::{
    collections::VecDeque, fmt::{Debug, Display}, io::BufRead, sync::{Arc, atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering}}, thread::{self}, time::{Duration, Instant}
};
use crate::{
    circular_queue::CircularQueue, ds::DsServer, dsp_filters::average_filter::AverageFilter, networking::UdpClient, presentation::PlotData 
    // networking::udp_server::{
    //     UdpServer,
    //     UDP_BUF_SIZE, UDP_HEADER_SIZE,
    // }, 
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
    name: Name,
    udp_client: Arc<UdpClient>,
    ds_server: DsServer,
    send: Sender<Point>,
    recv: Owner<Receiver<Point>>,
    pub delta: Arc<AtomicFloat<f64>>,
    pub f: Arc<AtomicFloat<f32>>,
    pub sampling_period: Arc<AtomicFloat<f64>>,
    pub t: Arc<AtomicFloat<f64>>,
    // pub complex0: Arc<RwLock<Vec<Complex<f64>>>>,
    pub complex: Arc<RwLock<CircularQueue<Complex<f64>>>>,
    pub fft_buflen: usize,
    pub fft_complex: Arc<RwLock<Vec<Complex<f64>>>>,
    hamming_window: Arc<RwLock<Vec<f64>>>,
    send_xy: Sender<u16>,
    recv_xy: Owner<Receiver<u16>>,
    fft: Arc<dyn Fft<f64> + 'static>,
    pub fft_xy_len: usize,
    pub fft_xy: Arc<PlotData>,
    pub fft_alarm_xy: Arc<PlotData>,
    pub fft_xy_dif: Arc<PlotData>,
    pub envelope_xy: Arc<PlotData>,
    pub limitations_xy: Arc<PlotData>,
    pub base_freq: Arc<AtomicFloat<f64>>,
    pub offset_freq: Arc<AtomicFloat<f64>>,
    pub udp_index: AtomicU8,
    pub udp_lost: Arc<AtomicFloat<f64>>,
    pub channel: Arc<AtomicUsize>,
    pub pause: Arc<AtomicBool>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}

impl FftAnalysis {
    ///
    pub fn new(
        parent: impl Into<String>,
        f: f32,
        fft_buflen: usize,
        udp_client: Arc<UdpClient>,
        ds_server: DsServer,
    ) -> Self {
        let parent = parent.into();
        let sampling_period = 1.0 / (f as f64);
        let delta = sampling_period / (fft_buflen as f64);
        // let i_to_nlist: Vec<f64> = (0..fft_buflen).into_iter().map(|i| {(i as f64) / (fft_buflen as f64)}).collect();
        // let phi_list: Vec<f64> = i_to_nlist.into_iter().map(|i_to_n| {PI * 2.0 * i_to_n}).collect();        
        // let complex0: Vec<Complex<f64>> = (0..fft_buflen).into_iter().map(|i| {
        //     Complex {
        //         re: phi_list[i].cos(), 
        //         im: phi_list[i].sin()
        //     }
        // }).collect();
        //
        // The length of the FFT display window 
        let fft_xy_len = 35_000;
        let mut planner = FftPlanner::new();
        let (send, recv) = channel::unbounded();
        let (send_xy, recv_xy) = channel::unbounded();
        Self {
            name: Name::new(&parent, "FftAnalysis"),
            udp_client,
            ds_server,
            send,
            recv: Owner::new(recv),
            send_xy,
            recv_xy: Owner::new(recv_xy),
            delta: Arc::new(AtomicFloat::new(delta)),
            f: Arc::new(AtomicFloat::new(f)),
            sampling_period: Arc::new(AtomicFloat::new(sampling_period)),
            t: Arc::new(AtomicFloat::new(0.0)),
            // complex0: Arc::new(RwLock::new(complex0)),
            complex: Arc::new(RwLock::new(CircularQueue::with_capacity_fill(fft_buflen, &mut vec![Complex{re: 0.0, im: 0.0}; fft_buflen]))),
            fft_buflen,
            fft_complex: Arc::new(RwLock::new(vec![Complex{re: 0.0, im: 0.0}; fft_buflen])),
            hamming_window: Arc::new(RwLock::new(Self::create_hamming_window(fft_buflen))),
            fft: planner.plan_fft_forward(fft_buflen),
            fft_xy_len,
            fft_xy: Arc::new(PlotData::new(fft_xy_len * 2)),
            fft_alarm_xy: Arc::new(PlotData::new(fft_xy_len * 2)),
            fft_xy_dif: Arc::new(PlotData::new(fft_buflen)),
            envelope_xy: Arc::new(PlotData::new(fft_buflen)),
            limitations_xy: Arc::new(PlotData::new(fft_xy_len)), // Self::buildLimitations(fftXyLen * 2, 0.0),
            base_freq: Arc::new(AtomicFloat::new(0.0)),
            offset_freq: Arc::new(AtomicFloat::new(0.0)),
            udp_index: AtomicU8::new(0),
            udp_lost: Arc::new(AtomicFloat::new(0.0)),
            channel: Arc::new(AtomicUsize::new(2)),
            pause: Arc::new(AtomicBool::new(false)),
            handles: Handles::new(&parent),
            exit: Arc::new(AtomicBool::new(false)),
            dbg: Dbg::new(parent, "FftAnalysis")
        }
    }
    ///
    /// Returns stream of XY values
    pub fn xy(&self) -> Receiver<u16> {
        self.recv_xy.take().unwrap()
    }
    ///
    /// Create Hamming window coefficients
    fn create_hamming_window(size: usize) -> Vec<f64> {
        let mut window = vec![0.0; size];
        for i in 0..size {
            // Hamming window formula: w(n) = 0.54 - 0.46 * cos(2πn/(N-1))
            let n = i as f64;
            let n_norm = 2.0 * std::f64::consts::PI * n / (size as f64 - 1.0);
            window[i] = 0.54 - 0.46 * n_norm.cos();
        }
        window
    }
    ///
    /// 
    fn build_limitations(dbg: &Dbg, limitations_xy: &Arc<PlotData>, len: usize, offset: f64) {
        limitations_xy.clear();
        const LOW: f64 = 50.0;
        // let linitationsConf: BTreeMap<f64, f64> = BTreeMap::from([                
        let linitations_conf: Vec<(f64, f64)> = vec![                
            (0.0, LOW),
            (100.0 - 10.0, 300.0),
            (100.0 + 10.0, LOW),
            (381.0 - 10.0, 300.0),
            (381.0 + 10.0, LOW),
            (3000.0 - 100.0, 300.0),
            (3000.0 + 100.0, LOW),
            (4000.0 - 100.0, 300.0),
            (4000.0 + 100.0, LOW),
            (len as f64, LOW),
        ];
        let mut prev_amplitude = LOW;
        for (freq, amplitude) in linitations_conf {
            let mut freq = freq - offset;
            if freq < 0.0 {
                freq = 0.0;
            }
            limitations_xy.push(&[freq, prev_amplitude]);
            limitations_xy.push(&[freq, amplitude]);
            prev_amplitude = amplitude;
        }
        log::trace!("{dbg}.build_limitations | limitations: {:?}", limitations_xy.xy());
    }
    ///
    ///
    pub fn restart(&self) {
        log::debug!("{}.restart | started...", self.dbg);
        self.udp_index.store(0, Ordering::Release);
        self.udp_lost.store(0.0);
        match self.udp_client.restart() {
            Ok(_) => log::debug!("{}.restart | done", self.dbg),
            Err(err) => log::debug!("{}.restart | Error: {:?}", self.dbg, err),
        }
    }
    ///
    fn enqueue(
        dbg: &Dbg,
        complex: &mut CircularQueue<Complex<f64>>,
        // complex0: &Vec<Complex<f64>>,
        values: &[u16],
        window: &[f64],
        dc_remove: f64,
    ) {
        for (i, val) in values.iter().enumerate() {
            // log::debug!("{} value: {:?}", self.dbg, value);
            // let i = complex.len();
            complex.push(
                Complex {
                    // re: (*val as f64) * complex0[i].re, 
                    // im: (*val as f64) * complex0[i].im, 
                    // re: (*val as f64 - dc) * window.get(i).unwrap_or(&0.0),
                    re: *val as f64 - dc_remove,
                    im: 0.0, 
                },
            );
        }
        log::trace!("{dbg}.enqueue | Done");
    }
    ///
    /// 
    fn fft_process(
        dbg: &Dbg,
        sampl_freq: usize,
        fft: &Arc<dyn Fft<f64>>,
        fft_buflen: usize,
        fft_complex: &mut Vec<Complex<f64>>,
        fft_xy_len: usize,
        fft_xy: &Arc<PlotData>,
        fft_xy_dif: &Arc<PlotData>,
        fft_alarm_xy: &Arc<PlotData>,
        envelope_xy: &Arc<PlotData>,
        limitations_xy: &Arc<PlotData>,
    ) {
        // let t = Instant::now();
        fft.process(fft_complex);
        // log::debug!("{dbg}.fft_process | fft elapsed: {:?}", t.elapsed());
        // self.fft.process_with_scratch(&mut self.fftComplex);
        Self::build_fft_xy(sampl_freq, fft_buflen, fft_complex, fft_xy_len, fft_xy, fft_alarm_xy, limitations_xy);
        Self::build_envelope(fft_xy_len, fft_xy, envelope_xy);
        Self::build_fft_xy_dif(fft_xy_len, fft_xy, fft_xy_dif);
    }    
    ///
    ///
    fn build_fft_xy(sampl_freq: usize, fft_buflen: usize, fft_complex: &Vec<Complex<f64>>, fft_xy_len: usize, fft_xy: &Arc<PlotData>, fft_alarm_xy: &Arc<PlotData>, limitations_xy: &Arc<PlotData>) {
        // let factor = 1.0 / ((sampl_freq / 4) as f64);
        let x_factor = sampl_freq as f64 / fft_buflen as f64;
        // let coherent_gain = 0.54;  // Hamming window coherent gain
        // let y_factor = coherent_gain * 2.0 / fft_buflen as f64;
        let y_factor = 2.0 / fft_buflen as f64;
        let mut x: f64;
        let mut y: f64;
        fft_xy.clear();
        fft_alarm_xy.clear();
        fft_xy.push(&[0.0, 0.0]);
        fft_xy.push(&[0.0, 0.0]);
        for i in 1..fft_xy_len {
            x = i as f64 * x_factor;
            y = fft_complex.get(i).unwrap_or(&Complex::new(0.0, 0.0)).abs() * y_factor;
            // y = ((self.fftComplex[i].re.powi(2) + self.fftComplex[i].im.powi(2)) * factor) as f64;
            // if Self::fft_point_overflowed(x, y, limitations_xy) {
            //     fft_alarm_xy.push([x, 0.0]);
            //     fft_alarm_xy.push([x, y]);    
            // }
            fft_xy.push(&[x, 0.0]);
            fft_xy.push(&[x, y]);
        }
    }
    ///
    ///
    fn fft_point_overflowed(freq: f64, amplitude: f64, limitations_xy: &Arc<PlotData>) -> bool {
        let mut range = Range {min: 0.0, max: 0.0};
        for [x, amplitude_limit] in limitations_xy.xy() {
            range.max = x;
            if range.contains(freq) {
                return amplitude >= amplitude_limit;
            }
        }
        false
    }
    ///
    /// 
    fn build_envelope(fft_xy_len: usize, fft_xy: &Arc<PlotData>, envelope_xy: &Arc<PlotData>) {
        let len = fft_xy_len;
        // let mut buf: heapless::spsc::Queue<f64, 3> = heapless::spsc::Queue::new();
        let filter_len: usize = 256;
        let mut filter_buf: CircularQueue<f64> = CircularQueue::with_capacity_fill(filter_len, &mut vec![0.0; filter_len]);
        // let factor = 1.0;// / ((self.fftBuflen / 2) as f64);
        let mut x: f64;
        let mut y: f64;
        let mut average: f64;
        envelope_xy.clear();
        for i in 0..len {
            x = fft_xy.get(i)[0];
            y = fft_xy.get(i)[1];
            filter_buf.push(y);
            average = filter_buf.buffer().iter().sum::<f64>() / (filter_len as f64);
            average = y  + 10.0 * average;
            // average = 100000.0 + 0.1 * y  + (filterBuf.buffer().iter().sum::<f64>() / (filterLen as f64));
            // self.envelopeXy.push([x, 0.0]);
            envelope_xy.push(&[x, average]);
        }
    }
    ///
    /// Производная от FFT
    fn build_fft_xy_dif(fft_xy_len: usize, fft_xy: &Arc<PlotData>, fft_xy_dif: &Arc<PlotData>) {
        let filter_len = 512;
        let mut filter: AverageFilter<f64> = AverageFilter::new(filter_len);
        let len = fft_xy_len;
        let mut y_dif: f64;
        let mut y: f64;
        let mut i: usize = 0;
        let mut y_prev: f64 = fft_xy.get(i)[1];
        fft_xy_dif.clear();
        fft_xy_dif.push(&[0.0, 0.0]);
        for j in 1..len {
            i = j * 2 - 1;
            y = fft_xy.get(j)[1];
            // yDif = (y - yPrev).abs();
            filter.add((y - y_prev).abs());
            y_dif = filter.value() * 100.0 + 1000000.0;
            y_prev = y;
            fft_xy_dif.push(&[fft_xy.get(j)[0] - (filter_len / 2) as f64, y_dif]);
        }
        // for j in 0..(filterLen / 2) {
        //     i = len - (filterLen / 2) + j;
        //     self.fftXyDif.remove(0);
        //     self.fftXyDif.push([self.fftXy[i][0], yDif]);
        // }
    }
}
//
//
impl Object for FftAnalysis {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for FftAnalysis {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClient")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for FftAnalysis {
    //
    //
    fn get_link(&self, _: &str) -> Sender<Point> {
        self.send.clone()
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        log::debug!("{dbg}.run | starting...");
        log::info!("{dbg}.run | enter");

        let queues = self.ds_server.queues.clone();
        let fft_xy_len = self.fft_xy_len;
        let limitations_xy = self.limitations_xy.clone();
        Self::build_limitations(&dbg, &limitations_xy, fft_xy_len, 0.0);
        let base_freq = self.base_freq.clone();
        let offset_freq = self.offset_freq.clone();
        let exit = self.exit.clone();
        let handle1 = thread::Builder::new().name("DsServer tread".to_string()).spawn(move || {
            while !(exit.load(Ordering::Acquire)) {
                // let mut buf = Some(Arc::new([0u8; UDP_BUF_SIZE]));
                for queue in &queues {
                    while !queue.is_empty() {
                        match queue.pop() {
                            Ok(point) => {
                                if point.name == "Drive.Counter" {
                                    let value = point.valueReal();
                                    base_freq.store(value as f64);
                                    offset_freq.store(value as f64 - 3000.0);
                                    let offset = (value as f64) - 3000.0 / 60.0;
                                    Self::build_limitations(&dbg, &limitations_xy, fft_xy_len, offset)
                                }
                            },
                            Err(_) => {},
                        }
                        // log::debug!("{} point ({:?}): {:?} {:?}", logLoc, point.dataType, point.name, point.value);
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            log::info!("{dbg} Exit");
        }).map_err(|err| Error::new(&self.dbg, "run").pass_with("DsServer start failed", err.to_string()))?;

        let dbg = self.dbg.clone();
        let receiver = self.recv.take().unwrap();
        let pause = self.pause.clone();
        let channel = self.channel.clone();
        let f = self.f.load() as usize;
        let dc = 0.0; //2048.0;    // Постоянная составляющая, которую можно удалить для уточнения и облегчения FFT
        let fft = self.fft.clone();
        let fft_buflen = self.fft_buflen;
        let complex = self.complex.clone();
        let window = self.hamming_window.clone();
        // let complex0 = self.complex0.clone();
        let fft_complex = self.fft_complex.clone();
        let fft_xy_len = self.fft_xy_len;
        let fft_xy = self.fft_xy.clone();
        let fft_xy_dif = self.fft_xy_dif.clone();
        let fft_alarm_xy = self.fft_alarm_xy.clone();
        let envelope_xy = self.envelope_xy.clone();
        let limitations_xy = self.limitations_xy.clone();
        let send_xy = self.send_xy.clone();
        let t = self.t.clone();
        let delta = self.delta.clone();
        let exit = self.exit.clone();
        let (fft_send, fft_rcv) = channel::unbounded();
        let handle2 = thread::Builder::new().name("FftEnqueue tread".to_string()).spawn(move || {
            log::debug!("{dbg}.run | Reading events...");
            let mut err_limit = ErrorLimit::new(30);
            let mut buf = vec![];
            let shift = fft_buflen / 32;     // frequency of fft calculations
            let mut enqued = 0;
            while !(exit.load(Ordering::Acquire)) {
                match receiver.recv_timeout(RECV_TIMEOUT) {
                    Ok(event) => {
                        if !pause.load(Ordering::Acquire) {
                            if event.name().ends_with(&format!("{}", channel.load(Ordering::Acquire))) {
                                // log::debug!("{dbg}.run | Event: {:?}", event);
                                let val = event.to_int().as_int().value as u16;
                                if let Err(err) = send_xy.send(val) {
                                    log::debug!("{dbg}.run | Send error {:?}", err);
                                }
                                buf.push(val);
                                // log::debug!("{dbg} received buf {:?}", buf);
                                if buf.len() >= 512 {
                                    // log::debug!("{dbg}.run | {} values received", buf.len());
                                    Self::enqueue(
                                        &dbg,
                                        &mut complex.write(),
                                        // &complex0.read(),
                                        &buf,
                                        &window.read(),
                                        dc,
                                    );
                                    enqued += buf.len();
                                    buf.clear();
                                    if enqued >= shift {
                                        if let Err(err) = fft_send.send(()) {
                                            log::warn!("{dbg}.run | Can't send 'Execute' to FftAnalysis: \n\t{:?}", err);
                                        }
                                        enqued = 0;
                                    }
                                }
                            }
                        }
                        err_limit.reset();
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {
                            if let Err(_) = err_limit.add() {
                                match buf.len() {
                                    0 => log::warn!("{dbg}.run | Can't receive values"),
                                    _ => log::warn!("{dbg}.run | Receiving resetarted because of long timeout")
                                }
                                err_limit.reset();
                                buf.clear();
                            }
                        }
                        _ => {
                            log::warn!("{dbg}.run | receive error: {:?}", err);
                            break;
                        }
                    }
                }
            }
            log::info!("{dbg} Exit");
            // this.lock().cancel = false;
        }).map_err(|err| Error::new(&self.dbg, "run").pass_with("FftEnqueue start failed", err.to_string()))?;
        let dbg = self.dbg.clone();
        let complex = self.complex.clone();
        let pause = self.pause.clone();
        let exit = self.exit.clone();
        let handle3 = thread::Builder::new().name("FftAnalysis tread".to_string()).spawn(move || {
            while !(exit.load(Ordering::Acquire)) {
                if !pause.load(Ordering::Acquire) {
                    if fft_rcv.len() > 3 {
                        log::warn!("{dbg}.run | FFT queue length: {}", fft_rcv.len());
                    }
                    match fft_rcv.recv_timeout(RECV_TIMEOUT) {
                        Ok(_) => {
                            complex.read().buffer().clone_into(&mut fft_complex.write());
                            Self::fft_process(
                                &dbg,
                                f,
                                &fft,
                                fft_buflen,
                                &mut fft_complex.write(),
                                fft_xy_len,
                                &fft_xy,
                                &fft_xy_dif,
                                &fft_alarm_xy,
                                &envelope_xy,
                                &limitations_xy,
                            );
                        }
                        Err(err) => match err {
                            RecvTimeoutError::Timeout => {}
                            _ => {
                                log::warn!("{dbg}.run | receive error: {:?}", err);
                                break;
                            }
                        }
                    }
                }
            }
        }).map_err(|err| Error::new(&self.dbg, "run").pass_with("FftAnalysis start failed", err.to_string()))?;
        self.handles.push(handle1);
        self.handles.push(handle2);
        self.handles.push(handle3);
        log::info!("{}.run | Starting - ok", self.dbg);
        Ok(())
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
///
/// Thread safe f64
pub struct AtomicFloat<T> {
    val: RwLock<T>,
}
impl<T: Copy> AtomicFloat<T> {
    pub fn new(val: T) -> Self {
        Self { val: RwLock::new(val) }
    }
    ///
    /// Returns stored value
    pub fn load(&self) -> T {
        *self.val.read()
    }
    ///
    /// Stores specified value
    pub fn store(&self, val: T) {
        *self.val.write() = val;
    }
}
impl<T: Display> Display for AtomicFloat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         write!(f, "{}", self.val.read())
    }
}
impl<T: Display> Debug for AtomicFloat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         write!(f, "{}", self.val.read())
    }
}
///
/// 
struct Range {
    pub min: f64,
    pub max: f64,
}
impl Range {
    fn contains(&self, value: f64) -> bool {
        self.min  <= value && value <= self.max
    }
}
