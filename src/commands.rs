use std::collections::HashMap;

use crate::handlers::*;

use crate::models::value::Value;
use crate::server::Server;
use lazy_static::lazy_static;

type CommandHandler = Box<dyn Fn(&mut Server, String, Vec<Value>) -> Option<Value> + Send + Sync>;

fn wrap_no_args<F>(
    f: F,
) -> Box<dyn Fn(&mut Server, String, Vec<Value>) -> Option<Value> + Send + Sync>
where
    F: Fn(&mut Server) -> Option<Value> + Send + Sync + 'static,
{
    Box::new(move |server, _, _| f(server))
}

fn wrap_immutable_no_args<F>(
    f: F,
) -> Box<dyn Fn(&mut Server, String, Vec<Value>) -> Option<Value> + Send + Sync>
where
    F: Fn(&Server) -> Option<Value> + Send + Sync + 'static,
{
    Box::new(move |server, _, _| f(server))
}

lazy_static! {
    pub static ref COMMAND_HANDLERS: HashMap<&'static str, CommandHandler> = {
        let mut handlers: HashMap<&str, CommandHandler> = HashMap::new();
        // Basic commands
        handlers.insert("PING", Box::new(ping_handler));
        handlers.insert("ECHO", Box::new(echo_handler));
        handlers.insert("SET", Box::new(set_handler));
        handlers.insert("GET", Box::new(get_handler));
        handlers.insert("INFO", wrap_immutable_no_args(info_handler));

        // Replication commands
        handlers.insert("REPLCONF", Box::new(replconf_handler));
        handlers.insert("PSYNC", wrap_no_args(psync_handler));

        // Server commands
        handlers.insert("FLUSHALL", Box::new(flushall_handler));


        // Key management commands
        // Returns all keys matching pattern.
        handlers.insert("KEYS", Box::new(keys_handler));

        // Returns the type of the value stored at key.
        handlers.insert("TYPE", Box::new(type_handler));

        // Removes the specified keys. A key is ignored if it does not exist.
        handlers.insert("DEL", Box::new(del_handler));

        // Removes the specified keys. A key is ignored if it does not exist.
        handlers.insert("UNLINK", Box::new(unlink_handler));

        // Set a key's time to live in seconds.
        handlers.insert("EXPIRE", Box::new(expire_handler));

        // Renames key to newkey. It returns an error if key does not exist.
        handlers.insert("RENAME", Box::new(rename_handler));

        // List commands
        // Returns the length of the list stored at key.
        handlers.insert("LLEN", Box::new(llen_handler));

        // Removes the first count occurrences of elements equal to element from the list stored at key.
        handlers.insert("LREM", Box::new(lrem_handler));

        // Returns the element at index index in the list stored at key.
        handlers.insert("LINDEX", Box::new(lindex_handler));

        // Removes and returns the first element of the list stored at key.
        handlers.insert("LPOP", Box::new(lpop_handler));

        // Removes and returns the last element of the list stored at key.
        handlers.insert("RPOP", Box::new(rpop_handler));

        // Returns the specified elements of the list stored at key.
        handlers.insert("LSET", Box::new(lset_handler));


        // Hash commands
        // Returns the value associated with field in the hash stored at key.
        handlers.insert("HGET", Box::new(hget_handler));

        // Returns the value associated with field in the hash stored at key.
        handlers.insert("HEXISTS", Box::new(hexists_handler));

        // Removes the specified fields from the hash stored at key.
        handlers.insert("HDEL", Box::new(hdel_handler));

        // Returns all field names in the hash stored at key.
        handlers.insert("HGETALL", Box::new(hgetall_handler));

        // Returns all field names in the hash stored at key.
        handlers.insert("HKEYS", Box::new(hkeys_handler));

        // Returns the number of fields contained in the hash stored at key.
        handlers.insert("HLEN", Box::new(hlen_handler));

        // Sets the specified fields to their respective values in the hash stored at key.
        handlers.insert("HMSET", Box::new(hmset_handler));

        // Sets field in the hash stored at key to value.
        handlers.insert("HSET", Box::new(hset_handler));

        // Returns all values in the hash stored at key.
        handlers.insert("HVALS", Box::new(hvals_handler));
        handlers
    };
}
