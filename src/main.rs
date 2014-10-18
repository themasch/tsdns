extern crate ini;

use ini::Ini;
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedReader;

fn handle_request(host: &str, mut client: TcpStream) {
    println!("handling request for {}", host);
    client.write(b"127.0.0.1:9001\n");
}

fn handle_new_client(mut stream: TcpStream) {
    let mut reader = BufferedReader::new(stream);
    let req = reader.read_line();
    stream = reader.unwrap();
    match req {
        Ok(host) => handle_request(host.as_slice(), stream),
        Err(msg) => {
            stream.write(b"error");
        }
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

    let socket = TcpListener::bind(host, port);
    let mut acceptor = socket.listen();

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => {
                println!("connection failed: {}", e);
            },
            Ok(stream) => spawn(proc() {
                // connection succeeded
                handle_new_client(stream)
            })
        }
    }

    drop(acceptor);
}
