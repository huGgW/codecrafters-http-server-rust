use std::{collections::HashMap, io::{self, BufRead, BufReader}};

pub struct Request {
    pub start_line: StartLine,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn parse<R: io::Read>(reader: &mut BufReader<R>) -> Result<Request, std::io::Error> {
        let first_line = Request::read_line(reader)?;
        let start_line = StartLine::parse(first_line.as_str())?;

        let mut headers = HashMap::new();
        loop {
            let line = Request::read_line(reader)?;
            if line.is_empty() {
                break;
            }

            let args = line.split(": ").collect::<Vec<_>>();
            if let [key, val] = args[..] {
                // since headers are case-insensitive, we convert the key to lowercase for convinience
                _ = headers.insert(key.to_string().to_lowercase(), val.to_string());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not valid header",
                ));
            }
        }

        // TODO: implement reading body
        Ok(Request {
            start_line,
            headers,
            body: Vec::new(),
        })
    }

    fn read_line<R: io::Read>(reader: &mut BufReader<R>) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        reader.read_until(b'\n', &mut buf)?;
        if buf.get(buf.len() - 2).filter(|&s| *s == b'\r').is_none()
            || buf.last().filter(|&s| *s == b'\n').is_none()
        {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "not valid line seperator",
            ));
        }

        match String::from_utf8(buf) {
            Ok(s) => Ok(s.trim_end().to_string()),
            Err(e) => Err(std::io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

pub struct StartLine {
    pub method: String,
    pub path: String,
    pub version: String,
}

impl StartLine {
    pub fn parse(s: &str) -> Result<StartLine, io::Error> {
        let args = s.split(' ').collect::<Vec<_>>();
        if args.len() != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "not valid start line",
            ));
        }

        let method = *args.first().unwrap();
        let path = *args.get(1).unwrap();

        let version_vec = args.get(2).unwrap().split('/').collect::<Vec<_>>();
        let version = match version_vec[..] {
            ["HTTP", v] => v,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not valid version",
                ))
            }
        };

        Ok(StartLine {
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
        })
    }
}
