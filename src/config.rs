use std::path::Path;
use serde::{Deserialize, Serialize};
use std::{fs, io};


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
    /// The port the http server will listen on
    pub port: Option<u16>,
    /// A list of keypairs that the server will handle.
    pub keys: Vec<KeyRing>,
}

#[derive(Debug, Clone)]
pub struct ConfigError;


/// Load configuration data from a path and parse it into `ConfigData`
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
