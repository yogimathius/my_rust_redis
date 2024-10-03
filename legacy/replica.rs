use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use tokio::net::TcpStream;

use crate::log;

#[derive(Clone, Debug)]
pub struct ReplicaClient {
    pub port: u16,
    pub stream: Arc<Mutex<TcpStream>>,
    pub handshakes: u8,
}

impl ReplicaClient {
    pub async fn new(vec: Vec<String>) -> Result<Self> {
        let mut iter = vec.into_iter();
        let addr = iter.next().unwrap();
        let port = iter.next().unwrap();
        log!("connecting to main at {}:{}", addr, port);
        let stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();
        let stream = Arc::new(Mutex::new(stream));

        Ok(Self {
            port: port.parse::<u16>().unwrap(),
            stream,
            handshakes: 0,
        })
    }
}
