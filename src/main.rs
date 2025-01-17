mod threadpool;

use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;
use std::net::TcpStream;

static mut DIRECTORY: &str = "/Users/andriinikolin";
fn main() {
    let params: Vec<String> = std::env::args().collect();
    if params.len() > 2 {
        let directory = params[2].clone();
        unsafe { DIRECTORY = Box::leak(Box::new(directory)); }
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let thread_pool = threadpool::ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");
    let mut buffer = [0; 1024];
    _ = stream.read(&mut buffer).unwrap();
    let request = parse_request(&buffer);

    let response = match request
        .path_parts
        .first()
        .unwrap_or(&String::new())
        .as_str()
    {
        "user-agent" => user_agent(request),
        "echo" => echo(request),
        "files" => files(request),
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
    _ = stream.write(&response_bytes).unwrap();
}

fn user_agent(request: Request) -> Response {
    let unknown = &"unknown".to_string();
    let user_agent = request
        .headers
        .iter()
        .find(|(key, _)| key.to_lowercase() == "user-agent")
        .map(|(_, value)| value)
        .unwrap_or(unknown);

    let content_length = user_agent.len();
    let content_type_header = ("Content-Type".to_string(), "text/plain".to_string());
    let content_length_header = ("Content-Length".to_string(), content_length.to_string());
    Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: vec![content_type_header, content_length_header],
        body: user_agent.as_bytes().to_vec(),
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

fn files(request: Request) -> Response {
    match request.method.as_str() {
        "GET" => files_get(request),
        "POST" => files_post(request),
        _ => panic!("Unsupported method"),
    }
}

fn files_post(request: Request) -> Response {
    let path = format!("{}/{}", unsafe { DIRECTORY }, request.path_parts.last().unwrap());
    println!("Writing file: {}", path);

    let mut file = std::fs::File::create(path).unwrap();
    _ = file.write(&request.body).unwrap();

    Response {
        status_code: 201,
        status_text: "Created".to_string(),
        headers: vec![],
        body: vec![],
    }
}

fn files_get(request: Request) -> Response {
    let path = format!("{}/{}", unsafe { DIRECTORY }, request.path_parts.last().unwrap());
    println!("Serving file: {}", path);
    let content = std::fs::read(path);

    match content {
        Ok(content) => {
            let content_length = content.len();
            let content_type_header = ("Content-Type".to_string(), "application/octet-stream".to_string());
            let content_length_header = ("Content-Length".to_string(), content_length.to_string());
            Response {
                status_code: 200,
                status_text: "OK".to_string(),
                headers: vec![content_type_header, content_length_header],
                body: content,
            }
        },
        Err(_) => Response {
            status_code: 404,
            status_text: "Not Found".to_string(),
            headers: vec![],
            body: vec![],
        }
    }
}

fn parse_request(buf: &[u8; 1024]) -> Request {
    let request = String::from_utf8_lossy(buf);

    let request = request
        .trim()
        .split("\r\n")
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<&str>>();
    let request_line = request[0].split(" ").collect::<Vec<&str>>();
    let method = request_line[0].to_string();
    let path = request_line[1].to_string();
    let path_parts = path
        .split("/")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let headers = request[1..]
        .iter()
        .filter(|line| line.contains(": "))
        .map(|line| {
            let parts = line.split(": ").collect::<Vec<&str>>();
            (parts[0].to_string(), parts[1].to_string())
        })
        .collect();

    let mut body: Vec<_> = request[1..]
        .iter()
        .filter(|line| !line.contains(": "))
        .flat_map(|line| line.as_bytes())
        .copied()
        .collect();
    while body.last() == Some(&0) {
        body.pop();
    }

    Request {
        method,
        path: path.clone(),
        path_parts,
        headers,
        body,
    }
}

struct Request {
    method: String,
    path: String,
    path_parts: Vec<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
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
