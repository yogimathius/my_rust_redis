use std::collections::HashMap;

use crate::{model::Value, server::Server};
use lazy_static::lazy_static;

type CommandHandler = Box<dyn Fn(&mut Server, Vec<Value>) -> Option<Value> + Send + Sync>;

fn ok_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

fn ping_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("PONG".to_string()))
}

fn echo_handler(_: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(args.first().unwrap().clone())
}

fn replconf_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

fn flushall_handler(server: &mut Server, _: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    cache.clear();
    Some(Value::SimpleString("OK".to_string()))
}

fn wrap_no_args<F>(f: F) -> Box<dyn Fn(&mut Server, Vec<Value>) -> Option<Value> + Send + Sync>
where
    F: Fn(&mut Server) -> Option<Value> + Send + Sync + 'static,
{
    Box::new(move |server, _| f(server))
}

fn wrap_immutable_no_args<F>(
    f: F,
) -> Box<dyn Fn(&mut Server, Vec<Value>) -> Option<Value> + Send + Sync>
where
    F: Fn(&Server) -> Option<Value> + Send + Sync + 'static,
{
    Box::new(move |server, _| f(server))
}

lazy_static! {
    pub static ref COMMAND_HANDLERS: HashMap<&'static str, CommandHandler> = {
        let mut handlers: HashMap<&str, CommandHandler> = HashMap::new();
        handlers.insert("PING", Box::new(ping_handler));
        handlers.insert("ECHO", Box::new(echo_handler));
        handlers.insert("SET", Box::new(Server::set));
        handlers.insert("GET", Box::new(Server::get));
        handlers.insert("INFO", wrap_immutable_no_args(Server::info));
        handlers.insert("REPLCONF", Box::new(replconf_handler));
        handlers.insert("PSYNC", wrap_no_args(Server::sync));
        handlers.insert("FLUSHALL", Box::new(flushall_handler));

                // Placeholder handlers
        handlers.insert("KEYS", Box::new(ok_handler));
        handlers.insert("TYPE", Box::new(ok_handler));
        handlers.insert("DEL", Box::new(ok_handler));
        handlers.insert("UNLINK", Box::new(ok_handler));
        handlers.insert("EXPIRE", Box::new(ok_handler));
        handlers.insert("RENAME", Box::new(ok_handler));
        handlers.insert("LLEN", Box::new(ok_handler));
        handlers.insert("LREM", Box::new(ok_handler));
        handlers.insert("LINDEX", Box::new(ok_handler));
        handlers.insert("LPOP", Box::new(ok_handler));
        handlers.insert("RPOP", Box::new(ok_handler));
        handlers.insert("LSET", Box::new(ok_handler));
        handlers.insert("HGET", Box::new(ok_handler));
        handlers.insert("HEXISTS", Box::new(ok_handler));
        handlers.insert("HDEL", Box::new(ok_handler));
        handlers.insert("HGETALL", Box::new(ok_handler));
        handlers.insert("HKEYS", Box::new(ok_handler));
        handlers.insert("HLEN", Box::new(ok_handler));
        handlers.insert("HMSET", Box::new(ok_handler));
        handlers.insert("HSET", Box::new(ok_handler));
        handlers.insert("HVALS", Box::new(ok_handler));
        handlers
    };
}
