use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortId([u8; 8]);

impl ShortId {
    pub fn new() -> Self {
        Default::default()
    }
}

impl fmt::Debug for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShortId({})", self)
    }
}

impl fmt::Display for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        write!(f, "{}", String::from_utf8_lossy(&self.0[..len]))
    }
}

impl FromStr for ShortId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.to_ascii_lowercase();
        if !str.is_ascii() || str.len() > 8 {
            return Err(Error::InvalidShortId { id: s.to_string() });
        }
        let bytes = str.as_bytes();
        let mut id = [0; 8];
        let len = bytes.len().min(id.len());
        id[..len].copy_from_slice(&bytes[..len]);
        Ok(ShortId(id))
    }
}

impl Serialize for ShortId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ShortId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ShortId::from_str(&s).map_err(serde::de::Error::custom)
    }
}
