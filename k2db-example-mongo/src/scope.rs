// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Ownership scope applied to scoped database operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Owner(String),
    All,
}

impl Scope {
    /// Construct an owner-bound scope.
    ///
    /// ```
    /// use k2db::Scope;
    ///
    /// let scope = Scope::owner("alice");
    /// assert_eq!(scope, Scope::Owner("alice".to_owned()));
    /// ```
    pub fn owner(owner: impl Into<String>) -> Self {
        Self::Owner(owner.into())
    }

    /// Construct the wildcard scope.
    ///
    /// ```
    /// use k2db::Scope;
    ///
    /// assert_eq!(Scope::all(), Scope::All);
    /// ```
    pub fn all() -> Self {
        Self::All
    }

    pub fn from_legacy(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed == "*" {
            return Some(Self::all());
        }

        Some(Self::owner(trimmed.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;

    #[test]
    fn owner_constructor_wraps_owned_value() {
        assert_eq!(Scope::owner("owner1"), Scope::Owner("owner1".to_owned()));
    }

    #[test]
    fn all_constructor_returns_wildcard_scope() {
        assert_eq!(Scope::all(), Scope::All);
    }

    #[test]
    fn from_legacy_maps_wildcard_and_owner_values() {
        assert_eq!(Scope::from_legacy("*"), Some(Scope::All));
        assert_eq!(Scope::from_legacy(" owner1 "), Some(Scope::Owner("owner1".to_owned())));
    }
}