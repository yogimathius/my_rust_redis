use crate::database::Database;
use crate::log;
use crate::models::args::Args;
use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::resp::RespHandler;
use crate::utilities::ServerState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::time::{interval, sleep, Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Main,
    Slave { host: String, port: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: i64,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    pub role: Role,
    pub port: u16,
    pub sync: bool,
    pub server_state: ServerState,
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
        let server = Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
            port: args.port,
            sync: false,
            server_state: ServerState::Initialising,
        };

        server
    }

    pub async fn match_replica(&mut self, args: Args) {
        match args.replicaof {
            Some(vec) => {
                let mut replica = ReplicaClient::new(vec).await.unwrap();
                replica.send_ping(&self).await.unwrap();

                while replica.sync == false {
                    match replica.read_response().await {
                        Ok(response) => {
                            log!("response in match: {}", response);
                            replica.handshakes += 1;
                            replica.handle_response(&response, self).await.unwrap();
                        }
                        Err(e) => {
                            log!("Failed to read from stream: {}", e);
                        }
                    }
                }
            }
            None => {}
        }
    }

    pub async fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        log!("Listening on Port {}", port);

        let db = Database::new(self.cache.clone(), "dump.rdb");

        if let Err(e) = db.read_backup() {
            eprintln!("Failed to load backup: {}", e);
        }

        let db_clone = db.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(300)).await;

            let mut interval_timer = interval(Duration::from_secs(300));
            loop {
                interval_timer.tick().await;
                if let Err(e) = db_clone.dump_backup() {
                    eprintln!("Failed to dump backup: {}", e);
                } else {
                    println!("Backup dumped successfully.");
                }
            }
        });

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let server_clone = self.clone();
                            tokio::spawn(async move {
                                let mut handler = RespHandler::new(stream);
                                log!("Handling client");
                                match handler.handle_client(server_clone).await {
                                    Ok(_) => log!("Client disconnected gracefully"),
                                    Err(e) => log!("Client disconnected with error: {}", e),
                                }
                            });
                        },
                        Err(e) => {
                            log!("Error accepting connection: {}", e);
                        }
                    }
                }

                _ = tokio::signal::ctrl_c() => {
                    println!("Received Ctrl+C, initiating graceful shutdown...");

                    if let Err(e) = db.dump_backup() {
                        eprintln!("Failed to dump backup on shutdown: {}", e);
                    } else {
                        println!("Backup dumped successfully on shutdown.");
                    }

                    println!("Server is shutting down gracefully.");
                    break;
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
        let mut msg = vec![Value::BulkString(command.to_string())];
        for (key, value) in params {
            msg.push(Value::BulkString(key.to_string()));
            msg.push(Value::BulkString(value.to_string()));
        }
        let payload = Value::Array(msg);
        Some(payload)
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
