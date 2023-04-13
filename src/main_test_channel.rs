#![allow(non_snake_case)]

mod circular_queue;

use log::info;
use std::{
    thread, 
    sync::{Mutex, Arc, mpsc}, 
    time::Duration, 
    env
};
use circular_queue::CircularQueue;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let cancel = false;
    // let queue: CircularQueue<i32> = CircularQueue::with_capacity(16);
    // let q1 = Arc::new(Mutex::new(queue));
    // let q2 = q1.clone();
    // let r1 = &mut queue;
    // let refQueue = ;
    let mut rec = vec![];

    let (tx, rx) = mpsc::channel::<i32>();

    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        // let mut q = q2.lock().unwrap();
        loop {            
            for x in 0..10 {
                match tx.send(x) {
                    Ok(_) => {},
                    Err(err) => {
                        info!("[tread tx] send error: {:?}", err);
                    },
                };
                info!("[tread tx] pushed: {:?}", x);
                thread::sleep(Duration::from_millis(50));
            }
            thread::sleep(Duration::from_millis(3000));
        }
    }).unwrap();

    info!("tread tx is started");
    // thread::sleep(Duration::from_secs_f64(1.0));
    info!("main loop starting...");
    // let q = q1.lock().unwrap();
    while !cancel {        
        // info!("queue len: {:?}", q1.lock().unwrap().len());
        // let q = q1.lock().unwrap();
        rec.clear();
        for item in rx.iter() {
            rec.push(item);
            // thread::sleep(Duration::from_secs_f64(0.001));
        }
        info!("received rec: {:?}", rec);
        thread::sleep(Duration::from_millis(500));
    }
    handle.join().unwrap();
}