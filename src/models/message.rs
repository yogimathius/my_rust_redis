use std::net::TcpStream;

use crate::command::Command;

#[derive(Debug)]
pub struct Message {
    pub connection: TcpStream,
    pub command: Command,
}
