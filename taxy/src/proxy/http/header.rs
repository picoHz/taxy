use hyper::{
    header::{FORWARDED, VIA},
    http::header::Entry,
    http::HeaderValue,
    HeaderMap,
};
use std::{iter, net::IpAddr};

#[derive(Default, Debug)]
pub struct HeaderRewriter {
    trust_upstream_headers: bool,
    use_std_forwarded: bool,
    set_via: Option<HeaderValue>,
}

impl HeaderRewriter {
    pub fn builder() -> Builder {
        Default::default()
    }

    fn remove_untrusted_headers(&self, headers: &mut HeaderMap) {
        let header_keys = &[
            FORWARDED.as_str(),
            "x-forwarded-for",
            "x-forwarded-host",
            "x-real-ip",
        ];
        for key in header_keys {
            if let Entry::Occupied(entry) = headers.entry(*key) {
                entry.remove_entry_mult();
            }
        }
    }

    fn parse_x_forwarded_for(&self, headers: &mut HeaderMap) -> Vec<IpAddr> {
        if let Entry::Occupied(entry) = headers.entry("x-forwarded-for") {
            return entry
                .remove_entry_mult()
                .1
                .flat_map(|v| {
                    v.to_str()
                        .ok()
                        .unwrap_or_default()
                        .split(',')
                        .filter_map(|ip| ip.trim().parse().ok())
                        .collect::<Vec<IpAddr>>()
                })
                .collect();
        }
        Vec::new()
    }

    fn parse_forwarded(&self, headers: &mut HeaderMap) -> Vec<String> {
        if let Entry::Occupied(entry) = headers.entry(FORWARDED) {
            return entry
                .remove_entry_mult()
                .1
                .flat_map(|v| {
                    v.to_str()
                        .ok()
                        .unwrap_or_default()
                        .split(',')
                        .map(|item| item.trim().to_string())
                        .collect::<Vec<String>>()
                })
                .collect();
        }
        Vec::new()
    }

    pub fn pre_process(&self, headers: &mut HeaderMap, remote_addr: IpAddr) {
        let mut x_forwarded_for = Vec::new();
        let mut forwarded = Vec::new();

        if self.trust_upstream_headers {
            x_forwarded_for = self.parse_x_forwarded_for(headers);
            forwarded = self.parse_forwarded(headers);
        } else {
            self.remove_untrusted_headers(headers);
        }

        if self.use_std_forwarded || !forwarded.is_empty() {
            if forwarded.is_empty() {
                forwarded = x_forwarded_for
                    .into_iter()
                    .map(forwarded_directive)
                    .collect();
            }
            if let Ok(forwarded_value) = HeaderValue::from_str(
                &forwarded
                    .into_iter()
                    .chain(iter::once(forwarded_directive(remote_addr)))
                    .collect::<Vec<_>>()
                    .join(", "),
            ) {
                headers.insert(FORWARDED, forwarded_value);
            }
        } else if let Ok(x_forwarded_value) = HeaderValue::from_str(
            &x_forwarded_for
                .iter()
                .chain(iter::once(&remote_addr))
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ) {
            headers.insert("x-forwarded-for", x_forwarded_value);
        }
    }

    pub fn post_process(&self, headers: &mut HeaderMap) {
        if let Some(via) = &self.set_via {
            headers.insert(VIA, via.clone());
        }
    }
}

#[derive(Default)]
pub struct Builder {
    inner: HeaderRewriter,
}

impl Builder {
    pub fn trust_upstream_headers(mut self, trust: bool) -> Self {
        self.inner.trust_upstream_headers = trust;
        self
    }

    pub fn use_std_forwarded(mut self, use_std: bool) -> Self {
        self.inner.use_std_forwarded = use_std;
        self
    }

    pub fn set_via(mut self, via: HeaderValue) -> Self {
        self.inner.set_via = Some(via);
        self
    }

    pub fn build(self) -> HeaderRewriter {
        self.inner
    }
}

fn forwarded_directive(addr: IpAddr) -> String {
    if addr.is_ipv6() {
        format!("for=\"[{addr}]\"")
    } else {
        format!("for={addr}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_header_rewriter_pre_process() {
        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = HeaderRewriter::builder().build();
        rewriter.pre_process(&mut headers, Ipv4Addr::new(127, 0, 0, 1).into());
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "127.0.0.1");

        let mut headers = HeaderMap::new();
        headers.append(FORWARDED, "for=192.168.0.1".parse().unwrap());

        let rewriter = HeaderRewriter::builder()
            .trust_upstream_headers(true)
            .build();
        rewriter.pre_process(&mut headers, Ipv4Addr::new(127, 0, 0, 1).into());
        assert_eq!(
            headers.get(FORWARDED).unwrap(),
            "for=192.168.0.1, for=127.0.0.1"
        );

        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = HeaderRewriter::builder()
            .trust_upstream_headers(true)
            .build();
        rewriter.pre_process(&mut headers, Ipv4Addr::new(127, 0, 0, 1).into());
        assert_eq!(
            headers.get("x-forwarded-for").unwrap(),
            "192.168.0.1, 127.0.0.1"
        );

        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = HeaderRewriter::builder()
            .trust_upstream_headers(true)
            .use_std_forwarded(true)
            .build();
        rewriter.pre_process(&mut headers, Ipv6Addr::LOCALHOST.into());
        assert_eq!(
            headers.get(FORWARDED).unwrap(),
            "for=192.168.0.1, for=\"[::1]\""
        );
    }

    #[test]
    fn test_header_rewriter_post_process() {
        let mut headers = HeaderMap::new();
        let rewriter = HeaderRewriter::builder()
            .set_via("taxy".parse().unwrap())
            .build();
        rewriter.post_process(&mut headers);
        assert_eq!(headers.get("via").unwrap(), "taxy");
    }
}
