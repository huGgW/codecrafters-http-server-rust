use std::io;
use std::net::TcpListener;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                if let Err(e) = handle_connection(&mut stream) {
                    println!("handle connection error: {}", e);
                }

                if let Err(e) = stream.shutdown(std::net::Shutdown::Both) {
                    println!("shutdown error: {}", e);
                }
                println!("connection closed");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let read_stream = stream.try_clone()?;
    let mut reader = BufReader::new(read_stream);

    let req_line = read_request(&mut reader)?;
    let handler = router(&req_line);

    handler(&req_line, stream)
}

fn read_request(reader: &mut BufReader<TcpStream>) -> Result<String, std::io::Error> {
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    if buf.get(buf.len() - 2).filter(|&s| *s == b'\r').is_none() {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "not valid line seperator",
        ));
    }

    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(e) => Err(std::io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

fn router(req_line: &str) -> fn(&str, &mut TcpStream) -> Result<(), std::io::Error> {
    let req_line_args = req_line.split(' ').collect::<Vec<_>>();

    // if not GET, 404
    if req_line_args.first().filter(|&&s| s == "GET").is_none() {
        return unknwon_handler;
    }

    // if not HTTP, 404
    if req_line_args
        .last()
        .filter(|&s| s.starts_with("HTTP"))
        .is_none()
    {
        return unknwon_handler;
    }

    match req_line_args.get(1) {
        Some(&"/") => default_handler,
        Some(&s) if s.starts_with("/echo") => echo_handler,
        _ => unknwon_handler,
    }
}

fn echo_handler(req_line: &str, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let req_line_args = req_line.split(' ').collect::<Vec<_>>();
    let path = req_line_args.get(1).unwrap(); // should be already checked
    let echo_paths = path.split('/').skip(2).collect::<Vec<_>>();

    // we care only first element

    let echo_str = match echo_paths.first() {
        Some(&s) => s,
        None => {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "no str given to echo",
            ))
        }
    };

    stream.write_all(b"HTTP/1.1 200 OK\r\n")?;
    stream.write_all(b"Content-Type: text/plain\r\n")?;
    stream.write_all(format!("Content-Length: {}\r\n", echo_str.len()).as_bytes())?;
    stream.write_all(b"\r\n")?;
    stream.write_all(echo_str.as_bytes())?;

    Ok(())
}

fn default_handler(_: &str, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;

    Ok(())
}

fn unknwon_handler(_: &str, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?;

    Ok(())
}
