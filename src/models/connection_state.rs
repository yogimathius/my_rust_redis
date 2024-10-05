use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

pub struct ConnectionState {
    pub stream: Arc<Mutex<TcpStream>>,
    pub buffer: Vec<u8>,
}

impl ConnectionState {
    pub fn new(stream: TcpStream) -> Self {
        ConnectionState {
            stream: Arc::new(Mutex::new(stream)),
            buffer: Vec::new(),
        }
    }
}
