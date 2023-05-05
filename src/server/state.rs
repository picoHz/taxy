use super::{listener::TcpListenerPool, table::ProxyTable};
use crate::{
    command::ServerCommand,
    config::{storage::ConfigStorage, AppConfig, Source},
    event::ServerEvent,
    keyring::{Keyring, KeyringItem},
    proxy::{PortContext, PortContextKind},
};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::convert::Infallible;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{io::AsyncBufReadExt, task::JoinHandle};
use tokio::{
    io::BufStream,
    net::TcpStream,
    sync::{broadcast, mpsc},
};
use tracing::{error, info, span, Instrument, Level};
use warp::http::{Request, Response};

pub struct ServerState {
    config: AppConfig,
    storage: ConfigStorage,
    table: ProxyTable,
    pool: TcpListenerPool,
    certs: Keyring,
    http_challenges: HashMap<String, String>,
    command_sender: mpsc::Sender<ServerCommand>,
    br_sender: broadcast::Sender<ServerEvent>,
}

impl ServerState {
    pub async fn new(
        storage: ConfigStorage,
        command_sender: mpsc::Sender<ServerCommand>,
        br_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        let config = storage.load_app_config().await;
        let _ = br_sender.send(ServerEvent::AppConfigUpdated {
            config: config.clone(),
            source: Source::File,
        });

        let certs = storage.load_keychain().await;
        let _ = br_sender.send(ServerEvent::KeyringUpdated {
            items: certs.list(),
        });

        let mut table = ProxyTable::new();
        let ports = storage.load_entries().await;
        for entry in ports {
            match PortContext::new(entry) {
                Ok(mut ctx) => {
                    if let Err(err) = ctx.prepare(&config).await {
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

        let mut this = Self {
            config,
            storage,
            table,
            pool: TcpListenerPool::new(),
            certs,
            http_challenges: HashMap::new(),
            command_sender,
            br_sender,
        };

        this.update_port_statuses().await;
        this.start_http_challenges().await;
        this
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn handle_command(&mut self, cmd: ServerCommand) {
        match cmd {
            ServerCommand::SetAppConfig { config } => {
                self.config = config.clone();
                let _ = self.br_sender.send(ServerEvent::AppConfigUpdated {
                    config,
                    source: Source::Api,
                });
            }
            ServerCommand::SetPort { mut ctx } => {
                if let Err(err) = ctx.setup(&self.certs).await {
                    error!(?err, "failed to setup port");
                }
                self.table.set_port(ctx);
                self.update_port_statuses().await;
            }
            ServerCommand::DeletePort { id } => {
                self.table.delete_port(&id);
                self.update_port_statuses().await;
            }
            ServerCommand::AddKeyringItem { item } => {
                match &item {
                    KeyringItem::Acme(entry) => {
                        self.storage.save_acme(entry).await;
                    }
                    KeyringItem::ServerCert(cert) => {
                        self.storage.save_cert(cert).await;
                    }
                }
                self.certs.add(item);
                let _ = self.br_sender.send(ServerEvent::KeyringUpdated {
                    items: self.certs.list(),
                });
                self.start_http_challenges().await;
            }
            ServerCommand::DeleteKeyringItem { id } => {
                match self.certs.delete(&id) {
                    Some(KeyringItem::Acme(_)) => {
                        self.storage.delete_acme(&id).await;
                    }
                    Some(KeyringItem::ServerCert(_)) => {
                        self.storage.delete_cert(&id).await;
                    }
                    _ => (),
                }
                let _ = self.br_sender.send(ServerEvent::KeyringUpdated {
                    items: self.certs.list(),
                });
            }
            ServerCommand::StopHttpChallenges => {
                self.pool.set_http_challenges(false);
                self.http_challenges.clear();
                self.pool.update(self.table.contexts_mut()).await;
            }
        }
    }

    pub async fn handle_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::AppConfigUpdated {
                config: app_config,
                source,
            } => {
                if source != Source::File {
                    self.storage.save_app_config(&app_config).await;
                }
            }
            ServerEvent::PortTableUpdated { entries, source } => {
                if source != Source::File {
                    self.storage.save_entries(&entries).await;
                }
            }
            _ => (),
        }
    }

    pub fn has_active_listeners(&self) -> bool {
        self.pool.has_active_listeners()
    }

    pub async fn select(&mut self) -> Option<(usize, TcpStream)> {
        self.pool.select().await
    }

    pub async fn handle_connection(&mut self, index: usize, stream: TcpStream) {
        let mut stream = BufStream::new(stream);

        if !self.http_challenges.is_empty() {
            if let Some(body) = self.handle_http_challenge(&mut stream).await {
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(
                            stream,
                            service_fn(|_: Request<Incoming>| {
                                let body = body.clone();
                                async move {
                                    Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
                                }
                            }),
                        )
                        .await
                    {
                        error!("Error serving connection: {:?}", err);
                    }
                });
                return;
            }
        }

        if index < self.table.contexts().len() {
            let state = &mut self.table.contexts_mut()[index];
            if let PortContextKind::Tcp(tcp) = state.kind_mut() {
                tcp.start_proxy(stream);
            }
        }
    }

    async fn update_port_statuses(&mut self) {
        self.pool.update(self.table.contexts_mut()).await;
        let _ = self.br_sender.send(ServerEvent::PortTableUpdated {
            entries: self.table.entries().to_vec(),
            source: Source::Api,
        });
        for (entry, ctx) in self.table.entries().iter().zip(self.table.contexts()) {
            let _ = self.br_sender.send(ServerEvent::PortStatusUpdated {
                id: entry.id.clone(),
                status: *ctx.status(),
            });
        }
    }

    async fn handle_http_challenge(&mut self, stream: &mut BufStream<TcpStream>) -> Option<String> {
        const HTTP_CHALLENGE_HEADER: &[u8] = b"GET /.well-known/acme-challenge/";
        if let Ok(buf) = stream.fill_buf().await {
            if buf.starts_with(HTTP_CHALLENGE_HEADER) {
                return buf[HTTP_CHALLENGE_HEADER.len()..]
                    .split(|&b| b == b' ')
                    .next()
                    .and_then(|line| {
                        let key = std::str::from_utf8(line).unwrap_or("");
                        self.http_challenges.get(key).cloned()
                    });
            }
        }
        None
    }

    pub async fn run_background_tasks(&mut self) {
        let _ = self.start_http_challenges().await.await;
        for ctx in self.table.contexts_mut() {
            if let Err(err) = ctx.refresh(&self.certs).await {
                error!(?err, "failed to refresh port");
            }
        }
    }

    async fn start_http_challenges(&mut self) -> JoinHandle<()> {
        let entries = self
            .certs
            .iter()
            .filter_map(|item| match item {
                KeyringItem::Acme(entry) => Some(entry.clone()),
                _ => None,
            })
            .filter(|entry| {
                self.certs
                    .find_server_cert_by_acme(&entry.id)
                    .iter()
                    .map(|cert| {
                        cert.metadata
                            .as_ref()
                            .map(|meta| meta.created_at)
                            .unwrap_or(SystemTime::UNIX_EPOCH)
                    })
                    .max()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .elapsed()
                    .unwrap_or_default()
                    > Duration::from_secs(60 * 60 * 24 * entry.renewal_days)
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return tokio::task::spawn(async {});
        }

        let mut requests = Vec::new();
        for acme in entries {
            let span = span!(Level::INFO, "acme", resource_id = acme.id);
            span.in_scope(|| {
                info!(
                    provider = acme.provider,
                    identifiers = ?acme.identifiers,
                    "starting acme request"
                );
            });
            match acme.request().instrument(span.clone()).await {
                Ok(request) => requests.push(request),
                Err(err) => {
                    let _enter = span.enter();
                    error!("failed to request challenge: {}", err)
                }
            }
        }
        let challenges = requests
            .iter()
            .flat_map(|req| req.http_challenges.clone())
            .collect();

        self.http_challenges = challenges;
        self.pool.set_http_challenges(true);
        self.pool.update(self.table.contexts_mut()).await;

        let command = self.command_sender.clone();
        tokio::task::spawn(async move {
            for mut req in requests {
                let span = span!(Level::INFO, "acme", resource_id = req.id);
                match req.start_challenge().instrument(span.clone()).await {
                    Ok(cert) => {
                        span.in_scope(|| {
                            info!(id = cert.id(), "acme request completed");
                        });
                        let _ = command
                            .send(ServerCommand::AddKeyringItem {
                                item: KeyringItem::ServerCert(Arc::new(cert)),
                            })
                            .await;
                    }
                    Err(err) => {
                        let _enter = span.enter();
                        error!(?err, "failed to start challenge");
                    }
                }
            }
            let _ = command.send(ServerCommand::StopHttpChallenges).await;
        })
    }
}
