use bincode;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::log;
use crate::models::redis_item::RedisItem;

#[derive(Clone)]
pub struct Database {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    pub path: String,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("I/O Error")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error")]
    Serialization(#[from] bincode::Error),

    #[error("Data Corruption Detected")]
    DataCorruption,
}

impl Database {
    /// Initializes the Database struct.
    pub fn new(cache: Arc<Mutex<HashMap<String, RedisItem>>>, path: &str) -> Self {
        Database {
            cache,
            path: path.to_string(),
        }
    }

    pub fn dump_backup(&self) -> Result<(), DatabaseError> {
        let temp_path = format!("{}.tmp", &self.path);
        let temp_path = Path::new(&temp_path);
        let file = File::create(temp_path)?;
        let mut writer = BufWriter::new(file);

        let cache = self.cache.lock().unwrap();
        let serialized = bincode::serialize(&*cache)?;

        writer.write_all(&serialized)?;
        writer.flush()?;
        drop(writer);

        fs::rename(temp_path, &self.path)?;

        Ok(())
    }

    pub fn read_backup(&self) -> Result<(), DatabaseError> {
        let path = Path::new(&self.path);
        if !path.exists() {
            log!("Backup file does not exist. Starting with an empty cache.");
            return Ok(());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let deserialized: HashMap<String, RedisItem> = bincode::deserialize(&buffer)?;

        let mut cache = self.cache.lock().unwrap();
        *cache = deserialized;

        log!("Backup loaded successfully.");
        Ok(())
    }
}
