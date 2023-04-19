use crate::config::AppConfig;
use crate::{config, error::Error};
use std::fmt;
use std::sync::Arc;
use std::{net::IpAddr, str::FromStr};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::{rustls::ServerName, TlsAcceptor};
use tracing::{debug, error};
use x509_parser::prelude::GeneralName;

use super::certs::{load_single_file, search_cert_from_name};

pub struct TlsTermination {
    pub server_names: Vec<SubjectName>,
    pub acceptor: Option<TlsAcceptor>,
}

impl fmt::Debug for TlsTermination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsTermination")
            .field("server_names", &self.server_names)
            .finish()
    }
}

impl TlsTermination {
    pub fn new(config: &config::tls::TlsTermination) -> Result<Self, Error> {
        let mut server_names = Vec::new();
        for name in &config.server_names {
            let name = SubjectName::from_str(name)?;
            server_names.push(name);
        }
        Ok(Self {
            server_names,
            acceptor: None,
        })
    }

    pub async fn setup(&mut self, config: &AppConfig) -> Result<(), Error> {
        let server_names = self.server_names.clone();
        let search_paths = config.certs.search_paths.clone();
        let (certs, key) = tokio::task::spawn_blocking(move || {
            for path in search_paths {
                debug!(?path, server_names = ?server_names, "searching certificates");
                if let Some(certs) = search_cert_from_name(&path, &server_names) {
                    match load_single_file(&certs.parent().unwrap()) {
                        Ok(result) => return Ok(result),
                        Err(err) => {
                            error!(?err);
                        }
                    }
                }
            }
            Err(Error::ValidTlsCertificatesNotFound)
        })
        .await
        .unwrap()?;

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key);

        let server_config = match server_config {
            Ok(config) => config,
            Err(err) => {
                error!(?err, server_names = ?self.server_names);
                return Err(Error::TlsServerConfigrationFailed);
            }
        };

        let server_config = Arc::new(server_config);
        self.acceptor = Some(TlsAcceptor::from(server_config));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectName {
    DnsName(String),
    WildcardDnsName(String),
    IPAddress(IpAddr),
}

impl SubjectName {
    pub fn test(&self, name: &GeneralName) -> bool {
        match (self, name) {
            (Self::DnsName(s), GeneralName::DNSName(n)) => {
                if n.starts_with("*.") {
                    s.trim_end_matches(n.trim_start_matches("*"))
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
                    return IpAddr::from([a, b, c, d]) == *s;
                }
                [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] => {
                    return IpAddr::from([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]) == *s;
                }
                _ => false,
            },
            _ => false,
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
