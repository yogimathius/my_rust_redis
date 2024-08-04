mod resp;
mod server;
use clap::Parser as ClapParser;
use resp::Value;

use anyhow::Result;
use server::{Role, Server};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u16,
    #[arg(short, long, value_delimiter = ' ', num_args = 1)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listener = TcpListener::bind(("127.0.0.1", args.port)).await.unwrap();

    println!("Listening on Port {}", args.port);
    let server = Server::new(match args.replicaof.clone() {
        Some(vec) => {
            let mut iter = vec.into_iter();
            let addr = iter.next().unwrap();
            let port = iter.next().unwrap();
            Role::Slave {
                host: addr,
                port: port.parse::<u16>().unwrap(),
            }
        }
        None => Role::Master,
    });
    match args.replicaof {
        Some(vec) => {
            let mut iter = vec.into_iter();
            let addr = iter.next().unwrap();
            let port = iter.next().unwrap();
            let mut stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();
            send_handshake(&mut stream, &server).await.unwrap();

            let mut buffer = [0; 1024];

            match stream.read(&mut buffer).await {
                Ok(n) => {
                    if n == 0 {
                        println!("Connection closed by the server");
                    } else {
                        let response = String::from_utf8_lossy(&buffer[..n]);
                        let response_str = response.as_ref();
                        println!("Response: {}", response.trim() == "+PONG");
                        match response_str.trim() {
                            "+PONG" => {
                                println!("Replication established");
                                send_handshake_two(stream, &server).await.unwrap();
                            }
                            _ => {
                                println!("Failed to establish replication: {}", response);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {}", e);
                }
            }
        }
        None => {}
    }

    loop {
        let stream = listener.accept().await;
        let server = server.clone();

        match stream {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_client(stream, server).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn send_handshake(stream: &mut TcpStream, server: &Server) -> Result<()> {
    let msg = server.ping().unwrap();
    stream.write_all(msg.serialize().as_bytes()).await?;
    Ok(())
}

async fn send_handshake_two(mut stream: TcpStream, server: &Server) -> Result<()> {
    let replconf = server.replconf().unwrap();
    stream.write_all(replconf.serialize().as_bytes()).await?;
    Ok(())
}

async fn handle_client(stream: TcpStream, mut server: Server) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();

            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "get" => server.get(args),
                "set" => server.set(args),
                "INFO" => server.info(),
                _ => panic!("Cannot handle command {}", command),
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected bulk string")),
    }
}
