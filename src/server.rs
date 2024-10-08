use bytes::{Buf, Bytes, BytesMut};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::command::{Command, RedisCommand};
use crate::connection_manager::ConnectionManager;
use crate::log;
use crate::message_processer::MessageProcessor;
use crate::models::connection_state::ConnectionState;
use crate::models::message::Message;
use crate::models::redis_item::RedisItem;
use crate::models::redis_type::RedisType;
use crate::models::role::Role;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::serializer::Serializer;
use crate::utilities::parse_message;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ServerState {
    Initialising,
    SendingHandshake,
    AwaitingReplConfOk,
    SendingCapabilities,
    AwaitingFullResync,
    ReceivingRdbDump,
    StreamingCommands,
}

pub struct Server {
    port: u16,
    pub state: ServerState,
    pub role: Role,
    master_addr: Option<String>,
    master_replid: String,
    master_repl_offset: u32,
    pub sync: bool,
    pub connection_manager: ConnectionManager,
    pub message_processor: MessageProcessor,
}

impl Server {
    pub fn new(port: u16, replicaof: Option<String>) -> Self {
        let role = match replicaof {
            Some(_) => Role::Slave,
            None => Role::Master,
        };
        let master_addr = if replicaof.is_some() {
            Some(replicaof.unwrap().replace(" ", ":"))
        } else {
            None
        };
        let master_replid = if role == Role::Master {
            "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()
        } else {
            "?".to_string()
        };
        Server {
            port,
            role,
            state: ServerState::Initialising,
            master_addr,
            master_replid: master_replid.clone(),
            master_repl_offset: 0,
            sync: false,
            connection_manager: ConnectionManager::new(),
            message_processor: MessageProcessor::new(
                role.clone(),
                HashMap::new(),
                master_replid,
                0,
                ServerState::Initialising,
            ),
        }
    }

    pub async fn run(&mut self) {
        if self.role == Role::Slave {
            self.connect_to_master().await;
        }

        let listener = match TcpListener::bind(format!("127.0.0.1:{}", self.port)).await {
            Ok(listener) => listener,
            Err(e) => panic!("Error binding to port: {}", e),
        };

        self.connection_manager.listener = Some(Arc::new(listener)); // Assign the listener
        self.state = ServerState::StreamingCommands;
        log!("Listening on 127.0.0.1:{}...", self.port);
    }

    async fn connect_to_master(&mut self) {
        let addr = self.master_addr.as_ref().unwrap();
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let connection = ConnectionState::new(stream);
                self.connection_manager.master_connection = Some(connection);
            }
            Err(e) => {
                panic!("Error connecting to master: {}", e);
            }
        }
        let connection = self.connection_manager.master_connection.as_ref().unwrap();

        let stream_clone = Arc::clone(&connection.stream);

        let mut replica = ReplicaClient::new(stream_clone, self.port).await.unwrap();
        replica.send_ping().await.unwrap();
        self.state = ServerState::SendingHandshake;
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

    pub async fn rdb_dump(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = Serializer::serialize_data(&self.message_processor.data)?;
        let mut file = File::create("dump.rdb")?;
        file.write_all(&data)?;
        Ok(())
    }
}
