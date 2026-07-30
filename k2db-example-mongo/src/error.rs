// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceError {
    SystemError,
    BadRequest,
    NotFound,
    ConfigurationError,
    AlreadyExists,
    ValidationError,
    BadGateway,
    ServiceUnavailable,
}

#[derive(Debug, Serialize)]
pub struct K2DbError {
    #[serde(rename = "error")]
    pub service_error: ServiceError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "error_description")]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<Value>,
    #[serde(skip)]
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl K2DbError {
    pub fn new(service_error: ServiceError, message: impl Into<String>, key: impl Into<Option<String>>) -> Self {
        Self {
            service_error,
            key: key.into(),
            message: message.into(),
            context: None,
            sensitive: None,
            source: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_sensitive(mut self, sensitive: Value) -> Self {
        self.sensitive = Some(sensitive);
        self
    }

    pub fn wrap<E>(
        error: E,
        service_error: ServiceError,
        key: impl Into<Option<String>>,
        message: impl Into<String>,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            service_error,
            key: key.into(),
            message: message.into(),
            context: None,
            sensitive: None,
            source: Some(Box::new(error)),
        }
    }
}

impl Display for K2DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for K2DbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|error| error as &(dyn StdError + 'static))
    }
}