use crate::keyring::subject_name::SubjectName;
use crate::keyring::Keyring;
use crate::{config, error::Error};
use serde_derive::Serialize;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TlsState {
    Active,
    NoValidCertificate,
    ConfigurationFailed,
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
        let server_names = self.server_names.clone();

        let cert = if let Some(cert) = keyring.find_server_cert(&server_names) {
            cert
        } else {
            return TlsState::NoValidCertificate;
        };

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
                return TlsState::ConfigurationFailed;
            }
        };

        let server_config = Arc::new(server_config);
        self.acceptor = Some(TlsAcceptor::from(server_config));

        TlsState::Active
    }

    pub async fn refresh(&mut self, certs: &Keyring) -> TlsState {
        self.setup(certs).await
    }
}
