use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                _stream.read(&mut buffer).unwrap();
                let request = parse_request(&buffer);

                let response = match request.path_parts.first().unwrap().as_str() {
                    "echo" => echo(request),
                    "" => Response {
                        status_code: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: b"Hello, World!".to_vec(),
                    },
                    _ => Response {
                        status_code: 404,
                        status_text: "Not Found".to_string(),
                        headers: vec![],
                        body: vec![],
                    },
                };

                let response_bytes = response.to_bytes();
                _stream.write(&response_bytes).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn echo(request: Request) -> Response {
    let content_length = request.path_parts[1].len();
    let content_type_header = ("Content-Type".to_string(), "text/plain".to_string());
    let content_length_header = ("Content-Length".to_string(), content_length.to_string());
    Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: vec![content_type_header, content_length_header],
        body: request.path_parts[1].as_bytes().to_vec(),
    }
}

fn parse_request(buf: &[u8; 1024]) -> Request {
    let request = String::from_utf8_lossy(buf);

    let request = request.trim().split("\r\n").next().unwrap();
    let request_parts: Vec<&str> = request.split_whitespace().collect();

    let method = request_parts[0].to_string();

    let path = request_parts[1].to_string().trim().to_string();
    let path_parts = path
        .split("/")
        .map(|s| s.to_string())
        .filter(|s| s != "")
        .collect();

    Request {
        method,
        path: path.clone(),
        path_parts,
    }
}

struct Request {
    method: String,
    path: String,
    path_parts: Vec<String>,
}

struct Response {
    status_code: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Response {
    fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));
        response.into_bytes()
    }
}
