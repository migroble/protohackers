use std::io::{prelude::*, BufReader, Result};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

const PORT: u16 = 10420;

enum Response {
    Ok(bool),
    Malformed(&'static str),
}

impl Response {
    fn to_str(&self) -> String {
        match self {
            Self::Ok(prime) => json::stringify(json::object! { method: "isPrime", prime: *prime }),
            Self::Malformed(reason) => reason.to_string(),
        }
    }
}

fn is_prime(n: u64) -> bool {
    println!("Prime test: {}", n);

    if n <= 1 {
        return false;
    } else if n == 2 || n == 3 {
        return true;
    } else if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    for i in (5..(n as f64).sqrt() as u64).step_by(6) {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
    }

    true
}

fn handle(mut stream: TcpStream) -> Result<()> {
    let input = BufReader::new(stream.try_clone()?);

    for line in input.lines() {
        let line = line?;
        println!("Received: {}", &line);

        let res = if let Ok(obj) = json::parse(&line) {
            if obj["method"].as_str() == Some("isPrime") {
                if let Some(n) = obj["number"].as_number() {
                    let n: f64 = n.into();
                    let rem = n % 1.0;
                    let prime = if rem == 0.0 {
                        is_prime(n as u64)
                    } else {
                        false
                    };

                    Response::Ok(prime)
                } else {
                    Response::Malformed("No number")
                }
            } else {
                Response::Malformed("Invalid method")
            }
        } else {
            Response::Malformed("Invalid JSON")
        };

        stream.write_all(res.to_str().as_bytes())?;
        stream.write(&['\n' as u8])?;
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
