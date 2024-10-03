use crate::{command::Command, connection::Connection};

#[derive(Debug)]
pub struct Message {
    pub connection: Connection,
    pub command: Command,
}
