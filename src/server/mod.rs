use crate::config::storage::ConfigStorage;
use crate::config::Source;
use crate::keyring::KeyringItem;
use crate::proxy::{PortContext, PortContextKind};
use crate::server::table::ProxyTable;
use crate::{command::ServerCommand, event::ServerEvent};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use listener::TcpListenerPool;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::TcpStream;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, warn};
use warp::http::{Request, Response};

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

    let http_challenges = HashMap::<String, String>::new();
    pool.set_http_challenges(!http_challenges.is_empty());

    let app_config = config.load_app_config().await;
    let _ = event.send(ServerEvent::AppConfigUpdated {
        config: app_config.clone(),
        source: Source::File,
    });

    let mut certs = config.load_certs().await;
    let _ = event.send(ServerEvent::KeyringUpdated {
        items: certs.list(),
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
                    Some(ServerCommand::AddKeyringItem { item }) => {
                        if let KeyringItem::ServerCert (cert) = &item {
                            config.save_cert(cert).await;
                        }
                        certs.add(item);
                        let _ = event.send(ServerEvent::KeyringUpdated { items: certs.list() } );
                    }
                    Some(ServerCommand::DeleteKeyringItem { id }) => {
                        config.delete_cert(&id).await;
                        certs.delete(&id);
                        let _ = event.send(ServerEvent::KeyringUpdated { items: certs.list() } );
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
                    let mut stream = BufStream::new(stream);

                    if !http_challenges.is_empty() {
                        if let Some(body) = handle_http_challenge(&mut stream, &http_challenges).await {
                            tokio::task::spawn(async move {
                                if let Err(err) = http1::Builder::new()
                                    .serve_connection(stream, service_fn(|_: Request<Incoming>| {
                                        let body = body.clone();
                                        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body)))) }
                                    }))
                                    .await
                                {
                                    error!("Error serving connection: {:?}", err);
                                }
                            });
                            continue;
                        }
                    }

                    if index < table.contexts().len() {
                        let state = &mut table.contexts_mut()[index];
                        if let PortContextKind::Tcp(tcp) = state.kind_mut() {
                            tcp.start_proxy(stream);
                        }
                    }
                }
            }
        };
    }

    Ok(())
}

async fn handle_http_challenge(
    stream: &mut BufStream<TcpStream>,
    challenges: &HashMap<String, String>,
) -> Option<String> {
    const HTTP_CHALLENGE_HEADER: &[u8] = b"GET /.well-known/acme-challenge/";
    if let Ok(buf) = stream.fill_buf().await {
        if buf.starts_with(HTTP_CHALLENGE_HEADER) {
            return buf[HTTP_CHALLENGE_HEADER.len()..]
                .split(|&b| b == b' ')
                .next()
                .and_then(|line| {
                    let key = std::str::from_utf8(line).unwrap_or("");
                    challenges.get(key).cloned()
                });
        }
    }
    None
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
            status: *ctx.status(),
        });
    }
}
