use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    
    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, _)) => {

                tokio::spawn(async move {
                    handle_client(stream).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf).await.unwrap();

        if bytes_read == 0 {
            return;
        }

        let buf_str = std::str::from_utf8(&buf[0..bytes_read]).unwrap().to_lowercase();
        if buf_str.lines().filter(|line| { line.contains("ping")}).count() > 0 {
            stream.write_all(b"+PONG\r\n").await.expect("Failed to write to client");
            println!("+PONG\r\n");
        } else {
            println!("Failed to extract command.");
        }
    }
}
