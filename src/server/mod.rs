use self::state::ServerState;
use crate::config::storage::ConfigStorage;
use crate::{command::ServerCommand, event::ServerEvent};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

mod listener;
mod state;
mod table;

pub async fn start_server(
    config: ConfigStorage,
    command_send: mpsc::Sender<ServerCommand>,
    mut command_recv: mpsc::Receiver<ServerCommand>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let mut event_recv = event.subscribe();
    let mut server = ServerState::new(config, command_send, event).await;
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
        }
    }

    Ok(())
}
