use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

use crate::command::Command;

#[derive(Debug)]
pub struct Message {
    pub connection: Arc<Mutex<TcpStream>>,
    pub command: Command,
}
