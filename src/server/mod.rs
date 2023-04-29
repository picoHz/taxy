use crate::config::storage::ConfigStorage;
use crate::config::Source;
use crate::keyring::acme::AcmeEntry;
use crate::keyring::{Keyring, KeyringItem};
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
use std::sync::Arc;
use std::time::SystemTime;
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
    command_send: mpsc::Sender<ServerCommand>,
    mut command_recv: mpsc::Receiver<ServerCommand>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let mut table = ProxyTable::new();
    let mut pool = TcpListenerPool::new();
    let mut event_recv = event.subscribe();

    let mut http_challenges = HashMap::<String, String>::new();

    let app_config = config.load_app_config().await;
    let _ = event.send(ServerEvent::AppConfigUpdated {
        config: app_config.clone(),
        source: Source::File,
    });

    let mut certs = config.load_keychain().await;
    let _ = event.send(ServerEvent::KeyringUpdated {
        items: certs.list(),
    });

    /*
    command_send
        .send(ServerCommand::AddKeyringItem {
            item: KeyringItem::Acme(Arc::new(
                AcmeEntry::new(
                    "Let's Encrypt",
                    "https://acme-staging-v02.api.letsencrypt.org/directory",
                    "d142-115-39-175-81.ngrok-free.app",
                )
                .await
                .unwrap(),
            )),
        })
        .await
        .unwrap();
    */

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

    start_http_challenges(
        command_send,
        &mut pool,
        &mut table,
        &certs,
        &mut http_challenges,
    )
    .await;

    loop {
        tokio::select! {
            cmd = command_recv.recv() => {
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
                        match &item {
                            KeyringItem::Acme (entry) => {
                                config.save_acme(entry).await;
                            }
                            KeyringItem::ServerCert (cert) => {
                                config.save_cert(cert).await;
                            }
                        }
                        certs.add(item);
                        let _ = event.send(ServerEvent::KeyringUpdated { items: certs.list() } );
                    }
                    Some(ServerCommand::DeleteKeyringItem { id }) => {
                        config.delete_cert(&id).await;
                        certs.delete(&id);
                        let _ = event.send(ServerEvent::KeyringUpdated { items: certs.list() } );
                    }
                    Some(ServerCommand::StopHttpChallenges) => {
                        pool.set_http_challenges(false);
                        http_challenges.clear();
                        pool.update(table.contexts_mut()).await;
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

async fn start_http_challenges(
    command: mpsc::Sender<ServerCommand>,
    pool: &mut TcpListenerPool,
    table: &mut ProxyTable,
    certs: &Keyring,
    http_challenges: &mut HashMap<String, String>,
) {
    let entries = certs
        .iter()
        .filter_map(|item| match item {
            KeyringItem::Acme(entry) => Some(entry.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut requests = Vec::new();
    for acme in entries {
        match acme.request().await {
            Ok(request) => requests.push((request, acme)),
            Err(err) => error!("failed to request challenge: {}", err),
        }
    }
    let challenges = requests
        .iter()
        .map(
            |(req, _): &(crate::keyring::acme::AcmeRequest, Arc<AcmeEntry>)| {
                req.http_challenges.clone()
            },
        )
        .flatten()
        .collect();

    println!("challenges: {:?}", challenges);

    *http_challenges = challenges;
    pool.set_http_challenges(true);
    pool.update(table.contexts_mut()).await;

    tokio::task::spawn(async move {
        for (mut req, mut entry) in requests {
            match req.start_challenge().await {
                Ok(cert) => {
                    println!("cert: {:?}", cert);
                    let _ = command
                        .send(ServerCommand::AddKeyringItem {
                            item: KeyringItem::ServerCert(Arc::new(cert)),
                        })
                        .await;
                    Arc::make_mut(&mut entry).last_updated = SystemTime::now();
                }
                Err(err) => {
                    error!(?err, "failed to start challenge");
                }
            }
        }
        let _ = command.send(ServerCommand::StopHttpChallenges).await;
    });
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
