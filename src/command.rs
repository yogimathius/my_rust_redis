use crate::{
    log,
    models::value::Value,
    server::ServerState,
    utilities::{extract_command, parse_message, unpack_bulk_str, unpack_integer},
};
use bytes::{Bytes, BytesMut};

#[derive(Debug)]
pub enum RedisCommand {
    Ping,
    Get(String),
    Set(String, String, Option<String>, Option<i64>),
    Echo(String),
    Command,
    Info,
    ReplConf(String, String),
    ReplConfGetAck,
    Psync(String, String),
    FullResync(String, i64),
    Rdb(Bytes),
}

#[derive(Debug)]
pub struct Command {
    pub command: RedisCommand,
    pub key: String,
    pub args: Vec<Value>,
    pub raw: Bytes,
}

impl Command {
    pub fn new(command: RedisCommand, key: String, args: Vec<Value>, raw: Bytes) -> Self {
        Command {
            command,
            key,
            args,
            raw,
        }
    }

    pub async fn parse(input: &mut BytesMut, server_state: ServerState) -> Result<Command, String> {
        log!("Parsing command: {:?}", input);

        // Attempt to parse a message from the input buffer
        let (value, bytes_consumed) = parse_message(input.clone()).map_err(|e| e.to_string())?;
        log!("Parsed value: {:?}", value);

        // Capture the raw bytes and advance the buffer
        let raw_command_bytes = input.split_to(bytes_consumed).freeze(); // Converts to Bytes

        // Extract the command and arguments
        let (command_name, args) =
            extract_command(&value).map_err(|e| format!("Error extracting command: {}", e))?;

        let command_upper = command_name.to_uppercase();
        log!("Command: {}", command_upper);
        match value {
            Value::BulkString(s) => {
                if server_state == ServerState::ReceivingRdbDump {
                    return Ok(Command::new(
                        RedisCommand::Rdb(s.into()),
                        String::new(),
                        vec![],
                        raw_command_bytes,
                    ));
                } else {
                    return Err("Unexpected RDB data".to_string());
                }
            }
            Value::SimpleString(s) => {
                if s.starts_with("FULLRESYNC") {
                    let parts: Vec<&str> = s.trim_end().split_whitespace().collect();
                    if parts.len() == 3 {
                        let replid = parts[1].to_string();
                        let offset = parts[2]
                            .parse::<i64>()
                            .map_err(|_| "Invalid replication offset".to_string())?;
                        Ok(Command::new(
                            RedisCommand::FullResync(replid, offset),
                            String::new(),
                            vec![],
                            raw_command_bytes,
                        ))
                    } else {
                        Err("Invalid FULLRESYNC response".to_string())
                    }
                } else {
                    // Handle other simple strings
                    log!("TODO: Handle other simple strings");
                    Err("Unknown command".to_string())
                }
            }
            Value::BulkBytes(data) => {
                // Handle RDB data
                Ok(Command::new(
                    RedisCommand::Rdb(data.into()),
                    String::new(),
                    vec![],
                    raw_command_bytes,
                ))
            }
            Value::Array(_) => {
                match command_upper.as_str() {
                    "PING" => Ok(Command::new(
                        RedisCommand::Ping,
                        String::new(),
                        args,
                        raw_command_bytes,
                    )),
                    "GET" => {
                        if args.len() != 1 {
                            return Err("GET command requires 1 argument".to_string());
                        }
                        let key = unpack_bulk_str(&args[0]).unwrap();
                        Ok(Command::new(
                            RedisCommand::Get(key.clone()),
                            key,
                            args,
                            raw_command_bytes,
                        ))
                    }
                    "SET" => {
                        if args.len() < 2 {
                            return Err("SET command requires at least 2 arguments".to_string());
                        }
                        let key = unpack_bulk_str(&args[0]).unwrap();
                        let value_arg = unpack_bulk_str(&args[1]).unwrap();

                        let mut expiry_flag: Option<String> = None;
                        let mut expiry_time: Option<i64> = None;

                        if args.len() >= 4 {
                            expiry_flag = Some(unpack_bulk_str(&args[2]).unwrap());
                            expiry_time = Some(unpack_integer(args[3].clone()).unwrap());
                        }

                        Ok(Command::new(
                            RedisCommand::Set(key.clone(), value_arg, expiry_flag, expiry_time),
                            key,
                            args,
                            raw_command_bytes,
                        ))
                    }
                    "ECHO" => {
                        if args.len() != 1 {
                            return Err("ECHO command requires 1 argument".to_string());
                        }
                        let message = unpack_bulk_str(&args[0]).unwrap();
                        Ok(Command::new(
                            RedisCommand::Echo(message),
                            String::new(),
                            args,
                            raw_command_bytes,
                        ))
                    }
                    "COMMAND" => Ok(Command::new(
                        RedisCommand::Command,
                        String::new(),
                        args,
                        raw_command_bytes,
                    )),
                    "INFO" => Ok(Command::new(
                        RedisCommand::Info,
                        String::new(),
                        args,
                        raw_command_bytes,
                    )),
                    "REPLCONF" => {
                        if !args.is_empty() {
                            let subcommand = unpack_bulk_str(&args[0]).unwrap().to_uppercase();
                            match subcommand.as_str() {
                                "LISTENING-PORT" | "CAPA" | "ACK" => {
                                    if args.len() != 2 {
                                        return Err(
                                            "REPLCONF subcommand requires an argument".to_string()
                                        );
                                    }
                                    let argument = unpack_bulk_str(&args[1]).unwrap();
                                    Ok(Command::new(
                                        RedisCommand::ReplConf(subcommand, argument),
                                        String::new(),
                                        args,
                                        raw_command_bytes,
                                    ))
                                }
                                "GETACK" => {
                                    log!("REPLCONF GETACK");
                                    Ok(Command::new(
                                        RedisCommand::ReplConfGetAck,
                                        String::new(),
                                        args,
                                        raw_command_bytes,
                                    ))
                                }
                                _ => Err(format!("Unknown REPLCONF subcommand: {}", subcommand)),
                            }
                        } else {
                            Err("REPLCONF requires at least one subcommand".to_string())
                        }
                    }
                    "PSYNC" => {
                        if args.len() != 2 {
                            return Err("PSYNC command requires 2 arguments".to_string());
                        }
                        let replid = unpack_bulk_str(&args[0]).unwrap();
                        let offset = unpack_bulk_str(&args[1]).unwrap();
                        Ok(Command::new(
                            RedisCommand::Psync(replid, offset),
                            String::new(),
                            args,
                            raw_command_bytes,
                        ))
                    }
                    // _ if command_upper.starts_with("FULLRESYNC") => {
                    //     // SimpleString("FULLRESYNC 75cd7bc10c49047e0d163660f3b90625b1af31dc 0")
                    //     // split into replid and offset
                    //     let split_command: Vec<&str> = command_upper.split_whitespace().collect();
                    //     let replid = split_command[1].to_string();
                    //     let offset = split_command[2].to_string();
                    //     Ok(Command::new(
                    //         RedisCommand::FullResync(replid, offset),
                    //         String::new(),
                    //         args,
                    //         raw_command_bytes,
                    //     ))
                    // }
                    _ => Err(format!("Unknown command: {}", command_upper)),
                }
            }
            _ => Err("TODO: handle other types later".to_string()),
        }
    }
}
