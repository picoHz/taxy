use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};
use tokio_rustls::rustls::ServerName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectName {
    DnsName(String),
    WildcardDnsName(String),
    IPAddress(IpAddr),
}

impl SubjectName {
    pub fn test(&self, name: &str) -> bool {
        match self {
            Self::DnsName(n) => n == name,
            Self::WildcardDnsName(n) => {
                n == name
                    .trim_start_matches(|c| c != '.')
                    .trim_start_matches('.')
            }
            Self::IPAddress(addr) => match addr {
                IpAddr::V4(addr) => name == addr.to_string(),
                IpAddr::V6(addr) => name == addr.to_string(),
            },
        }
    }
}

impl Serialize for SubjectName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SubjectName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl ToString for SubjectName {
    fn to_string(&self) -> String {
        match self {
            Self::DnsName(name) => name.to_owned(),
            Self::WildcardDnsName(name) => format!("*.{}", name),
            Self::IPAddress(addr) => addr.to_string(),
        }
    }
}

impl FromStr for SubjectName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let wildcard = s.starts_with("*.");
        let name = ServerName::try_from(s.trim_start_matches("*."))
            .map_err(|_| Error::InvalidSubjectName { name: s.to_owned() })?;
        match name {
            ServerName::DnsName(name) => {
                if wildcard {
                    Ok(Self::WildcardDnsName(name.as_ref().to_string()))
                } else {
                    Ok(Self::DnsName(name.as_ref().to_string()))
                }
            }
            ServerName::IpAddress(addr) => Ok(Self::IPAddress(addr)),
            _ => Err(Error::InvalidSubjectName { name: s.to_owned() }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subject_name() {
        assert_eq!(
            SubjectName::from_str("*.example.com").unwrap(),
            SubjectName::WildcardDnsName("example.com".to_owned())
        );
        assert_eq!(
            SubjectName::from_str("example.com").unwrap(),
            SubjectName::DnsName("example.com".to_owned())
        );
        assert_eq!(
            SubjectName::from_str("127.0.0.1").unwrap(),
            SubjectName::IPAddress(IpAddr::V4([127, 0, 0, 1].into()))
        )
    }

    #[test]
    fn test_subject_name_test() {
        assert!(SubjectName::from_str("*.example.com")
            .unwrap()
            .test("app.example.com"));
        assert!(SubjectName::from_str("example.com")
            .unwrap()
            .test("example.com"));
        assert!(SubjectName::from_str("127.0.0.1")
            .unwrap()
            .test("127.0.0.1"));
    }
}
