use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;
use anyhow::Result;
use std::collections::HashMap;
use resp::Value;
use std::sync::Mutex;
mod resp;
mod args;

pub struct RedisItem {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

type Storage = std::sync::Arc<Mutex<HashMap<String, RedisItem>>>;

// mod replication {
//     pub(super) const MASTER: &[u8] = b"$25\r\n# Replication\nrole:master\r\n";
//     pub(super) const SLAVE: &[u8] = b"$24\r\n# Replication\nrole:slave\r\n";
// }

#[derive(Debug, Clone, Copy)]
enum Role {
    Master,
    Slave {
        host: ::std::net::Ipv4Addr,
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    // let port = 6379;
    let args::Args { port, role } = args::Args::parse().unwrap();

    let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    println!("Listening on Port {}", port);

    let storage: Storage = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));

    loop {
        let stream = listener.accept().await;
        let storage: Storage = storage.clone();

        match stream {
            Ok((stream, _)) => {

                tokio::spawn(async move {
                    handle_client(stream, storage, role).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream, storage: Storage, role: Role) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
           let (command, args) = extract_command(value).unwrap();

           match command.as_str() {
            "ping" => Value::SimpleString("PONG".to_string()),
            "echo" => args.first().unwrap().clone(),
            "get" => handle_get(args, storage.clone()),
            "set" => handle_set(args, storage.clone()),
            "INFO" => handle_info(role),
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

fn handle_get(args: Vec<Value>, storage: Storage) -> Value {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let storage = storage.lock().unwrap();
    match storage.get(&key) {
        Some(value) => {
            let response = if let Some(expiration) = value.expiration {
                let now = Instant::now();
                if now.duration_since(value.created_at).as_millis()
                    > expiration as u128
                {
                    Value::NullBulkString
                } else {
                    Value::BulkString(value.value.clone())
                }
            } else {
                Value::BulkString(value.value.clone())
            };
            response
        },
        None => Value::NullBulkString,
    }
}

fn handle_set(args: Vec<Value>, storage: Storage) -> Value {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();
    let mut storage = storage.lock().unwrap();
        // add Expiration
        let expiration_time = match args.get(2) {
            None => None,
            Some(Value::BulkString(sub_command)) => {
                println!("sub_command = {:?} {}:?", sub_command, sub_command != "px");
                if sub_command != "px" {
                    panic!("Invalid expiration time")
                }
                match args.get(3) {
                    None => None,
                    Some(Value::BulkString(time)) => {
                        // add expiration
                        // parse time to i64
                        let time = time.parse::<i64>().unwrap();
                        Some(time)
                    }
                    _ => panic!("Invalid expiration time"),
                }
            }
            _ => panic!("Invalid expiration time"),
        };
        let redis_item = if let Some(exp_time) = expiration_time {
            RedisItem {
                value,
                created_at: Instant::now(),
                expiration: Some(exp_time),
            }
        } else {
            RedisItem {
                value,
                created_at: Instant::now(),
                expiration: None,
            }
        };
        storage.insert(key, redis_item);

    Value::SimpleString("OK".to_string())
}

fn handle_info(role: Role) -> Value {
    print!("args = {:?}", role);
    if let Role::Master = role {
        return Value::BulkString("role:master\nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb\nmaster_repl_offset:0".to_string());
    }
    Value::BulkString("role:slave".to_string())
}