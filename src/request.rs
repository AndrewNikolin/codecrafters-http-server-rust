pub struct Request {
    pub(crate) method: String,
    path: String,
    pub(crate) path_parts: Vec<String>,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl Request {
    pub fn new(buf: &[u8; 1024]) -> Request {
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
}