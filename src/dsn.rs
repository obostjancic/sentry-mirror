use std::collections::HashMap;

//use std::collections::HashMap;
use url::Url;

use crate::config::Key;

/// DSN components parsed from a DSN string
#[derive(Debug, Clone, PartialEq)]
pub struct Dsn {
    pub public_key: String,
    pub secret_key: String,
    pub project_id: String,
    pub host: String,
    pub path: String,
}

#[derive(Debug)]
pub enum DsnParseError {
    MissingPublicKey,
    MissingHost,
    MissingPath,
    MissingProjectId,
    InvalidUrl,
}

impl Dsn {
    pub fn from_string(input: String) -> Result<Self, DsnParseError> {
        let url = match Url::parse(&input) {
            Ok(u) => u,
            Err(_) => return Err(DsnParseError::InvalidUrl),
        };
        if url.username().len() < 1 {
            return Err(DsnParseError::MissingPublicKey);
        }
        let public_key = url.username().to_string();
        let secret_key = match url.password() {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };
        let host = match url.host_str() {
            Some(h) => h.to_string(),
            None => return Err(DsnParseError::MissingHost),
        };
        let path = url.path().to_string();
        let path_segments = match url.path_segments() {
            Some(s) => s,
            None => return Err(DsnParseError::MissingPath),

        };
        let project_id = match path_segments.last() {
            Some(p) => p.to_string(),
            None => return Err(DsnParseError::MissingProjectId),
        };
        println!("{:?}", project_id);
        if project_id == "/" || project_id == "" {
            return Err(DsnParseError::MissingProjectId);
        }

        Ok(Dsn {
            public_key,
            secret_key,
            project_id,
            host,
            path,
        })
    }

    /// Get a string of the key's identity.
    pub fn key_id(&self) -> String {
        let pubkey = &self.public_key;
        let projectid = &self.project_id;

        return format!("{pubkey}:{projectid}");
    }
}

pub fn make_key_map(keys: Vec<Key>) -> HashMap<String, Vec<Dsn>> {
    let mut keymap: HashMap<String, Vec<Dsn>> = HashMap::new();
    for item in keys {
        let inbound_dsn = match Dsn::from_string(item.inbound.expect("Missing inbound key")) {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        };
        let outbound = item.outbound.iter()
            .filter_map(|item| match item {
                Some(i) => Some(i),
                None => None,
            })
            .map(|outbound_str| {
                return Dsn::from_string(outbound_str.to_owned()).expect("Invalid outbound DSN")
            }).collect::<Vec<Dsn>>();
        keymap.insert(inbound_dsn.key_id(), outbound);
    }
    keymap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Key;

    #[test]
    fn parse_from_string_valid() {
        let dsn = Dsn::from_string("http://390bf7f953b7492c9007d2cf69078adf@localhost:8765/1847101".to_string()).unwrap();
        assert_eq!("390bf7f953b7492c9007d2cf69078adf", dsn.public_key);
        assert_eq!("localhost", dsn.host);
        assert_eq!("1847101", dsn.project_id);
    }

    #[test]
    fn parse_from_string_orgdomain() {
        let dsn_str = "https://d2030950946a6197f9cdb9633c069eea@o4507063958255996.ingest.de.sentry.io/4501063980026892";
        let dsn = Dsn::from_string(dsn_str.to_string()).unwrap();
        assert_eq!("d2030950946a6197f9cdb9633c069eea", dsn.public_key);
        assert_eq!("o4507063958255996.ingest.de.sentry.io", dsn.host);
        assert_eq!("4501063980026892", dsn.project_id);
        assert_eq!("", dsn.secret_key);
    }

    #[test]
    fn parse_from_string_missing_project_id() {
        let dsn_str = "https://abcdef@sentry.internal";
        let dsn = Dsn::from_string(dsn_str.to_string());
        assert_eq!(true, dsn.is_err());
    }

    #[test]
    fn parse_from_string_missing_empty_string() {
        let dsn_str = "";
        let dsn = Dsn::from_string(dsn_str.to_string());
        assert_eq!(true, dsn.is_err());
    }

    #[test]
    fn make_key_map_valid() {
        let keys = vec![
            Key {
                inbound: Some("https://abcdef@sentry.io/1234".to_string()),
                outbound: vec![
                    Some("https://ghijkl@sentry.io/567".to_string()),
                    Some("https://mnopq@sentry.io/890".to_string())
                ]
            }
        ];
        let keymap = make_key_map(keys);
        assert_eq!(keymap.len(), 1);
        let value = keymap.get("abcdef:1234").expect("Should have a value");
        assert_eq!(value.len(), 2);
    }
}
