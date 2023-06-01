use super::sites::SiteTable;
use super::{listener::TcpListenerPool, rpc::RpcCallback, table::ProxyTable};
use crate::config::site::SiteEntry;
use crate::keyring::certs::{Cert, CertInfo};
use crate::keyring::KeyringInfo;
use crate::proxy::PortStatus;
use crate::{
    command::ServerCommand,
    config::{port::PortEntry, storage::ConfigStorage, AppConfig, Source},
    error::Error,
    event::ServerEvent,
    keyring::{
        acme::{AcmeEntry, AcmeInfo},
        Keyring, KeyringItem,
    },
    proxy::{PortContext, PortContextKind},
};
use hyper::server::conn::Http;
use hyper::{service::service_fn, Body};
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
use warp::http::Response;
use x509_parser::time::ASN1Time;

pub struct ServerState {
    config: AppConfig,
    storage: ConfigStorage,
    table: ProxyTable,
    sites: SiteTable,
    pool: TcpListenerPool,
    certs: Keyring,
    http_challenges: HashMap<String, String>,
    command_sender: mpsc::Sender<ServerCommand>,
    br_sender: broadcast::Sender<ServerEvent>,
    callback_sender: mpsc::Sender<RpcCallback>,
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
        let table = ProxyTable::new();
        let ports = storage.load_entries().await;
        let sites = storage.load_sites().await;

        let mut this = Self {
            config,
            storage,
            table,
            sites: SiteTable::new(sites),
            pool: TcpListenerPool::new(),
            certs,
            http_challenges: HashMap::new(),
            command_sender,
            br_sender,
            callback_sender,
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

        let _ = this.br_sender.send(ServerEvent::AcmeUpdated {
            items: this.get_acme_list(),
        });
        let _ = this.br_sender.send(ServerEvent::ServerCertsUpdated {
            items: this.get_server_cert_list(),
        });
        let _ = this.br_sender.send(ServerEvent::SitesUpdated {
            items: this.get_site_list(),
        });

        this.update_port_statuses().await;
        this.start_http_challenges().await;
        this
    }

    pub async fn handle_command(&mut self, cmd: ServerCommand) {
        match cmd {
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
                let _ = self.br_sender.send(ServerEvent::AcmeUpdated {
                    items: self.get_acme_list(),
                });
                let _ = self.br_sender.send(ServerEvent::ServerCertsUpdated {
                    items: self.get_server_cert_list(),
                });
                self.start_http_challenges().await;
            }
            ServerCommand::StopHttpChallenges => {
                self.pool.set_http_challenges(false);
                self.http_challenges.clear();
                self.pool.update(self.table.contexts_mut()).await;
            }
            ServerCommand::CallMethod { id, mut arg } => {
                let result = arg.call(self).await;
                let _ = self.callback_sender.send(RpcCallback { id, result }).await;
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
            ServerEvent::PortTableUpdated { entries } => {
                self.storage.save_entries(&entries).await;
            }
            ServerEvent::ServerCertsUpdated { .. } => {
                for ctx in self.table.contexts_mut() {
                    let _ = ctx.refresh(&self.certs).await;
                }
            }
            ServerEvent::SitesUpdated { items } => {
                self.storage.save_sites(&items).await;
                for ctx in self.table.contexts_mut() {
                    let sites = items
                        .iter()
                        .filter(|entry: &&SiteEntry| entry.site.ports.contains(&ctx.entry.id))
                        .cloned()
                        .collect();
                    let span = span!(Level::INFO, "port", resource_id = ctx.entry.id);
                    if let Err(err) = ctx.setup(&self.certs, sites).instrument(span.clone()).await {
                        span.in_scope(|| {
                            error!(?err, "failed to setup port");
                        });
                    }
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

    async fn update_port_statuses(&mut self) {
        self.pool.update(self.table.contexts_mut()).await;
        let _ = self.br_sender.send(ServerEvent::PortTableUpdated {
            entries: self.table.entries().to_vec(),
        });
        for (entry, ctx) in self.table.entries().iter().zip(self.table.contexts()) {
            let _ = self.br_sender.send(ServerEvent::PortStatusUpdated {
                id: entry.id.clone(),
                status: *ctx.status(),
            });
        }
    }

    async fn update_sites(&mut self) {
        let _ = self
            .br_sender
            .send(ServerEvent::SitesUpdated { items: vec![] });
    }

    async fn update_port_ctx(&mut self, mut ctx: PortContext) {
        let sites = self
            .sites
            .entries()
            .into_iter()
            .filter(|entry: &SiteEntry| entry.site.ports.contains(&ctx.entry.id))
            .collect();
        let span = span!(Level::INFO, "port", resource_id = ctx.entry.id);
        if let Err(err) = ctx.setup(&self.certs, sites).instrument(span.clone()).await {
            span.in_scope(|| {
                error!(?err, "failed to setup port");
            });
        }
        self.table.set_port(ctx);
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
        self.remove_expired_certs();
    }

    fn remove_expired_certs(&mut self) {
        let mut removing_items = Vec::new();
        for acme in self.certs.acme_entries() {
            let certs = self.certs.find_server_certs_by_acme(&acme.id);
            let mut expired = certs
                .iter()
                .filter(|cert| cert.not_after < ASN1Time::now())
                .map(|cert| cert.id.clone())
                .collect::<Vec<_>>();
            if expired.len() >= certs.len() {
                expired.pop();
            }
            removing_items.append(&mut expired);
        }
        for id in &removing_items {
            self.certs.delete(id);
        }
        if !removing_items.is_empty() {
            let _ = self.br_sender.send(ServerEvent::ServerCertsUpdated {
                items: self.get_server_cert_list(),
            });
        }
    }

    async fn start_http_challenges(&mut self) -> JoinHandle<()> {
        let entries = self.certs.acme_entries();
        let entries = entries
            .iter()
            .filter(|entry| {
                self.certs
                    .find_server_certs_by_acme(&entry.id)
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
            return tokio::task::spawn(async {});
        }

        let mut requests = Vec::new();
        for entry in entries {
            let span = span!(Level::INFO, "acme", resource_id = entry.id);
            span.in_scope(|| {
                info!(
                    provider = entry.acme.provider,
                    identifiers = ?entry.acme.identifiers,
                    "starting acme request"
                );
            });
            match entry.request().instrument(span.clone()).await {
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

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn set_config(&mut self, config: AppConfig) -> Result<(), Error> {
        self.config = config.clone();
        let _ = self.br_sender.send(ServerEvent::AppConfigUpdated {
            config,
            source: Source::Api,
        });
        Ok(())
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

    pub async fn add_port(&mut self, entry: PortEntry) -> Result<(), Error> {
        if self.get_port_status(&entry.id).is_ok() {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.update_port_ctx(PortContext::new(entry)?).await;
            self.update_port_statuses().await;
            Ok(())
        }
    }

    pub async fn update_port(&mut self, entry: PortEntry) -> Result<(), Error> {
        if self.get_port_status(&entry.id).is_ok() {
            self.update_port_ctx(PortContext::new(entry)?).await;
            self.update_port_statuses().await;
            Ok(())
        } else {
            Err(Error::IdNotFound { id: entry.id })
        }
    }

    pub async fn delete_port(&mut self, id: &str) -> Result<(), Error> {
        if self.table.delete_port(id) {
            self.update_port_statuses().await;
            Ok(())
        } else {
            Err(Error::IdNotFound { id: id.to_string() })
        }
    }

    pub fn reset_port(&mut self, id: &str) -> Result<(), Error> {
        if self.table.reset_port(id) {
            Ok(())
        } else {
            Err(Error::IdNotFound { id: id.to_string() })
        }
    }

    pub fn get_acme_list(&self) -> Vec<AcmeInfo> {
        self.certs
            .list()
            .into_iter()
            .filter_map(|item| match item {
                KeyringInfo::Acme(acme) => Some(acme),
                _ => None,
            })
            .collect()
    }

    pub async fn add_acme(&mut self, entry: AcmeEntry) -> Result<(), Error> {
        if self.certs.iter().any(|item| item.id() == entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            let _ = self
                .command_sender
                .send(ServerCommand::AddKeyringItem {
                    item: KeyringItem::Acme(Arc::new(entry)),
                })
                .await;
            Ok(())
        }
    }

    pub async fn delete_keyring_item(&mut self, id: &str) -> Result<(), Error> {
        if !self.certs.iter().any(|item| item.id() == id) {
            return Err(Error::IdNotFound { id: id.to_string() });
        }

        match self.certs.delete(id) {
            Some(KeyringItem::Acme(_)) => {
                self.storage.delete_acme(id).await;
            }
            Some(KeyringItem::ServerCert(_)) => {
                self.storage.delete_cert(id).await;
            }
            _ => (),
        }
        let _ = self.br_sender.send(ServerEvent::AcmeUpdated {
            items: self.get_acme_list(),
        });
        let _ = self.br_sender.send(ServerEvent::ServerCertsUpdated {
            items: self.get_server_cert_list(),
        });

        Ok(())
    }

    pub fn get_server_cert_list(&self) -> Vec<CertInfo> {
        self.certs
            .list()
            .into_iter()
            .filter_map(|item| match item {
                KeyringInfo::ServerCert(cert) => Some(cert),
                _ => None,
            })
            .collect()
    }

    pub async fn add_server_cert(&mut self, cert: Cert) -> Result<(), Error> {
        if self.certs.iter().any(|item| item.id() == cert.id()) {
            Err(Error::IdAlreadyExists {
                id: cert.id().into(),
            })
        } else {
            let _ = self
                .command_sender
                .send(ServerCommand::AddKeyringItem {
                    item: KeyringItem::ServerCert(Arc::new(cert)),
                })
                .await;
            Ok(())
        }
    }

    pub fn get_site_list(&self) -> Vec<SiteEntry> {
        self.sites.entries()
    }

    pub async fn add_site(&mut self, entry: SiteEntry) -> Result<(), Error> {
        self.sites.add(entry)?;
        self.update_sites().await;
        Ok(())
    }

    pub async fn update_site(&mut self, entry: SiteEntry) -> Result<(), Error> {
        self.sites.update(entry)?;
        self.update_sites().await;
        Ok(())
    }

    pub async fn delete_site(&mut self, id: &str) -> Result<(), Error> {
        self.sites.delete(id)?;
        self.update_sites().await;
        Ok(())
    }
}
