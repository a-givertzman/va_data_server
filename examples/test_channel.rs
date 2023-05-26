#![allow(non_snake_case)]


use log::info;
use std::{
    thread, 
    sync::{Mutex, Arc, mpsc}, 
    time::{Duration, Instant}, 
    env
};

fn main() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    testRawSlice();
    testArcSlice();
}

///
/// 
fn testRawSlice() {
    let mut cancel = false;
    let mut rec = vec![];

    let (tx, rx) = mpsc::channel::<[u16; 1024]>();

    let t = Instant::now();
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        // let mut q = q2.lock().unwrap();
        let mut loopIndex = 1;
        loop {
            for x in 0..10 {
                let value = [loopIndex * 100 + x; 1024];
                match tx.send(value) {
                    Ok(_) => {
                        // info!("[tread tx] sent: {:?}", value);
                    },
                    Err(err) => {
                        info!("[tread tx] send error: {:?}", err);
                    },
                };
                // thread::sleep(Duration::from_millis(10));
            }
            // thread::sleep(Duration::from_millis(300));
            loopIndex += 1;
            if loopIndex >= 1000 {break}
        }
    }).unwrap();

    info!("[main] tread tx is started");
    // thread::sleep(Duration::from_secs_f64(1.0));
    info!("[main] loop starting...");
    // let q = q1.lock().unwrap();
    // thread::sleep(Duration::from_millis(100));
    while !cancel {
        // info!("[main] loop trying read channel...");
        // info!("queue len: {:?}", q1.lock().unwrap().len());
        // let q = q1.lock().unwrap();
        rec.clear();
        for _ in 0..10 {
            match rx.recv() {
                Ok(value) => {
                    rec.push(value);
                },
                Err(err) => {
                    info!("[main] error read channel: {:?}", err);
                    cancel = true
                },
            };

        }
        // for item in rx.iter() {
        //     rec.push(item);
        //     // thread::sleep(Duration::from_secs_f64(0.001));
        // }
        // info!("received rec: {:?}", rec);
        // thread::sleep(Duration::from_millis(500));
    }
    handle.join().unwrap();
    let elapsed = Instant::now().duration_since(t);
    info!("[testRawSlice] elapsed: {:?}", elapsed);
}

///
/// 
fn testArcSlice() {
    let mut cancel = false;
    let mut rec = vec![];

    let (tx, rx) = mpsc::channel::<Arc<[u16; 1024]>>();

    let t = Instant::now();
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        // let mut q = q2.lock().unwrap();
        let mut loopIndex = 1;
        loop {
            for x in 0..10 {
                let value = [loopIndex * 100 + x; 1024];
                match tx.send(Arc::new(value)) {
                    Ok(_) => {
                        // info!("[tread tx] sent: {:?}", value);
                    },
                    Err(err) => {
                        info!("[tread tx] send error: {:?}", err);
                    },
                };
                // thread::sleep(Duration::from_millis(10));
            }
            // thread::sleep(Duration::from_millis(300));
            loopIndex += 1;
            if loopIndex >= 1000 {break}
        }
    }).unwrap();

    info!("[main] tread tx is started");
    // thread::sleep(Duration::from_secs_f64(1.0));
    info!("[main] loop starting...");
    // let q = q1.lock().unwrap();
    // thread::sleep(Duration::from_millis(100));
    while !cancel {
        // info!("[main] loop trying read channel...");
        // info!("queue len: {:?}", q1.lock().unwrap().len());
        // let q = q1.lock().unwrap();
        rec.clear();
        for _ in 0..10 {
            match rx.recv() {
                Ok(value) => {
                    rec.push(value);
                },
                Err(err) => {
                    info!("[main] error read channel: {:?}", err);
                    cancel = true
                },
            };

        }
        // for item in rx.iter() {
        //     rec.push(item);
        //     // thread::sleep(Duration::from_secs_f64(0.001));
        // }
        // info!("received rec: {:?}", rec);
        // thread::sleep(Duration::from_millis(500));
    }
    handle.join().unwrap();
    let elapsed = Instant::now().duration_since(t);
    info!("[testArcSlice] elapsed: {:?}", elapsed);
}