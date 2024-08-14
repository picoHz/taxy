use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, net::IpAddr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectName {
    DnsName(String),
    WildcardDnsName(String),
    IPAddress(IpAddr),
}

impl SubjectName {
    pub fn test(&self, name: &str) -> bool {
        match self {
            Self::DnsName(n) => n.eq_ignore_ascii_case(name),
            Self::WildcardDnsName(n) => n.eq_ignore_ascii_case(
                name.trim_start_matches(|c| c != '.')
                    .trim_start_matches('.'),
            ),
            Self::IPAddress(addr) => match addr {
                IpAddr::V4(addr) => name.eq_ignore_ascii_case(&addr.to_string()),
                IpAddr::V6(addr) => name.eq_ignore_ascii_case(&addr.to_string()),
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

impl Display for SubjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DnsName(name) => write!(f, "{}", name),
            Self::WildcardDnsName(name) => write!(f, "*.{}", name),
            Self::IPAddress(addr) => write!(f, "{}", addr),
        }
    }
}

impl FromStr for SubjectName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(Error::InvalidSubjectName {
                name: s.to_string(),
            });
        }
        let wildcard = s.starts_with("*.");
        let name = s.trim_start_matches("*.");
        let ipaddr: Result<IpAddr, _> = name.parse();
        match ipaddr {
            Ok(addr) => Ok(Self::IPAddress(addr)),
            _ => {
                if wildcard {
                    Ok(Self::WildcardDnsName(name.to_string()))
                } else {
                    Ok(Self::DnsName(name.to_string()))
                }
            }
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
