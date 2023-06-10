use std::io::{prelude::*, Result};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::thread;

const PORT: u16 = 10420;

fn to_hex(data: &[u8]) -> String {
    data.iter().map(|d| format!("{d:X}")).collect()
}

fn handle(mut stream: TcpStream) -> Result<()> {
    let mut data = Vec::new();
    stream.read_to_end(&mut data)?;

    println!(
        "Received {} bytes: {:?}",
        data.len(),
        str::from_utf8(&data).unwrap_or(&to_hex(&data))
    );

    stream.write_all(&data)
}

fn main() -> Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], PORT)))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle(stream) {
                        println!("Error handling connection: {:?}", e)
                    }
                });
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
