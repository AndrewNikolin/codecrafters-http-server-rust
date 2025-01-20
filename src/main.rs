mod threadpool;
mod request;
mod response;

use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;
use std::net::TcpStream;
use request::Request;
use response::Response;

static mut DIRECTORY: &str = "public";
fn main() {
    let params: Vec<String> = std::env::args().collect();
    if params.len() > 2 {
        let directory = params[2].clone();
        unsafe {
            DIRECTORY = Box::leak(Box::new(directory));
        }
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let thread_pool = threadpool::ThreadPool::new(5);

    println!("Running on http://127.0.0.1:4221");
    println!("Test with http://127.0.0.1:4221/user-agent");

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
    let request = Request::new(&buffer);

    let mut response = match request
        .path_parts
        .first()
        .unwrap_or(&String::new())
        .as_str()
    {
        "user-agent" => user_agent(&request),
        "echo" => echo(&request),
        "files" => files(&request),
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

    if request
        .headers
        .iter()
        .any(|(key, _)| key.to_lowercase() == "Accept-Encoding".to_lowercase())
    {
        let accepted_encodings = request
            .headers
            .iter()
            .find(|(key, _)| key.to_lowercase() == "Accept-Encoding".to_lowercase())
            .unwrap()
            .1
            .split(", ")
            .collect::<Vec<&str>>();

        if accepted_encodings.contains(&"gzip") {
            response
                .headers
                .push(("Content-Encoding".to_string(), "gzip".to_string()));
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            let body_text = String::from_utf8_lossy(&response.body);
            _ = encoder.write_all(body_text.as_bytes());
            response.body = encoder.finish().unwrap();

            let content_length = response.body.len();

            if let Some((_, content_length_header)) = response
                .headers
                .iter_mut()
                .find(|(key, _)| key.to_lowercase() == "Content-Length".to_lowercase())
            {
                *content_length_header = content_length.to_string();
            } else {
                response
                    .headers
                    .push(("Content-Length".to_string(), content_length.to_string()));
            }
        }
    }

    let response_bytes = response.to_bytes();
    _ = stream.write(&response_bytes).unwrap();
}

fn user_agent(request: &Request) -> Response {
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

fn echo(request: &Request) -> Response {
    let echo = &request.path_parts[1];

    if echo == "echo" {
        panic!("Infinite loop detected");
    }

    let content_length = echo.len();
    let content_type_header = ("Content-Type".to_string(), "text/plain".to_string());
    let content_length_header = ("Content-Length".to_string(), content_length.to_string());
    Response {
        status_code: 200,
        status_text: "OK".to_string(),
        headers: vec![content_type_header, content_length_header],
        body: echo.as_bytes().to_vec(),
    }
}

fn files(request: &Request) -> Response {
    match request.method.as_str() {
        "GET" => files_get(request),
        "POST" => files_post(request),
        _ => panic!("Unsupported method"),
    }
}

fn files_post(request: &Request) -> Response {
    let path = format!(
        "{}/{}",
        unsafe { DIRECTORY },
        request.path_parts.last().unwrap()
    );
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

fn files_get(request: &Request) -> Response {
    let path = format!(
        "{}/{}",
        unsafe { DIRECTORY },
        request.path_parts.last().unwrap()
    );
    println!("Serving file: {}", path);
    let content = std::fs::read(path);

    match content {
        Ok(content) => {
            let content_length = content.len();
            let content_type_header = (
                "Content-Type".to_string(),
                "application/octet-stream".to_string(),
            );
            let content_length_header = ("Content-Length".to_string(), content_length.to_string());
            Response {
                status_code: 200,
                status_text: "OK".to_string(),
                headers: vec![content_type_header, content_length_header],
                body: content,
            }
        }
        Err(_) => Response {
            status_code: 404,
            status_text: "Not Found".to_string(),
            headers: vec![],
            body: vec![],
        },
    }
}
