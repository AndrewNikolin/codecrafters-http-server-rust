use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                _stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer);
                let request_parts: Vec<&str> = request.split_whitespace().collect();
                let path = request_parts[1];
                println!("Request: {}", path);
                if path != "/" {
                    _stream.write(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").unwrap();
                    continue;
                } else {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
