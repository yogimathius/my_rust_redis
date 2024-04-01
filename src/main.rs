mod resp;
mod server;
use resp::Value;
use clap::Parser as ClapParser;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use server::{Role, Server};


#[derive(ClapParser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u16,
    #[arg(short, long, value_delimiter = ' ', num_args = 2)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    // let port = 6379;
    let args = Args::parse();

    let listener = TcpListener::bind(("127.0.0.1", args.port))
        .await
        .unwrap();

    println!("Listening on Port {}", args.port);

    let server = Server::new(match args.replicaof {
        Some(_) => Role::Slave { host: "localhost".to_string(), port: 6379 },
        None => Role::Master,
    });
    match args.replicaof {
        Some(vec) => {
            let mut iter = vec.into_iter();
            let addr = iter.next().unwrap();
            let port = iter.next().unwrap();
            let stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();
            send_handhshake(stream, &server).await.unwrap();
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

async fn send_handhshake(mut stream: TcpStream, server: &Server) -> Result<()> {
    let msg = server.ping().unwrap();
    stream.write_all(msg.serialize().as_bytes()).await?;
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

fn extract_command(value: Value) -> Result<(String, Vec<Value>)>{
    match value {
        Value::Array(a) => {
            Ok((
                unpack_bulk_str(a.first().unwrap().clone())?,
                a.into_iter().skip(1).collect(),
            ))
        },
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) ->  Result<String>{
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected bulk string")),

    }
}