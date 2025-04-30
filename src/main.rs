mod request;
mod response;

use request::Request;
use response::{Response, Status};
use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::{
    io::{BufReader, Write},
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
    let request = Request::parse(&mut reader)?;

    let handler = router(&request);

    handler(&request, stream)
}

fn router(request: &Request) -> fn(&Request, &mut TcpStream) -> Result<(), std::io::Error> {
    // if not GET, 404
    if request.start_line.method != "GET" {
        return unknwon_handler;
    }

    match request.start_line.path.as_str() {
        "/" => default_handler,
        s if s.starts_with("/echo") => echo_handler,
        u if u.starts_with("/user-agent") => user_agent_handler,
        _ => unknwon_handler,
    }
}

fn echo_handler(request: &Request, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let echo_paths = request
        .start_line
        .path
        .split('/')
        .skip(2)
        .collect::<Vec<_>>();

    // we care only first element
    let echo_str = match echo_paths[..] {
        [s] => s,
        _ => {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidData,
                "not valid echo path given",
            ))
        }
    };

    let response = Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 200,
            status: "OK".to_string(),
        },
        headers: HashMap::from([
            (String::from("Content-Type"), String::from("text/plain")),
            (String::from("Content-Length"), echo_str.len().to_string()),
        ]),
        body: echo_str.as_bytes().to_vec(),
    };
    stream.write_all(&response.to_bytes())?;

    Ok(())
}

fn user_agent_handler(request: &Request, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let user_agent = match request.headers.get("User-Agent".to_lowercase().as_str()) {
        Some(s) => s,
        None => {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "not valid user agent header given",
            ))
        }
    };

    let response = Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 200,
            status: "OK".to_string(),
        },
        headers: HashMap::from([
            (String::from("Content-Type"), String::from("text/plain")),
            (String::from("Content-Length"), user_agent.len().to_string()),
        ]),
        body: user_agent.as_bytes().to_vec(),
    };
    stream.write_all(&response.to_bytes())?;

    Ok(())
}

fn default_handler(_: &Request, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let response = Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 200,
            status: "OK".to_string(),
        },
        headers: HashMap::new(),
        body: Vec::new(),
    };
    stream.write_all(&response.to_bytes())?;

    Ok(())
}

fn unknwon_handler(_: &Request, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let response = Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 404,
            status: "Not Found".to_string(),
        },
        headers: HashMap::new(),
        body: Vec::new(),
    };
    stream.write_all(&response.to_bytes())?;

    Ok(())
}
