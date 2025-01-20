pub struct Response {
    pub(crate) status_code: u16,
    pub(crate) status_text: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl Response {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        let mut bytes = response.into_bytes();
        bytes.extend(&self.body);
        bytes
    }
}