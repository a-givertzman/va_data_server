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
    }, 
    io::{
        BufReader, 
        BufRead, Read,
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
            // let buf_reader = BufReader::new(&mut stream);
            // let http_request: Vec<_> = buf_reader
            //     .lines()
            //     .map(|result| result.unwrap())
            //     .take_while(|line| !line.is_empty())
            //     .collect();
        
            // println!("Request: {:#?}", http_request);
            let string = String::from_utf8_lossy(&buf);
            // println!("buf: {:#?}", buf);
            println!("string: {:#?}", string);
            std::thread::sleep(self.reconnectDelay);
        }
    }
}