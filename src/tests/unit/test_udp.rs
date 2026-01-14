#[cfg(test)]
use std::net::UdpSocket;
use std::time::Duration;

#[test]
fn test_udp() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:15180")?;
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;
    println!("Bound to 127.0.0.1:15180, waiting for packets...");
    
    let mut buf = [0; 1500];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            println!("Received {} bytes from {}", len, addr);
            println!("First 16 bytes: {:?}", &buf[..16.min(len)]);
            println!("Header: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", 
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6]);
        }
        Err(e) => {
            println!("Error or timeout: {}", e);
        }
    }
    Ok(())
}
