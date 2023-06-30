use crate::certs::Cert;
use crate::server::cert_list::CertList;
use dashmap::DashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use taxy_api::cert::CertKind;
use taxy_api::error::Error;
use taxy_api::subject_name::SubjectName;
use taxy_api::tls::TlsState;
use tokio_rustls::rustls::server::{ClientHello, ResolvesServerCert};
use tokio_rustls::rustls::sign::CertifiedKey;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::error;

pub struct TlsTermination {
    pub server_names: Vec<SubjectName>,
    pub acceptor: Option<TlsAcceptor>,
    pub alpn_protocols: Vec<Vec<u8>>,
}

impl fmt::Debug for TlsTermination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsTermination")
            .field("server_names", &self.server_names)
            .finish()
    }
}

impl TlsTermination {
    pub fn new(
        config: &taxy_api::tls::TlsTermination,
        alpn_protocols: Vec<Vec<u8>>,
    ) -> Result<Self, Error> {
        let mut server_names = Vec::new();
        for name in &config.server_names {
            let name = SubjectName::from_str(name)?;
            server_names.push(name);
        }
        Ok(Self {
            server_names,
            acceptor: None,
            alpn_protocols,
        })
    }

    pub async fn setup(&mut self, certs: &CertList) -> TlsState {
        let resolver: Arc<dyn ResolvesServerCert> = Arc::new(CertResolver::new(
            certs
                .iter()
                .filter(|cert| cert.kind == CertKind::Server)
                .cloned()
                .collect(),
            self.server_names.clone(),
            true,
        ));

        let mut server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_cert_resolver(resolver);
        server_config.alpn_protocols = self.alpn_protocols.clone();

        let server_config = Arc::new(server_config);
        self.acceptor = Some(TlsAcceptor::from(server_config));

        TlsState::Active
    }
}

pub struct CertResolver {
    certs: Vec<Arc<Cert>>,
    default_names: Vec<SubjectName>,
    sni: bool,
    cache: DashMap<String, Arc<CertifiedKey>>,
}

impl CertResolver {
    pub fn new(certs: Vec<Arc<Cert>>, default_names: Vec<SubjectName>, sni: bool) -> Self {
        Self {
            certs,
            default_names,
            sni,
            cache: DashMap::new(),
        }
    }
}

impl ResolvesServerCert for CertResolver {
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

        if let Some(cert) = self.cache.get(cert.id()) {
            Some(cert.clone())
        } else {
            let certified = match cert.certified() {
                Ok(certified) => Arc::new(certified),
                Err(err) => {
                    error!("failed to load certified key: {}", err);
                    return None;
                }
            };
            self.cache.insert(cert.id().to_string(), certified.clone());
            Some(certified)
        }
    }
}
