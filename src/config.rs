use std::path::Path;
use serde::{Deserialize, Serialize};
use std::{fs, io};


/// A set of inbound and outbound keys.
/// Requests sent to an inbound DSN are mirrored to all outbound DSNs
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub inbound: Option<String>,
    pub outbound: Vec<Option<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigData {
    pub port: Option<String>,
    pub keys: Vec<Key>,
}

#[derive(Debug, Clone)]
pub struct ConfigError;


pub fn load_config(path: &Path) -> Result<ConfigData, ConfigError> {
    let f = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return Err(ConfigError),
    };
    let configdata = match serde_yaml::from_reader(io::BufReader::new(f)) {
        Ok(data) => data,
        Err(_) => return Err(ConfigError),
    };
    Ok(configdata)
}
