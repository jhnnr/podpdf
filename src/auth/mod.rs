use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Unauthorized: valid session token or API key required")]
    Unauthorized,
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(alias = "username")]
    pub email: String,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistentAuthStore {
    pub users: Vec<User>,
    pub api_keys: Vec<ApiKeyEntry>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub email: String,
    pub created_at: std::time::Instant,
}

pub struct AuthManager {
    store_path: PathBuf,
    store: RwLock<PersistentAuthStore>,
    sessions: RwLock<HashMap<String, Session>>,
}

impl AuthManager {
    /// Initializes or loads the authentication store from disk.
    pub fn new<P: AsRef<Path>>(store_path: P) -> Arc<Self> {
        let path = store_path.as_ref().to_path_buf();
        let mut store: PersistentAuthStore = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => PersistentAuthStore::default(),
            }
        } else {
            PersistentAuthStore::default()
        };

        let mut needs_save = !path.exists();

        // Seed default admin user if store has no users
        if store.users.is_empty() {
            let (salt, hash) = hash_password("admin");
            store.users.push(User {
                email: "admin@podpdf.local".to_string(),
                password_hash: hash,
                salt,
            });
            needs_save = true;
        } else {
            // Automatically migrate legacy "admin" username to "admin@podpdf.local"
            for user in &mut store.users {
                if user.email.eq_ignore_ascii_case("admin") {
                    user.email = "admin@podpdf.local".to_string();
                    needs_save = true;
                }
            }
        }

        if needs_save {
            let _ = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")));
            let _ = std::fs::write(
                &path,
                serde_json::to_string_pretty(&store).unwrap_or_default(),
            );
        }

        Arc::new(Self {
            store_path: path,
            store: RwLock::new(store),
            sessions: RwLock::new(HashMap::new()),
        })
    }

    /// Authenticates a user with email and password, returning a new session token.
    pub async fn login(&self, email: &str, password: &str) -> Result<String, AuthError> {
        let store = self.store.read().await;
        let id_clean = email.trim();
        let user = store
            .users
            .iter()
            .find(|u| {
                u.email.eq_ignore_ascii_case(id_clean)
                    || (u.email.eq_ignore_ascii_case("admin@podpdf.local")
                        && id_clean.eq_ignore_ascii_case("admin"))
                    || (u.email.eq_ignore_ascii_case("admin")
                        && id_clean.eq_ignore_ascii_case("admin@podpdf.local"))
            })
            .ok_or(AuthError::InvalidCredentials)?;

        let computed_hash = hash_password_with_salt(password, &user.salt);
        if computed_hash != user.password_hash {
            return Err(AuthError::InvalidCredentials);
        }

        let token = generate_random_token();
        let mut sessions = self.sessions.write().await;
        sessions.insert(
            token.clone(),
            Session {
                email: user.email.clone(),
                created_at: std::time::Instant::now(),
            },
        );

        Ok(token)
    }

    /// Terminates an active session token.
    pub async fn logout(&self, token: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(token);
    }

    /// Checks if a session token is valid and returns the user's email.
    pub async fn validate_session(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(token).map(|s| s.email.clone())
    }

    /// Validates an API key against stored hashed keys.
    pub async fn validate_api_key(&self, key: &str) -> bool {
        let key_hash = hash_string(key);
        let store = self.store.read().await;
        store.api_keys.iter().any(|k| k.key_hash == key_hash)
    }

    /// Validates authentication credentials from either a session token or an API key.
    pub async fn validate_any(&self, auth_header_or_token: &str) -> bool {
        let token = auth_header_or_token
            .trim_start_matches("Bearer ")
            .trim_start_matches("bearer ")
            .trim();

        if token.starts_with("podpdf_") {
            self.validate_api_key(token).await
        } else {
            self.validate_session(token).await.is_some()
        }
    }

    /// Changes email and/or password for an existing authenticated user.
    pub async fn change_credentials(
        &self,
        current_email: &str,
        new_email: Option<String>,
        current_password: &str,
        new_password: Option<String>,
    ) -> Result<(), AuthError> {
        let mut store = self.store.write().await;
        let user = store
            .users
            .iter_mut()
            .find(|u| u.email.eq_ignore_ascii_case(current_email))
            .ok_or(AuthError::UserNotFound)?;

        let computed_current = hash_password_with_salt(current_password, &user.salt);
        if computed_current != user.password_hash {
            return Err(AuthError::InvalidCredentials);
        }

        if let Some(new_addr) = new_email {
            let trimmed = new_addr.trim();
            if !validate_email(trimmed) {
                return Err(AuthError::BadRequest(
                    "Invalid email address format".to_string(),
                ));
            }
            user.email = trimmed.to_string();
        }

        if let Some(new_pwd) = new_password {
            if new_pwd.len() < 4 {
                return Err(AuthError::BadRequest(
                    "Password must be at least 4 characters long".to_string(),
                ));
            }
            let (new_salt, new_hash) = hash_password(&new_pwd);
            user.salt = new_salt;
            user.password_hash = new_hash;
        }

        self.save_store(&store)
    }

    /// Generates a new API key, storing its hash and returning the raw secret key.
    pub async fn generate_api_key(&self, name: &str) -> Result<(String, ApiKeyEntry), AuthError> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(AuthError::BadRequest(
                "API key name cannot be empty".to_string(),
            ));
        }

        let id = generate_random_token();
        let raw_key = format!("podpdf_{}", generate_random_token());
        let key_hash = hash_string(&raw_key);
        let key_prefix = format!("{}...", &raw_key[..14]);

        let entry = ApiKeyEntry {
            id,
            name: trimmed_name.to_string(),
            key_hash,
            key_prefix,
            created_at: format_timestamp_now(),
        };

        let mut store = self.store.write().await;
        store.api_keys.push(entry.clone());
        self.save_store(&store)?;

        Ok((raw_key, entry))
    }

    /// Lists all existing API keys without secrets.
    pub async fn list_api_keys(&self) -> Vec<ApiKeyEntry> {
        let store = self.store.read().await;
        store.api_keys.clone()
    }

    /// Revokes an API key by identifier.
    pub async fn revoke_api_key(&self, id: &str) -> Result<(), AuthError> {
        let mut store = self.store.write().await;
        let initial_len = store.api_keys.len();
        store.api_keys.retain(|k| k.id != id);

        if store.api_keys.len() == initial_len {
            return Err(AuthError::BadRequest("API key not found".to_string()));
        }

        self.save_store(&store)
    }

    fn save_store(&self, store: &PersistentAuthStore) -> Result<(), AuthError> {
        let json =
            serde_json::to_string_pretty(store).map_err(|e| AuthError::Storage(e.to_string()))?;
        if let Some(parent) = self.store_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.store_path, json).map_err(|e| AuthError::Storage(e.to_string()))?;
        Ok(())
    }
}

fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return false;
    }
    let domain = parts[1];
    domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn generate_random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    let mut hex = String::with_capacity(32);
    for b in bytes {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

fn hash_password(password: &str) -> (String, String) {
    let mut salt_bytes = [0u8; 16];
    rand::rng().fill(&mut salt_bytes);
    let mut salt = String::with_capacity(32);
    for b in salt_bytes {
        salt.push_str(&format!("{:02x}", b));
    }
    let hash = hash_password_with_salt(password, &salt);
    (salt, hash)
}

fn hash_password_with_salt(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in result {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

fn hash_string(val: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(val.as_bytes());
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in result {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

fn format_timestamp_now() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("Unix timestamp {}", d.as_secs()),
        Err(_) => "Unknown".to_string(),
    }
}
