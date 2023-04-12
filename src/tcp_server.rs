#![allow(non_snake_case)]

use log::{
    info,
    // trace,
    debug,
    warn,
};
use std::{
    net::{
        SocketAddr, 
        TcpStream, 
        TcpListener, 
        Shutdown,
    }, 
    io::{
        // BufReader, 
        // BufRead, 
        Read, Write,
    }, 
    sync::{
        Arc, 
        Mutex,
    },     
    thread,
    time::Duration,
    error::Error, 
};
use std::time::SystemTime;
use chrono::{
    DateTime,
    Utc,
    SecondsFormat,
};
use crate::{
    input_signal::PI2,
    ds_point::DsPoint,
};


const EOF: u8 = 4;


///
/// 
pub struct TcpServer {
    addr: SocketAddr,
    // stream: Option<TcpStream>,
    // listener: Option<TcpListener>,
    reconnectDelay: Duration,
    pub isConnected: bool,
    // cancel: bool,
}

impl TcpServer {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.parse().unwrap(),
            // stream: None,
            // listener: None,
            reconnectDelay: Duration::from_secs(3),
            isConnected: false,
            // cancel: false,
        }
    }
    pub fn run(this: Arc<Mutex<Self>>) -> Result<(), Box<dyn Error>> {
        debug!("[TcpServer] trying to open...");
        let mut listener: Option<TcpListener> = None;
        let mut tryAgain = 3;
        let addr = this.lock().unwrap().addr;
        let reconnectDelay = this.lock().unwrap().reconnectDelay;
        while tryAgain > 0 {
            debug!("[TcpServer] {:?} attempts left", tryAgain);
            listener = match TcpListener::bind(addr) {
                Ok(stream) => {
                    info!("[TcpServer] opened on: {:?}\n", addr);
                    tryAgain = -1;
                    Some(stream)
                },
                Err(err) => {
                    debug!("[TcpServer] binding error on: {:?}\n\tdetailes: {:?}", addr, err);
                    std::thread::sleep(reconnectDelay);
                    None
                },
            };
            tryAgain -= 1;
        };
        debug!("[TcpServer] listening for incoming clients");
        match listener {
            Some(listener) => {
                for result in listener.incoming() {
                    let mut stream = result.unwrap();
                    let mut streamSend = stream.try_clone().unwrap();
                    let me = this.clone();
                    info!("[TcpServer] incoming connection: {:?}", stream.peer_addr());
                    Some(
                        thread::Builder::new().name("TcpServer tread".to_string()).spawn(move || {
                            debug!("[TcpServer] started in {:?}", thread::current().name().unwrap());
                            me.lock().unwrap().listenStream(&mut stream);

                        })?
                    );        
                    this.lock().unwrap().sendToConnection(&mut streamSend);
                    // this.lock().unwrap().handleConnection(streamSend)?;
                }
            },
            None => {
                warn!("[TcpServer] connection failed");
            },
        };
        Ok(())
    }
    ///
    /// 
    // fn handleConnection(&mut self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    //     // let mut s1 = Arc::new(Mutex::new(stream.try_clone().unwrap()));
    //     Ok(())
    // }
    ///
    /// 
    fn buildPoint(&self, name: &str, value: f64) -> DsPoint<f64> {
        DsPoint {
            class: String::from("commonCmd"),
            datatype: String::from("real"),
            name: format!("/line1/ied12/db902_panel_controls/{}", name.to_owned()),
            value,
            status: 0,
            timestamp: DateTime::<Utc>::from(SystemTime::now()).to_rfc3339_opts(SecondsFormat::Micros, true),
        }
    }
    ///
    /// Sending messages to remote client
    fn sendToConnection(&mut self, stream: &mut TcpStream) {
        debug!("[TcpServer] start to sending messages...");
        let len = 4096;
        let delay = 1.0 / (len as f64);
        let mut i = 0;
        let mut phi = 0.0;
        println!("sending delay: {:#?}", delay);
        let now: DateTime<Utc> = SystemTime::now().into();
        println!("first: {:?}", now.to_rfc3339_opts(SecondsFormat::Micros, true));
        let mut points;
        let mut errHappen = false;
        loop {
            // println!("buf: {:#?}", buf);
            points = vec![
                // self.buildPoint("Platform.i", i as f64),
                self.buildPoint("Platform.phi", phi),
                self.buildPoint(
                    "Platform.sin", 
                    100.0 * (1.0 * phi).sin()
                    + 100.0 * (2.0 * phi).sin()
                ),
            ];
            for point in points {
                // debug!("sending point: {:#?}", point);
                let jsonString = point.toJson();
                errHappen = false;
                match jsonString {
                    Ok(value) => {
                        match Self::writeToTcpStream(stream, value.as_bytes()) {
                            Ok(_) => {},
                            Err(_) => {
                                errHappen = true;
                                break;
                            },
                        };
                        match Self::writeToTcpStream(stream, &[EOF]) {
                            Ok(_) => {},
                            Err(_) => {
                                errHappen = true;
                                break;
                            },
                        };
                    },
                    Err(err) => {
                        warn!("error converting point to json: {:?},\n\tdetales: {:?}", point, err);
                    },
                }
                if errHappen { break };
            }
            if errHappen { break };
            i = (i + 1) % len;
            phi = PI2 * (i as f64) / (len as f64);
            thread::sleep(Duration::from_secs_f64(delay));
        }
        match stream.shutdown(Shutdown::Both) {
            Ok(_) => {
                warn!("[TcpServer] sendToConnection stream.shutdown done");
            },
            Err(err) => {
                warn!("[TcpServer] sendToConnection stream.shutdown error: {:?}", err);
            },
        };
        warn!("[TcpServer] sendToConnection exit");
    }
    ///
    /// 
    fn writeToTcpStream(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        match stream.write(bytes) {
            Ok(_) => Ok(()),
            Err(err) => {
                warn!("[TcpStream] write error, data: {:?},\n\tdetales: {:?}", bytes, err);
                Err(Box::new(err))
            },
        }
    }
    ///
    /// Listening incoming messages from remote client
    fn listenStream(&mut self, stream: &mut TcpStream) {
        debug!("[TcpServer] start to reading messages...");
        let mut cancel = false;
        while !cancel {
            let mut buf = [0; 256];
            match stream.read(&mut buf) {
                Ok(bytesRead) => {
                    debug!("[TcpServer] bytes read: {:#?}", bytesRead);
                    cancel = bytesRead <= 0;
                },
                Err(err) => {
                    warn!("[TcpServer] TcpStream read error: {:#?}", err);
                    cancel = true;
                },
            };
            // debug!("[TcpServer] buf: {:#?}", buf);
            let parts = buf.split(|b| {*b == EOF});
            let bytes: Vec<_> = parts.take(1).collect();
            // debug!("[TcpServer] bytes: {:#?}", bytes[0]);
            let point = DsPoint::<f64>::fromBytes(bytes[0]);
            debug!("[TcpServer] received point: {:#?}", point);
            thread::sleep(self.reconnectDelay);
            if cancel { break };
        }
        warn!("[TcpServer] listenStream exit");
    }
}