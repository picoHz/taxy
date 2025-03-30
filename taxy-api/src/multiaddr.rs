use crate::error::Error;
use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Multiaddr {
    protocols: Vec<Protocol>,
}

impl Multiaddr {
    pub fn is_tls(&self) -> bool {
        self.protocols.iter().any(|p| matches!(p, Protocol::Tls))
    }

    pub fn is_http(&self) -> bool {
        self.protocols
            .iter()
            .any(|p| matches!(p, Protocol::Http(_)))
    }

    pub fn is_udp(&self) -> bool {
        self.protocols.iter().any(|p| matches!(p, Protocol::Udp(_)))
    }

    pub fn is_quic(&self) -> bool {
        self.protocols.iter().any(|p| matches!(p, Protocol::Quic))
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, Error> {
        self.ip_addr()
            .and_then(|ip| self.port().map(|port| SocketAddr::new(ip, port)))
    }

    pub fn ip_addr(&self) -> Result<IpAddr, Error> {
        self.protocols
            .iter()
            .find_map(|p| match p {
                Protocol::Ip(addr) => Some(*addr),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidMultiaddr {
                addr: self.to_string(),
            })
    }

    pub fn port(&self) -> Result<u16, Error> {
        self.protocols
            .iter()
            .find_map(|p| match p {
                Protocol::Tcp(port) => Some(*port),
                Protocol::Udp(port) => Some(*port),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidMultiaddr {
                addr: self.to_string(),
            })
    }

    pub fn host(&self) -> Result<String, Error> {
        self.protocols
            .iter()
            .find_map(|p| match p {
                Protocol::Dns(host) => Some(host.clone()),
                Protocol::Ip(addr) => Some(addr.to_string()),
                _ => None,
            })
            .ok_or_else(|| Error::InvalidMultiaddr {
                addr: self.to_string(),
            })
    }

    pub fn protocol_name(&self) -> &'static str {
        match (self.is_udp(), self.is_http(), self.is_tls(), self.is_quic()) {
            (_, false, _, true) => "QUIC",
            (_, true, _, true) => "HTTP over QUIC",
            (true, _, _, _) => "UDP",
            (false, true, true, _) => "HTTPS",
            (false, true, false, _) => "HTTP",
            (false, false, true, _) => "TCP over TLS",
            (false, false, false, _) => "TCP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Dns(String),
    Ip(IpAddr),
    Tcp(u16),
    Udp(u16),
    Tls,
    Http(String),
    Quic,
}

impl FromStr for Multiaddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut protocols = Vec::new();
        let mut rest = s.trim_start_matches('/');
        while !rest.is_empty() {
            let (protocol, next) = rest.split_once('/').unwrap_or((rest, ""));
            match protocol {
                "dns" => {
                    let (host, next) = next.split_once('/').unwrap_or((next, ""));
                    protocols.push(Protocol::Dns(host.to_string()));
                    rest = next;
                }
                "ip4" | "ip6" => {
                    let (addr, next) = next.split_once('/').unwrap_or((next, ""));
                    let addr = addr
                        .parse::<IpAddr>()
                        .map_err(|_| Error::InvalidMultiaddr {
                            addr: s.to_string(),
                        })?;
                    protocols.push(Protocol::Ip(addr));
                    rest = next;
                }
                "tcp" => {
                    let (port, next) = next.split_once('/').unwrap_or((next, ""));
                    let port = port.parse::<u16>().map_err(|_| Error::InvalidMultiaddr {
                        addr: s.to_string(),
                    })?;
                    protocols.push(Protocol::Tcp(port));
                    rest = next;
                }
                "tls" => {
                    protocols.push(Protocol::Tls);
                    rest = next;
                }
                "quic" => {
                    protocols.push(Protocol::Quic);
                    rest = next;
                }
                "http" => {
                    protocols.push(Protocol::Http(format!("/{next}")));
                    rest = "";
                }
                "https" => {
                    protocols.push(Protocol::Tls);
                    protocols.push(Protocol::Http(format!("/{next}")));
                    rest = "";
                }
                "udp" => {
                    let (port, next) = next.split_once('/').unwrap_or((next, ""));
                    let port = port.parse::<u16>().map_err(|_| Error::InvalidMultiaddr {
                        addr: s.to_string(),
                    })?;
                    protocols.push(Protocol::Udp(port));
                    rest = next;
                }
                _ => rest = next,
            }
        }
        Ok(Self { protocols })
    }
}

impl fmt::Display for Multiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for protocol in &self.protocols {
            match protocol {
                Protocol::Dns(host) => write!(f, "/dns/{}", host)?,
                Protocol::Ip(addr) => {
                    if addr.is_ipv4() {
                        write!(f, "/ip4/{}", addr)?;
                    } else {
                        write!(f, "/ip6/{}", addr)?;
                    }
                }
                Protocol::Tcp(port) => write!(f, "/tcp/{}", port)?,
                Protocol::Udp(port) => write!(f, "/udp/{}", port)?,
                Protocol::Tls => {
                    if !self.is_http() {
                        write!(f, "/tls")?
                    }
                }
                Protocol::Http(path) => {
                    let path = if path == "/" || path.is_empty() {
                        ""
                    } else {
                        path.as_str()
                    };
                    if self.is_tls() {
                        write!(f, "/https{}", path)?;
                    } else {
                        write!(f, "/http{}", path)?;
                    }
                }
                Protocol::Quic => write!(f, "/quic")?,
            }
        }
        Ok(())
    }
}

impl serde::Serialize for Multiaddr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Multiaddr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Multiaddr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_multiaddr() {
        let addr = Multiaddr::from_str("/dns/example.com/tcp/8080").unwrap();
        assert_eq!(addr.to_string(), "/dns/example.com/tcp/8080");
        assert!(!addr.is_http());
        assert!(!addr.is_tls());

        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap();
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/8080");
        assert!(!addr.is_http());
        assert!(!addr.is_tls());

        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080/tls").unwrap();
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/8080/tls");
        assert!(!addr.is_http());
        assert!(addr.is_tls());

        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080/http").unwrap();
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/8080/http");
        assert!(addr.is_http());
        assert!(!addr.is_tls());

        let addr = Multiaddr::from_str("/ip6/::/tcp/8080/https/example.com/index.html").unwrap();
        assert_eq!(
            addr.to_string(),
            "/ip6/::/tcp/8080/https/example.com/index.html"
        );
        assert!(addr.is_http());
        assert!(addr.is_tls());

        let addr = Multiaddr::from_str("/ip4/127.0.0.1/udp/8080").unwrap();
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/udp/8080");
        assert!(!addr.is_http());
        assert!(!addr.is_tls());
        assert!(addr.is_udp());

        let addr = Multiaddr::from_str("/ip4/127.0.0.1/udp/8080/quic/http").unwrap();
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/udp/8080/quic/http");
        assert!(addr.is_http());
        assert!(!addr.is_tls());
        assert!(addr.is_udp());
        assert!(addr.is_quic());
    }
}
