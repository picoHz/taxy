use crate::proxy::{PortContext, PortContextEvent, PortContextKind};
use futures::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use taxy_api::port::SocketState;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, span, Level};

const SOCKET_BACKLOG_SIZE: i32 = 128;

#[derive(Debug)]
pub struct TcpListenerPool {
    listeners: Vec<TcpListenerStream>,
    http_challenge_addr: Option<SocketAddr>,
}

impl TcpListenerPool {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            http_challenge_addr: None,
        }
    }

    pub fn set_http_challenge_addr(&mut self, addr: Option<SocketAddr>) {
        self.http_challenge_addr = addr;
    }

    pub fn has_active_listeners(&self) -> bool {
        !self.listeners.is_empty()
    }

    pub async fn remove_unused_ports(&mut self, ports: &[PortContext]) {
        let used_addrs = ports
            .iter()
            .filter_map(|ctx| match ctx.kind() {
                PortContextKind::Tcp(state) => Some(state.listen),
                PortContextKind::Http(state) => Some(state.listen),
                _ => None,
            })
            .collect::<HashSet<_>>();

        self.listeners.retain(|listener| {
            if let Ok(addr) = listener.inner.local_addr() {
                used_addrs.contains(&addr)
            } else {
                false
            }
        });
    }

    pub async fn update(&mut self, ports: &mut [PortContext]) {
        let mut reserved_ports = Vec::new();
        if let Some(reserved_addr) = self.http_challenge_addr {
            let port_used = ports.iter().any(|ctx| match ctx.kind() {
                PortContextKind::Tcp(state) => state.listen.port() == reserved_addr.port(),
                PortContextKind::Http(state) => state.listen.port() == reserved_addr.port(),
                _ => false,
            });
            if !port_used {
                reserved_ports.push(PortContext::reserved());
            }
        }

        let used_addrs = ports
            .iter()
            .chain(&reserved_ports)
            .filter_map(|ctx| match ctx.kind() {
                PortContextKind::Tcp(state) => Some(state.listen),
                PortContextKind::Http(state) => Some(state.listen),
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

        for (index, ctx) in ports
            .iter_mut()
            .chain(reserved_ports.iter_mut())
            .enumerate()
        {
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
            let bind = match ctx.kind() {
                PortContextKind::Tcp(state) => state.listen,
                PortContextKind::Http(state) => state.listen,
                _ => {
                    if let Some(addr) = self.http_challenge_addr {
                        addr
                    } else {
                        continue;
                    }
                }
            };
            let (listener, state) = if !ctx.entry.port.active {
                (None, SocketState::Inactive)
            } else if let Some(listener) = listeners.remove(&bind) {
                (Some(listener), SocketState::Listening)
            } else {
                span.in_scope(|| {
                    info!(%bind, "listening on tcp port");
                });
                match create_tcp_listener(bind) {
                    Ok(sock) => (
                        Some(TcpListenerStream {
                            index: 0,
                            inner: sock,
                        }),
                        SocketState::Listening,
                    ),
                    Err(err) => {
                        let _enter = span.enter();
                        error!(%bind, %err, "failed to listen on tcp port");
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
                sock.index = index;
                self.listeners.push(sock);
            }
            ctx.event(PortContextEvent::SocketStateUpdated(state));
        }
    }

    pub async fn select(&mut self) -> Option<(usize, TcpStream)> {
        let streams = &mut self.listeners;
        match futures::stream::select_all(streams).next().await {
            Some((index, Ok(sock))) => Some((index, sock)),
            _ => None,
        }
    }
}

fn create_tcp_listener(addr: SocketAddr) -> io::Result<TcpListener> {
    let socket = socket2::Socket::new(
        socket2::Domain::for_address(addr),
        socket2::Type::STREAM,
        None,
    )?;
    if addr.is_ipv6() {
        socket.set_only_v6(true)?;
    }
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(SOCKET_BACKLOG_SIZE)?;
    TcpListener::from_std(socket.into())
}

#[derive(Debug)]
struct TcpListenerStream {
    index: usize,
    inner: TcpListener,
}

impl Stream for TcpListenerStream {
    type Item = (usize, io::Result<TcpStream>);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<(usize, io::Result<TcpStream>)>> {
        match self.inner.poll_accept(cx) {
            Poll::Ready(Ok((stream, _))) => Poll::Ready(Some((self.index, Ok(stream)))),
            Poll::Ready(Err(err)) => Poll::Ready(Some((self.index, Err(err)))),
            Poll::Pending => Poll::Pending,
        }
    }
}
