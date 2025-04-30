use std::collections::HashMap;

pub struct Response {
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut status_bytes = self.status.to_bytes();

        let mut buf = String::new();
        buf.push_str("\r\n");

        self.headers
            .iter()
            .map(|(k, v)| k.to_owned() + ": " + v + "\r\n")
            .for_each(|s| buf.push_str(&s));
        buf.push_str("\r\n");

        let mut resp = Vec::from(buf.as_bytes());

        resp.extend(&self.body);

        status_bytes.extend(resp);
        status_bytes
    }
}

pub struct Status {
    pub version: String,
    pub status_code: i16,
    pub status: String,
}

impl Status {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = String::new();

        buf.push_str("HTTP/");
        buf.push_str(&self.version);
        buf.push(' ');

        buf.push_str(&self.status_code.to_string());
        buf.push(' ');

        buf.push_str(&self.status);

        Vec::from(buf.as_bytes())
    }
}
