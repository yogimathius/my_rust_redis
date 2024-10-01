use std::{collections::HashMap, sync::Arc, time::Instant};

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
    models::{redis_type::RedisType, value::Value},
    replication::Replication,
    utilities::extract_command,
};

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

pub struct Server {
    pub replication: Replication,
    pub config: Config,
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
}

impl Server {
    pub fn new(config: Config) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Server {
            replication: Replication::new(&config),
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }))
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
            let Ok(value) = conn.read_value().await else {
                break Err(Error::msg("Unable to read value"));
            };

            if let Some(value) = value {
                log!("value: {:?}", value);
                self.execute_command(self.cache.clone(), value, sender.clone(), &mut conn)
                    .await
                    .unwrap();
                conn.write_value(Value::SimpleString("OK".to_string()))
                    .await?;
            }
        }
    }

    pub async fn connect_to_master(&self) -> Result<TcpStream, Error> {
        if let Some(replicaof) = self.config.replicaof.clone() {
            let stream = TcpStream::connect(replicaof).await?;
            return Ok(stream);
        }

        Err(Error::msg("no replica of"))
    }

    pub async fn handshake(&self, mut conn: Connection) -> Result<(), Error> {
        conn.write_value(Value::SimpleString("PING".to_string()))
            .await?;

        let _frame = conn.read_value().await?;

        let replconf = Value::Array(vec![
            Value::SimpleString("REPLCONF".to_string()),
            Value::SimpleString("listening-port".to_string()),
        ]);

        conn.write_value(replconf).await?;

        let _frame = conn.read_value().await?;

        let replconf = Value::Array(vec![
            Value::SimpleString("REPLCONF".to_string()),
            Value::SimpleString("capa".to_string()),
            Value::SimpleString("psync2".to_string()),
        ]);

        conn.write_value(replconf).await?;

        let _frame = conn.read_value().await?;

        let psync = Value::Array(vec![
            Value::SimpleString("PSYNC".to_string()),
            Value::SimpleString("?".to_string()),
            Value::SimpleString("-1".to_string()),
        ]);

        conn.write_value(psync).await?;

        let _frame = conn.read_value().await?;

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
        log!("command: {:?}", command);
        match command.as_str() {
            "PING" => {
                let value = Value::SimpleString("PONG".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "REPLCONF" => {
                let value = Value::SimpleString("OK".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "PSYNC" => {
                let value =
                    Value::SimpleString(format!("FULLRESYNC {} 0", self.replication.master_replid));
                conn.write_value(value).await.unwrap();
                self.write_rdb(conn).await.unwrap();
                let mut receiver = sender.subscribe();

                while let Ok(f) = receiver.recv().await {
                    conn.write_value(f).await.unwrap();
                }
                Ok(())
            }
            "SET" => {
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
        conn.write_value(Value::BulkString(header)).await?;
        conn.write_value(Value::BulkBytes(contents)).await?;

        Ok(())
    }
}
