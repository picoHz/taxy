use crate::proxy::{PortContext, PortContextEvent, PortContextKind};
use futures::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use taxy_api::port::SocketState;
use tokio::net::UdpSocket;
use tracing::{error, info, span, Instrument, Level};

#[derive(Debug)]
pub struct UdpListenerPool {
    listeners: Vec<UdpListenerStream>,
}

impl UdpListenerPool {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn has_active_listeners(&self) -> bool {
        !self.listeners.is_empty()
    }

    pub async fn update(&mut self, ports: &mut [PortContext]) {
        let used_addrs = ports
            .iter()
            .filter_map(|ctx| match ctx.kind() {
                PortContextKind::Udp(state) => Some(state.listen),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let mut listeners: HashMap<_, _> = self
            .listeners
            .drain(..)
            .filter_map(|listener| {
                listener
                    .inner
                    .local_addr()
                    .ok()
                    .map(|addr| (addr, listener))
            })
            .filter(|(addr, _)| used_addrs.contains(addr))
            .collect();

        for (index, ctx) in ports.iter_mut().enumerate() {
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
            let bind = match ctx.kind() {
                PortContextKind::Udp(state) => state.listen,
                _ => continue,
            };
            let (listener, state) = if !ctx.entry.port.active {
                (None, SocketState::Inactive)
            } else if let Some(listener) = listeners.remove(&bind) {
                (Some(listener), SocketState::Listening)
            } else {
                span.in_scope(|| {
                    info!(%bind, "listening on udp port");
                });
                match UdpSocket::bind(bind).instrument(span.clone()).await {
                    Ok(sock) => (
                        Some(UdpListenerStream {
                            index: 0,
                            config_index: 0,
                            inner: sock,
                        }),
                        SocketState::Listening,
                    ),
                    Err(err) => {
                        let _enter = span.enter();
                        error!(%bind, %err, "failed to listen on udp port");
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
                sock.index = self.listeners.len();
                sock.config_index = index;
                self.listeners.push(sock);
            }
            ctx.event(PortContextEvent::SocketStateUpdated(state));
        }
    }

    pub async fn select(&mut self) -> Option<(usize, usize, SocketAddr, Vec<u8>)> {
        let streams = &mut self.listeners;
        match futures::stream::select_all(streams.iter_mut())
            .next()
            .await
        {
            Some((index, config_index, Ok((addr, data)))) => {
                Some((index, config_index, addr, data))
            }
            _ => None,
        }
    }

    pub async fn send_to(&self, index: usize, addr: SocketAddr, data: &[u8]) {
        if let Some(listener) = self.listeners.get(index) {
            let _ = listener.inner.send_to(data, addr).await;
        }
    }
}

#[derive(Debug)]
struct UdpListenerStream {
    index: usize,
    config_index: usize,
    inner: UdpSocket,
}

impl Stream for UdpListenerStream {
    type Item = (usize, usize, io::Result<(SocketAddr, Vec<u8>)>);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<(usize, usize, io::Result<(SocketAddr, Vec<u8>)>)>> {
        let mut buf = vec![0; 65527];
        let mut read_buf = tokio::io::ReadBuf::new(&mut buf);
        match self.inner.poll_recv_from(cx, &mut read_buf) {
            Poll::Ready(Ok(addr)) => Poll::Ready(Some((
                self.index,
                self.config_index,
                Ok((addr, read_buf.filled().to_vec())),
            ))),
            Poll::Ready(Err(err)) => Poll::Ready(Some((self.index, self.config_index, Err(err)))),
            Poll::Pending => Poll::Pending,
        }
    }
}
