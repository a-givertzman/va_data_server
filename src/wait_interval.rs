#![allow(non_snake_case)]

use std::{
    time::Duration, sync::{Arc, Mutex}, 
};
use log::{
    // info,
    // trace,
    debug,
    // warn,
};

// type BuilderCallback = FnMut();

pub struct Interval {
    period: f64,
    cancel: bool,
    
    interval:Duration,
    sleepDelta:Duration,
    // exitInterval: ;
}
impl Interval {
    /// 
    /// `builder: fn()` - callback will be called at the start of each iteration
    /// `period`, seconds - looping interval
    pub fn new(period: f64, builder: Arc<Mutex<dyn FnMut()>>) -> Self {
        let interval = Duration::from_secs_f64(self.period);
        Self {
            period,
            cancel: false,

            interval: interval,
            sleepDelta: Duration::from_secs_f64(self.period / 1000.0),
            exitInterval: interval.as_nanos(),
        }
    }
    ///
    /// Looped iterations will be started
    pub fn run(&mut self) {
        // let builder = self.builder;
        debug!("interval : {:?}", self.interval);
        debug!("interval ms : {:?}", self.interval.as_millis());
        debug!("interval mcs: {:?}", self.interval.as_micros());
        debug!("interval ns : {:?}", self.interval.as_nanos());
        debug!("sleepDelta : {:?}", self.sleepDelta);
        debug!("exitInterval : {:?}ns\n", self.exitInterval);
        // let mut times = vec![];
        let mut prevose = 0;//Duration::ZERO;
        let mut sleeped = 0;
        let mut limit = 0;
        let start = std::time::Instant::now();
        prevose = start.elapsed().as_nanos();
        while !self.cancel {
            (self.builder.lock().unwrap())();
            limit = exitInterval + prevose;
            while start.elapsed().as_nanos() < limit {
                std::thread::sleep(sleepDelta);
                sleeped += 1;
            }
            // times.push([start.elapsed().as_nanos() - prevose, sleeped]);
            debug!("elapsed : {:?}ns\n", [start.elapsed().as_nanos() - prevose, sleeped]);
            prevose = start.elapsed().as_nanos();
        }
        // for t in times {
        //     trace!("at: {:?}", t);
        // }
    }
    ///
    /// Looped iterations will be stopped
    pub fn cancel(&mut self) {
        self.cancel = true;
    }
}


// fn main_() {
//     env::set_var("RUST_LOG", "trace");
//     env::set_var("RUST_BACKTRACE", "1");
//     env_logger::init();

//     let f = 10000.0;
//     let period = 1.0 / f;
//     info!("f Hz : {:?}", f);
//     info!("T sec: {:?}", period);

//     let interval = Duration::from_secs_f64(period);
//     let sleepDelta = Duration::from_secs_f64(period / 1000.0);
//     let exitInterval = interval.as_nanos();
//     info!("interval : {:?}", interval);
//     info!("interval ms : {:?}", interval.as_millis());
//     info!("interval mcs: {:?}", interval.as_micros());
//     info!("interval ns : {:?}", interval.as_nanos());
//     info!("sleepDelta : {:?}", sleepDelta);
//     info!("exitInterval : {:?}ns\n", exitInterval);
//     let mut times = vec![];
//     let mut prevose = 0;//Duration::ZERO;
//     let mut sleeped = 0;
//     let mut limit = 0;
//     let start = std::time::Instant::now();
//     prevose = start.elapsed().as_nanos();
//     for i in 0..1000 {
//         sleeped = 0;
//         // for j in 0..10 {
//         let p = start.elapsed().as_nanos();
//         // }
//         limit = exitInterval + prevose;
//         while start.elapsed().as_nanos() < limit {
//             std::thread::sleep(sleepDelta);
//             sleeped += 1;
//         }
//         times.push([start.elapsed().as_nanos() - prevose, sleeped]);
//         prevose = start.elapsed().as_nanos();
//     }
//     for t in times {
//         trace!("at: {:?}", t);
//     }
// }