mod handler;
mod middleware;
mod request;
mod response;

use clap::{arg, Parser};
use handler::*;
use middleware::gzip_compress_middleware;
use request::Request;
use std::net::TcpListener;
use std::thread;
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

    let mut should_close = false;
    while !should_close {
        let request = Request::parse(&mut reader)?;

        let handler = router(&request, args);
        let wrapped_handler = apply_middleware(handler);

        let mut response =
            wrapped_handler(&request).unwrap_or_else(|_| unknwon_handler(&request).unwrap());

        should_close = request
            .headers
            .get("Connection".to_lowercase().as_str())
            .map_or(false, |s| s == "close");

        if should_close {
            (&mut response)
                .headers
                .insert(String::from("Connection"), String::from("close"));
        }

        stream.write_all(&response.to_bytes())?;
    }

    Ok(())
}

fn router(request: &Request, args: &Args) -> Box<Handler> {
    match (
        request.start_line.method.as_str(),
        request.start_line.path.as_str(),
    ) {
        ("GET", "/") => Box::new(default_handler),
        ("GET", s) if s.starts_with("/echo") => Box::new(echo_handler),
        ("GET", u) if u.starts_with("/user-agent") => Box::new(user_agent_handler),
        ("GET", f) if f.starts_with("/files") => Box::new(files_read_handler_provider(args)),
        ("POST", f) if f.starts_with("/files") => Box::new(files_write_handler_provider(args)),
        _ => Box::new(unknwon_handler),
    }
}

fn apply_middleware(handler: Box<Handler>) -> Box<Handler> {
    let middlewares = vec![gzip_compress_middleware];
    middlewares.iter().rev().fold(handler, |acc, m| m(acc))
}
