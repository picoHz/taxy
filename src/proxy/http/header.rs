use std::net::IpAddr;

use hyper::{
    header::{FORWARDED, VIA},
    http::header::Entry,
    http::HeaderValue,
    HeaderMap,
};

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
