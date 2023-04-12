use log::{
    info,
    // trace,
    debug,
    warn,
};
use serde::{
    Serialize,
    Deserialize,
};
use std::{
    net::{
        SocketAddr, 
        TcpStream, 
        TcpListener,
    }, 
    io::{
        BufReader, 
        BufRead, Read, Write,
    }, 
    error::Error, 
    time::Duration,
};
use std::time::SystemTime;
use chrono::{
    DateTime,
    Utc,
    SecondsFormat,
};

use crate::input_signal::PI2;

pub struct TcpServer {
    addr: SocketAddr,
    stream: Option<TcpStream>,
    // listener: Option<TcpListener>,
    reconnectDelay: Duration,
    pub isConnected: bool,
}

impl TcpServer {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.parse().unwrap(),
            stream: None,
            // listener: None,
            reconnectDelay: Duration::from_secs(3),
            isConnected: false,
        }
    }
    pub fn run(&mut self) {
        let mut listener: Option<TcpListener> = None;
        let mut tryAgain = 3;
        while tryAgain > 0 {
            listener = match TcpListener::bind(self.addr) {
                Ok(stream) => {
                    info!("[TcpServer] opened on: {:?}\n", self.addr);
                    tryAgain = -1;
                    Some(stream)
                },
                Err(err) => {
                    debug!("[TcpServer] binding error on: {:?}\n\tdetailes: {:?}", self.addr, err);
                    std::thread::sleep(self.reconnectDelay);
                    None
                },
            };
            tryAgain -= 1;
        }
        match listener {
            Some(listener) => {
                for result in listener.incoming() {
                    let stream = result.unwrap();
                    info!("[TcpServer] incoming connection: {:?}", stream.peer_addr());
                    // stream.
                    self.handleConnection(stream);
                }
            },
            None => {
                warn!("[TcpServer] connection failed");
            },
        }
    }
    ///
    /// 
    fn handleConnection(&mut self, mut stream: TcpStream) {
        self.listenConnection(&mut stream);
        self.sendToConnection(&mut stream);
    }
    ///
    /// 
    fn buildPoint(&self, value: f64) -> DsPoint<f64> {
        DsPoint {
            class: String::from("commonCmd"),
            datatype: String::from("real"),
            name: String::from("/line1/ied12/db902_panel_controls/Platform.SensorMRU"),
            value: value,
            status: 0,
            timestamp: DateTime::<Utc>::from(SystemTime::now()).to_rfc3339_opts(SecondsFormat::Micros, true),
        }
    }
    ///
    /// Sending messages to remote client
    fn sendToConnection(&mut self, stream: &mut TcpStream) {
        debug!("[TcpServer] start to sending messages...");
        // stream.set_nonblocking(true).expect("set_nonblocking call failed");
        let delay = 1.0 / 16_384.0;
        let phi = 0.0;
        println!("sending delay: {:#?}", delay);
        let now: DateTime<Utc> = SystemTime::now().into();
        println!("first: {:?}", now.to_rfc3339_opts(SecondsFormat::Micros, true));
        let mut point = self.buildPoint(phi);
        loop {
            // println!("buf: {:#?}", buf);
            point = self.buildPoint(phi);
            debug!("sending point: {:#?}", point);
            let jsonString = point.toJson();
            match jsonString {
                Ok(value) => {
                    stream.write(value.as_bytes()).unwrap();
                },
                Err(err) => {
                    warn!("error converting point to json: {:?},\n\tdetales: {:?}", point, err)
                },
            }
            std::thread::sleep(Duration::from_secs_f64(delay));
        }
    }
    ///
    /// Listening incoming messages from remote client
    fn listenConnection(&mut self, stream: &mut TcpStream) {
        debug!("[TcpServer] start to reading messages...");
        // stream.set_nonblocking(true).expect("set_nonblocking call failed");
        loop {
            let mut buf = [0; 256];
            // let mut string = String::new();
            match stream.read(&mut buf) {
                Ok(bytesRead) => {
                    debug!("[TcpServer] bytes read: {:#?}", bytesRead);
                },
                Err(err) => {
                    warn!("[TcpServer] TcpStream read error: {:#?}", err);
                },
            };
            // println!("buf: {:#?}", buf);
            let parts = buf.split(|b| {*b == EOF});
            let bytes: Vec<_> = parts.take(1).collect();
            // debug!("bytes: {:#?}", bytes[0]);
            let point = DsPoint::<f64>::fromBytes(bytes[0]);
            debug!("received point: {:#?}", point);
            std::thread::sleep(self.reconnectDelay);
        }
    }
}


const EOF: u8 = 4;

#[derive(Debug, Deserialize, Serialize)]
struct DsPoint<T> {
    class: String,
    #[serde(rename(deserialize = "type", serialize = "type"))]
    datatype: String,
    name: String,
    value: T,
    status: i64,
    timestamp: String,
}
impl<'a, T> DsPoint<T> 
where
    for<'de> T: Deserialize<'de> + 'a,
    T: Serialize + 'a,
{
    pub fn fromBytes(bytes: &[u8]) -> Self {
        let string = String::from_utf8_lossy(&bytes).into_owned();
        debug!("string: {:#?}", string);
        // let eof = String::from_utf8_lossy(&[4]).into_owned();
        // println!("eof: {:#?}", eof);
        // let parts: Vec<&str> = string.split(&eof).collect();
        // debug!("parts: {:#?}", parts);
        // let pointJson = parts[0];
        let point: DsPoint<T> = serde_json::from_str(&string).unwrap();
        // debug!("point: {:#?}", point);
        point
    }
    pub fn toJson(&self) -> Result<String, serde_json::error::Error>{
        let result = serde_json::to_string(&self);
        debug!("point: {:#?}", result);
        result
    }
}


// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type")]
// #[serde(rename_all = "lowercase")]
// enum Item {
//     bool {
//         #[serde(default)]
//         value: i32,
//     },
//     int {
//         #[serde(default)]
//         value: i32,
//     },
//     real {
//         #[serde(default)]
//         value: f64,
//     },
// }
