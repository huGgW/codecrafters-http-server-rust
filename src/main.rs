#[allow(unused_imports)]
use std::net::TcpListener;
use std::{io::Write, net::TcpStream};

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
    write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?;
    stream.flush()?;

    Ok(())
}
