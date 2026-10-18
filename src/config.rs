use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path;

/// Server runtime configuration loaded from file, environment variables, or .env file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_payload_size: usize,
    pub log_level: String,
    pub auth_enabled: bool,
    pub auth_store_path: String,
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_payload_size: 50 * 1024 * 1024, // 50 MB
            log_level: "info".to_string(),
            auth_enabled: true,
            auth_store_path: "data/auth_store.json".to_string(),
            custom_headers: HashMap::new(),
        }
    }
}

impl Config {
    /// Loads configuration from config.toml, .env, and system environment variables.
    pub fn load() -> Self {
        let env_map = parse_env_file();

        let mut config = if Path::new("config.toml").exists() {
            match std::fs::read_to_string("config.toml") {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        };

        let get_val = |key: &str| -> Option<String> {
            env::var(key).ok().or_else(|| env_map.get(key).cloned())
        };

        if let Some(host) = get_val("HOST") {
            config.host = host;
        }

        if let Some(port_str) = get_val("PORT")
            && let Ok(port) = port_str.parse()
        {
            config.port = port;
        }

        if let Some(max_size_str) = get_val("MAX_PAYLOAD_SIZE")
            && let Ok(max_size) = max_size_str.parse()
        {
            config.max_payload_size = max_size;
        }

        if let Some(auth_enabled_str) = get_val("AUTH_ENABLED") {
            config.auth_enabled =
                auth_enabled_str.eq_ignore_ascii_case("true") || auth_enabled_str == "1";
        }

        if let Some(auth_store) = get_val("AUTH_STORE_PATH") {
            config.auth_store_path = auth_store;
        }

        if let Some(log_level) = get_val("RUST_LOG") {
            config.log_level = log_level;
        }

        // Parse custom headers from env (format: "Header-One: Value1; Header-Two: Value2")
        if let Some(headers_str) = get_val("CUSTOM_HEADERS") {
            for pair in headers_str.split(';') {
                if let Some((k, v)) = pair.split_once(':') {
                    let k = k.trim().to_string();
                    let v = v.trim().to_string();
                    if !k.is_empty() && !v.is_empty() {
                        config.custom_headers.insert(k, v);
                    }
                }
            }
        }

        config
    }

    /// Resolves the socket address configured for the server.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}

fn parse_env_file() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let env_path = Path::new(".env");
    if env_path.exists()
        && let Ok(content) = std::fs::read_to_string(env_path)
    {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let k = k.trim().to_string();
                let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                map.insert(k, v);
            }
        }
    }
    map
}
