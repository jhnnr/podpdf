use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path;

pub const HARDENED_AUTH_STORE_PATH: &str = "data/auth_store.json";

/// Server runtime configuration loaded exclusively from .env and environment variables.
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
            auth_store_path: HARDENED_AUTH_STORE_PATH.to_string(),
            custom_headers: HashMap::new(),
        }
    }
}

impl Config {
    /// Loads configuration from .env and system environment variables.
    pub fn load() -> Self {
        let env_map = parse_env_file();
        let mut config = Self::default();

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

        // AUTH_STORE_PATH is hardened in source code
        config.auth_store_path = HARDENED_AUTH_STORE_PATH.to_string();

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

    /// Persists dynamic runtime configuration changes back to the .env file.
    pub fn save_to_env(&self) {
        let env_path = Path::new(".env");
        let mut existing_lines: Vec<String> = if env_path.exists() {
            std::fs::read_to_string(env_path)
                .unwrap_or_default()
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        let headers_str = self
            .custom_headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(";");

        let updates: [(&str, String); 5] = [
            ("HOST", self.host.clone()),
            ("PORT", self.port.to_string()),
            ("MAX_PAYLOAD_SIZE", self.max_payload_size.to_string()),
            ("AUTH_ENABLED", self.auth_enabled.to_string()),
            ("CUSTOM_HEADERS", headers_str),
        ];

        for (key, val) in updates {
            let prefix = format!("{}=", key);
            let mut found = false;
            for line in &mut existing_lines {
                if line.trim_start().starts_with(&prefix) {
                    *line = format!("{}={}", key, val);
                    found = true;
                    break;
                }
            }
            if !found {
                existing_lines.push(format!("{}={}", key, val));
            }
        }

        let _ = std::fs::write(env_path, existing_lines.join("\n") + "\n");
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
