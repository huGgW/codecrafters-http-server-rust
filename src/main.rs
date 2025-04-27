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

fn handle_connection(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let read_stream = stream.try_clone()?;
    let mut reader = BufReader::new(read_stream);
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    if !(buf.get(buf.len() - 2).is_some_and(|b| b.eq(&b'\r'))) {
        return handle_unknown(stream);
    }

    let req_line = String::from_utf8(buf)?;
    let req_line_args = req_line.split(' ').collect::<Vec<_>>();
    if !(req_line_args.first().is_some_and(|&s| s.eq("GET"))
        && req_line_args.get(1).is_some_and(|&s| s.eq("/")))
    {
        return handle_unknown(stream);
    }

    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
    Ok(())
}

fn handle_unknown(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?;

    Ok(())
}
