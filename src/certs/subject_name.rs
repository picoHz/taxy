use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};
use tokio_rustls::rustls::ServerName;
use x509_parser::prelude::GeneralName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectName {
    DnsName(String),
    WildcardDnsName(String),
    IPAddress(IpAddr),
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

impl SubjectName {
    pub fn test(&self, name: &GeneralName) -> bool {
        match (self, name) {
            (Self::DnsName(s), GeneralName::DNSName(n)) => {
                if n.starts_with("*.") {
                    s.trim_end_matches(n.trim_start_matches('*'))
                        .chars()
                        .all(|c| c == '-' || c.is_ascii_alphanumeric())
                } else {
                    s == n
                }
            }
            (Self::WildcardDnsName(s), GeneralName::DNSName(n)) => {
                n.starts_with("*.") && n.trim_start_matches("*.") == s
            }
            (Self::IPAddress(s), GeneralName::IPAddress(n)) => match **n {
                [a, b, c, d] => {
                    IpAddr::from([a, b, c, d]) == *s
                }
                [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] => {
                    IpAddr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]) == *s
                }
                _ => false,
            },
            _ => false,
        }
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
        let name = SubjectName::from_str("*.example.com").unwrap();
        assert!(name.test(&GeneralName::DNSName("*.example.com")));
        assert!(!name.test(&GeneralName::DNSName("example.com")));
        assert!(!name.test(&GeneralName::DNSName("www.example.org")));
        assert!(!name.test(&GeneralName::IPAddress(&[127, 0, 0, 1])));

        let name = SubjectName::from_str("app.app.example.com").unwrap();
        assert!(name.test(&GeneralName::DNSName("*.app.example.com")));
        assert!(!name.test(&GeneralName::DNSName("*.example.com")));
        assert!(!name.test(&GeneralName::DNSName("example.com")));
        assert!(!name.test(&GeneralName::DNSName("www.example.org")));
        assert!(!name.test(&GeneralName::IPAddress(&[127, 0, 0, 1])));
    }
}
