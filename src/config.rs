use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io, env};
use base64::{engine::general_purpose, Engine as _};

/// A set of inbound and outbound keys.
/// Requests sent to an inbound DSN are mirrored to all outbound DSNs
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyRing {
    /// Inbound keys are virtual DSNs that the mirror will accept traffic on
    pub inbound: Option<String>,
    /// One or more upstream DSN keys that the mirror will forward traffic to.
    pub outbound: Vec<Option<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigData {
    /// The inbound IP to use. Defaults to 127.0.0.1
    pub ip: Option<String>,
    /// The port the http server will listen on
    pub port: Option<u16>,
    /// A list of keypairs that the server will handle.
    pub keys: Vec<KeyRing>,
}

#[derive(Debug, Clone)]
pub struct ConfigError;

/// Load configuration data from a path and parse it into `ConfigData`
pub fn load_config(path: &Path) -> Result<ConfigData, ConfigError> {
    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Err(ConfigError),
    };
    let configdata = match serde_yaml::from_reader(io::BufReader::new(f)) {
        Ok(data) => data,
        Err(_) => return Err(ConfigError),
    };
    Ok(configdata)
}

pub fn load_b64_config(key: &str) -> Result<ConfigData, ConfigError> {
    let b64 = match env::var(key) {
        Ok(b64) => b64,
        Err(_) => return Err(ConfigError),
    };
    let decoded = match general_purpose::STANDARD.decode(&b64) {
        Ok(decoded) => decoded,
        Err(_) => return Err(ConfigError),
    };
    let decoded_str = match String::from_utf8(decoded) {
        Ok(decoded_str) => decoded_str,
        Err(_) => return Err(ConfigError),
    };

    let configdata = match serde_yaml::from_str(&decoded_str) {
        Ok(data) => data,
        Err(_) => return Err(ConfigError),
    };
    Ok(configdata)
}