// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce, Tag};
use base64::Engine;
use mongodb::bson::{Bson, Document};

use crate::config::EncryptionConfig;
use crate::error::{K2DbError, ServiceError};

pub fn has_secure_encryption(encryption: Option<&EncryptionConfig>) -> bool {
    encryption.is_some()
}

pub fn encrypt_secure_fields_deep(
    value: Bson,
    secure_prefixes: &[String],
    encryption: Option<&EncryptionConfig>,
    aad_prefix: &str,
) -> Result<Bson, K2DbError> {
    let Some(encryption) = encryption else {
        return Ok(value);
    };
    if secure_prefixes.is_empty() {
        return Ok(value);
    }

    match value {
        Bson::Document(document) => {
            let mut out = Document::new();
            for (key, value) in document {
                if is_secure_field_key(&key, secure_prefixes) {
                    let aad = format!("{aad_prefix}|{key}");
                    out.insert(key, encrypt_secure_value(value, encryption, &aad)?);
                } else {
                    out.insert(
                        key,
                        encrypt_secure_fields_deep(value, secure_prefixes, Some(encryption), aad_prefix)?,
                    );
                }
            }
            Ok(Bson::Document(out))
        }
        Bson::Array(values) => Ok(Bson::Array(
            values
                .into_iter()
                .map(|value| encrypt_secure_fields_deep(value, secure_prefixes, Some(encryption), aad_prefix))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        other => Ok(other),
    }
}

pub fn decrypt_secure_fields_deep(
    value: Bson,
    secure_prefixes: &[String],
    encryption: Option<&EncryptionConfig>,
    aad_prefixes: &[String],
) -> Result<Bson, K2DbError> {
    let Some(encryption) = encryption else {
        return Ok(value);
    };
    if secure_prefixes.is_empty() {
        return Ok(value);
    }

    match value {
        Bson::Document(document) => {
            let mut out = Document::new();
            for (key, value) in document {
                if is_secure_field_key(&key, secure_prefixes) {
                    let aads = aad_prefixes
                        .iter()
                        .map(|prefix| format!("{prefix}|{key}"))
                        .collect::<Vec<_>>();
                    out.insert(key, decrypt_secure_value(value, encryption, &aads)?);
                } else {
                    out.insert(
                        key,
                        decrypt_secure_fields_deep(value, secure_prefixes, Some(encryption), aad_prefixes)?,
                    );
                }
            }
            Ok(Bson::Document(out))
        }
        Bson::Array(values) => Ok(Bson::Array(
            values
                .into_iter()
                .map(|value| decrypt_secure_fields_deep(value, secure_prefixes, Some(encryption), aad_prefixes))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        other => Ok(other),
    }
}

fn is_secure_field_key(key: &str, secure_prefixes: &[String]) -> bool {
    secure_prefixes.iter().any(|prefix| key.starts_with(prefix))
}

fn encrypt_secure_value(
    value: Bson,
    encryption: &EncryptionConfig,
    aad: &str,
) -> Result<Bson, K2DbError> {
    let plaintext = serde_json::to_vec(&value).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::SystemError,
            Some("sys_mdb_sav".to_owned()),
            "Error saving object to database",
        )
    })?;

    let iv = rand::random::<[u8; 12]>();
    let cipher = Aes256Gcm::new_from_slice(&encryption.key).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::ConfigurationError,
            Some("sys_mdb_secure_key_invalid".to_owned()),
            "secureFieldEncryptionKey must decode to 32 bytes (AES-256)",
        )
    })?;

    let mut buffer = plaintext;
    let tag = cipher
        .encrypt_in_place_detached(Nonce::from_slice(&iv), aad.as_bytes(), &mut buffer)
        .map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Error saving object to database",
                Some("sys_mdb_sav".to_owned()),
            )
        })?;

    let payload = format!(
        "{}.{}.{}",
        base64::engine::general_purpose::STANDARD.encode(iv),
        base64::engine::general_purpose::STANDARD.encode(tag),
        base64::engine::general_purpose::STANDARD.encode(buffer),
    );
    Ok(Bson::String(format!("{}:{payload}", encryption.key_id)))
}

fn decrypt_secure_value(
    value: Bson,
    encryption: &EncryptionConfig,
    aad_candidates: &[String],
) -> Result<Bson, K2DbError> {
    let Bson::String(value) = value else {
        return Ok(value);
    };

    let Some((kid, payload)) = value.split_once(':') else {
        return Ok(Bson::String(value));
    };
    if kid != encryption.key_id {
        return Ok(Bson::String(value));
    }

    let parts = payload.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Ok(Bson::String(value));
    }

    let iv = base64::engine::general_purpose::STANDARD.decode(parts[0]).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::SystemError,
            Some("sys_mdb_secure_decrypt_failed".to_owned()),
            "Failed to decrypt secure field",
        )
    })?;
    let tag = base64::engine::general_purpose::STANDARD.decode(parts[1]).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::SystemError,
            Some("sys_mdb_secure_decrypt_failed".to_owned()),
            "Failed to decrypt secure field",
        )
    })?;
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(parts[2]).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::SystemError,
            Some("sys_mdb_secure_decrypt_failed".to_owned()),
            "Failed to decrypt secure field",
        )
    })?;

    let cipher = Aes256Gcm::new_from_slice(&encryption.key).map_err(|error| {
        K2DbError::wrap(
            error,
            ServiceError::ConfigurationError,
            Some("sys_mdb_secure_key_invalid".to_owned()),
            "secureFieldEncryptionKey must decode to 32 bytes (AES-256)",
        )
    })?;

    for aad in aad_candidates {
        let mut buffer = ciphertext.clone();
        let result = cipher.decrypt_in_place_detached(
            Nonce::from_slice(&iv),
            aad.as_bytes(),
            &mut buffer,
            Tag::from_slice(&tag),
        );
        if result.is_ok() {
            return serde_json::from_slice(&buffer).map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_secure_decrypt_failed".to_owned()),
                    "Failed to decrypt secure field",
                )
            });
        }
    }

    Err(K2DbError::new(
        ServiceError::SystemError,
        "Failed to decrypt secure field",
        Some("sys_mdb_secure_decrypt_failed".to_owned()),
    ))
}

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;

    use crate::config::EncryptionConfig;

    use super::{decrypt_secure_fields_deep, encrypt_secure_fields_deep};

    fn encryption() -> EncryptionConfig {
        EncryptionConfig {
            key_id: "kid-1".to_owned(),
            key: [7_u8; 32],
        }
    }

    #[test]
    fn round_trip_encrypts_and_decrypts_secure_fields() {
        let encrypted = encrypt_secure_fields_deep(
            mongodb::bson::Bson::Document(doc! { "#secret": "x", "name": "ok" }),
            &["#".to_owned()],
            Some(&encryption()),
            "k2db|items|uuid1",
        )
        .unwrap();

        let decrypted = decrypt_secure_fields_deep(
            encrypted,
            &["#".to_owned()],
            Some(&encryption()),
            &["k2db|items|uuid1".to_owned()],
        )
        .unwrap();

        assert_eq!(decrypted, mongodb::bson::Bson::Document(doc! { "#secret": "x", "name": "ok" }));
    }
}