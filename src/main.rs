use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use std::collections::HashMap;
use resp::Value;
use std::sync::Mutex;
mod resp;

type Storage = std::sync::Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let port = 6379;

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on Port {}", port);

    let storage: Storage = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));

    loop {
        let stream = listener.accept().await;
        let storage: Storage = storage.clone();

        match stream {
            Ok((stream, _)) => {

                tokio::spawn(async move {
                    handle_client(stream, storage).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream, storage: Storage) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
           let (command, args) = extract_command(value).unwrap();

           match command.as_str() {
            "ping" => Value::SimpleString("PONG".to_string()),
            "echo" => args.first().unwrap().clone(),
            "get" => {
                let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                let storage = storage.lock().unwrap();
                match storage.get(&key) {
                    Some(value) => Value::BulkString(value.clone()),
                    None => Value::NullBulkString,
                }
            }
            "set" => {
                let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();
                let mut storage = storage.lock().unwrap();
                storage.insert(key, value);
                Value::SimpleString("OK".to_string())
            },
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