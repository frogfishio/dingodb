// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use mongodb::Client;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::error::{K2DbError, ServiceError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientCacheKey(String);

impl ClientCacheKey {
    pub fn build(config: &DatabaseConfig) -> Self {
        let hosts = config
            .hosts
            .iter()
            .map(|host| format!("{}:{}", host.host.trim(), host.port.map(|port| port.to_string()).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join(",");

        let password_hash = config.password.as_ref().map(|password| {
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            format!("sha256:{:x}", hasher.finalize())
        }).unwrap_or_default();

        let key = format!(
            "hosts={hosts}|user={}|pass={password_hash}|authSource={}|rs={}",
            config.user.as_deref().unwrap_or_default(),
            config.auth_source.as_deref().unwrap_or("admin"),
            config.replica_set.as_deref().unwrap_or_default(),
        );

        Self(key)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
struct PoolEntry {
    client: Arc<Client>,
    refs: usize,
}

fn registry() -> &'static Mutex<HashMap<String, PoolEntry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, PoolEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn acquire(config: &DatabaseConfig) -> Result<(ClientCacheKey, Arc<Client>), K2DbError> {
    let key = ClientCacheKey::build(config);

    {
        let mut locked = registry().lock().await;
        if let Some(existing) = locked.get_mut(key.as_str()) {
            existing.refs += 1;
            return Ok((key, Arc::clone(&existing.client)));
        }
    }

    let client = Arc::new(
        Client::with_options(config.client_options().await?).map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::ServiceUnavailable,
                Some("sys_mdb_init".to_owned()),
                "Failed to create MongoDB client",
            )
        })?,
    );

    let mut locked = registry().lock().await;
    if let Some(existing) = locked.get_mut(key.as_str()) {
        existing.refs += 1;
        return Ok((key, Arc::clone(&existing.client)));
    }

    locked.insert(
        key.as_str().to_owned(),
        PoolEntry {
            client: Arc::clone(&client),
            refs: 1,
        },
    );

    Ok((key, client))
}

pub async fn release(key: &ClientCacheKey) {
    let mut locked = registry().lock().await;
    let should_remove = match locked.get_mut(key.as_str()) {
        Some(entry) if entry.refs > 1 => {
            entry.refs -= 1;
            false
        }
        Some(_) => true,
        None => false,
    };

    if should_remove {
        locked.remove(key.as_str());
    }
}

pub async fn reset_for_tests() {
    registry().lock().await.clear();
}