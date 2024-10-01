use crate::log;
use crate::models::args::Args;
use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::resp::RespHandler;
use anyhow::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    io::AsyncWriteExt,
    sync::{broadcast, Mutex},
};

use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Main,
    Slave { host: String, port: u16 },
}

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    pub role: Role,
    pub port: u16,
    pub sync: bool,
    pub replica_ports: Arc<Mutex<Vec<u16>>>,
}

impl Server {
    pub fn new(args: Args) -> Arc<Mutex<Self>> {
        let role = match args.replicaof {
            Some(vec) => {
                let mut iter = vec.into_iter();
                let addr = iter.next().unwrap();
                let _ = iter.next().unwrap();
                Role::Slave {
                    host: addr,
                    port: args.port,
                }
            }
            None => Role::Main,
        };
        Arc::new(Mutex::new(Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
            port: args.port,
            sync: false,
            replica_ports: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    pub async fn connect_to_master(&self, args: Args) -> Result<TcpStream, Error> {
        if let Some(replicaof) = args.replicaof.clone() {
            let replicaof = format!("{}:{}", replicaof[0], replicaof[1]);
            let stream = TcpStream::connect(replicaof).await?;
            return Ok(stream);
        }

        Err(Error::msg("no replica of"))
    }

    pub async fn match_replica(&mut self, args: Args) {
        log!("checking replica args {:?}", args);
        match args.replicaof {
            Some(vec) => {
                log!("vec {:?}", vec);
                let mut replica = ReplicaClient::new(vec).await.unwrap();
                replica.send_ping(&self).await.unwrap();

                while replica.handshakes <= 3 {
                    match replica.read_response().await {
                        Ok(response) => {
                            log!("response: {}", response);
                            log!("replica.handshakes: {}", replica.handshakes);
                            replica.handshakes += 1;
                            replica.handle_response(&response, &self).await.unwrap();
                        }
                        Err(e) => {
                            log!("Failed to read from stream: {}", e);
                        }
                    }
                }
                println!("self {:?}", self);
                println!("replica {:?}", replica);
            }
            None => {}
        }
    }

    pub async fn listen(&mut self, port: u16, sender: Arc<broadcast::Sender<Value>>) {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        log!("Listening on Port {}", port);

        loop {
            let stream = listener.accept().await;

            let server: Arc<Mutex<Server>> = Arc::new(Mutex::new(self.clone()));
            let sender = Arc::clone(&sender);

            match stream {
                Ok((stream, _)) => {
                    log!("stream: {:?}", stream);
                    let server_clone = Arc::clone(&server);

                    tokio::spawn(async move {
                        let mut handler = RespHandler::new(stream, server_clone.clone());
                        if let Err(e) = handler.handle_client(server_clone, sender).await {
                            log!("Error handling client: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    log!("error: {}", e);
                }
            }
        }
    }

    pub fn send_ping(&self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let msg = vec![Value::BulkString(String::from("PING"))];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn send_psync(&self) -> Option<Value> {
        log!("Syncing with master");
        log!("self.role {:?}", self.role);

        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let msg = vec![
                    Value::BulkString(String::from("PSYNC")),
                    Value::BulkString(String::from("?")),
                    Value::BulkString(String::from("-1")),
                ];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn generate_replconf(&self, command: &str, params: Vec<(&str, String)>) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let mut msg = vec![Value::BulkString(command.to_string())];
                for (key, value) in params {
                    msg.push(Value::BulkString(key.to_string()));
                    msg.push(Value::BulkString(value.to_string()));
                }
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Self::Main => String::from("master"),
            Self::Slave { host: _, port: _ } => String::from("slave"),
        }
    }
}
