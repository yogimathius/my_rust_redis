use core::panic;
#[derive(Debug)]
pub enum RedisCommand {
    Ping,
    Get(String),
    Set(String, String, Option<String>, Option<u32>),
    Echo(String),
    Command,
    Info,
    ReplConf(String, String),
    Psync(String, i32),
}
#[derive(Debug)]
pub struct Command {
    pub command: RedisCommand,
    pub raw: String,
}
impl Command {
    pub fn new(command: RedisCommand, raw: String) -> Self {
        Command { command, raw }
    }
    pub fn parse(input: &str) -> Command {
        println!("Parsing command: {}", input);
        let mut lines = input.split("\r\n");
        // The first line is the number of parameters
        lines.next();
        // The second line is the length of the command
        lines.next();
        // The third line is the actual command
        let command = lines.next().unwrap();
        match command.to_uppercase().as_str() {
            "PING" => Command::new(RedisCommand::Ping, format!("{}", input)),
            "GET" => {
                lines.next(); // Skip the length of the key
                let key = lines.next().unwrap();
                Command::new(RedisCommand::Get(key.to_string()), format!("{}", input))
            }
            "SET" => {
                lines.next(); // Skip the length of the key
                let key = lines.next().unwrap();
                lines.next(); // Skip the length of the value
                let value = lines.next().unwrap();
                let mut expiry_flag: Option<String> = None;
                let mut expiry_time: Option<u32> = None;
                // This is the length of the flag
                match lines.next() {
                    Some(flag_len) => {
                        if !flag_len.is_empty() {
                            expiry_flag = Some(lines.next().unwrap().to_string());
                            lines.next(); // Skip the length of the value
                            match lines.next() {
                                Some(time_str) => {
                                    expiry_time = Some(time_str.parse().unwrap());
                                }
                                None => {}
                            }
                        }
                    }
                    None => {}
                }
                Command::new(
                    RedisCommand::Set(key.to_string(), value.to_string(), expiry_flag, expiry_time),
                    format!("{}", input),
                )
            }
            "ECHO" => {
                lines.next(); // Skip the length of the value
                let value = lines.next().unwrap();
                Command::new(RedisCommand::Echo(value.to_string()), format!("{}", input))
            }
            "COMMAND" => Command::new(RedisCommand::Command, format!("{}", input)),
            "INFO" => Command::new(RedisCommand::Info, format!("{}", input)),
            "REPLCONF" => {
                lines.next(); // Skip the length of the key
                let key = lines.next().unwrap();
                lines.next(); // Skip the length of the value
                let value = lines.next().unwrap();
                Command::new(
                    RedisCommand::ReplConf(key.to_string(), value.to_string()),
                    format!("*{}", input),
                )
            }
            "PSYNC" => {
                lines.next(); // Skip the length of the replid
                let replid = lines.next().unwrap();
                lines.next(); // Skip the length of the offset
                let offset = lines.next().unwrap();
                Command::new(
                    RedisCommand::Psync(replid.to_string(), offset.parse().unwrap()),
                    format!("{}", input),
                )
            }
            _ => {
                panic!("Unknown command: {}", command);
            }
        }
    }
}
