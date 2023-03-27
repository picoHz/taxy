use crate::config::storage::ConfigStorage;
use crate::config::Source;
use crate::proxy::PortContext;
use crate::server::table::ProxyTable;
use crate::{command::ServerCommand, event::ServerEvent};
use listener::TcpListenerPool;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

mod listener;
mod table;

pub async fn start_server(
    config: ConfigStorage,
    mut command: mpsc::Receiver<ServerCommand>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let mut table = ProxyTable::new();
    let mut pool = TcpListenerPool::new();
    let mut event_recv = event.subscribe();

    let ports = config.load_entries().await;
    for port in &ports {
        table.set_port(&port.name, port.clone());
    }
    update_port_statuses(&event, &mut pool, &mut table).await;

    loop {
        tokio::select! {
            cmd = command.recv() => {
                match cmd {
                    Some(ServerCommand::SetPort { name, entry }) => {
                        table.set_port(&name, entry);
                        update_port_statuses(&event, &mut pool, &mut table).await;
                    },
                    Some(ServerCommand::DeletePort { name }) => {
                        table.delete_port(&name);
                        update_port_statuses(&event, &mut pool, &mut table).await;
                    },
                    _ => (),
                }
            }
            event = event_recv.recv() => {
                match event {
                    Ok(ServerEvent::PortTableUpdated { entries, source }) => {
                        if source != Source::File {
                            config.save_entries(&entries).await;
                        }
                    },
                    Ok(ServerEvent::Shutdown) => break,
                    Err(RecvError::Lagged(n)) => {
                        warn!("event stream lagged: {}", n);
                    }
                    _ => (),
                }
            }
            sock = pool.select(), if pool.has_active_listeners() => {
                if let Some((index, stream)) = sock {
                    let state = &mut table.contexts_mut()[index];
                    match state {
                        PortContext::Tcp(tcp) => {
                            tcp.start_proxy(stream);
                        }
                    }
                }
            }
        };
    }

    Ok(())
}

async fn update_port_statuses(
    event: &broadcast::Sender<ServerEvent>,
    pool: &mut TcpListenerPool,
    table: &mut ProxyTable,
) {
    pool.update(table.contexts_mut()).await;
    let _ = event.send(ServerEvent::PortTableUpdated {
        entries: table.entries().to_vec(),
        source: Source::Api,
    });
    for (entry, ctx) in table.entries().iter().zip(table.contexts()) {
        let _ = event.send(ServerEvent::PortStatusUpdated {
            name: entry.name.clone(),
            status: ctx.status().clone(),
        });
    }
}
