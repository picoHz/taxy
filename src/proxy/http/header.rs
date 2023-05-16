use hyper::{
    header::{FORWARDED, VIA},
    http::header::Entry,
    http::HeaderValue,
    HeaderMap,
};
use std::net::IpAddr;

#[derive(Default)]
pub struct HeaderRewriter {
    trust_upstream_headers: bool,
    set_via: Option<HeaderValue>,
}

impl HeaderRewriter {
    pub fn builder() -> Builder {
        Default::default()
    }

    pub fn pre_process(&self, headers: &mut HeaderMap, remote_addr: IpAddr) {
        let mut x_forwarded_for: Vec<IpAddr> = Vec::new();
        if self.trust_upstream_headers {
            if let Entry::Occupied(entry) = headers.entry("x-forwarded-for") {
                x_forwarded_for = entry
                    .remove_entry_mult()
                    .1
                    .flat_map(|v| {
                        v.to_str()
                            .ok()
                            .unwrap_or_default()
                            .split(',')
                            .filter_map(|ip| ip.trim().parse().ok())
                            .collect::<Vec<_>>()
                    })
                    .collect();
            }
        } else {
            if let Entry::Occupied(entry) = headers.entry(FORWARDED) {
                entry.remove_entry_mult();
            }
            if let Entry::Occupied(entry) = headers.entry("x-forwarded-for") {
                entry.remove_entry_mult();
            }
            if let Entry::Occupied(entry) = headers.entry("x-forwarded-host") {
                entry.remove_entry_mult();
            }
            if let Entry::Occupied(entry) = headers.entry("x-real-ip") {
                entry.remove_entry_mult();
            }
        }
        x_forwarded_for.push(remote_addr);
        if let Ok(x_forwarded_value) = HeaderValue::from_str(
            &x_forwarded_for
                .iter()
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

    pub fn set_via(mut self, via: HeaderValue) -> Self {
        self.inner.set_via = Some(via);
        self
    }

    pub fn build(self) -> HeaderRewriter {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_header_rewriter_pre_process() {
        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = HeaderRewriter::builder().build();
        rewriter.pre_process(&mut headers, Ipv4Addr::new(127, 0, 0, 1).into());
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "127.0.0.1");

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
