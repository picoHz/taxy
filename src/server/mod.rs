use crate::config::storage::ConfigStorage;
use crate::config::Source;
use crate::proxy::{PortContext, PortContextKind};
use crate::server::table::ProxyTable;
use crate::{command::ServerCommand, event::ServerEvent};
use listener::TcpListenerPool;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, warn};

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

    let app_config = config.load_app_config().await;
    let _ = event.send(ServerEvent::AppConfigUpdated {
        config: app_config.clone(),
        source: Source::File,
    });

    let mut certs = config.load_certs().await;
    let _ = event.send(ServerEvent::CertListUpdated {
        certs: certs.list(),
    });

    let ports = config.load_entries().await;
    for entry in ports {
        match PortContext::new(entry) {
            Ok(mut ctx) => {
                if let Err(err) = ctx.prepare(&app_config).await {
                    error!(?err, "failed to prepare port");
                }
                if let Err(err) = ctx.setup(&certs).await {
                    error!(?err, "failed to setup port");
                }
                table.set_port(ctx);
            }
            Err(err) => {
                error!(?err, "failed to create proxy state");
            }
        };
    }
    update_port_statuses(&event, &mut pool, &mut table).await;

    loop {
        tokio::select! {
            cmd = command.recv() => {
                match cmd {
                    Some(ServerCommand::SetAppConfig { config }) => {
                        let _ = event.send(ServerEvent::AppConfigUpdated {
                            config,
                            source: Source::Api,
                        });
                    },
                    Some(ServerCommand::SetPort { mut ctx }) => {
                        if let Err(err) = ctx.setup(&certs).await {
                            error!(?err, "failed to setup port");
                        }
                        table.set_port(ctx);
                        update_port_statuses(&event, &mut pool, &mut table).await;
                    },
                    Some(ServerCommand::DeletePort { name }) => {
                        table.delete_port(&name);
                        update_port_statuses(&event, &mut pool, &mut table).await;
                    },
                    Some(ServerCommand::AddCert { cert }) => {
                        config.save_cert(&cert).await;
                        certs.add(cert);
                        let _ = event.send(ServerEvent::CertListUpdated { certs: certs.list() } );
                    }
                    Some(ServerCommand::DeleteCert { id }) => {
                        config.delete_cert(&id).await;
                        certs.delete(&id);
                        let _ = event.send(ServerEvent::CertListUpdated { certs: certs.list() } );
                    }
                    _ => (),
                }
            }
            event = event_recv.recv() => {
                match event {
                    Ok(ServerEvent::AppConfigUpdated { config: app_config, source }) => {
                        if source != Source::File {
                            config.save_app_config(&app_config).await;
                        }
                    },
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
                    match state.kind_mut() {
                        PortContextKind::Tcp(tcp) => {
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
