use crate::certs::store::CertStore;
use crate::certs::SubjectName;
use crate::{config, error::Error};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::error;

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

    pub async fn setup(&mut self, certs: &CertStore) -> Result<(), Error> {
        let server_names = self.server_names.clone();

        let cert = certs
            .find(&server_names)
            .ok_or(Error::ValidTlsCertificatesNotFound)?;

        let chain = cert.chain.clone();
        let key = cert.key.clone();

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(chain, key);

        let server_config = match server_config {
            Ok(config) => config,
            Err(err) => {
                error!(?err, server_names = ?self.server_names);
                return Err(Error::TlsServerConfigrationFailed);
            }
        };

        let server_config = Arc::new(server_config);
        self.acceptor = Some(TlsAcceptor::from(server_config));

        Ok(())
    }
}
