use crate::log;
use crate::models::args::Args;
use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::resp::RespHandler;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpListener;

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
    pub replicas: Arc<Mutex<Vec<ReplicaClient>>>,
}

impl Server {
    pub fn new(args: Args) -> Self {
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
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
            port: args.port,
            sync: false,
            replicas: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn match_replica(&mut self, args: Args) {
        match args.replicaof {
            Some(vec) => {
                let mut replica = ReplicaClient::new(vec).await.unwrap();
                replica.send_ping(&self).await.unwrap();

                while replica.handshakes < 4 {
                    match replica.read_response().await {
                        Ok(response) => {
                            replica.handshakes += 1;
                            replica.handle_response(&response, &self).await.unwrap();
                        }
                        Err(e) => {
                            log!("Failed to read from stream: {}", e);
                        }
                    }
                }
                self.replicas.lock().unwrap().push(replica);
            }
            None => {}
        }
    }

    pub async fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        log!("Listening on Port {}", port);

        loop {
            let stream = listener.accept().await;
            let server: Server = self.clone();
            match stream {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut handler = RespHandler::new(stream);
                        handler.handle_client(server).await.unwrap();
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

    pub async fn propagate_command(&self, command: &str, args: Vec<Value>) {
        let replicas = self.replicas.lock().unwrap();
        for replica in replicas.iter() {
            let mut replica = replica.clone();
            replica
                .propagate_command(command, args.clone())
                .await
                .unwrap();
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
