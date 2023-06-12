#![allow(non_snake_case)]

#[path = "../src/interval.rs"]
mod interval;
use std::{
    env,
    time::Duration, sync::Arc, 
};
use log::{
    info,
    // trace,
    debug,
    // warn,
};
use interval::Interval;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    run();
}

fn run<'a>() {
    let f = 10000.0;
    let period = 1.0 / f;
    info!("f Hz : {:?}", f);
    info!("T sec: {:?}", period);
    // let mut iterations = 0;
    let mut il = Interval::new(
        period,
        // Box::new(move|| {
        //     let mut r = vec![];
        //     for i in 0..10 {
        //         r.push(i);
        //     }
        //     info!("r: {:?}", r);
        //     // info!("n: {:?}", iterations);
        //     // iterations += 1;
        //     // if iterations > 100 {
        //     //     // interval.cancel();
        //     // }
        // }), 
    );
    il.wait();
}
