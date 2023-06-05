#![allow(non_snake_case)]



pub mod udp_server {
    use concurrent_queue::ConcurrentQueue;
    use log::{
        info,
        // trace,
        debug,
        warn,
    };
    use std::{
        net::UdpSocket, 
        time::Duration, 
        sync::{Arc, Mutex}, 
        thread::{self, JoinHandle},
    };

    pub(crate) const SYN: u8 = 22;
    pub(crate) const EOT: u8 = 4;
    // pub(crate) const QSIZE: usize = 512;
    pub(crate) const UDP_HEADER_SIZE: usize = 4;
    pub(crate) const UDP_BUF_SIZE: usize = 1024 + UDP_HEADER_SIZE;
    pub(crate) const MAX_QUEUE_SIZE: usize = 1024;
    
    pub struct UdpServer {
        handle: Option<JoinHandle<()>>,
        localAddr: String, //SocketAddr,
        remoteAddr: String, //SocketAddr,
        reconnectDelay: Duration,
        pub isConnected: bool,
        cancel: bool,
        restart: bool,
        sender: Arc<ConcurrentQueue<[u8; UDP_BUF_SIZE]>>,
        pub receiver: Arc<ConcurrentQueue<[u8; UDP_BUF_SIZE]>>,
    }
    
    impl UdpServer {
        ///
        pub fn new(
            localAddr: &str,
            remoteAddr: &str,
            reconnectDelay: Option<Duration>,
        ) -> Self {
            let sender = Arc::new(ConcurrentQueue::bounded(MAX_QUEUE_SIZE)); 
            Self {
                handle: None,
                localAddr: String::from(localAddr),
                remoteAddr: String::from(remoteAddr),
                reconnectDelay: match reconnectDelay {Some(rd) => rd, None => Duration::from_secs(3)},
                isConnected: false,
                cancel: false,
                restart: false,
                // queue: queue, //heapless::spsc::Queue::new(),
                sender: sender.clone(),
                receiver: sender,
            }
        }
        ///
        ///
        pub fn restart(&mut self) {
            const logLoc: &str = "[UdpServer.restart]";
            debug!("{} started...", logLoc);
            self.restart = true;
            self.cancel = true;
            debug!("{} done", logLoc);
        }
        ///
        pub fn run(this: Arc<Mutex<Self>>) -> () {
            const logLoc: &str = "[UdpServer.run]";
            debug!("{} starting...", logLoc);
            info!("{} enter", logLoc);
            let me = this.clone();
            let me1 = this.clone();
            let localAddr = this.lock().unwrap().localAddr.clone();
            let remoteAddr = this.lock().unwrap().remoteAddr.clone();
            let reconnectDelay = this.lock().unwrap().reconnectDelay;
            let handle = thread::Builder::new().name("UdpServer tread".to_string()).spawn(move || {
                debug!("{} started in {:?}", logLoc, thread::current().name().unwrap());
                info!("{} started", logLoc);
                while !(this.lock().unwrap().cancel) {
                    info!("{} try to bind on: {:?}", logLoc, localAddr.clone());
                    let result = match UdpSocket::bind(localAddr.clone()) {
                        Ok(socket) => {
                            info!("{} ready on: {:?}\n", logLoc, localAddr);
                            this.lock().unwrap().isConnected = true;
                            info!("{} isConnected: {:?}\n", logLoc, this.lock().unwrap().isConnected);
                            let mut udpBuf = [0; UDP_BUF_SIZE];
                            let handshake = Self::handshake();
                            info!("{} sending handshake({}): {:?}", logLoc, handshake.len(), handshake);
                            match socket.send_to(&handshake, remoteAddr.clone()) {
                                Ok(_) => {
                                    info!("{} handshake done\n", logLoc);
                                },
                                Err(err) => {
                                    warn!("{} send error: {:#?}", logLoc, err);
                                },
                            };
                            socket.set_read_timeout(Some(Duration::from_millis(3000))).unwrap();
                            let mut cancel = match this.try_lock() {
                                Ok(m) => {
                                    m.cancel
                                }
                                Err(_) => {
                                    false
                                }
                            };
                            while !cancel {
                                // debug!("{} reading from udp socket...", logLoc);
                                match socket.recv_from(&mut udpBuf) {
                                    Ok((_amt, _src)) => {
                                        // debug!("{} receaved bytes({}) from{:?}: {:?}", logLoc, _amt, _src, udpBuf);
                                        this.lock().unwrap().enqueue(udpBuf);
                                    },
                                    Err(err) => {
                                        warn!("{} read error: {:#?}", logLoc, err);
                                    },
                                };
                                // debug!("{} udp read done", logLoc);
                                // std::thread::sleep(Duration::from_millis(100));
                                cancel = match this.try_lock() {
                                    Ok(m) => {
                                        m.cancel
                                    }
                                    Err(_) => {
                                        false
                                    }
                                };
                            }
                            info!("{} exit read cycle", logLoc);
                            Some(socket)
                        }
                        Err(err) => {
                            me1.lock().unwrap().isConnected = false;
                            debug!("{} binding error on: {:?}\n\tdetailes: {:?}", logLoc, localAddr, err);
                            std::thread::sleep(reconnectDelay);
                            None
                        }
                    };
                    if this.lock().unwrap().restart {
                        info!("{} restart detected", logLoc);
                        match result {
                            Some(socket) => {
                                info!("{} trying to drop socket...", logLoc);
                                drop(socket);
                                info!("{} drop socket - done", logLoc);
                            },
                            None => {},
                        }
                        this.lock().unwrap().cancel = false;
                        this.lock().unwrap().restart = false;
                    } else {
                        info!("{} sleep reconnectDelay: {:?}", logLoc, reconnectDelay);
                        std::thread::sleep(reconnectDelay);    
                    }
                }
                info!("{} exit", logLoc);
                this.lock().unwrap().cancel = false;
            }).unwrap();
            me.lock().unwrap().handle = Some(handle);
            debug!("{} started\n", logLoc);
        }
        ///
        fn enqueue(&mut self, buf: [u8; UDP_BUF_SIZE]) {
            const logLoc: &str = "[UdpServer.enqueue]";
            // debug!("{} started..", logLoc);
            match self.sender.push(buf) {
                Ok(_) => {
                    // debug!("{} added to queue: {:?}", logLoc, buf);
                },
                Err(err) => {
                    warn!("{} error add to queue: {:?}", logLoc, err);
                },
            };
        }
        ///
        fn handshake() -> [u8; UDP_HEADER_SIZE] {
            [SYN, EOT, 0, 0]
        }
    }
}

