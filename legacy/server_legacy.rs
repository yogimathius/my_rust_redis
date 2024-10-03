use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Sender},
        Mutex,
    },
};

use crate::{
    commands::COMMAND_HANDLERS,
    config::Config,
    connection::Connection,
    handlers::{info_handler, set_handler},
    log,
    models::value::Value,
    replication::Replication,
    server::RedisItem,
    utilities::extract_command,
};

pub struct Server {
    port: u16,
    role: Role,
    master_addr: Option<String>,
    master_stream: Option<TcpStream>,
    master_replid: String,
    master_repl_offset: u32,
    listener: Option<TcpListener>,
    connections: Vec<TcpStream>,
    messages: VecDeque<Message>,
    slaves: HashMap<String, TcpStream>,
    pub config: Config,
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let role = match config.replicaof {
            Some(_) => Role::Slave,
            None => Role::Master,
        };
        let master_addr = if replicaof.is_some() {
            Some(config.replicaof.unwrap().replace(" ", ":"))
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
            master_addr,
            master_stream: None,
            master_replid,
            master_repl_offset: 0,
            listener: None,
            connections: Vec::new(),
            messages: VecDeque::new(),
            slaves: HashMap::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn listen(&self) -> Result<TcpListener, Error> {
        let address = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(address).await?;

        Ok(listener)
    }

    pub async fn handle_connection(
        &mut self,
        mut conn: Connection,
        sender: Arc<Sender<Value>>,
    ) -> Result<(), Error> {
        let sender = Arc::clone(&sender);
        log!("Handling connection");
        loop {
            match conn.read_value().await {
                Ok(Some(value)) => {
                    log!("CHECKING value: {:?}", value);
                    log!("value: {:?}", value);
                    self.execute_command(self.cache.clone(), value, sender.clone(), &mut conn)
                        .await
                        .unwrap();
                    conn.write_value(Value::SimpleString("OK".to_string()))
                        .await?;
                }
                Ok(None) => {
                    log!("No value read, continuing to wait for new values");
                    // Continue the loop to wait for new values
                }
                Err(e) => {
                    log!("Error reading value: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    pub async fn connect_to_master(&self) -> Result<TcpStream, Error> {
        log!("Connecting to master {:?}", self.config);
        if let Some(replicaof) = self.config.replicaof.clone() {
            log!("Connecting to master");
            let stream = TcpStream::connect(replicaof).await?;
            return Ok(stream);
        }

        Err(Error::msg("no replica of"))
    }

    pub async fn handshake(&mut self, mut conn: Connection) -> Result<(), Error> {
        log!("Starting handshake");
        conn.write_value(Value::SimpleString("PING".to_string()))
            .await?;

        let _response = conn.read_value().await?;

        self.send_replconf_two(&mut conn).await?;

        let _response = conn.read_value().await?;

        self.send_replconf(&mut conn).await?;

        let _response = conn.read_value().await?;

        let psync = Value::Array(vec![
            Value::BulkString("PSYNC".to_string()),
            Value::BulkString("?".to_string()),
            Value::BulkString("-1".to_string()),
        ]);

        conn.write_value(psync).await?;

        let response: Option<Value> = conn.read_value().await?;
        log!("Handshake response: {:?}", response);
        Ok(())
    }

    pub async fn execute_command(
        &mut self,
        cache: Arc<Mutex<HashMap<String, RedisItem>>>,
        value: Value,
        sender: Arc<broadcast::Sender<Value>>,
        conn: &mut Connection,
    ) -> Result<(), Error> {
        let (command, key, args) = extract_command(value.clone()).unwrap();
        log!("executing command: {:?}", command);
        match command.as_str() {
            "PING" => {
                let value = Value::SimpleString("PONG".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "REPLCONF" => {
                log!("REPLCONF command");
                let value = Value::SimpleString("OK".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "PSYNC" => {
                let value =
                    Value::SimpleString(format!("FULLRESYNC {} 0", self.replication.master_replid));
                conn.write_value(value).await.unwrap();
                self.write_rdb(conn).await.unwrap();
                let mut conn_clone = conn.clone();

                let receiver = Arc::new(Mutex::new(sender.subscribe()));
                log!("Subscribing to sender");
                conn_clone.spawn_pubsub_task(receiver);
                log!("Done sending values");
                Ok(())
            }
            "SET" => {
                log!("SET command");
                let response = sender.send(value);
                log!("response: {:?}", response);
                set_handler(cache, key, args).await;
                Ok(())
            }
            "INFO" => {
                let response = info_handler(self).await;
                conn.write_value(response.unwrap()).await.unwrap();
                Ok(())
            }
            _ => {
                if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
                    command_function.handle(cache, key, args).await;
                    Ok(())
                } else {
                    let value = Value::Error("ERR unknown command".to_string());
                    conn.write_value(value).await.unwrap();
                    Err(anyhow::Error::msg("Unknown command"))
                }
            }
        }
    }

    pub async fn write_rdb(&mut self, conn: &mut Connection) -> Result<(), Error> {
        let mut rdb_buf: Vec<u8> = vec![];
        log!("Opening dump.rdb");
        let _ = File::open("rdb")
            .await
            .unwrap()
            .read_to_end(&mut rdb_buf)
            .await;
        log!("Read {} bytes from dump.rdb", rdb_buf.len());
        let contents = hex::decode(&rdb_buf).unwrap();
        let header = format!("${}\r\n", contents.len());
        log!("Writing RDB file");
        log!("header: {:?}", header);
        conn.write_all(header.as_bytes()).await?;
        log!("contents: {:?}", contents);
        conn.write_all(&contents).await?;
        log!("Wrote RDB file");
        Ok(())
    }

    pub async fn send_replconf(&mut self, conn: &mut Connection) -> Result<(), Error> {
        let command = "REPLCONF";
        let msg = Value::Array(vec![
            Value::BulkString(String::from(command)),
            Value::BulkString(String::from("listening-port")),
            Value::BulkString(self.config.port.to_string()),
        ]);
        conn.write_value(msg).await?;
        Ok(())
    }

    pub async fn send_replconf_two(&mut self, conn: &mut Connection) -> Result<(), Error> {
        let command = "REPLCONF";
        let msg = Value::Array(vec![
            Value::BulkString(String::from(command)),
            Value::BulkString(String::from("capa")),
            Value::BulkString(String::from("psync2")),
        ]);
        conn.write_value(msg).await?;
        Ok(())
    }
}
