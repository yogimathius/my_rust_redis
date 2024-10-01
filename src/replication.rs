use std::fmt::Display;

use crate::config::Config;

#[derive(PartialEq, Eq)]
pub(crate) enum Role {
    Main,
    Slave,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Main => write!(f, "master"),
            Role::Slave => write!(f, "slave"),
        }
    }
}

#[allow(dead_code)]
pub struct Replication {
    pub(crate) role: Role,
    connected_slaves: u16,
    pub(crate) master_replid: String,
    pub(crate) master_repl_offset: i8,
    second_repl_offset: i8,
    pub(crate) port: u16,
    pub(crate) host: String,
}

impl Replication {
    pub(crate) fn new(config: &Config) -> Self {
        Replication {
            role: if config.replicaof.is_some() {
                Role::Slave
            } else {
                Role::Main
            },
            connected_slaves: 0,
            master_replid: String::from("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"),
            master_repl_offset: 0,
            second_repl_offset: 0,
            port: 6379,
            host: String::from("localhost"),
        }
    }
}
