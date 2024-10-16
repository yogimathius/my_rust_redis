use lazy_static::lazy_static;

use std::collections::{HashMap, HashSet};
use std::fmt::Arguments;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::BytesMut;

use crate::models::redis_item::RedisItem;
use crate::models::value::Value;
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
        ':' => parse_integer(buffer),
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

fn parse_integer(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let integer = parse_int(line)?;
        Ok((Value::Integer(integer), len + 1))
    } else {
        Err(anyhow::anyhow!("Invalid integer format {:?}", buffer))
    }
}

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
