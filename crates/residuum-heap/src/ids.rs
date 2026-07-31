//! Durable and session identity types (`HEAP_SPEC` §30.6).

use crate::error::HeapError;
use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! uuid_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name([u8; 16]);

        impl $name {
            /// Validate raw RFC 4122 network-order bytes (non-zero, version 4, RFC variant).
            pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, HeapError> {
                if bytes == [0u8; 16] {
                    return Err(HeapError::InvalidArgument(concat!(stringify!($name), " is zero")));
                }
                let version = (bytes[6] & 0xf0) >> 4;
                if version != 4 {
                    return Err(HeapError::InvalidArgument(concat!(stringify!($name), " must be UUIDv4")));
                }
                let variant = (bytes[8] & 0xc0) >> 6;
                if variant != 0b10 {
                    return Err(HeapError::InvalidArgument(concat!(stringify!($name), " must be RFC variant")));
                }
                Ok(Self(bytes))
            }

            /// Construct without validating version bits (for recovery of integrity-valid bytes).
            pub fn from_bytes_unchecked_nonzero(bytes: [u8; 16]) -> Result<Self, HeapError> {
                if bytes == [0u8; 16] {
                    return Err(HeapError::InvalidArgument(concat!(stringify!($name), " is zero")));
                }
                Ok(Self(bytes))
            }

            /// Cryptographically random UUIDv4.
            pub fn new_random() -> Result<Self, HeapError> {
                let mut bytes = [0u8; 16];
                getrandom::fill(&mut bytes).map_err(|_| HeapError::InvalidArgument("getrandom failed"))?;
                bytes[6] = (bytes[6] & 0x0f) | 0x40;
                bytes[8] = (bytes[8] & 0x3f) | 0x80;
                Ok(Self(bytes))
            }

            /// Raw bytes.
            pub fn as_bytes(&self) -> &[u8; 16] {
                &self.0
            }

            /// Copy of raw bytes.
            pub fn to_bytes(self) -> [u8; 16] {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let b = &self.0;
                write!(
                    f,
                    "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                    b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
                )
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl std::str::FromStr for $name {
            type Err = HeapError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                parse_canonical_uuid(s).and_then(Self::from_bytes)
            }
        }
    };
}

uuid_id!(/// Durable heap identity.
    HeapId);
uuid_id!(/// Durable collection identity.
    CollectionId);
uuid_id!(/// Durable stream identity.
    StreamId);
uuid_id!(/// Serving deployment identity.
    DeploymentId);
uuid_id!(/// Certificate identity.
    CertificateId);
uuid_id!(/// Capability-instance identity.
    CapabilityId);

fn parse_canonical_uuid(s: &str) -> Result<[u8; 16], HeapError> {
    let b = s.as_bytes();
    if b.len() != 36 || b[8] != b'-' || b[13] != b'-' || b[18] != b'-' || b[23] != b'-' {
        return Err(HeapError::InvalidArgument(
            "UUID must be canonical lowercase hyphenated",
        ));
    }
    let mut out = [0u8; 16];
    let positions = [0, 2, 4, 6, 9, 11, 14, 16, 19, 21, 24, 26, 28, 30, 32, 34];
    for (i, &p) in positions.iter().enumerate() {
        out[i] = hex_byte(b[p], b[p + 1])?;
    }
    Ok(out)
}

fn hex_byte(a: u8, b: u8) -> Result<u8, HeapError> {
    Ok((hex_nibble(a)? << 4) | hex_nibble(b)?)
}

fn hex_nibble(c: u8) -> Result<u8, HeapError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(HeapError::InvalidArgument("UUID must be lowercase hex")),
    }
}

macro_rules! nonzero_u64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(u64);

        impl $name {
            /// Construct; zero is rejected.
            pub fn new(v: u64) -> Result<Self, HeapError> {
                if v == 0 {
                    return Err(HeapError::InvalidArgument(concat!(stringify!($name), " must be non-zero")));
                }
                Ok(Self(v))
            }

            /// Raw value.
            pub fn get(self) -> u64 {
                self.0
            }
        }
    };
}

nonzero_u64!(/// Authority epoch.
    AuthorityEpoch);
nonzero_u64!(/// Authority generation.
    AuthorityGeneration);
nonzero_u64!(/// Security revision.
    SecurityRevision);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_and_bad_version() {
        assert!(HeapId::from_bytes([0u8; 16]).is_err());
        let mut b = [1u8; 16];
        b[6] = 0x10; // v1
        b[8] = 0x80;
        assert!(HeapId::from_bytes(b).is_err());
    }

    #[test]
    fn roundtrip_display() {
        let id = HeapId::new_random().unwrap();
        let s = id.to_string();
        let parsed: HeapId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }
}
