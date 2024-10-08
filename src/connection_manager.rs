use std::sync::Arc;

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    command::{Command, RedisCommand},
    log,
    models::{connection_state::ConnectionState, message::Message, value::Value},
    server::ServerState,
    utilities::parse_message,
};

pub struct ConnectionManager {
    pub listener: Option<Arc<TcpListener>>,
    pub connections: Vec<ConnectionState>,
    pub master_connection: Option<ConnectionState>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            listener: None,
            connections: Vec::new(),
            master_connection: None,
        }
    }

    pub async fn accept_connections(&mut self) {
        log!("Accepting connections...");
        let listener = self.listener.as_ref().unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    log!("Accepted connection");
                    log!("Stream: {:?}", stream);
                    let connection_state = ConnectionState::new(stream);
                    self.connections.push(connection_state);
                }
                Err(e) => {
                    panic!("Error accepting connection: {}", e);
                }
            }
            tokio::task::yield_now().await;
        }
    }

    pub async fn read_messages(&mut self, server_state: ServerState) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut closed_indices = Vec::new();
        for (index, connection) in self.connections.iter_mut().enumerate() {
            let (should_close, new_commands) =
                ConnectionManager::read_messages_from(server_state.clone(), &connection.stream)
                    .await;
            if should_close {
                closed_indices.push(index);
            }
            log!("Received {} commands from client", new_commands.len());
            messages.extend(new_commands.into_iter().map(|command| Message {
                connection: Arc::clone(&connection.stream),
                command,
            }));
        }
        for &index in closed_indices.iter().rev() {
            self.connections.remove(index);
        }
        if let Some(master_connection) = self.master_connection.as_mut() {
            let (_should_close, new_commands) = ConnectionManager::read_messages_from(
                server_state.clone(),
                &master_connection.stream,
            )
            .await;
            log!("Received {} commands from master", new_commands.len());
            if new_commands.len() > 0 {
                log!("Received {} commands from master", new_commands.len());
                messages.extend(new_commands.into_iter().map(|command| Message {
                    connection: Arc::clone(&master_connection.stream),
                    command,
                }));
            }
        }
        messages
    }

    async fn read_messages_from(
        server_state: ServerState,
        stream: &Arc<Mutex<TcpStream>>,
    ) -> (bool, Vec<Command>) {
        log!("Reading messages from client...");
        let mut buf = BytesMut::with_capacity(1024);
        let mut new_commands: Vec<Command> = Vec::new();
        let mut should_close = false;

        let mut stream = stream.lock().await;

        match stream.read_buf(&mut buf).await {
            Ok(0) => {
                log!("Read 0 bytes, closing connection");
                should_close = true;
            }
            Ok(bytes_read) => {
                log!("Read {} bytes", bytes_read);
                log!("Buffer: {:?}", buf);
                log!("Server state: {:?}", server_state);
                if server_state == ServerState::ReceivingRdbDump {
                    log!("Received RDB data");
                    loop {
                        if buf.is_empty() {
                            break;
                        }
                        match parse_message(buf.clone().split()) {
                            Ok((value, bytes_consumed)) => match value {
                                Value::BulkString(data) => {
                                    buf.advance(bytes_consumed);
                                    new_commands.push(Command::new(
                                        RedisCommand::Rdb(data.into()),
                                        String::new(),
                                        vec![],
                                        Bytes::new(),
                                    ));
                                    return (true, new_commands);
                                }
                                Value::Array(_) => {
                                    buf.advance(bytes_consumed);

                                    let command =
                                        Command::parse(&mut buf, server_state).await.unwrap();

                                    new_commands.push(command);
                                    return (true, new_commands);
                                }
                                _ => {
                                    log!("TODO: Handle unexpected value types");
                                    buf.advance(bytes_consumed);
                                    return (true, vec![]);
                                }
                            },
                            Err(ref e) if e.to_string() == "Incomplete" => {
                                // Need more data to complete the message
                                log!("Incomplete RDB data, waiting for more data");
                                return (false, vec![]); // Don't close the connection
                            }
                            Err(e) => {
                                // Handle parsing error
                                log!("Error parsing RDB data: {:?}", e);
                                return (true, vec![]); // Close the connection or handle error
                            }
                        }
                    }
                } else {
                    let command = Command::parse(&mut buf, server_state).await.unwrap();
                    new_commands.extend(vec![command]);
                    log!("Added commands to new_commands: {:?}", new_commands);
                }
            }
            Err(e) => {
                log!("Error reading from stream: {:?}", e);
                should_close = true;
            }
        }

        log!("Finished reading messages from client: {:?}", new_commands);
        (should_close, new_commands)
    }
}
