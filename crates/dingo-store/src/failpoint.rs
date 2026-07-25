//! Named failpoints for crash-consistency testing (DEF-022).
//!
//! Failpoints are **always compiled** but are no-ops unless armed. Production
//! paths pay only a mutex lock + map lookup when any failpoint has ever been
//! armed in the process; when the registry is empty, [`hit`] returns immediately
//! after a cheap empty check.
//!
//! ## Actions
//!
//! - [`Action::Panic`] — process-local crash simulation (catchable with
//!   `catch_unwind` in tests; drop the store handle and reopen).
//! - [`Action::Error`] — inject a [`StoreError::Failpoint`] without panicking.
//! - [`Action::Return`] — same as Error (alias for matrix wording).
//!
//! Arm with [`arm`] / [`arm_once`]. Clear with [`disarm`] / [`clear`].
//!
//! Names are stable identifiers listed in `crash_matrix.v1.json`.

use crate::error::StoreError;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// What happens when a named failpoint is hit while armed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Unwind the current thread (crash simulation).
    Panic,
    /// Return [`StoreError::Failpoint`] to the caller.
    Error,
    /// Same as [`Action::Error`] (matrix synonym for injected I/O failure).
    Return,
}

#[derive(Debug, Clone)]
struct Armed {
    action: Action,
    /// Remaining hits; `None` means fire every time until disarmed.
    remaining: Option<u32>,
}

static REGISTRY: OnceLock<Mutex<HashMap<&'static str, Armed>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<&'static str, Armed>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Arm `name` so the next hits perform `action` until disarmed.
pub fn arm(name: &'static str, action: Action) {
    let mut g = registry().lock().expect("failpoint registry");
    g.insert(
        name,
        Armed {
            action,
            remaining: None,
        },
    );
}

/// Arm `name` to fire `action` at most `times` times, then auto-disarm.
pub fn arm_once(name: &'static str, action: Action) {
    arm_n(name, action, 1);
}

/// Arm `name` to fire `action` at most `n` times.
pub fn arm_n(name: &'static str, action: Action, n: u32) {
    let mut g = registry().lock().expect("failpoint registry");
    g.insert(
        name,
        Armed {
            action,
            remaining: Some(n),
        },
    );
}

/// Remove a single failpoint.
pub fn disarm(name: &str) {
    let mut g = registry().lock().expect("failpoint registry");
    g.remove(name);
}

/// Remove all failpoints (call from test teardown).
pub fn clear() {
    let mut g = registry().lock().expect("failpoint registry");
    g.clear();
}

/// Whether any failpoint is currently armed.
pub fn any_armed() -> bool {
    let g = registry().lock().expect("failpoint registry");
    !g.is_empty()
}

/// Hit a named failpoint. No-op when not armed.
///
/// # Panics
///
/// Panics when the armed action is [`Action::Panic`] (intentional).
pub fn hit(name: &'static str) -> Result<(), StoreError> {
    let action = {
        let mut g = registry().lock().expect("failpoint registry");
        let Some(entry) = g.get_mut(name) else {
            return Ok(());
        };
        let action = entry.action;
        if let Some(ref mut rem) = entry.remaining {
            if *rem == 0 {
                g.remove(name);
                return Ok(());
            }
            *rem = rem.saturating_sub(1);
            if *rem == 0 {
                g.remove(name);
            }
        }
        action
    };
    match action {
        Action::Panic => panic!("dingo failpoint: {name}"),
        Action::Error | Action::Return => Err(StoreError::Failpoint(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unarmed_is_noop() {
        clear();
        assert!(hit("test.never_armed").is_ok());
    }

    #[test]
    fn error_action_returns_failpoint() {
        clear();
        arm_once("test.error", Action::Error);
        let err = hit("test.error").unwrap_err();
        assert!(matches!(err, StoreError::Failpoint("test.error")));
        // second hit auto-disarmed
        assert!(hit("test.error").is_ok());
        clear();
    }

    #[test]
    fn panic_action_unwinds() {
        clear();
        arm_once("test.panic", Action::Panic);
        let r = std::panic::catch_unwind(|| {
            let _ = hit("test.panic");
        });
        assert!(r.is_err());
        clear();
    }
}
