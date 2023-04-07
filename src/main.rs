#![allow(non_snake_case)]
mod circular_queue;

use std::time::Instant;

use circular_queue::CircularQueue;
use heapless::spsc::Queue;

const COUNT: usize = 1000;
const QSIZE: usize = 16_384;
const QSIZEADD: usize = QSIZE + 1;
fn main() {
    let mut queue: Queue<f64, QSIZEADD> = Queue::new();
    let mut buf = vec![0.0; QSIZE];
    {
        buf.fill(0.0);
        // println!("buf: {:?}", buf);
        let start = Instant::now();
        testQueue(&mut queue, &mut buf);
        println!("elapsed: {:?}", start.elapsed());
    }
    {
        buf.fill(0.0);
        // println!("buf: {:?}", buf);
        let mut cQueue: CircularQueue<f64> = CircularQueue::with_capacity(QSIZE);
        let start = Instant::now();
        testCQeque(&mut cQueue, &mut buf);
        println!("elapsed: {:?}", start.elapsed());
    }
    {
        buf.fill(0.0);
        // println!("buf: {:?}", buf);
        let mut cQueue: CircularQueue<f64> = CircularQueue::with_capacity(QSIZE);
        let start = Instant::now();
        testCQeque1(&mut cQueue, &mut buf);
        println!("elapsed: {:?}", start.elapsed());
    }
    {
        buf.fill(0.0);
        // println!("buf: {:?}", buf);
        let mut cQueue: CircularQueue<f64> = CircularQueue::with_capacity(QSIZE);
        let start = Instant::now();
        testCQeque2(&mut cQueue, &mut buf);
        println!("elapsed: {:?}", start.elapsed());
    }
}


fn testQueue(queue: &mut Queue<f64, QSIZEADD>, buf: &mut Vec<f64>) {
    // let mut cloned: Queue<f64, 8>;
    for i in 0..COUNT {
        match queue.enqueue(i as f64) {
            Ok(_) => {},
            Err(val) => {
                println!("error adding value: {:?}", val);
            },
        };
        
        for (i, item) in queue.iter().enumerate() {
            buf[i] = *item;
            // println!("readed buf: {:?}\t{:?}", i, item);
        }
        if queue.is_full() {
            queue.dequeue().unwrap();
            // println!("dequeue: {:?}", d);
        }
        // println!("readed buf: {:?}", &buf);
        // println!("queue: {:?}", &queue);
        // let oldest = queue.peek();
        // println!("readed: {:?}", oldest);    
    }
}

fn testCQeque(cQueue: &mut CircularQueue<f64>, buf: &mut [f64]) {
    // let mut cloned: Queue<f64, 8>;
    for i in 0..COUNT {
        cQueue.push(i as f64);
        
        for (i, item) in cQueue.iter().enumerate() {
            buf[i] = *item;
            // println!("readed buf: {:?}\t{:?}", i, item);
        }
        // println!("readed: {:?}", &buf);
        // println!("queue: {:?}", &cQueue);
    }
}

fn testCQeque1(cQueue: &mut CircularQueue<f64>, buf: &mut Vec<f64>) {
    // let mut cloned: Queue<f64, 8>;
    for i in 0..COUNT {
        cQueue.push(i as f64);
        buf.clear();
        buf.append(
            &mut Vec::<f64>::from(cQueue.buffer())
        );
        // cQueue.buffer().to_owned().clone();
        // println!("readed: {:?}", &buf);
        // println!("queue: {:?}", &cQueue);
    }
}

fn testCQeque2(cQueue: &mut CircularQueue<f64>, buf: &mut Vec<f64>) {
    // let mut cloned: Queue<f64, 8>;
    for i in 1..COUNT {
        cQueue.push(0.0);
    }
    for i in 0..COUNT {
        cQueue.push(i as f64);
        println!("readed: {:?}   {:?}", buf.len(), cQueue.buffer().len());
        // buf.clone_from_slice(
        //     cQueue.buffer()
        // );
        // cQueue.buffer().to_owned().clone();
        // println!("readed: {:?}", &buf);
        // println!("queue: {:?}", &cQueue);
    }
}