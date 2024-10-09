use bincode;
use serde::{Deserialize, Serialize};
use serde_json; // Not needed if using bincode
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::server::RedisItem;

// Ensure your RedisItem, Value, and RedisType structs/enums derive Serialize and Deserialize

pub struct Database {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    pub path: String,
}

impl Database {
    /// Initializes the Database struct.
    pub fn new(cache: Arc<Mutex<HashMap<String, RedisItem>>>, path: &str) -> Self {
        Database {
            cache,
            path: path.to_string(),
        }
    }

    /// Dumps the in-memory cache to a backup file using Bincode.
    pub fn dump_backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.path);
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let cache = self.cache.lock().unwrap();
        let serialized = bincode::serialize(&*cache)?;

        writer.write_all(&serialized)?;
        writer.flush()?;
        Ok(())
    }
}
