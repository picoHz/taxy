use self::rpc::RpcCallback;
use self::state::ServerState;
use crate::command::ServerCommand;
use crate::config::storage::Storage;
use std::time::Duration;
use taxy_api::event::ServerEvent;
use taxy_api::site::ProxyEntry;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, trace, warn};

mod acme_list;
pub mod cert_list;
mod health_checker;
mod listener;
mod port_list;
mod proxy_list;
pub mod rpc;
mod state;

pub struct Server {
    server_state: ServerState,
    command_recv: mpsc::Receiver<ServerCommand>,
    event_recv: broadcast::Receiver<ServerEvent>,
}

impl Server {
    pub async fn new<S>(config: S) -> (Self, ServerChannels)
    where
        S: Storage,
    {
        let (command_send, command_recv) = mpsc::channel(1);
        let (callback_send, callback_recv) = mpsc::channel(16);
        let (event_send, _) = broadcast::channel(16);
        let server_state = ServerState::new(
            config,
            command_send.clone(),
            callback_send,
            event_send.clone(),
        )
        .await;
        let server = Self {
            server_state,
            command_recv,
            event_recv: event_send.subscribe(),
        };
        let channels = ServerChannels {
            command: command_send,
            callback: callback_recv,
            event: event_send,
        };
        (server, channels)
    }

    pub async fn start(self) -> anyhow::Result<()> {
        start_server(self.server_state, self.command_recv, self.event_recv).await
    }
}

pub struct ServerChannels {
    pub command: mpsc::Sender<ServerCommand>,
    pub callback: mpsc::Receiver<RpcCallback>,
    pub event: broadcast::Sender<ServerEvent>,
}

impl ServerChannels {
    pub fn shutdown(&self) {
        let _ = self.event.send(ServerEvent::Shutdown);
    }
}

async fn start_server(
    mut server: ServerState,
    mut command_recv: mpsc::Receiver<ServerCommand>,
    mut event_recv: broadcast::Receiver<ServerEvent>,
) -> anyhow::Result<()> {
    let mut background_task_interval =
        tokio::time::interval(server.config().background_task_interval);
    background_task_interval.tick().await;

    let mut healthcheck_interval = tokio::time::interval(gcd_iter(server.proxies.entries()));
    loop {
        tokio::select! {
            cmd = command_recv.recv() => {
                if let Some(cmd) = cmd {
                    server.handle_command(cmd).await;
                }
            }
            event = event_recv.recv() => {
                match event {
                    Ok(ServerEvent::Shutdown) => break,
                    Ok(ServerEvent::AppConfigUpdated { config }) => {
                        let mut new_interval = tokio::time::interval(config.background_task_interval);
                        new_interval.tick().await;
                        background_task_interval = new_interval;
                    },
                    Err(RecvError::Lagged(n)) => {
                        warn!("event stream lagged: {}", n);
                    }
                    _ => ()
                }
            }
            sock = server.select(), if server.has_active_listeners() => {
                if let Some((index, stream)) = sock {
                    server.handle_connection(index, stream).await;
                }
            }
            _ = background_task_interval.tick() => {
                info!("Starting background tasks (interval: {:?})", background_task_interval.period());
                server.run_background_tasks().await;
                let mut new_interval = tokio::time::interval(server.config().background_task_interval);
                new_interval.tick().await;
                background_task_interval = new_interval;
            }
            _ = healthcheck_interval.tick() => {
                trace!("Starting healthchecks (interval: {:?})", healthcheck_interval.period());
                server.run_healthchecks().await;
                let mut new_interval = tokio::time::interval(gcd_iter(server.proxies.entries()));
                new_interval.tick().await;
                healthcheck_interval = new_interval;
            }
        }
    }

    Ok(())
}

fn gcd_iter<'a>(iter: impl Iterator<Item = &'a ProxyEntry>) -> Duration {
    let mut iter = iter
        .filter_map(|entry| entry.proxy.health_check.as_ref())
        .map(|hc| hc.interval)
        .filter(|d| !d.is_zero());
    if let Some(first) = iter.next() {
        iter.fold(first, gcd).max(Duration::from_secs(1))
    } else {
        Duration::from_secs(60 * 60)
    }
}

fn gcd(a: Duration, b: Duration) -> Duration {
    let mut a = a;
    let mut b = b;
    while !b.is_zero() {
        let t = b;
        b = Duration::from_secs(a.as_secs() % b.as_secs());
        a = t;
    }
    a
}
