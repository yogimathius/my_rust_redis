use std::env;

pub struct Config {
    pub port: String,
    pub replicaof: Option<String>,
}

impl Config {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut config = Config {
            port: String::from("6379"),
            replicaof: None,
        };

        for (index, arg) in args.iter().enumerate() {
            if arg == "--port" {
                if let Some(port) = args.get(index + 1) {
                    config.port = port.to_owned();
                }
            }

            if arg == "--replicaof" {
                if let (Some(host), Some(port)) = (args.get(index + 1), args.get(index + 2)) {
                    config.replicaof = Some(format!("{}:{}", host, port));
                }
            }
        }

        config
    }
}
