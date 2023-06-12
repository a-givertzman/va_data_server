use log::{
    info,
    // trace,
    debug,
    warn,
};
use std::{
    net::{
        TcpStream, 
        SocketAddr,
    }, 
    io::Result, 
    error::Error, time::Duration,
};

pub struct TcpServer {
    addr: SocketAddr,
    stream: Option<TcpStream>,
    reconnectDelay: Duration,
    pub isConnected: bool,
}

impl TcpServer {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.parse().unwrap(),
            stream: None,
            reconnectDelay: Duration::from_secs(3),
            isConnected: false,
        }
    }
    pub fn run(&mut self) {
        let mut stream: Option<TcpStream> = None;
        let mut tryAgain = 3;
        while tryAgain > 0 {
            stream = match TcpStream::connect(self.addr) {
                Ok(stream) => {
                    info!("[TcpServer] connected on: {:?}", self.addr);
                    Some(stream)
                },
                Err(err) => {
                    debug!("[TcpServer] connect error: {:?}", err);
                    std::thread::sleep(self.reconnectDelay);
                    None
                },
            };
            tryAgain -= 1;
        }
        match stream {
            Some(stream) => {
                self.stream = Some(stream);
            },
            None => {
                warn!("[TcpServer] connection failed");
            },
        }
    }
}