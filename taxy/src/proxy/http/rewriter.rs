use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::header::ALT_SVC;
use hyper::{body::Body, Response};
use hyper::{
    header::{FORWARDED, VIA},
    http::{header::Entry, HeaderValue},
    HeaderMap,
};
use sailfish::TemplateOnce;
use std::{iter, net::IpAddr};

use super::error::{map_error, ErrorTemplate};

#[derive(Default, Debug)]
pub struct RequestRewriter {
    trust_upstream_headers: bool,
    set_via: Option<HeaderValue>,
}

impl RequestRewriter {
    pub fn builder() -> RequestRewriterBuilder {
        Default::default()
    }

    fn remove_untrusted_headers(&self, headers: &mut HeaderMap) {
        let header_keys = &[FORWARDED.as_str(), "x-forwarded-for", "x-real-ip"];
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

    pub fn pre_process(
        &self,
        headers: &mut HeaderMap,
        remote_addr: IpAddr,
        header_host: Option<String>,
        forwarded_proto: &'static str,
    ) {
        let mut x_forwarded_for = Vec::new();
        let mut forwarded = Vec::new();

        if self.trust_upstream_headers {
            x_forwarded_for = self.parse_x_forwarded_for(headers);
            forwarded = self.parse_forwarded(headers);
        } else {
            self.remove_untrusted_headers(headers);
        }

        if forwarded.is_empty() {
            forwarded = x_forwarded_for
                .iter()
                .map(|ip| forwarded_for_directive(*ip))
                .collect();
        }
        if let Ok(forwarded_value) = HeaderValue::from_str(
            &forwarded
                .into_iter()
                .chain(iter::once(forwarded_for_directive(remote_addr)))
                .chain(
                    header_host
                        .as_ref()
                        .map(|host| forwarded_host_directive(host)),
                )
                .chain(iter::once(forwarded_proto_directive(forwarded_proto)))
                .collect::<Vec<_>>()
                .join(", "),
        ) {
            headers.insert(FORWARDED, forwarded_value);
        }

        if let Ok(x_forwarded_value) = HeaderValue::from_str(
            &x_forwarded_for
                .iter()
                .chain(iter::once(&remote_addr))
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ) {
            headers.insert("x-forwarded-for", x_forwarded_value);
        }

        headers.insert(
            "x-forwarded-proto",
            HeaderValue::from_static(forwarded_proto),
        );

        if let Some(host) = &header_host {
            if let Ok(host) = HeaderValue::from_str(host) {
                headers.insert("x-forwarded-host", host);
            }
        }
    }

    pub fn post_process(&self, headers: &mut HeaderMap) {
        if let Some(via) = &self.set_via {
            headers.insert(VIA, via.clone());
        }
    }
}

#[derive(Default)]
pub struct RequestRewriterBuilder {
    inner: RequestRewriter,
}

impl RequestRewriterBuilder {
    pub fn trust_upstream_headers(mut self, trust: bool) -> Self {
        self.inner.trust_upstream_headers = trust;
        self
    }

    pub fn set_via(mut self, via: HeaderValue) -> Self {
        self.inner.set_via = Some(via);
        self
    }

    pub fn build(self) -> RequestRewriter {
        self.inner
    }
}

fn forwarded_for_directive(addr: IpAddr) -> String {
    if addr.is_ipv6() {
        format!("for=\"[{addr}]\"")
    } else {
        format!("for={addr}")
    }
}

fn forwarded_host_directive(host: &str) -> String {
    format!("host={host}")
}

fn forwarded_proto_directive(proto: &str) -> String {
    format!("proto={proto}")
}

#[derive(Default, Debug)]
pub struct ResponseRewriter {
    https_port: Option<u16>,
    quic_port: Option<u16>,
}

impl ResponseRewriter {
    pub fn builder() -> ResponseRewriterBuilder {
        Default::default()
    }

    pub fn map_response<B>(
        &self,
        res: Result<Response<B>, anyhow::Error>,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>, anyhow::Error>
    where
        B: Body<Data = Bytes, Error = anyhow::Error> + Send + Sync + 'static,
    {
        match res {
            Ok(mut res) => {
                res.headers_mut().remove(ALT_SVC);
                let alt_svc = match (self.https_port, self.quic_port) {
                    (Some(https), Some(quic)) => Some(format!(
                        "h2=\":{}\", h3=\":{}\", h3-25=\":{}\"",
                        https, quic, quic
                    )),
                    (Some(https), None) => Some(format!("h2=\":{}\"", https)),
                    (None, Some(quic)) => Some(format!("h3=\":{}\", h3-25=\":{}\"", quic, quic)),
                    _ => None,
                };
                if let Some(alt_svc) = alt_svc {
                    res.headers_mut()
                        .insert(ALT_SVC, HeaderValue::from_str(&alt_svc).unwrap());
                }
                Ok(res.map(|body| BoxBody::new(body)))
            }
            Err(err) => {
                let code = map_error(err);
                let ctx = ErrorTemplate {
                    code: code.as_u16(),
                };
                let mut res = Response::new(BoxBody::new(
                    Full::new(Bytes::from(ctx.render_once().unwrap())).map_err(Into::into),
                ));
                *res.status_mut() = code;
                Ok(res)
            }
        }
    }
}

#[derive(Default)]
pub struct ResponseRewriterBuilder {
    inner: ResponseRewriter,
}

impl ResponseRewriterBuilder {
    pub fn https_port(mut self, port: Option<u16>) -> Self {
        self.inner.https_port = port;
        self
    }

    pub fn quic_port(mut self, port: Option<u16>) -> Self {
        self.inner.quic_port = port;
        self
    }

    pub fn build(self) -> ResponseRewriter {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_header_rewriter_pre_process() {
        let forwarded_proto = "http";

        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = RequestRewriter::builder().build();
        rewriter.pre_process(
            &mut headers,
            Ipv4Addr::new(127, 0, 0, 1).into(),
            Some("example.com".into()),
            forwarded_proto,
        );
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "127.0.0.1");
        assert_eq!(headers.get("x-forwarded-proto").unwrap(), "http");
        assert_eq!(headers.get("x-forwarded-host").unwrap(), "example.com");

        let mut headers = HeaderMap::new();
        headers.append(FORWARDED, "for=192.168.0.1".parse().unwrap());

        let rewriter = RequestRewriter::builder()
            .trust_upstream_headers(true)
            .build();
        rewriter.pre_process(
            &mut headers,
            Ipv4Addr::new(127, 0, 0, 1).into(),
            Some("example.com".into()),
            forwarded_proto,
        );
        assert_eq!(
            headers.get(FORWARDED).unwrap(),
            "for=192.168.0.1, for=127.0.0.1, host=example.com, proto=http"
        );
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "127.0.0.1");
        assert_eq!(headers.get("x-forwarded-proto").unwrap(), "http");
        assert_eq!(headers.get("x-forwarded-host").unwrap(), "example.com");

        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = RequestRewriter::builder()
            .trust_upstream_headers(true)
            .build();
        rewriter.pre_process(
            &mut headers,
            Ipv4Addr::new(127, 0, 0, 1).into(),
            Some("example.com".into()),
            forwarded_proto,
        );
        assert_eq!(
            headers.get("x-forwarded-for").unwrap(),
            "192.168.0.1, 127.0.0.1"
        );
        assert_eq!(headers.get("x-forwarded-proto").unwrap(), "http");
        assert_eq!(headers.get("x-forwarded-host").unwrap(), "example.com");

        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", "192.168.0.1".parse().unwrap());

        let rewriter = RequestRewriter::builder()
            .trust_upstream_headers(true)
            .build();
        rewriter.pre_process(
            &mut headers,
            Ipv6Addr::LOCALHOST.into(),
            Some("example.com".into()),
            forwarded_proto,
        );
        assert_eq!(
            headers.get(FORWARDED).unwrap(),
            "for=192.168.0.1, for=\"[::1]\", host=example.com, proto=http"
        );
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "192.168.0.1, ::1");
        assert_eq!(headers.get("x-forwarded-proto").unwrap(), "http");
        assert_eq!(headers.get("x-forwarded-host").unwrap(), "example.com");
    }

    #[test]
    fn test_header_rewriter_post_process() {
        let mut headers = HeaderMap::new();
        let rewriter = RequestRewriter::builder()
            .set_via("taxy".parse().unwrap())
            .build();
        rewriter.post_process(&mut headers);
        assert_eq!(headers.get("via").unwrap(), "taxy");
    }
}
