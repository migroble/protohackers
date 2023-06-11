use std::collections::HashMap;
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::str;

const PORT: u16 = 10420;
const MAX_REQ_SIZE: usize = 1000;

fn main() -> Result<()> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], PORT)))?;
    let mut data = HashMap::new();
    data.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

    loop {
        let mut req = [0; MAX_REQ_SIZE];
        let (count, src) = socket.recv_from(&mut req)?;

        let req = str::from_utf8(&req[..count]).unwrap();
        println!("Request: {}", req);

        let mut split = req.splitn(2, '=');
        let key = split.next().unwrap();
        let value = split.next();

        if let Some(value) = value {
            if key != "version" {
                data.insert(key.to_string(), value.to_string());
            }
        } else {
            socket.send_to(
                format!("{}={}", key, data.get(key).unwrap_or(&"".to_string())).as_bytes(),
                &src,
            )?;
        }
    }
}
