use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use utoipa::ToSchema;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema)]
pub struct ShortId([u8; 8]);

impl ShortId {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<[u8; 7]> for ShortId {
    fn from(id: [u8; 7]) -> Self {
        let mut bytes = [0; 8];
        bytes[1..].copy_from_slice(&id);
        Self(bytes)
    }
}

impl fmt::Debug for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShortId({})", self)
    }
}

impl fmt::Display for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0[0] == 0 {
            write!(f, "{}", hex::encode(&self.0[1..]))
        } else {
            let len = self.0.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
            write!(f, "{}", String::from_utf8_lossy(&self.0[..len]))
        }
    }
}

impl FromStr for ShortId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = [0; 8];
        if hex::decode_to_slice(s, &mut id[1..]).is_ok() {
            return Ok(ShortId(id));
        }
        let str = s.to_ascii_lowercase();
        if !str.is_ascii() || str.len() > 8 {
            return Err(Error::InvalidShortId { id: s.to_string() });
        }
        let bytes = str.as_bytes();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_hex() {
        let id = ShortId::from_str("f9cf7e3faa1aca").unwrap();
        assert_eq!(id.to_string(), "f9cf7e3faa1aca");
    }

    #[test]
    fn parse_hyphen_id() {
        let id = ShortId::from_str("djs-vjd").unwrap();
        assert_eq!(id.to_string(), "djs-vjd");
    }
}
