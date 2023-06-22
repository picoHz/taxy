use self::rpc::RpcCallback;
use self::state::ServerState;
use crate::command::ServerCommand;
use crate::config::file::FileStorage;
use taxy_api::event::ServerEvent;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};

mod acme_list;
pub mod cert_list;
mod listener;
mod port_list;
pub mod rpc;
mod site_list;
mod state;

pub struct Server {
    config: FileStorage,
    command_send: mpsc::Sender<ServerCommand>,
    command_recv: mpsc::Receiver<ServerCommand>,
    callback: mpsc::Sender<RpcCallback>,
    event: broadcast::Sender<ServerEvent>,
}

impl Server {
    pub fn new(config: FileStorage) -> (Self, ServerChannels) {
        let (command_send, command_recv) = mpsc::channel(1);
        let (callback_send, callback_recv) = mpsc::channel(16);
        let (event_send, _) = broadcast::channel(16);
        let server = Self {
            config,
            command_send,
            command_recv,
            callback: callback_send,
            event: event_send,
        };
        let channels = ServerChannels {
            command: server.command_send.clone(),
            callback: callback_recv,
            event: server.event.clone(),
        };
        (server, channels)
    }

    pub async fn start(self) -> anyhow::Result<()> {
        start_server(
            self.config,
            self.command_send,
            self.command_recv,
            self.callback,
            self.event,
        )
        .await
    }
}

pub struct ServerChannels {
    pub command: mpsc::Sender<ServerCommand>,
    pub callback: mpsc::Receiver<RpcCallback>,
    pub event: broadcast::Sender<ServerEvent>,
}

async fn start_server(
    config: FileStorage,
    command_send: mpsc::Sender<ServerCommand>,
    mut command_recv: mpsc::Receiver<ServerCommand>,
    callback: mpsc::Sender<RpcCallback>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let mut event_recv = event.subscribe();
    let mut server = ServerState::new(config, command_send, callback, event).await;

    let mut background_task_interval =
        tokio::time::interval(server.config().background_task_interval);
    background_task_interval.tick().await;

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
                    Ok(ServerEvent::AppConfigUpdated { config, .. }) => {
                        let mut new_interval = tokio::time::interval(config.background_task_interval);
                        new_interval.tick().await;
                        background_task_interval = new_interval;
                    },
                    Ok(event) => server.handle_event(event).await,
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
        }
    }

    Ok(())
}
