// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::env;
use std::time::Duration;

use base64::Engine;
use mongodb::options::{ClientOptions, Credential};
use serde::{Deserialize, Serialize};

use crate::error::{K2DbError, ServiceError};
use crate::observability::QueryHooks;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConfig {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OwnershipMode {
    Lax,
    Strict,
}

impl Default for OwnershipMode {
    fn default() -> Self {
        Self::Lax
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationMode {
    Loose,
    Guarded,
    Strict,
}

impl Default for AggregationMode {
    fn default() -> Self {
        Self::Loose
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub key_id: String,
    #[serde(skip)]
    pub key: [u8; 32],
}

impl EncryptionConfig {
    pub fn from_base64(key_id: impl Into<String>, key_b64: &str) -> Result<Self, K2DbError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(key_b64.trim())
            .map_err(|_| {
                K2DbError::new(
                    ServiceError::ConfigurationError,
                    "secureFieldEncryptionKey must be base64-encoded",
                    Some("sys_mdb_secure_key_invalid".to_owned()),
                )
            })?;

        let key: [u8; 32] = decoded.try_into().map_err(|_| {
            K2DbError::new(
                ServiceError::ConfigurationError,
                "secureFieldEncryptionKey must decode to 32 bytes (AES-256)",
                Some("sys_mdb_secure_key_invalid".to_owned()),
            )
        })?;

        Ok(Self {
            key_id: key_id.into(),
            key,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub hosts: Vec<HostConfig>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub auth_source: Option<String>,
    pub replica_set: Option<String>,
    pub slow_query_ms: Option<u64>,
    pub ownership_mode: OwnershipMode,
    pub aggregation_mode: AggregationMode,
    pub secure_field_prefixes: Vec<String>,
    pub secure_field_encryption: Option<EncryptionConfig>,
    #[serde(skip, default)]
    pub hooks: QueryHooks,
}

impl DatabaseConfig {
    pub fn validate(&self) -> Result<(), K2DbError> {
        if self.name.trim().is_empty() {
            return Err(K2DbError::new(
                ServiceError::ConfigurationError,
                "Database name is required",
                Some("sys_mdb_no_hosts".to_owned()),
            ));
        }

        if self.hosts.is_empty() || self.hosts.iter().all(|host| host.host.trim().is_empty()) {
            return Err(K2DbError::new(
                ServiceError::ConfigurationError,
                "No valid hosts provided in configuration",
                Some("sys_mdb_no_hosts".to_owned()),
            ));
        }

        Ok(())
    }

    pub fn build_connection_uri(&self) -> Result<String, K2DbError> {
        self.validate()?;

        let auth = match (&self.user, &self.password) {
            (Some(user), Some(password)) => {
                format!("{}:{}@", urlencoding::encode(user), urlencoding::encode(password))
            }
            _ => String::new(),
        };

        let single_no_port = self.hosts.len() == 1 && self.hosts[0].port.is_none();

        if single_no_port {
            let host = self.hosts[0].host.trim();
            return Ok(format!(
                "mongodb+srv://{auth}{host}/?retryWrites=true&w=majority"
            ));
        }

        let hosts = self
            .hosts
            .iter()
            .map(|host| format!("{}:{}", host.host.trim(), host.port.unwrap_or(27017)))
            .collect::<Vec<_>>()
            .join(",");

        let mut params = vec!["retryWrites=true".to_owned(), "w=majority".to_owned()];
        if let Some(replica_set) = &self.replica_set {
            params.push(format!("replicaSet={replica_set}"));
        }

        Ok(format!("mongodb://{auth}{hosts}/?{}", params.join("&")))
    }

    pub async fn client_options(&self) -> Result<ClientOptions, K2DbError> {
        let uri = self.build_connection_uri()?;
        let mut options = ClientOptions::parse(uri)
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::ConfigurationError,
                    Some("sys_mdb_init".to_owned()),
                    "Failed to parse MongoDB connection options",
                )
            })?;

        options.connect_timeout = Some(Duration::from_millis(2_000));
        options.server_selection_timeout = Some(Duration::from_millis(2_000));

        let auth_source = self.auth_source.as_deref().or_else(|| match (&self.user, &self.password) {
            (Some(_), Some(_)) => Some("admin"),
            _ => None,
        });

        if let Some(source) = auth_source {
            let credential = options.credential.get_or_insert_with(Credential::default);
            credential.source = Some(source.to_owned());
        }

        Ok(options)
    }

    pub fn masked_uri(&self) -> Result<String, K2DbError> {
        let uri = self.build_connection_uri()?;
        Ok(uri.replace(|c: char| c == '\n' || c == '\r', ""))
    }
    pub fn from_env(prefix: &str) -> Result<Self, K2DbError> {
        let read = |key: &str| env::var(format!("{prefix}{key}")).ok();

        let name = read("NAME").ok_or_else(|| {
            K2DbError::new(
                ServiceError::ConfigurationError,
                "K2DB_NAME and K2DB_HOSTS are required in environment",
                Some("sys_mdb_no_hosts".to_owned()),
            )
        })?;
        let hosts_raw = read("HOSTS").ok_or_else(|| {
            K2DbError::new(
                ServiceError::ConfigurationError,
                "K2DB_NAME and K2DB_HOSTS are required in environment",
                Some("sys_mdb_no_hosts".to_owned()),
            )
        })?;

        let hosts = hosts_raw
            .split(',')
            .filter_map(|part| {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let mut split = trimmed.splitn(2, ':');
                let host = split.next()?.trim().to_owned();
                let port = split.next().and_then(|port| port.parse::<u16>().ok());
                Some(HostConfig { host, port })
            })
            .collect::<Vec<_>>();

        let encryption = match (read("SECURE_FIELD_ENCRYPTION_KEY_ID"), read("SECURE_FIELD_ENCRYPTION_KEY")) {
            (Some(key_id), Some(key)) if !key_id.trim().is_empty() && !key.trim().is_empty() => {
                Some(EncryptionConfig::from_base64(key_id, &key)?)
            }
            _ => None,
        };

        Ok(Self {
            name,
            hosts,
            user: read("USER"),
            password: read("PASSWORD"),
            auth_source: read("AUTH_SOURCE"),
            replica_set: read("REPLICASET"),
            slow_query_ms: read("SLOW_MS").and_then(|value| value.parse::<u64>().ok()),
            ownership_mode: OwnershipMode::default(),
            aggregation_mode: AggregationMode::default(),
            secure_field_prefixes: Vec::new(),
            secure_field_encryption: encryption,
            hooks: QueryHooks::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> DatabaseConfig {
        DatabaseConfig {
            name: "example".to_owned(),
            hosts: vec![HostConfig {
                host: "127.0.0.1".to_owned(),
                port: Some(27017),
            }],
            user: Some("editor".to_owned()),
            password: Some("secret".to_owned()),
            auth_source: None,
            replica_set: None,
            slow_query_ms: Some(250),
            ownership_mode: OwnershipMode::Strict,
            aggregation_mode: AggregationMode::Loose,
            secure_field_prefixes: Vec::new(),
            secure_field_encryption: None,
            hooks: QueryHooks::default(),
        }
    }

    #[tokio::test]
    async fn client_options_default_auth_source_to_admin_when_credentials_exist() {
        let options = base_config().client_options().await.expect("client options");
        let credential = options.credential.expect("credential");
        assert_eq!(credential.source.as_deref(), Some("admin"));
    }
}