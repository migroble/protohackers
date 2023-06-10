use std::collections::HashMap;
use std::io::{prelude::*, BufReader, Result};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

const PORT: u16 = 10420;
const WELCOME_MSG: &'static str = "Welcome to budgetchat! What shall I call you?\n";
const INVALID_NAME_MSG: &'static str = "Invalid username. Disconnecting.";
const MAX_NAME_LEN: usize = 16;

fn handle(chat: Arc<Mutex<Chatroom>>, mut stream: TcpStream) -> Result<()> {
    stream.write(WELCOME_MSG.as_bytes())?;

    let mut name = [0; MAX_NAME_LEN];
    let count = stream.read(&mut name)?;

    let name = str::from_utf8(&name[..count])
        .unwrap_or("")
        .trim()
        .to_string();
    if name.len() == 0 || !name.chars().all(|c| c.is_alphanumeric()) {
        stream.write(INVALID_NAME_MSG.as_bytes())?;
        return Ok(());
    }

    let input = BufReader::new(stream.try_clone()?);

    chat.lock().unwrap().start_session(name.clone(), stream);

    for line in input.lines() {
        let line = line?;
        chat.lock().unwrap().send_message(&name, &line);
    }

    chat.lock().unwrap().end_session(&name);

    Ok(())
}

struct Chatroom {
    sessions: HashMap<String, TcpStream>,
}

impl Chatroom {
    fn start_session(&mut self, name: String, mut stream: TcpStream) {
        self.run(|_name, stream| {
            stream
                .write(format!("* {} has entered the room\n", &name).as_bytes())
                .unwrap();
        });

        stream
            .write(
                format!(
                    "* The room contains: {}\n",
                    self.sessions
                        .keys()
                        .map(|name| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .as_bytes(),
            )
            .unwrap();

        self.sessions.insert(name, stream);
    }

    fn end_session(&mut self, name: &str) {
        self.sessions.remove(name);

        self.run(|_name, stream| {
            stream
                .write(format!("* {} has left the room\n", &name).as_bytes())
                .unwrap();
        });
    }

    fn send_message(&mut self, name: &str, message: &str) {
        let name = name.clone();

        self.run(|sname, stream| {
            if name != sname {
                stream
                    .write(format!("[{}] {}\n", &name, &message).as_bytes())
                    .unwrap();
            }
        });
    }

    fn run<F: Fn(&str, &mut TcpStream)>(&mut self, f: F) {
        self.sessions
            .iter_mut()
            .for_each(|(name, stream)| f(name, stream));
    }
}

fn main() -> Result<()> {
    let chat = Arc::new(Mutex::new(Chatroom {
        sessions: HashMap::new(),
    }));

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], PORT)))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let chat = Arc::clone(&chat);
                thread::spawn(move || {
                    if let Err(e) = handle(chat, stream.try_clone().unwrap()) {
                        println!("Error handling connection: {:?}", e)
                    }
                });
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
