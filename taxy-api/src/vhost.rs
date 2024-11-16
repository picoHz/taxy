use crate::{error::Error, subject_name::SubjectName};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum VirtualHost {
    SubjectName(SubjectName),
    Regex(Regex),
}

impl VirtualHost {
    pub fn test(&self, name: &str) -> bool {
        match self {
            VirtualHost::SubjectName(n) => n.test(name),
            VirtualHost::Regex(r) => r.is_match(name),
        }
    }
}

impl PartialEq for VirtualHost {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VirtualHost::SubjectName(a), VirtualHost::SubjectName(b)) => a == b,
            (VirtualHost::Regex(a), VirtualHost::Regex(b)) => a.as_str() == b.as_str(),
            _ => false,
        }
    }
}

impl Eq for VirtualHost {}

impl fmt::Display for VirtualHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualHost::SubjectName(name) => write!(f, "{}", name),
            VirtualHost::Regex(r) => write!(f, "{}", r),
        }
    }
}

impl FromStr for VirtualHost {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = SubjectName::from_str(s) {
            Ok(VirtualHost::SubjectName(n))
        } else if let Ok(r) = Regex::new(s) {
            Ok(VirtualHost::Regex(r))
        } else {
            Err(Error::InvalidVirtualHost { host: s.into() })
        }
    }
}

impl Serialize for VirtualHost {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VirtualHost {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_virtual_host() {
        use super::VirtualHost;
        use std::str::FromStr;

        let host = VirtualHost::from_str("localhost").unwrap();
        assert_eq!(host.to_string(), "localhost");
        assert!(host.test("localhost"));
        assert!(!host.test("example.com"));

        let host = VirtualHost::from_str("^.*\\.example\\.com$").unwrap();
        assert_eq!(host.to_string(), "^.*\\.example\\.com$");
        assert!(!host.test("localhost"));
        assert!(host.test("www.example.com"));

        let host = VirtualHost::from_str("^([a-z]+\\.)+my\\.vow$").unwrap();
        assert_eq!(host.to_string(), "^([a-z]+\\.)+my\\.vow$");
        assert!(!host.test("localhost"));
        assert!(host.test("sphinx.of.black.quartz.judge.my.vow"));
    }
}
