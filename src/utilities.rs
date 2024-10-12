use lazy_static::lazy_static;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Arguments;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::BytesMut;

use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use crate::server::RedisItem;
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Initialising,
    AwaitingFullResync,
    ReceivingRdbDump,
    AwaitingGetAck,
    StreamingCommands,
}
pub fn log_message(file: &str, line: u32, args: Arguments) {
    println!("{}:{}: {:?}", file, line, args);
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::utilities::log_message(file!(), line!(), format_args!($($arg)*))
    };
}

lazy_static! {
    static ref NO_ARG_COMMANDS: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("PING");
        m.insert("INFO");
        m.insert("PSYNC");
        m.insert("FLUSHALL");
        m
    };
}

lazy_static! {
    static ref NO_KEY_COMMANDS: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("UNLINK");
        m.insert("DEL");
        m.insert("KEYS");
        m
    };
}

pub fn extract_command(value: Value) -> Result<(String, String, Vec<Value>)> {
    match value {
        Value::Array(a) => {
            let command = unpack_bulk_str(a.first().unwrap().clone()).unwrap();
            let mut iter = a.into_iter();
            if NO_ARG_COMMANDS.contains(command.as_str()) {
                return Ok((command, "".to_string(), vec![]));
            }
            if NO_KEY_COMMANDS.contains(command.as_str()) {
                iter.next();
                return Ok((command, "".to_string(), iter.collect()));
            }
            iter.next();
            let key = unpack_bulk_str(iter.next().ok_or_else(|| anyhow::anyhow!("Missing key"))?)?;

            Ok((command, key, iter.collect()))
        }
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

pub fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected bulk string")),
    }
}

pub fn unpack_integer(value: Value) -> Result<i64> {
    match value {
        Value::Integer(i) => Ok(i),
        _ => Err(anyhow::anyhow!("Expected integer")),
    }
}

pub fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    log!("Parsing message: {:?}", buffer);
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Unknown value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();

        return Ok((Value::SimpleString(string), len + 1));
    }

    return Err(anyhow::anyhow!("Invalid string {:?}", buffer));
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_length, mut bytes_consumed) =
        if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
            let array_length = parse_int(line)?;

            (array_length, len + 1)
        } else {
            return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
        };

    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(array_item);
        bytes_consumed += len
    }

    return Ok((Value::Array(items), bytes_consumed));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;

        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;

    Ok((
        Value::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_of_bulk_str].to_vec(),
        )?),
        total_parsed,
    ))
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }

    return None;
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}

// pub fn get_expiration(args: Vec<Value>) -> Result<Option<i64>, String> {
//     match args.get(2) {
//         Some(Value::BulkString(sub_command)) => {
//             if sub_command != "px" {
//                 panic!("Invalid expiration time")
//             }
//             match args.get(3) {
//                 None => Ok(None),
//                 Some(Value::Integer(time)) => {
//                     let time = time.parse::<i64>().unwrap();
//                     Ok(Some(time))
//                 }
//                 _ => Err("Invalid expiration time".to_string()),
//             }
//         }
//         None => Ok(None),
//         _ => Err("Invalid expiration time".to_string()),
//     }
// }

/**
NX -- Set expiry only when the key has no expiry
XX -- Set expiry only when the key has an existing expiry
GT -- Set expiry only when the new expiry is greater than current one
LT -- Set expiry only when the new expiry is less than current one
 */
// helper function

pub fn should_set_expiry(item: &RedisItem, expiration: i64, option: String) -> bool {
    log!("item {:?}", item);
    log!("expiration {:?}", expiration);
    log!("option {:?}", option);

    match option.to_uppercase().as_str() {
        "NX" => item.expiration.is_none(),
        "XX" => {
            log!("item.expiration {:?}", item.expiration);
            item.expiration.is_some()
        }
        "GT" => item.expiration.map_or(true, |exp| expiration > exp),
        "LT" => item.expiration.map_or(true, |exp| expiration < exp),
        _ => true,
    }
}

pub fn extract_args(args: Vec<Value>) -> (String, Option<String>, Option<String>, Vec<Value>) {
    let mut iter = args.into_iter();

    let key = match iter.next() {
        Some(Value::BulkString(s)) => s,
        _ => "".to_string(),
    };

    let arg1 = match iter.next() {
        Some(Value::BulkString(s)) => Some(s),
        _ => None,
    };

    let arg2 = match iter.next() {
        Some(Value::BulkString(s)) => Some(s),
        _ => None,
    };

    let additional_args: Vec<Value> = iter.collect();

    (key, arg1, arg2, additional_args)
}

pub fn lock_and_get_item<'a, F, R>(
    cache: &Arc<Mutex<HashMap<String, RedisItem>>>,
    key: &str,
    callback: F,
) -> Result<R, Value>
where
    F: FnOnce(&mut RedisItem) -> R,
{
    let mut cache = cache.lock().unwrap();
    match cache.get_mut(key) {
        Some(item) => Ok(callback(item)),
        None => Err(Value::Error("ERR no such key".to_string())),
    }
}

pub fn read_length_encoding(buffer: &[u8], index: usize) -> Result<(u64, usize), Box<dyn Error>> {
    if index >= buffer.len() {
        return Err("Index out of bounds while reading length encoding".into());
    }
    let first_byte = buffer[index];
    let prefix = first_byte >> 6;
    match prefix {
        0b00 => {
            let length = (first_byte & 0x3F) as u64;
            Ok((length, index + 1))
        }
        0b01 => {
            if index + 2 > buffer.len() {
                return Err("Insufficient bytes for 2-byte length encoding".into());
            }
            let length = (((first_byte & 0x3F) as u64) << 8) | (buffer[index + 1] as u64);
            Ok((length, index + 2))
        }
        0b10 => {
            if index + 5 > buffer.len() {
                return Err("Insufficient bytes for 5-byte length encoding".into());
            }
            let length = u32::from_be_bytes([
                buffer[index + 1],
                buffer[index + 2],
                buffer[index + 3],
                buffer[index + 4],
            ]) as u64;
            Ok((length, index + 5))
        }
        _ => {
            // Handle special encoding (11)
            // For simplicity, skip implementation here
            Err("Special length encoding not implemented".into())
        }
    }
}

pub fn read_expiry(buffer: &[u8], index: usize) -> Result<(Option<i64>, usize), Box<dyn Error>> {
    if index >= buffer.len() {
        return Err("Index out of bounds while reading expiry".into());
    }
    let opcode = buffer[index];
    match opcode {
        0xFD => {
            if index + 5 > buffer.len() {
                return Err("Insufficient bytes for EXPIRETIME".into());
            }
            let expiry = u32::from_be_bytes([
                buffer[index + 1],
                buffer[index + 2],
                buffer[index + 3],
                buffer[index + 4],
            ]) as i64;
            Ok((Some(expiry), index + 5))
        }
        0xFC => {
            if index + 9 > buffer.len() {
                return Err("Insufficient bytes for EXPIRETIMEMS".into());
            }
            let expiry = u64::from_be_bytes([
                buffer[index + 1],
                buffer[index + 2],
                buffer[index + 3],
                buffer[index + 4],
                buffer[index + 5],
                buffer[index + 6],
                buffer[index + 7],
                buffer[index + 8],
            ]) as i64;
            Ok((Some(expiry), index + 9))
        }
        _ => Ok((None, index)),
    }
}

pub fn read_byte(buffer: &[u8], index: usize) -> Result<(u8, usize), Box<dyn Error>> {
    if index >= buffer.len() {
        return Err("Index out of bounds while reading a byte".into());
    }
    Ok((buffer[index], index + 1))
}

pub fn read_string(buffer: &[u8], index: usize) -> Result<(String, usize), Box<dyn Error>> {
    let (length, new_index) = read_length_encoding(buffer, index)?;
    if new_index + (length as usize) > buffer.len() {
        return Err("String length exceeds buffer size".into());
    }
    let s = String::from_utf8_lossy(&buffer[new_index..new_index + (length as usize)]).to_string();
    Ok((s, new_index + (length as usize)))
}

pub fn read_encoded_value(
    buffer: &[u8],
    index: usize,
    value_type: u8,
) -> Result<(Value, usize), Box<dyn Error>> {
    match value_type {
        0 => {
            // String Encoding
            let (s, new_index) = read_string(buffer, index)?;
            Ok((Value::BulkString(s), new_index))
        }
        // Implement other value types based on your needs
        _ => Err(format!("Unsupported value type: {}", value_type).into()),
    }
}

pub fn infer_redis_type(value_type: u8) -> RedisType {
    match value_type {
        0 => RedisType::String,
        1 => RedisType::List,
        2 => RedisType::Set,
        3 => RedisType::ZSet,
        4 => RedisType::Hash,
        _ => RedisType::None,
    }
}
