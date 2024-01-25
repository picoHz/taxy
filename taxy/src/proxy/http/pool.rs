use super::{
    compression::{is_compressed, CompressionStream},
    error::map_response,
};
use crate::proxy::http::{
    error::ProxyError, hyper_tls::client::HttpsConnector, upgrade, IoStream, HTTP2_MAX_FRAME_SIZE,
};
use hyper::{
    client::HttpConnector,
    header::UPGRADE,
    http::{uri::Scheme, HeaderValue},
    Body, Client, Request, Response,
};
use std::sync::Arc;
use tokio::net::{self, TcpSocket};
use tokio_rustls::{rustls::ClientConfig, TlsConnector};
use tracing::{debug, error};
use warp::host::Authority;

pub struct ConnectionPool {
    tls_client_config: Arc<ClientConfig>,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl ConnectionPool {
    pub fn new(tls_client_config: Arc<ClientConfig>) -> Self {
        let https = HttpsConnector::new(tls_client_config.clone());
        let client = Client::builder()
            .http2_max_frame_size(Some(HTTP2_MAX_FRAME_SIZE as u32))
            .build::<_, hyper::Body>(https);

        Self {
            tls_client_config,
            client,
        }
    }

    pub async fn request(&self, mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
        let conn = Conn {
            scheme: req.uri().scheme().unwrap().clone(),
            authority: req.uri().authority().unwrap().clone(),
        };

        if req.headers().contains_key(UPGRADE) {
            return start_upgrading_connection(conn, req, self.tls_client_config.clone()).await;
        }

        let accept_brotli = req
            .headers()
            .get(hyper::header::ACCEPT_ENCODING)
            .map(|value| value.to_str().unwrap_or_default().contains("br"))
            .unwrap_or_default();

        *req.version_mut() = hyper::Version::HTTP_11;

        let result: Result<_, anyhow::Error> =
            self.client.request(req).await.map_err(|err| err.into());

        let http2 = result
            .as_ref()
            .map(|res| res.version() == hyper::Version::HTTP_2)
            .unwrap_or_default();

        let accept_brotli = accept_brotli & http2;

        let result = result.map(|res| {
            let (mut parts, body) = res.into_parts();

            let is_compressed = parts
                .headers
                .get(hyper::header::CONTENT_TYPE)
                .map(|value| is_compressed(value.as_bytes()))
                .unwrap_or_default();

            if !is_compressed {
                let encoding = parts.headers.entry(hyper::header::CONTENT_ENCODING);
                if let hyper::header::Entry::Vacant(entry) = encoding {
                    if accept_brotli {
                        entry.insert(HeaderValue::from_static("br"));
                        parts.headers.remove(hyper::header::CONTENT_LENGTH);
                        let stream = CompressionStream::new(body, HTTP2_MAX_FRAME_SIZE);
                        return Response::from_parts(parts, hyper::Body::wrap_stream(stream));
                    }
                }
            }

            Response::from_parts(parts, body)
        });

        if let Err(err) = &result {
            error!(%err);
        }

        map_response(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Conn {
    scheme: Scheme,
    authority: Authority,
}

async fn start_upgrading_connection(
    conn: Conn,
    req: Request<Body>,
    tls_client_config: Arc<ClientConfig>,
) -> Result<Response<Body>, anyhow::Error> {
    let resolved = net::lookup_host(conn.authority.as_str())
        .await
        .map_err(|_| ProxyError::DnsLookupFailed)?
        .next()
        .ok_or(ProxyError::DnsLookupFailed)?;
    debug!(authority = %conn.authority, %resolved);

    let sock = if resolved.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    let stream = sock.connect(resolved).await?;
    debug!(%resolved, "connected");

    let mut stream: Box<dyn IoStream> = Box::new(stream);
    if conn.scheme == Scheme::HTTPS {
        debug!(%resolved, "client: tls handshake");
        let tls = TlsConnector::from(tls_client_config);
        let host = conn.authority.host().to_string();
        let tls_stream = tls.connect(host.try_into().unwrap(), stream).await?;
        stream = Box::new(tls_stream);
    }

    upgrade::connect(req, stream).await
}
