use super::acme_list::AcmeList;
use super::cert_list::CertList;
use super::proxy_list::ProxyList;
use super::{listener::TcpListenerPool, port_list::PortList, rpc::RpcCallback};
use crate::certs::acme::AcmeOrder;
use crate::config::storage::Storage;
use crate::log::DatabaseLayer;
use crate::{
    command::ServerCommand,
    proxy::{PortContext, PortContextKind},
};
use hyper::server::conn::Http;
use hyper::{service::service_fn, Body};
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::convert::Infallible;
use std::str;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use taxy_api::app::{AppConfig, AppInfo};
use taxy_api::error::Error;
use taxy_api::event::ServerEvent;
use taxy_api::id::ShortId;
use taxy_api::site::ProxyEntry;
use tokio::io::AsyncBufReadExt;
use tokio::{
    io::BufStream,
    net::TcpStream,
    sync::{broadcast, mpsc},
};
use tracing::{error, info, span, Instrument, Level};
use warp::http::Response;
use x509_parser::time::ASN1Time;

pub struct ServerState {
    pub proxies: ProxyList,
    pub certs: CertList,
    pub acmes: AcmeList,
    pub ports: PortList,
    pub storage: Box<dyn Storage>,
    config: AppConfig,
    pool: TcpListenerPool,
    http_challenges: HashMap<String, String>,
    command_sender: mpsc::Sender<ServerCommand>,
    br_sender: broadcast::Sender<ServerEvent>,
    callback_sender: mpsc::Sender<RpcCallback>,
    broadcast_events: bool,
}

impl ServerState {
    pub async fn new(
        storage: impl Storage,
        command_sender: mpsc::Sender<ServerCommand>,
        callback_sender: mpsc::Sender<RpcCallback>,
        br_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        let config = storage.load_app_config().await;
        let _ = br_sender.send(ServerEvent::AppConfigUpdated {
            config: config.clone(),
        });

        let certs = storage.load_certs().await;
        let acmes = storage.load_acmes().await;
        let ports = storage.load_ports().await;
        let proxies = storage.load_proxies().await;

        let mut this = Self {
            proxies: proxies.into_iter().collect(),
            certs: CertList::new(certs).await,
            acmes: acmes.into_iter().collect(),
            ports: PortList::default(),
            storage: Box::new(storage),
            config,
            pool: TcpListenerPool::new(),
            http_challenges: HashMap::new(),
            command_sender,
            br_sender,
            callback_sender,
            broadcast_events: false,
        };

        for entry in ports {
            match PortContext::new(entry) {
                Ok(ctx) => {
                    this.update_port_ctx(ctx).await;
                }
                Err(err) => {
                    error!(?err, "failed to create proxy state");
                }
            };
        }

        this.update_port_statuses().await;
        this.update_certs().await;
        this.update_proxies().await;
        this.update_acmes().await;
        this
    }

    pub async fn handle_command(&mut self, cmd: ServerCommand) {
        match cmd {
            ServerCommand::AddCert { cert } => {
                self.certs.add(cert.clone());
                self.update_certs().await;
                self.storage.save_cert(&cert).await;
            }
            ServerCommand::SetBroadcastEvents { enabled } => {
                self.broadcast_events = enabled;
            }
            ServerCommand::SetHttpChallenges { orders } => {
                if orders.is_empty() {
                    self.stop_http_challenges().await;
                } else {
                    self.continue_http_challenges(orders).await;
                }
            }
            ServerCommand::CallMethod { id, mut arg } => {
                let result = arg.call(self).await;
                let _ = self.callback_sender.send(RpcCallback { id, result }).await;
            }
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

        if index < self.ports.as_slice().len() {
            let state = &mut self.ports.as_mut_slice()[index];
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

    pub async fn update_port_statuses(&mut self) {
        let entries = self.ports.entries().cloned().collect::<Vec<_>>();
        self.pool.update(self.ports.as_mut_slice()).await;
        self.storage.save_ports(&entries).await;
        if self.proxies.remove_incompatible_ports(&entries) {
            self.update_proxies().await;
        }
        let _ = self
            .br_sender
            .send(ServerEvent::PortTableUpdated { entries });
        if self.broadcast_events {
            for (entry, ctx) in self.ports.entries().cloned().zip(self.ports.as_slice()) {
                let _ = self.br_sender.send(ServerEvent::PortStatusUpdated {
                    id: entry.id,
                    status: *ctx.status(),
                });
            }
        }
    }

    pub async fn update_proxies(&mut self) {
        let entries = self.proxies.entries().cloned().collect::<Vec<_>>();
        self.storage.save_proxies(&entries).await;
        let _ = self.br_sender.send(ServerEvent::ProxiesUpdated {
            entries: entries.clone(),
        });
        for ctx in self.ports.as_mut_slice() {
            let proxies = entries
                .iter()
                .filter(|entry: &&ProxyEntry| entry.proxy.ports.contains(&ctx.entry.id))
                .cloned()
                .collect();
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
            if let Err(err) = ctx
                .setup(&self.certs, proxies)
                .instrument(span.clone())
                .await
            {
                span.in_scope(|| {
                    error!(?err, "failed to setup port");
                });
            }
        }
    }

    pub async fn update_certs(&mut self) {
        let _ = self.br_sender.send(ServerEvent::CertsUpdated {
            entries: self.certs.iter().map(|item| item.info()).collect(),
        });
        for ctx in self.ports.as_mut_slice() {
            let proxies = self
                .proxies
                .entries()
                .filter(|entry: &&ProxyEntry| entry.proxy.ports.contains(&ctx.entry.id))
                .cloned()
                .collect();
            let _ = ctx.setup(&self.certs, proxies).await;
        }
    }

    pub async fn update_acmes(&mut self) {
        let _ = self.br_sender.send(ServerEvent::AcmeUpdated {
            entries: self.acmes.entries().map(|acme| acme.info()).collect(),
        });
        self.start_http_challenges().await;
    }

    pub async fn update_port_ctx(&mut self, mut ctx: PortContext) -> bool {
        let proxies = self
            .proxies
            .entries()
            .filter(|entry: &&ProxyEntry| entry.proxy.ports.contains(&ctx.entry.id))
            .cloned()
            .collect();
        let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
        if let Err(err) = ctx
            .setup(&self.certs, proxies)
            .instrument(span.clone())
            .await
        {
            span.in_scope(|| {
                error!(?err, "failed to setup port");
            });
        }
        self.ports.update(ctx)
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

    pub async fn run_background_tasks(&mut self, app_info: &AppInfo) {
        if let Err(err) = self.cleanup_old_logs(app_info).await {
            error!(?err, "failed to cleanup old logs");
        }

        self.start_http_challenges().await;
        for ctx in self.ports.as_mut_slice() {
            let proxies = self
                .proxies
                .entries()
                .filter(|entry: &&ProxyEntry| entry.proxy.ports.contains(&ctx.entry.id))
                .cloned()
                .collect();
            let span = span!(Level::INFO, "port", resource_id = ctx.entry.id.to_string());
            if let Err(err) = ctx
                .setup(&self.certs, proxies)
                .instrument(span.clone())
                .await
            {
                span.in_scope(|| {
                    error!(?err, "failed to refresh port");
                });
            }
        }
        self.remove_expired_certs();
    }

    fn remove_expired_certs(&mut self) {
        let mut removing_items = Vec::new();
        for acme in self.acmes.entries() {
            let certs = self.certs.find_certs_by_acme(acme.id);
            let mut expired = certs
                .iter()
                .filter(|cert| cert.not_after < ASN1Time::now())
                .map(|cert| cert.id)
                .collect::<Vec<_>>();
            if expired.len() >= certs.len() {
                expired.pop();
            }
            removing_items.append(&mut expired);
        }
        for id in &removing_items {
            if let Err(err) = self.certs.delete(*id) {
                error!(?err, "failed to delete cert");
            }
        }
        if !removing_items.is_empty() {
            let _ = self.br_sender.send(ServerEvent::CertsUpdated {
                entries: self.certs.iter().map(|item| item.info()).collect(),
            });
        }
    }

    async fn cleanup_old_logs(&mut self, app_info: &AppInfo) -> anyhow::Result<()> {
        let path = app_info.log_path.join("log.db");
        let database = DatabaseLayer::new(&path, tracing::level_filters::LevelFilter::OFF).await?;
        database
            .cleanup(self.config.log.database_log_retention)
            .await?;
        Ok(())
    }

    async fn start_http_challenges(&mut self) {
        let entries = self.acmes.entries().cloned();
        let entries = entries
            .filter(|entry| {
                self.certs
                    .find_certs_by_acme(entry.id)
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
                    > Duration::from_secs(60 * 60 * 24 * entry.acme.renewal_days)
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return;
        }

        let command = self.command_sender.clone();
        tokio::task::spawn(async move {
            let mut orders = Vec::new();
            for entry in entries {
                let span = span!(Level::INFO, "acme", resource_id = entry.id.to_string());
                span.in_scope(|| {
                    info!(
                        provider = entry.acme.provider,
                        identifiers = ?entry.acme.identifiers,
                        "starting acme request"
                    );
                });
                match entry.request().instrument(span.clone()).await {
                    Ok(request) => orders.push(request),
                    Err(err) => {
                        let _enter = span.enter();
                        error!("failed to request challenge: {}", err)
                    }
                }
            }
            let _ = command
                .send(ServerCommand::SetHttpChallenges { orders })
                .await;
        });
    }

    async fn stop_http_challenges(&mut self) {
        self.http_challenges.clear();
        self.pool.set_http_challenge_addr(None);
        self.pool.update(self.ports.as_mut_slice()).await;
    }

    async fn continue_http_challenges(&mut self, orders: Vec<AcmeOrder>) {
        let challenges = orders
            .iter()
            .flat_map(|req| req.http_challenges.clone())
            .collect();

        self.http_challenges = challenges;
        self.pool
            .set_http_challenge_addr(Some(self.config.http_challenge_addr));
        self.pool.update(self.ports.as_mut_slice()).await;

        let command = self.command_sender.clone();
        tokio::task::spawn(async move {
            for mut order in orders {
                let span = span!(Level::INFO, "acme", resource_id = order.id.to_string());
                match order.start_challenge().instrument(span.clone()).await {
                    Ok(cert) => {
                        span.in_scope(|| {
                            info!(id = cert.id().to_string(), "acme request completed");
                        });
                        let _ = command
                            .send(ServerCommand::AddCert {
                                cert: Arc::new(cert),
                            })
                            .await;
                    }
                    Err(err) => {
                        let _enter = span.enter();
                        error!(?err, "failed to start challenge");
                    }
                }
            }
            let _ = command
                .send(ServerCommand::SetHttpChallenges { orders: vec![] })
                .await;
        });
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn set_config(&mut self, config: AppConfig) -> Result<(), Error> {
        self.config.clone_from(&config);
        let _ = self
            .br_sender
            .send(ServerEvent::AppConfigUpdated { config });
        Ok(())
    }

    pub fn generate_id(&self) -> ShortId {
        const TABLE: &[u8] = b"bcdfghjklmnpqrstvwxyz";

        let used_ids = self
            .acmes
            .entries()
            .map(|acme| acme.id)
            .chain(self.ports.entries().map(|port| port.id))
            .chain(self.proxies.entries().map(|site| site.id))
            .collect::<HashSet<_>>();

        let mut rng = rand::thread_rng();
        let mut id = [b'a'; 6];
        loop {
            for c in &mut id {
                *c = *TABLE.choose(&mut rng).unwrap();
            }
            let id = format!(
                "{}-{}",
                str::from_utf8(&id[..3]).unwrap(),
                str::from_utf8(&id[3..]).unwrap()
            )
            .parse()
            .unwrap();
            if !used_ids.contains(&id) {
                return id;
            }
        }
    }
}
