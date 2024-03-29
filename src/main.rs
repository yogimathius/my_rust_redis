use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use crate::resp::Value;
mod resp;

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
    let mut handler = resp::RespHandler::new(stream);

    let mut buf = [0; 512];
    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
           let (command, args) = extract_command(value).unwrap();

           match command.as_str() {
            "ping" => Value::SimpleString("PONG".to_string()),
            "echo" => args.first().unwrap().clone(),
            _ => panic!("Cannot handle comman {}", command),
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