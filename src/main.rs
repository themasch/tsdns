extern crate ini;

use ini::Ini;
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedReader;

use std::comm::{Sender, Receiver};

use std::io::Timer;
use std::time::Duration;

fn start_random_backend(rx: Receiver<(String, Sender<Option<String>>)>) {
    loop {
        let (host, answer) = rx.recv();
        let mut timer = Timer::new().unwrap();
        timer.sleep(Duration::milliseconds(1000));
        answer.send(Some(host + " found"));
    }
}

fn handle_new_client(mut stream: TcpStream, tx: Sender<(String, Sender<Option<String>>)>) {
    let mut reader = BufferedReader::new(stream);
    let req = reader.read_line();
    stream = reader.unwrap();
    let result = match req {
        Ok(host) => {
            let thost = host.as_slice().trim().to_string();
            let (mytx, myrx) = channel();
            tx.send((thost, mytx));
            let answer = myrx.recv();
            println!("answer: {}", answer);
            match answer {
                Some(address) => {
                    stream.write(address.as_bytes())
                },
                None => {
                    stream.write(b"404")
                }
            }

        },
        Err(msg) => {
            stream.write(format!("error: {}", msg).as_bytes())
        }
    };
    match result {
        Err(msg) => {
            println!("ERROR: {}", msg);
        }
        _ => ()
    };
}

fn main() {

    let mut config = Ini::load_from_file("config.ini");

    config.begin_section("config");
    let port: u16 = match from_str(config.get("port").as_slice()) {
        Some(p) => p,
        None => 41144
    };

    let host = config.get("hostname").as_slice();

    // start backend
    let (tx, rx) = channel();

    spawn(proc() {
        start_random_backend(rx);
    });

    let socket = TcpListener::bind(host, port);
    let mut acceptor = socket.listen();

    for stream in acceptor.incoming() {
        let ptx = tx.clone();
        match stream {
            Err(e) => {
                println!("connection failed: {}", e);
            },
            Ok(stream) => spawn(proc() {
                // connection succeeded
                handle_new_client(stream, ptx)
            })
        }
    }

    drop(acceptor);
}
