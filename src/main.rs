mod request;
mod response;

use clap::{arg, Parser};
use request::Request;
use response::{Response, Status};
use std::collections::HashMap;
use std::net::TcpListener;
use std::{io, thread};
use std::{
    io::{BufReader, Write},
    net::TcpStream,
};

#[derive(Parser, Clone)]
struct Args {
    #[arg(long)]
    directory: Option<String>,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args = Args::parse();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut thread_handles = Vec::new();

    for stream in listener.incoming() {
        let args = args.clone();
        let handle = thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                if let Err(e) = handle_connection(&mut stream, &args) {
                    println!("handle connection error: {}", e);
                }

                println!("connection closed");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}

fn handle_connection(stream: &mut TcpStream, args: &Args) -> Result<(), std::io::Error> {
    let read_stream = stream.try_clone()?;
    let mut reader = BufReader::new(read_stream);
    let request = Request::parse(&mut reader)?;

    let handler = router(&request, &args);
    let response = handler(&request).unwrap_or_else(|_| unknwon_handler(&request).unwrap());

    stream.write_all(&response.to_bytes())?;
    Ok(())
}

fn router(
    request: &Request,
    args: &Args,
) -> Box<dyn FnOnce(&Request) -> Result<Response, std::io::Error>> {
    // if not GET, 404
    if request.start_line.method != "GET" {
        return Box::new(unknwon_handler);
    }

    match request.start_line.path.as_str() {
        "/" => Box::new(default_handler),
        s if s.starts_with("/echo") => Box::new(echo_handler),
        u if u.starts_with("/user-agent") => Box::new(user_agent_handler),
        f if f.starts_with("/files") => Box::new(files_handler_provider(args)),
        _ => Box::new(unknwon_handler),
    }
}

fn echo_handler(request: &Request) -> Result<Response, std::io::Error> {
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

    Ok(Response {
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
    })
}

fn user_agent_handler(request: &Request) -> Result<Response, std::io::Error> {
    let user_agent = match request.headers.get("User-Agent".to_lowercase().as_str()) {
        Some(s) => s,
        None => {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "not valid user agent header given",
            ))
        }
    };

    Ok(Response {
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
    })
}

fn files_handler_provider(
    args: &Args,
) -> impl FnOnce(&Request) -> Result<Response, std::io::Error> {
    let directory_arg = args.directory.clone();

    move |request: &Request| -> Result<Response, std::io::Error> {
        let directory = match directory_arg {
            Some(d) => d.clone(),
            None => {
                return Err(std::io::Error::new(
                    io::ErrorKind::NotFound,
                    "directory arg not given",
                ))
            }
        };

        let filename = match request.start_line.path[1..].split_once('/') {
            Some(("files", filename)) => filename,
            _ => {
                return Err(std::io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "file path is not valid",
                ))
            }
        };

        let file = std::fs::read(directory + filename)?;

        Ok(Response {
            status: Status {
                version: String::from("1.1"),
                status_code: 200,
                status: String::from("OK"),
            },
            headers: HashMap::from([
                (
                    String::from("Content-Type"),
                    String::from("application/octet-stream"),
                ),
                (String::from("Content-Length"), file.len().to_string()),
            ]),
            body: file,
        })
    }
}

fn default_handler(_: &Request) -> Result<Response, std::io::Error> {
    Ok(Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 200,
            status: "OK".to_string(),
        },
        headers: HashMap::new(),
        body: Vec::new(),
    })
}

fn unknwon_handler(_: &Request) -> Result<Response, std::io::Error> {
    Ok(Response {
        status: Status {
            version: "1.1".to_string(),
            status_code: 404,
            status: "Not Found".to_string(),
        },
        headers: HashMap::new(),
        body: Vec::new(),
    })
}
