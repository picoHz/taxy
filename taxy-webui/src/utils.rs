use multiaddr::{Multiaddr, Protocol};

pub fn format_multiaddr(addr: &Multiaddr) -> String {
    let mut interface = String::new();

    let mut kind = "";
    for protocol in addr.iter() {
        match protocol {
            Protocol::Dns(name) | Protocol::Dns4(name) | Protocol::Dns6(name) => {
                interface.push_str(&format!("{name}:"));
            }
            Protocol::Ip4(addr) => {
                interface.push_str(&format!("{addr}:"));
            }
            Protocol::Ip6(addr) => {
                interface.push_str(&format!("{addr}:"));
            }
            Protocol::Tcp(port) => {
                interface.push_str(&format!("{port}"));
                kind = "TCP";
            }
            Protocol::Tls => {
                kind = "TLS";
            }
            Protocol::Http => {
                kind = "HTTP";
            }
            Protocol::Https => {
                kind = "HTTPS";
            }
            _ => (),
        }
    }

    format!("{kind} [{interface}]")
}
