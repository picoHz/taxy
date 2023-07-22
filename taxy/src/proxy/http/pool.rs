use super::{
    compression::{is_compressed, CompressionStream},
    error::map_response,
};
use crate::proxy::http::{error::ProxyError, upgrade, IoStream, HTTP2_MAX_FRAME_SIZE};
use dashmap::DashMap;
use hyper::{
    client::{self, conn::Connection, conn::SendRequest},
    header::UPGRADE,
    http::{uri::Scheme, HeaderValue},
    Body, Request, Response,
};
use std::sync::Arc;
use tokio::{
    net::{self, TcpSocket},
    sync::{mpsc, oneshot},
};
use tokio_rustls::{rustls::ClientConfig, TlsConnector};
use tracing::{debug, error};
use warp::host::Authority;

pub struct ConnectionPool {
    tls_client_config: Option<Arc<ClientConfig>>,
    connections: DashMap<Conn, mpsc::Sender<Req>>,
}

impl ConnectionPool {
    pub fn new(tls_client_config: Option<Arc<ClientConfig>>) -> Self {
        Self {
            tls_client_config,
            connections: DashMap::new(),
        }
    }

    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
        let conn = Conn {
            scheme: req.uri().scheme().unwrap().clone(),
            authority: req.uri().authority().unwrap().clone(),
        };

        if req.headers().contains_key(UPGRADE) {
            return start_upgrading_connection(conn, req, self.tls_client_config.clone()).await;
        }

        let (tx_res, rx_res) = oneshot::channel();
        let req = Req {
            request: req,
            result: tx_res,
        };

        let req = if let Some(tx) = self.connections.get(&conn) {
            match tx.send(req).await {
                Ok(_) => return rx_res.await?,
                Err(err) => {
                    self.connections.remove(&conn);
                    err.0
                }
            }
        } else {
            req
        };

        let (tx, rx) = mpsc::channel(1);
        self.connections.insert(conn.clone(), tx.clone());

        let conn = match start_connection(rx, conn, self.tls_client_config.clone()).await {
            Ok(conn) => conn,
            Err(err) => {
                return map_response(Err(err));
            }
        };

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {:?}", err);
            }
        });

        let _ = tx.send(req).await;
        rx_res.await?
    }
}

struct Req {
    request: Request<Body>,
    result: oneshot::Sender<Result<Response<Body>, anyhow::Error>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Conn {
    scheme: Scheme,
    authority: Authority,
}

async fn start_connection(
    mut recv: mpsc::Receiver<Req>,
    conn: Conn,
    tls_client_config: Option<Arc<ClientConfig>>,
) -> anyhow::Result<Connection<Box<dyn IoStream>, Body>> {
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

    let mut client_http2 = false;

    let mut stream: Box<dyn IoStream> = Box::new(stream);
    if let Some(config) = tls_client_config.filter(|_| conn.scheme == Scheme::HTTPS) {
        debug!(%resolved, "client: tls handshake");
        let tls = TlsConnector::from(config.clone());
        let tls_stream = tls
            .connect(conn.authority.host().try_into().unwrap(), stream)
            .await?;
        client_http2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
        stream = Box::new(tls_stream);
    }

    let (mut sender, conn) = client::conn::Builder::new()
        .http2_only(client_http2)
        .http2_max_frame_size(Some(HTTP2_MAX_FRAME_SIZE as u32))
        .handshake::<_, Body>(stream)
        .await?;

    tokio::spawn(async move {
        while let Some(req) = recv.recv().await {
            handle_request(req, &mut sender, client_http2).await;
        }
    });

    Ok(conn)
}

async fn handle_request(req: Req, sender: &mut SendRequest<Body>, client_http2: bool) {
    let accept_brotli = client_http2
        && req
            .request
            .headers()
            .get(hyper::header::ACCEPT_ENCODING)
            .map(|value| value.to_str().unwrap_or_default().contains("br"))
            .unwrap_or_default();

    let result: Result<_, anyhow::Error> = sender
        .send_request(req.request)
        .await
        .map_err(|err| err.into());

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

    let result = map_response(result);
    let _ = req.result.send(result);
}

async fn start_upgrading_connection(
    conn: Conn,
    req: Request<Body>,
    tls_client_config: Option<Arc<ClientConfig>>,
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
    if let Some(config) = tls_client_config.filter(|_| conn.scheme == Scheme::HTTPS) {
        debug!(%resolved, "client: tls handshake");
        let tls = TlsConnector::from(config.clone());
        let tls_stream = tls
            .connect(conn.authority.host().try_into().unwrap(), stream)
            .await?;
        stream = Box::new(tls_stream);
    }

    upgrade::connect(req, stream).await
}
