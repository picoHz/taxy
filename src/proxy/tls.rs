use crate::keyring::certs::Cert;
use crate::keyring::subject_name::SubjectName;
use crate::keyring::Keyring;
use crate::{config, error::Error};
use serde_derive::Serialize;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tokio_rustls::rustls::server::{ClientHello, ResolvesServerCert};
use tokio_rustls::rustls::sign::CertifiedKey;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TlsState {
    Active,
}

pub struct TlsTermination {
    pub server_names: Vec<SubjectName>,
    pub acceptor: Option<TlsAcceptor>,
}

impl fmt::Debug for TlsTermination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsTermination")
            .field("server_names", &self.server_names)
            .finish()
    }
}

impl TlsTermination {
    pub fn new(config: &config::tls::TlsTermination) -> Result<Self, Error> {
        let mut server_names = Vec::new();
        for name in &config.server_names {
            let name = SubjectName::from_str(name)?;
            server_names.push(name);
        }
        Ok(Self {
            server_names,
            acceptor: None,
        })
    }

    pub async fn setup(&mut self, keyring: &Keyring) -> TlsState {
        let resolver: Arc<dyn ResolvesServerCert> = Arc::new(ServerCertResolver::new(
            keyring.certs(),
            self.server_names.clone(),
            true,
        ));

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        let server_config = Arc::new(server_config);
        self.acceptor = Some(TlsAcceptor::from(server_config));

        TlsState::Active
    }

    pub async fn refresh(&mut self, certs: &Keyring) -> TlsState {
        self.setup(certs).await
    }
}

pub struct ServerCertResolver {
    certs: Vec<Arc<Cert>>,
    default_names: Vec<SubjectName>,
    sni: bool,
}

impl ServerCertResolver {
    pub fn new(certs: Vec<Arc<Cert>>, default_names: Vec<SubjectName>, sni: bool) -> Self {
        Self {
            certs,
            default_names,
            sni,
        }
    }
}

impl ResolvesServerCert for ServerCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let sni = client_hello
            .server_name()
            .filter(|_| self.sni)
            .map(|sni| SubjectName::DnsName(sni.into()))
            .into_iter()
            .collect::<Vec<_>>();

        let names = if sni.is_empty() {
            &self.default_names
        } else {
            &sni
        };

        let cert = self
            .certs
            .iter()
            .find(|cert| cert.is_valid() && names.iter().all(|name| cert.has_subject_name(name)))?;

        Some(cert.certified.clone())
    }
}
