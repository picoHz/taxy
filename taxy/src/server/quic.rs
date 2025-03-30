use crate::proxy::tls::CertResolver;
use crate::proxy::{PortContext, PortContextEvent, PortContextKind};
use futures::{Stream, StreamExt};
use h3_quinn::quinn::{self, Incoming};
use quinn::crypto::rustls::QuicServerConfig;
use std::collections::{HashMap, HashSet};
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use taxy_api::port::SocketState;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_rustls::rustls::server::ResolvesServerCert;
use tokio_rustls::rustls::ServerConfig;
use tracing::{error, info, span, Level};

#[derive(Debug)]
pub struct QuicListenerPool {
    listeners: Vec<QuicListenerStream>,
}

impl QuicListenerPool {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub async fn remove_unused_ports(&mut self, ports: &[PortContext]) {
        let used_addrs = ports
            .iter()
            .filter_map(|ctx| match ctx.kind() {
                PortContextKind::Http3(state) => Some(state.listen),
                _ => None,
            })
            .collect::<HashSet<_>>();

        for listener in self.listeners.iter_mut() {
            if !used_addrs.contains(&listener.local_addr) {
                let _ = listener.close.send(()).await;
                let _ = listener.closed.recv().await;
            }
        }

        self.listeners
            .retain(|listener| used_addrs.contains(&listener.local_addr));
    }

    pub async fn update(&mut self, ports: &mut [PortContext]) {
        let resolver: Arc<dyn ResolvesServerCert> = Arc::new(CertResolver::default());
        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        let used_addrs = ports
            .iter()
            .filter_map(|ctx| match ctx.kind() {
                PortContextKind::Http3(state) => Some(state.listen),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let mut listeners: HashMap<_, _> = self
            .listeners
            .drain(..)
            .map(|listener| (listener.local_addr, listener))
            .filter(|(addr, _)| used_addrs.contains(addr))
            .collect();

        for (index, ctx) in ports.iter_mut().enumerate() {
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
            let bind = match ctx.kind() {
                PortContextKind::Http3(state) => state.listen,
                _ => continue,
            };
            let (listener, state) = if !ctx.entry.port.active {
                (None, SocketState::Inactive)
            } else if let Some(listener) = listeners.remove(&bind) {
                (Some(listener), SocketState::Listening)
            } else {
                span.in_scope(|| {
                    info!(%bind, "listening on quic port");
                });

                let server_config = quinn::ServerConfig::with_crypto(Arc::new(
                    QuicServerConfig::try_from(tls_config.clone()).unwrap(),
                ));
                match quinn::Endpoint::server(server_config, bind) {
                    Ok(sock) => {
                        let local_addr = sock.local_addr().unwrap();
                        let (send, recv) = tokio::sync::mpsc::channel(1);
                        let (close_send, mut close_recv) = tokio::sync::mpsc::channel::<()>(1);
                        let (closed_send, closed_recv) = tokio::sync::mpsc::channel::<()>(1);
                        tokio::spawn(async move {
                            loop {
                                tokio::select! {
                                    _ = close_recv.recv() => break,
                                    conn = sock.accept() => {
                                        match conn {
                                            Some(conn) => {
                                                if send.send(conn).await.is_err() {
                                                    break;
                                                }
                                            }
                                            None => {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            sock.close(1001u16.into(), &[]);
                            sock.wait_idle().await;
                            std::mem::drop(send);
                            let _ = closed_send.send(()).await;
                        });
                        (
                            Some(QuicListenerStream {
                                config_index: 0,
                                local_addr,
                                inner: recv,
                                close: close_send,
                                closed: closed_recv,
                            }),
                            SocketState::Listening,
                        )
                    }
                    Err(err) => {
                        let _enter = span.enter();
                        error!(%bind, %err, "failed to listen on quic port");
                        let error = match err.kind() {
                            io::ErrorKind::AddrInUse => SocketState::AddressAlreadyInUse,
                            io::ErrorKind::PermissionDenied => SocketState::PermissionDenied,
                            io::ErrorKind::AddrNotAvailable => SocketState::AddressNotAvailable,
                            _ => SocketState::Error,
                        };
                        (None, error)
                    }
                }
            };
            if let Some(mut sock) = listener {
                sock.config_index = index;
                self.listeners.push(sock);
            }
            ctx.event(PortContextEvent::SocketStateUpdated(state));
        }
    }

    pub fn has_active_listeners(&self) -> bool {
        !self.listeners.is_empty()
    }

    pub async fn select(&mut self) -> Option<(usize, Incoming)> {
        let streams = &mut self.listeners;
        match futures::stream::select_all(streams).next().await {
            Some((index, incoming)) => Some((index, incoming)),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct QuicListenerStream {
    config_index: usize,
    local_addr: SocketAddr,
    inner: Receiver<Incoming>,
    close: Sender<()>,
    closed: Receiver<()>,
}

impl Stream for QuicListenerStream {
    type Item = (usize, Incoming);

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<(usize, Incoming)>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(incoming)) => Poll::Ready(Some((self.config_index, incoming))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
