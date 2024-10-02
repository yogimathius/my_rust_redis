use crate::{log, models::args::Args};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    pub port: String,
    pub replicaof: Option<String>,
}

impl Config {
    pub fn parse() -> Self {
        let args = Args::parse();
        let mut config: Config = Config {
            port: String::from("6379"),
            replicaof: None,
        };
        log!("{:?}", args);
        if let Some(replicaof) = args.replicaof {
            config.replicaof = Some(replicaof.join(":"));
        }

        config.port = args.port.to_string();

        config
    }
}
