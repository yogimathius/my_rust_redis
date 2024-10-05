use crate::log;
#[derive(Debug)]
pub enum RedisCommand {
    Ping,
    Get(String),
    Set(String, String, Option<String>, Option<u32>),
    Echo(String),
    Command,
    Info,
    ReplConf(String, String),
    ReplConfGetAck, // New variant
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

    pub fn parse(input: &str) -> Result<Command, String> {
        log!("Parsing command: {}", input);

        let elements = parse_resp_array(input)?;
        if elements.is_empty() {
            return Err("Empty command".to_string());
        }

        let command = elements[0].to_uppercase();
        match command.as_str() {
            "PING" => {
                log!("Parsing PING command");
                Ok(Command::new(RedisCommand::Ping, input.to_string()))
            }
            "GET" => {
                if elements.len() != 2 {
                    return Err("GET command requires 1 argument".to_string());
                }
                let key = elements[1].clone();
                Ok(Command::new(RedisCommand::Get(key), input.to_string()))
            }
            "SET" => {
                if elements.len() < 3 {
                    return Err("SET command requires at least 2 arguments".to_string());
                }
                let key = elements[1].clone();
                let value = elements[2].clone();

                let mut expiry_flag: Option<String> = None;
                let mut expiry_time: Option<u32> = None;

                if elements.len() >= 5 {
                    expiry_flag = Some(elements[3].clone());
                    expiry_time = Some(elements[4].parse().map_err(|_| "Invalid expiry time")?);
                }

                Ok(Command::new(
                    RedisCommand::Set(key, value, expiry_flag, expiry_time),
                    input.to_string(),
                ))
            }
            "ECHO" => {
                if elements.len() != 2 {
                    return Err("ECHO command requires 1 argument".to_string());
                }
                let message = elements[1].clone();
                Ok(Command::new(RedisCommand::Echo(message), input.to_string()))
            }
            "COMMAND" => Ok(Command::new(RedisCommand::Command, input.to_string())),
            "INFO" => Ok(Command::new(RedisCommand::Info, input.to_string())),
            "REPLCONF" => {
                if elements.len() >= 2 {
                    let subcommand = elements[1].to_uppercase();
                    match subcommand.as_str() {
                        "LISTENING-PORT" | "CAPA" | "ACK" => {
                            if elements.len() != 3 {
                                return Err("REPLCONF subcommand requires an argument".to_string());
                            }
                            let argument = elements[2].clone();
                            Ok(Command::new(
                                RedisCommand::ReplConf(subcommand, argument),
                                input.to_string(),
                            ))
                        }
                        "GETACK" => {
                            // REPLCONF GETACK has no additional arguments
                            Ok(Command::new(
                                RedisCommand::ReplConfGetAck,
                                input.to_string(),
                            ))
                        }
                        _ => Err(format!("Unknown REPLCONF subcommand: {}", subcommand)),
                    }
                } else {
                    Err("REPLCONF requires at least one subcommand".to_string())
                }
            }
            "PSYNC" => {
                if elements.len() != 3 {
                    return Err("PSYNC command requires 2 arguments".to_string());
                }
                let replid = elements[1].clone();
                let offset = elements[2]
                    .parse()
                    .map_err(|_| "Invalid PSYNC offset".to_string())?;
                Ok(Command::new(
                    RedisCommand::Psync(replid, offset),
                    input.to_string(),
                ))
            }
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

fn parse_resp_array(input: &str) -> Result<Vec<String>, String> {
    let mut lines = input.lines();
    let first_line = lines.next().ok_or("Empty input")?;
    if !first_line.starts_with('*') {
        return Err("Not a RESP array".to_string());
    }

    let num_elements: usize = first_line[1..]
        .parse()
        .map_err(|_| "Invalid array length")?;
    let mut elements = Vec::with_capacity(num_elements);

    while let Some(line) = lines.next() {
        if line.starts_with('$') {
            let bulk_len: usize = line[1..]
                .parse()
                .map_err(|_| "Invalid bulk string length")?;
            let mut bulk_string = String::new();
            let mut bytes_read = 0;

            while bytes_read < bulk_len {
                if let Some(content_line) = lines.next() {
                    bytes_read += content_line.len();
                    bulk_string.push_str(content_line);
                } else {
                    return Err("Unexpected end of input while reading bulk string".to_string());
                }
            }
            elements.push(bulk_string);
        } else {
            return Err("Expected bulk string".to_string());
        }
    }

    if elements.len() != num_elements {
        return Err("Mismatched number of elements".to_string());
    }

    Ok(elements)
}
