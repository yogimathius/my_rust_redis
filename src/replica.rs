// use std::sync::Arc;
// use tokio::sync::Mutex;

// use anyhow::Result;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;

// use crate::log;
// use crate::server::Server;

// #[derive(Clone, Debug)]
// pub struct ReplicaClient {
//     pub port: u16,
//     pub stream: Arc<Mutex<TcpStream>>,
//     pub handshakes: u8,
// }

// impl ReplicaClient {
//     pub async fn new(vec: Vec<String>) -> Result<Self> {
//         let mut iter = vec.into_iter();
//         let addr = iter.next().unwrap();
//         let port = iter.next().unwrap();
//         log!("connecting to main at {}:{}", addr, port);
//         let stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();
//         let stream = Arc::new(Mutex::new(stream));

//         Ok(Self {
//             port: port.parse::<u16>().unwrap(),
//             stream,
//             handshakes: 0,
//         })
//     }

//     pub async fn send_ping(&mut self, server: &Server) -> Result<()> {
//         let msg = server.send_ping().unwrap();
//         let mut stream = self.stream.lock().await;
//         stream.write_all(msg.serialize().as_bytes()).await?;
//         Ok(())
//     }

//     pub async fn send_replconf(&mut self, server: &Server) -> Result<()> {
//         let command = "REPLCONF";
//         let params = match self.handshakes {
//             1 => vec![("listening-port", server.port.to_string())],
//             2 => vec![("capa", "psync2".to_string())],
//             _ => vec![],
//         };
//         let replconf = server.generate_replconf(command, params).unwrap();
//         let mut stream = self.stream.lock().await;

//         stream.write_all(replconf.serialize().as_bytes()).await?;
//         Ok(())
//     }

//     pub async fn send_psync(&mut self, server: &Server) -> Result<()> {
//         let msg = server.send_psync().unwrap();
//         let mut stream = self.stream.lock().await;

//         stream.write_all(msg.serialize().as_bytes()).await?;
//         Ok(())
//     }

//     pub async fn read_response(&mut self) -> Result<String, std::io::Error> {
//         let mut buffer = [0; 512];
//         let mut stream = self.stream.lock().await;

//         let n = stream.read(&mut buffer).await?;
//         if n == 0 {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::UnexpectedEof,
//                 "Connection closed by the server",
//             ));
//         }
//         Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
//     }

//     pub async fn handle_response(
//         &mut self,
//         response: &str,
//         server: &Server,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         println!("response: {}", response);
//         match response.trim() {
//             "+PONG" => {
//                 self.send_replconf(server).await?;
//             }
//             "+OK" => {
//                 if self.handshakes == 3 {
//                     self.send_psync(server).await?;
//                 } else {
//                     self.send_replconf(server).await?;
//                 }
//             }
//             _ if response.starts_with("+FULLRESYNC") => {
//                 log!("ready for rdbsync");
//             }
//             _ => {
//                 println!("response: {}", response);
//                 let _server = Arc::new(Mutex::new(server.clone()));
//                 let stream = Arc::clone(&self.stream);
//                 let _stream = stream.lock().await;
//                 // let mut handler = RespHandler::new(stream, server);
//                 // handler.handle_client(server, sender).await?;
//             }
//         }
//         Ok(())
//     }
// }
