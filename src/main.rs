// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        println!("Incoming connection");
        match stream {
            Ok(_stream) => {
                println!("+PONG\r\n");
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

        stream.write_all(&buf[0..bytes_read]).expect("Failed to write to client");
    }
}