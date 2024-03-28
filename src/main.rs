use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf).expect("Failed to read from client");

        if bytes_read == 0 {
            return;
        }

        let buf_str = std::str::from_utf8(&buf[0..bytes_read]).unwrap().to_lowercase();
        if buf_str.lines().filter(|line| { line.contains("ping")}).count() > 0 {
            stream.write_all(b"+PONG\r\n").expect("Failed to write to client");
            println!("+PONG\r\n");
        } else {
            println!("Failed to extract command.");
        }
    }
}
