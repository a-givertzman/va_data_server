#![allow(non_snake_case)]

mod circular_queue;

use log::{
    info, 
    debug, 
    warn
};
use std::{
    thread, 
    sync::{Mutex, Arc, mpsc}, 
    time::{Duration, Instant}, 
    env
};
// use circular_queue::CircularQueue;
use ringbuf::HeapRb;
use crossbeam_channel::{bounded};
// use heapless::spsc::{Queue, Producer, Consumer};

const QSIZE: usize = 16_384 / 2;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let length = 16_777_216 * 4;

    heaplessQueue(length);      // 1.260381531s
    mpscChannel(length);        // 2.195514803s
    heapRb(length);             // 6.042734658s
    crossbeamChannel(length);   // 8.248474629s

}



fn heaplessQueue<'a>(length: i32) {
    info!("heapless Queue");
    let cancel = Arc::new(Mutex::new(false));
    let mut rec = vec![];

    // let queue = heapless::spsc::Queue::<i32, QSIZE>::new();
    // let arc = Arc::<Queue<i32, QSIZE>>::new(queue.clone());
    static mut RB: heapless::spsc::Queue<i32, QSIZE> = heapless::spsc::Queue::<i32, QSIZE>::new();
    let queue = unsafe { &mut RB };
    let (mut tx, mut rx) = queue.split();

    let start = Instant::now();
    
    let cancelTx = Arc::clone(&cancel);
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        info!("tread tx is started");
        // let mut tx = queueTx.lock().unwrap();
        info!("queue.lock() done");
        // loop {            
            for x in 0..length {
                let mut sent = false;
                while !sent {
                    match tx.enqueue(x) {
                        Ok(_) => {
                            sent = true;
                            // debug!("[tread tx] sent: {:?}", x);
                        },
                        Err(_) => {
                            thread::sleep(Duration::from_micros(100));
                            // warn!("[tread tx] sending: {:?} error", err);
                        },
                    };
                    // thread::sleep(Duration::from_millis(10));
                }
            }
            let mut c = cancelTx.lock().unwrap();
            *c = true;
            thread::sleep(Duration::from_millis(300));
        // }
    }).unwrap();

    info!("main loop starting...");
    // let mut rx = queueRx.lock().unwrap();
    while !(*cancel.lock().unwrap() && rx.len() == 0) {
        if !(rx.len() == 0) {
            // rec.clear();
            for _ in 0..rx.len() {
                match rx.dequeue() {
                    Some(item) => {
                        // debug!("[tread rx] received: {:?}", item);
                        rec.push(item);
                    },
                    None => {},
                }
                // thread::sleep(Duration::from_secs_f64(0.001));
            }
        // } else {
            // thread::sleep(Duration::from_millis(100));
        }
        // thread::sleep(Duration::from_millis(100));
    }
    info!("elapsed: {:?}", start.elapsed());
    // info!("received rec: {:?}", rec);
    for x in 0..length {
        if rec[x as usize] != x {
            info!("missing value: {:?}", x);
        }
        // if !rec.contains(&x) {
        //     info!("missing value: {:?}", x);
        // }
    }

    handle.join().unwrap();    
}


///
/// 
fn heapRb(length: i32) {
    info!("ringbuf::HeapRb");
    let cancel = Arc::new(Mutex::new(false));
    let mut rec = vec![];
    let queue: HeapRb<i32> = HeapRb::<i32>::new(QSIZE);

    let (mut tx, mut rx) = queue.split();

    let start = Instant::now();
    
    let cancelTxArc = Arc::clone(&cancel);
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        info!("tread tx is started");
        // loop {            
            for x in 0..length {
                let mut sent = false;
                while !sent {
                    match tx.push(x) {
                        Ok(_) => {
                            sent = true;
                            // debug!("[tread tx] sent: {:?}", x);
                        },
                        Err(err) => {
                            thread::sleep(Duration::from_micros(100));
                            warn!("[tread tx] sending: {:?} error", err);
                        },
                    };
                    // thread::sleep(Duration::from_millis(10));
                }
            }
            let mut cancelTx = cancelTxArc.lock().unwrap();
            *cancelTx = true;
            thread::sleep(Duration::from_millis(300));
        // }
    }).unwrap();

    // info!("main loop starting...");
    while !(*cancel.lock().unwrap() && rx.is_empty()) {
        if !rx.is_empty() {
            // rec.clear();
            for item in rx.pop_iter() {
                rec.push(item);
                // thread::sleep(Duration::from_secs_f64(0.001));
            }
        // } else {
            // thread::sleep(Duration::from_millis(50));
        }
    }
    info!("elapsed: {:?}", start.elapsed());
    // info!("received rec: {:?}", rec);
    info!("verifing transmitted data...");
    for x in 0..length {
        if rec[x as usize] != x {
            info!("missing value: {:?}", x);
        }
        // if !rec.contains(&x) {
        //     info!("missing value: {:?}", x);
        // }
    }
    info!("verification done");

    handle.join().unwrap();    
}


///
/// 
fn mpscChannel(length: i32) {
    info!("mpscChannel");
    let cancel = Arc::new(Mutex::new(false));
    let mut rec = vec![];
    let (tx, rx) = mpsc::sync_channel(QSIZE);//  ::channel::<i32>();

    let start = Instant::now();
    
    let cancelTxArc = Arc::clone(&cancel);
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        info!("tread tx is started");
        // loop {            
            for x in 0..length {
                let mut sent = false;
                while !sent {
                    match tx.send(x) {
                        Ok(_) => {
                            sent = true;
                            // debug!("[tread tx] sent: {:?}", x);
                        },
                        Err(err) => {
                            thread::sleep(Duration::from_micros(100));
                            warn!("[tread tx] sending: {:?} error", err);
                        },
                    };
                    // thread::sleep(Duration::from_millis(10));
                }
            }
            let mut cancelTx = cancelTxArc.lock().unwrap();
            *cancelTx = true;
            thread::sleep(Duration::from_millis(300));
        // }
    }).unwrap();

    // info!("main loop starting...");
    while !(*cancel.lock().unwrap()) {
        // if !rx.is_empty() {
            // rec.clear();
            for item in rx.iter() {
                rec.push(item);
                // thread::sleep(Duration::from_secs_f64(0.001));
            }
        // } else {
            // thread::sleep(Duration::from_millis(50));
        // }
    }
    info!("elapsed: {:?}", start.elapsed());
    // info!("received rec: {:?}", rec);
    info!("verifing transmitted data...");
    for x in 0..length {
        if rec[x as usize] != x {
            info!("missing value: {:?}", x);
        }
        // if !rec.contains(&x) {
        //     info!("missing value: {:?}", x);
        // }
    }
    info!("verification done");

    handle.join().unwrap();    
}


///
/// 
fn crossbeamChannel(length: i32) {
    info!("crossbeamChannel");
    let cancel = Arc::new(Mutex::new(false));
    let mut rec = vec![];

    let (tx, rx) = bounded(QSIZE);
    let start = Instant::now();
    
    let cancelTxArc = Arc::clone(&cancel);
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        info!("tread tx is started");
        // loop {            
            for x in 0..length {
                match tx.send(x) {
                    Ok(_) => {
                        // debug!("[tread tx] sent: {:?}", x);
                    },
                    Err(err) => {
                        thread::sleep(Duration::from_micros(100));
                        warn!("[tread tx] sending: {:?} error", err);
                    },
                };
                // thread::sleep(Duration::from_millis(10));
            }
            let mut cancelTx = cancelTxArc.lock().unwrap();
            *cancelTx = true;
            thread::sleep(Duration::from_millis(300));
        // }
    }).unwrap();

    // info!("main loop starting...");
    while !(*cancel.lock().unwrap()) {
        // if !rx.is_empty() {
            // rec.clear();
            while !rx.is_empty() {
                match rx.try_recv() {
                    Ok(item) => {
                        rec.push(item);
                        // debug!("[tread rx] received: {:?}", item);
                    },
                    Err(err) => {
                        warn!("[tread rx] receiving error: {:?}", err);
                    },
                };
                // thread::sleep(Duration::from_secs_f64(0.001));
            }
        // } else {
            // thread::sleep(Duration::from_millis(50));
        // }
    }
    info!("elapsed: {:?}", start.elapsed());
    // info!("received rec: {:?}", rec);
    info!("verifing transmitted data...");
    for x in 0..rec.len() {
        if rec[x as usize] != (x as i32) {
            info!("missing value: {:?}", x);
        }
        // if !rec.contains(&x) {
        //     info!("missing value: {:?}", x);
        // }
    }
    info!("verification done");

    handle.join().unwrap();    
}
