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
                    self.handle_connection(stream);
                }
            },
            None => {
                warn!("[TcpServer] connection failed");
            },
        }
    }
    ///
    /// 
    fn handle_connection(&mut self, mut stream: TcpStream) {
        debug!("[TcpServer] trying to read bytes...");
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
            let mut point = DsPoint::fromBytes(bytes[0]);
            debug!("point: {:#?}", point);
            std::thread::sleep(self.reconnectDelay);
            point.value = point.value + 1;
            let jsonString = point.toJson();
            match jsonString {
                Ok(value) => {
                    stream.write(value.as_bytes()).unwrap();
                },
                Err(err) => {
                    warn!("error converting point to json: {:?},\n\tdetales: {:?}", point, err)
                },
            }
        }
    }
}


const EOF: u8 = 4;

#[derive(Debug, Deserialize, Serialize)]
struct DsPoint {
    class: String,
    #[serde(rename(deserialize = "type", serialize = "type"))]
    datatype: String,
    name: String,
    value: i64,
    status: i64,
    timestamp: String,
}
impl DsPoint {
    pub fn fromBytes(bytes: &[u8]) -> Self {
        let string = String::from_utf8_lossy(&bytes).into_owned();
        // debug!("string: {:#?}", string);
        // let eof = String::from_utf8_lossy(&[4]).into_owned();
        // println!("eof: {:#?}", eof);
        // let parts: Vec<&str> = string.split(&eof).collect();
        // debug!("parts: {:#?}", parts);
        // let pointJson = parts[0];
        let point: DsPoint = serde_json::from_str(&string).unwrap();
        // debug!("point: {:#?}", point);
        point
    }
    pub fn toJson(&self) -> Result<String, serde_json::error::Error>{
        let result = serde_json::to_string(&self);
        debug!("point: {:#?}", result);
        result
    }
}