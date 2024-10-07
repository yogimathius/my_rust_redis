use lazy_static::lazy_static;

use std::collections::{HashMap, HashSet};
use std::fmt::Arguments;

use anyhow::Result;
use bytes::BytesMut;

use crate::models::value::Value;
use crate::my_redis_server::redis_item::RedisItem;

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

pub fn extract_command(value: &Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => {
            if a.is_empty() {
                return Err(anyhow::anyhow!("Empty command array"));
            }
            let command_value = &a[0];
            let command = unpack_bulk_str(command_value)?;
            log!("Command: {:?}", command);
            let args = a[1..].to_vec(); // Collect the rest as args
            Ok((command, args))
        }
        Value::SimpleString(s) => Ok((s.clone(), vec![])),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

pub fn unpack_bulk_str(value: &Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(std::str::from_utf8(s).unwrap().to_string()),
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
    log!("Parsing message in parse_message: {:?}", buffer);
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(&buffer),
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

fn parse_bulk_string(buffer: &[u8]) -> Result<(Value, usize), anyhow::Error> {
    log!("Parsing bulk string: {:?}", buffer);
    let mut idx = 1; // Skip the '$' character
    let (length_line, len_consumed) = read_until_crlf(&buffer[idx..]).unwrap();
    idx += len_consumed;

    let bulk_length = parse_int(length_line)?;
    if bulk_length < 0 {
        return Ok((Value::NullBulkString, idx));
    }
    let bulk_length = bulk_length as usize;

    let total_needed = idx + bulk_length + 2; // +2 for '\r\n'
    log!("Total needed: {}", total_needed);
    log!("Buffer length: {}", buffer.len());
    if buffer.len() < total_needed {
        return Err(anyhow::anyhow!("Incomplete"));
    }

    let data = &buffer[idx..idx + bulk_length];
    idx += bulk_length + 2; // Move past data and '\r\n'
    log!("Data: {:?}", data);
    Ok((Value::BulkString(data.to_vec()), idx))
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

    match option.as_str() {
        "NX" => {
            return item.expiration.is_none();
        }
        "XX" => {
            log!("item.expiration {:?}", item.expiration);
            return item.expiration.is_some();
        }
        "GT" => {
            return item.expiration.is_some() && item.expiration.unwrap() < expiration;
        }
        "LT" => {
            return item.expiration.is_some() && item.expiration.unwrap() > expiration;
        }
        _ => {
            log!("no option");

            return true;
        }
    }
}

pub fn extract_args(args: Vec<Value>) -> (Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>, Vec<Value>) {
    let mut iter = args.into_iter();

    let key = match iter.next() {
        Some(Value::BulkString(s)) => s,
        _ => "".to_string().into(),
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

pub async fn lock_and_get_item<'a, F, R>(
    mut cache: HashMap<String, RedisItem>,
    key: &str,
    callback: F,
) -> Result<R, Value>
where
    F: FnOnce(&mut RedisItem) -> R,
{
    match cache.get_mut(key) {
        Some(item) => Ok(callback(item)),
        None => Err(Value::Error("ERR no such key".to_string())),
    }
}
