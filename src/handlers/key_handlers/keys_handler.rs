use crate::{log, models::value::Value, server::Server};
use regex::Regex;

pub fn keys_handler(server: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    log!("keys_handler handler {:?}", args);

    let pattern = match args.get(0) {
        Some(Value::BulkString(s)) => s,
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'keys' command".into(),
            ))
        }
    };

    let regex_pattern = glob_to_regex(pattern);
    let re = match Regex::new(&regex_pattern) {
        Ok(re) => re,
        Err(_) => return Some(Value::Error("ERR invalid pattern".into())),
    };

    let cache = server.cache.lock().unwrap();

    let mut matching_keys: Vec<Value> = cache
        .keys()
        .filter(|key| re.is_match(key))
        .map(|key| Value::BulkString(key.clone()))
        .collect();

    // Sort the keys as strings
    matching_keys.sort_by(|a, b| {
        match (a, b) {
            (Value::BulkString(a_str), Value::BulkString(b_str)) => a_str.cmp(b_str),
            _ => std::cmp::Ordering::Equal, // This case should not occur, but we need to handle it
        }
    });

    // 5. Return matching keys as a BulkString array
    Some(Value::Array(matching_keys))
}

fn glob_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::new();
    regex_pattern.push('^');
    for c in pattern.chars() {
        match c {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '%' => {
                regex_pattern.push('\\');
                regex_pattern.push(c);
            }
            _ => regex_pattern.push(c),
        }
    }
    regex_pattern.push('$');
    regex_pattern
}
