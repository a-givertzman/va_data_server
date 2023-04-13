#![allow(non_snake_case)]

mod circular_queue;
mod bq;

use crossbeam_queue::ArrayQueue;
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
// use heapless::spsc::{Queue, Producer, Consumer};
use bq::{BlockingQueue};

const QSIZE: usize = 16_384;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let length = 16_777_216;

    // heaplessQueue(length);
    heapRb(length);
    mpscChannel(length);
    blockingQueue(length);

}



// fn heaplessQueue<'a>(length: i32) where 'a: 'static{
//     info!("heapless Queue");
//     let cancel = Arc::new(Mutex::new(false));
//     let mut rec = vec![];

//     let queue = heapless::spsc::Queue::<i32, QSIZE>::new();
//     let arc = Arc::<Queue<i32, QSIZE>>::new(queue.clone());
//     // let queue = Arc::new(Mutex::new(heapless::spsc::Queue::<i32, QSIZE>::new()));
//     // let queueTx = Arc::clone(&queue);
//     // let queueRx = Arc::clone(&queue);
//     let r = arc.clone();
//     let arcRefTx: &'a Arc<Queue<i32, QSIZE>> = &r;
//     let arcRefRx = &arc;

//     let mut tx = Producer::<'static, i32, QSIZE> {rb: arcRefTx};
//     let mut rx = Consumer {rb: &queue};
//     // let (mut tx, mut rx) = queue.split();

//     let start = Instant::now();
    
//     let cancelTx = Arc::clone(&cancel);
//     let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
//         info!("tread tx is started");
//         // let mut tx = *queueTx.lock().unwrap();
//         info!("queue.lock() done");
//         // loop {            
//             for x in 0..length {
//                 let mut sent = false;
//                 while !sent {
//                     match tx.enqueue(x) {
//                         Ok(_) => {
//                             sent = true;
//                             // debug!("[tread tx] sent: {:?}", x);
//                         },
//                         Err(_) => {
//                             thread::sleep(Duration::from_micros(100));
//                             // warn!("[tread tx] sending: {:?} error", err);
//                         },
//                     };
//                     // thread::sleep(Duration::from_millis(10));
//                 }
//             }
//             let mut c = cancelTx.lock().unwrap();
//             *c = true;
//             thread::sleep(Duration::from_millis(300));
//         // }
//     }).unwrap();

//     // info!("main loop starting...");
//     // let mut rx = queueRx.lock().unwrap();
//     while !(*cancel.lock().unwrap() && rx.len() == 0) {
//         if !(rx.len() == 0) {
//             // rec.clear();
//             for _ in 0..rx.len() {
//                 match rx.dequeue() {
//                     Some(item) => {
//                         rec.push(item);
//                     },
//                     None => {},
//                 }
//                 // thread::sleep(Duration::from_secs_f64(0.001));
//             }
//         // } else {
//             // thread::sleep(Duration::from_millis(50));
//         }
//     }
//     info!("elapsed: {:?}", start.elapsed());
//     // info!("received rec: {:?}", rec);
//     for x in 0..length {
//         if !rec.contains(&x) {
//             info!("missing value: {:?}", x);
//         }
//     }

//     handle.join().unwrap();    
// }


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
fn blockingQueue(length: i32) {
    info!("BlockingQueue");
    let cancel = Arc::new(Mutex::new(false));
    let mut rec = vec![];

    let share = Arc::new(BlockingQueue::<i32>::new());
    let tx = Arc::clone(&share);
    let rx = Arc::clone(&share);
    let start = Instant::now();
    
    let cancelTxArc = Arc::clone(&cancel);
    let handle = thread::Builder::new().name("tread tx".to_string()).spawn(move || {
        info!("tread tx is started");
        // loop {            
            for x in 0..length {
                let mut sent = false;
                while !sent {
                    tx.en_q(x)
                    // {
                    //     Ok(_) => {
                    //         sent = true;
                    //         // debug!("[tread tx] sent: {:?}", x);
                    //     },
                    //     Err(err) => {
                    //         thread::sleep(Duration::from_micros(100));
                    //         warn!("[tread tx] sending: {:?} error", err);
                    //     },
                    // };
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
            for _ in 0..rx.len() {
                rec.push(rx.de_q());
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
