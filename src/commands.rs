use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::handlers::*;
use crate::models::value::Value;
use crate::server::Server;
use lazy_static::lazy_static;

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(
        &self,
        server: Arc<Mutex<Server>>,
        key: String,
        args: Vec<Value>,
    ) -> Option<Value>;
}

pub struct SyncCommandHandler<F>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Option<Value> + Send + Sync,
{
    handler: F,
}

impl<F> SyncCommandHandler<F>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Option<Value> + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> CommandHandler for SyncCommandHandler<F>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Option<Value> + Send + Sync,
{
    async fn handle(
        &self,
        server: Arc<Mutex<Server>>,
        key: String,
        args: Vec<Value>,
    ) -> Option<Value> {
        (self.handler)(server, key, args)
    }
}

pub struct AsyncCommandHandler<F, Fut>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Option<Value>> + Send,
{
    handler: F,
}

impl<F, Fut> AsyncCommandHandler<F, Fut>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Option<Value>> + Send,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F, Fut> CommandHandler for AsyncCommandHandler<F, Fut>
where
    F: Fn(Arc<Mutex<Server>>, String, Vec<Value>) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Option<Value>> + Send,
{
    async fn handle(
        &self,
        server: Arc<Mutex<Server>>,
        key: String,
        args: Vec<Value>,
    ) -> Option<Value> {
        (self.handler)(server, key, args).await
    }
}

lazy_static! {
    pub static ref COMMAND_HANDLERS: HashMap<&'static str, Box<dyn CommandHandler>> = {
        let mut handlers: HashMap<&str, Box<dyn CommandHandler>> = HashMap::new();
        // Basic commands
        handlers.insert("PING", Box::new(SyncCommandHandler::new(ping_handler)));
        handlers.insert("ECHO", Box::new(SyncCommandHandler::new(echo_handler)));
        handlers.insert("SET", Box::new(AsyncCommandHandler::new(set_handler)));
        handlers.insert("GET", Box::new(AsyncCommandHandler::new(get_handler)));
        handlers.insert("INFO", Box::new(AsyncCommandHandler::new(info_handler)));

        // Replication commands
        handlers.insert("REPLCONF", Box::new(SyncCommandHandler::new(replconf_handler)));
        handlers.insert("PSYNC", Box::new(AsyncCommandHandler::new(psync_handler)));

        // Server commands
        handlers.insert("FLUSHALL", Box::new(AsyncCommandHandler::new(flushall_handler)));

        // Key management commands
        handlers.insert("KEYS", Box::new(AsyncCommandHandler::new(keys_handler)));
        handlers.insert("TYPE", Box::new(AsyncCommandHandler::new(type_handler)));
        handlers.insert("DEL", Box::new(AsyncCommandHandler::new(del_handler)));
        handlers.insert("UNLINK", Box::new(AsyncCommandHandler::new(unlink_handler)));
        handlers.insert("EXPIRE", Box::new(AsyncCommandHandler::new(expire_handler)));
        handlers.insert("RENAME", Box::new(AsyncCommandHandler::new(rename_handler)));

        // List commands
        handlers.insert("LLEN", Box::new(AsyncCommandHandler::new(llen_handler)));
        handlers.insert("LREM", Box::new(AsyncCommandHandler::new(lrem_handler)));
        handlers.insert("LINDEX", Box::new(AsyncCommandHandler::new(lindex_handler)));
        handlers.insert("LPOP", Box::new(AsyncCommandHandler::new(lpop_handler)));
        handlers.insert("RPOP", Box::new(AsyncCommandHandler::new(rpop_handler)));
        handlers.insert("LSET", Box::new(AsyncCommandHandler::new(lset_handler)));

        // Hash commands
        handlers.insert("HGET", Box::new(AsyncCommandHandler::new(hget_handler)));
        handlers.insert("HEXISTS", Box::new(AsyncCommandHandler::new(hexists_handler)));
        handlers.insert("HDEL", Box::new(AsyncCommandHandler::new(hdel_handler)));
        handlers.insert("HGETALL", Box::new(AsyncCommandHandler::new(hgetall_handler)));
        handlers.insert("HKEYS", Box::new(AsyncCommandHandler::new(hkeys_handler)));
        handlers.insert("HLEN", Box::new(AsyncCommandHandler::new(hlen_handler)));
        handlers.insert("HMSET", Box::new(SyncCommandHandler::new(hmset_handler)));
        handlers.insert("HSET", Box::new(AsyncCommandHandler::new(hset_handler)));
        handlers.insert("HVALS", Box::new(AsyncCommandHandler::new(hvals_handler)));
        handlers
    };
}
