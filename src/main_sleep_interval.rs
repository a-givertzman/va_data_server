#![allow(non_snake_case)]

use std::{
    env,
    cmp::Ordering,
    time::Duration, 
};
use log::{
    info,
    trace,
    debug,
    warn,
};

fn main() {
    env::set_var("RUST_LOG", "trace");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let f = 10_000.0;
    let period = 1.0 / f;
    info!("f Hz : {:?}", f);
    info!("T sec: {:?}", period);

    let interval = Duration::from_secs_f64(period);
    let intervalNanos = interval.as_nanos() as i128;
    info!("interval ms : {:?}", interval.as_millis());
    info!("interval mcs: {:?}", interval.as_micros());
    info!("interval ns : {:?}\n", interval.as_nanos());
    let startGlobal = std::time::Instant::now();
    let mut prevose = startGlobal.elapsed();
    let mut deltaIntegral: i128 = 0;
    let mut delta = 0;
    let mut intervalFact = Duration::ZERO;
    let mut start = std::time::Instant::now();
    for i in 0..1000000 {

        start = std::time::Instant::now();
        intervalFact = startGlobal.elapsed() - prevose;
        delta = (intervalNanos - (intervalFact.as_nanos() as i128)) / 50000;
        deltaIntegral += delta as i128;
        trace!("started: {:?}\t at: {:?}\t elapsed: {:?}\t delta: {:?}({:?})", i, intervalFact, start.elapsed(), delta, deltaIntegral);
        prevose = startGlobal.elapsed();
        for j in 0..1000000 {
            let p = j * j * j;
        }
        let elapsed = (start.elapsed().as_nanos() as i128) + 50000;
        if intervalNanos > elapsed {
            let durNanos = intervalNanos - elapsed;
            // trace!("durNanos: {:?}", durNanos);
            std::thread::sleep(Duration::from_nanos(durNanos as u64));
        }
        // let elapsed = start.elapsed();
        // if elapsed.as_nanos() > intervalNanos {
        //     trace!("dur: {:?},\t elapsed: {:?}", (durNanos as f64) / 1000.0, elapsed);
        // }
    }
}