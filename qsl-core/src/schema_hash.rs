//! Build-time schema identities over a versioned, architecture-independent descriptor.
//! Names belong only to the strict identity. Storage, inlining, RT pool settings,
//! initializers and optional-table implementation details do not belong to either.
use crate::QslError;

/// Bump when the canonical descriptor or any represented wire codec changes.
pub const SCHEMA_HASH_VERSION: u16 = 1;

/// xxHash is non-cryptographic, including its wider 128-bit variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SchemaHashAlgorithm {
    /// `!config USE_HASH = xxHash;` (XXH3-64, seed zero).
    #[default]
    Xxh3_64,
    /// `!config USE_HASH = xxHash128;` (XXH3-128, seed zero).
    Xxh3_128,
}
impl SchemaHashAlgorithm {
    pub const fn bits(self) -> u32 {
        match self {
            Self::Xxh3_64 => 64,
            Self::Xxh3_128 => 128,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xxh3_64 => "xxHash",
            Self::Xxh3_128 => "xxHash128",
        }
    }
    pub const fn rust_variant(self) -> &'static str {
        match self {
            Self::Xxh3_64 => "Xxh3_64",
            Self::Xxh3_128 => "Xxh3_128",
        }
    }
    pub fn digest(self, bytes: &[u8]) -> u128 {
        match self {
            Self::Xxh3_64 => u128::from(xxhash_rust::xxh3::xxh3_64(bytes)),
            Self::Xxh3_128 => xxhash_rust::xxh3::xxh3_128(bytes),
        }
    }
}
impl core::str::FromStr for SchemaHashAlgorithm {
    type Err = QslError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "xxHash" | "XXH3_64" => Ok(Self::Xxh3_64),
            "xxHash128" | "XXH3_128" => Ok(Self::Xxh3_128),
            _ => Err(QslError::InvalidSchema),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaHashKind {
    Strict,
    Light,
}

/// A digest must be compared together with algorithm, version and completeness.
/// For XXH3-64 the upper 64 bits of each value are zero. Neither digest is added
/// to a serialized record; sending or negotiating it remains explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemaHashes {
    pub version: u16,
    pub algorithm: SchemaHashAlgorithm,
    pub strict: u128,
    pub light: u128,
    /// False for an opaque QslCustom: only its Rust path and declared size are
    /// known. Its external implementation must be agreed separately.
    pub complete: bool,
}
/// A fixed-size hexadecimal digest view; formatting never allocates.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HexDigest {
    bytes: [u8; 32],
    len: u8,
}
impl HexDigest {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len as usize]).expect("hex ASCII")
    }
}
impl core::ops::Deref for HexDigest {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl core::fmt::Display for HexDigest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl core::fmt::Debug for HexDigest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_str().fmt(f)
    }
}
impl SchemaHashes {
    pub fn strict_hex(self) -> HexDigest {
        self.hex(self.strict)
    }
    pub fn light_hex(self) -> HexDigest {
        self.hex(self.light)
    }
    fn hex(self, mut value: u128) -> HexDigest {
        let len = (self.algorithm.bits() / 4) as usize;
        let mut bytes = [b'0'; 32];
        let digits = b"0123456789abcdef";
        let mut i = len;
        while i != 0 {
            i -= 1;
            bytes[i] = digits[(value & 15) as usize];
            value >>= 4;
        }
        HexDigest {
            bytes,
            len: len as u8,
        }
    }
}

/// Generated implementations contain literal constants, never runtime hashing.
pub trait SchemaIdentity {
    const SCHEMA_HASHES: SchemaHashes;
}

impl AsRef<str> for HexDigest {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl PartialEq<&str> for HexDigest {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<str> for HexDigest {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<HexDigest> for &str {
    fn eq(&self, other: &HexDigest) -> bool {
        *self == other.as_str()
    }
}
