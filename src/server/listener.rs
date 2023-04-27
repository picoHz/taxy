use crate::proxy::{PortContext, PortContextEvent, PortContextKind, SocketState};
use futures::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

#[derive(Debug)]
pub struct TcpListenerPool {
    listeners: Vec<TcpListenerStream>,
}

impl TcpListenerPool {
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
            .map(|ctx| {
                let PortContextKind::Tcp(state) = ctx.kind();
                state.listen
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
            let PortContextKind::Tcp(state) = ctx.kind();
            let (listener, state) = if let Some(listener) = listeners.remove(&state.listen) {
                (Some(listener), SocketState::Listening)
            } else {
                let bind = state.listen;
                info!(%bind, "listening on tcp port");
                match TcpListener::bind(bind).await {
                    Ok(sock) => (
                        Some(TcpListenerStream {
                            index: 0,
                            inner: sock,
                        }),
                        SocketState::Listening,
                    ),
                    Err(error) => {
                        error!(%bind, %error, "failed to listen on tcp port");
                        let error = match error.kind() {
                            io::ErrorKind::AddrInUse => SocketState::PortAlreadyInUse,
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
            ctx.event(PortContextEvent::SokcetStateUpadted(state));
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
