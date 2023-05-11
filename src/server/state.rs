use super::rpc;
use super::{
    listener::TcpListenerPool,
    rpc::{RpcCallback, RpcCallbackFunc, RpcMethod},
    table::ProxyTable,
};
use crate::proxy::PortStatus;
use crate::{
    command::ServerCommand,
    config::{port::PortEntry, storage::ConfigStorage, AppConfig, Source},
    error::Error,
    event::ServerEvent,
    keyring::{Keyring, KeyringItem},
    proxy::{PortContext, PortContextKind},
};
use hyper::server::conn::Http;
use hyper::{service::service_fn, Body};
use std::{any::Any, convert::Infallible};
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
use warp::http::Response;

pub struct ServerState {
    config: AppConfig,
    storage: ConfigStorage,
    table: ProxyTable,
    pool: TcpListenerPool,
    certs: Keyring,
    http_challenges: HashMap<String, String>,
    command_sender: mpsc::Sender<ServerCommand>,
    br_sender: broadcast::Sender<ServerEvent>,
    callback_sender: mpsc::Sender<RpcCallback>,
    callbacks: HashMap<String, RpcCallbackFunc>,
}

impl ServerState {
    pub async fn new(
        storage: ConfigStorage,
        command_sender: mpsc::Sender<ServerCommand>,
        callback_sender: mpsc::Sender<RpcCallback>,
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
            let span = span!(Level::INFO, "port", resource_id = entry.id);
            match PortContext::new(entry) {
                Ok(mut ctx) => {
                    if let Err(err) = ctx.prepare(&config).instrument(span.clone()).await {
                        span.in_scope(|| {
                            error!(?err, "failed to prepare port");
                        });
                    }
                    if let Err(err) = ctx.setup(&certs).instrument(span.clone()).await {
                        span.in_scope(|| {
                            error!(?err, "failed to setup port");
                        });
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
            callback_sender,
            callbacks: HashMap::new(),
        };

        this.register_callback::<rpc::ports::GetPortList>();
        this.register_callback::<rpc::ports::GetPortStatus>();
        this.register_callback::<rpc::ports::DeletePort>();
        this.register_callback::<rpc::ports::AddPort>();
        this.register_callback::<rpc::ports::UpdatePort>();

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
            ServerCommand::CallMethod { id, method, arg } => {
                if let Some(cb) = self.callbacks.remove(&method) {
                    let result = cb(self, arg);
                    self.callbacks.insert(method, cb);
                    let _ = self.callback_sender.send(RpcCallback { id, result }).await;
                }
            }
            ServerCommand::UpdatePorts => {
                self.update_port_statuses().await;
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
                    if let Err(err) = Http::new()
                        .serve_connection(
                            stream,
                            service_fn(|_| {
                                let body = body.clone();
                                async move { Ok::<_, Infallible>(Response::new(Body::from(body))) }
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
            match state.kind_mut() {
                PortContextKind::Tcp(tcp) => {
                    tcp.start_proxy(stream);
                }
                PortContextKind::Http(http) => {
                    http.start_proxy(stream);
                }
                PortContextKind::Reserved => (),
            }
        }
    }

    fn register_callback<M>(&mut self)
    where
        M: RpcMethod,
    {
        let func = move |this: &mut ServerState,
                         data: Box<dyn Any>|
              -> Result<Box<dyn Any + Send + Sync>, Error> {
            let input = data.downcast::<M>().map_err(|_| Error::RpcError)?;
            match input.call(this) {
                Ok(output) => Ok(Box::new(output) as Box<dyn Any + Send + Sync>),
                Err(err) => Err(err),
            }
        };
        let func = Box::new(func) as RpcCallbackFunc;
        self.callbacks.insert(M::NAME.to_string(), func);
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
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id);
            if let Err(err) = ctx.refresh(&self.certs).instrument(span.clone()).await {
                span.in_scope(|| {
                    error!(?err, "failed to refresh port");
                });
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

    pub fn get_port_list(&self) -> Vec<PortEntry> {
        self.table.entries()
    }

    pub fn get_port_status(&self, id: &str) -> Result<PortStatus, Error> {
        self.table
            .contexts()
            .iter()
            .find(|ctx| ctx.entry.id == id)
            .map(|ctx| *ctx.status())
            .ok_or_else(|| Error::IdNotFound { id: id.to_string() })
    }

    pub fn add_port(&mut self, entry: PortEntry) -> Result<(), Error> {
        if self.get_port_status(&entry.id).is_ok() {
             Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            let _ = self.command_sender.try_send(ServerCommand::SetPort {
                ctx: PortContext::new(entry)?,
            });
            Ok(())
        }
    }

    pub fn update_port(&mut self, entry: PortEntry) -> Result<(), Error> {
        if self.get_port_status(&entry.id).is_ok() {
            let _ = self.command_sender.try_send(ServerCommand::SetPort {
                ctx: PortContext::new(entry)?,
            });
            Ok(())
        } else {
            Err(Error::IdNotFound { id: entry.id })
        }
    }

    pub fn delete_port(&mut self, id: &str) -> Result<(), Error> {
        if self.table.delete_port(id) {
            let _ = self.command_sender.try_send(ServerCommand::UpdatePorts);
            Ok(())
        } else {
            Err(Error::IdNotFound { id: id.to_string() })
        }
    }
}
