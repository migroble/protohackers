use std::collections::HashMap;
use std::io::{prelude::*, Result};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

const PORT: u16 = 10420;

#[derive(Debug)]
enum Message {
    Insert(i32, i32),
    Query(i32, i32),
    Invalid,
}

impl Message {
    fn from_bytes(bytes: [u8; 9]) -> Self {
        let v1 = i32::from_be_bytes(bytes[1..5].try_into().unwrap());
        let v2 = i32::from_be_bytes(bytes[5..9].try_into().unwrap());
        match bytes[0] as char {
            'I' => Self::Insert(v1, v2),
            'Q' => Self::Query(v1, v2),
            _ => Self::Invalid,
        }
    }
}

fn handle(mut stream: TcpStream) -> Result<()> {
    let mut prices = HashMap::new();

    loop {
        let mut packet = [0; 9];
        if let Err(_) = stream.read_exact(&mut packet) {
            break;
        }

        let msg = Message::from_bytes(packet);
        println!("{:?}", msg);

        match msg {
            Message::Insert(ts, price) => {
                prices.insert(ts, price);
            }
            Message::Query(min, max) => {
                let iter = prices
                    .iter()
                    .filter(|(ts, _)| min <= **ts && **ts <= max)
                    .map(|(_, price)| *price as i64);
                let n = iter.clone().count() as i64;

                let avg = if n == 0 { 0 } else { iter.sum::<i64>() / n };
                println!("Average: {}", avg);
                stream.write(&(avg as u32).to_be_bytes())?;
            }
            Message::Invalid => {}
        }
    }

    Ok(())
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
