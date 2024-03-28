use crate::{certs::Cert, server::cert_list::CertList};
use anyhow::bail;
use backoff::{backoff::Backoff, ExponentialBackoffBuilder};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, ExternalAccountKey,
    Identifier, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    sync::Arc,
    time::{Duration, SystemTime},
};
use taxy_api::{
    acme::Acme,
    cert::{CertKind, CertMetadata},
    id::ShortId,
};
use taxy_api::{acme::AcmeInfo, subject_name::SubjectName};
use taxy_api::{acme::AcmeRequest, error::Error};
use tracing::{error, info};

const HTTP_CHALLENGE_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: ShortId,
    #[serde(flatten)]
    pub acme: Acme,
    pub account: Arc<AccountCredentials>,
}

impl fmt::Debug for AcmeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcmeEntry")
            .field("config", &self.acme.config)
            .field("identifiers", &self.acme.identifiers)
            .finish()
    }
}

impl AcmeEntry {
    pub async fn new(id: ShortId, req: AcmeRequest) -> Result<Self, Error> {
        let contact = req.contacts.iter().map(|c| c.as_str()).collect::<Vec<_>>();
        let external_account = req
            .eab
            .map(|eab| ExternalAccountKey::new(eab.key_id, &eab.hmac_key));
        let account = Account::create(
            &NewAccount {
                contact: &contact,
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            &req.server_url,
            external_account.as_ref(),
        )
        .await;

        let (_, account) = match account {
            Ok(account) => account,
            Err(e) => {
                error!("failed to create account: {}", e);
                return Err(Error::AcmeAccountCreationFailed);
            }
        };

        Ok(Self {
            id,
            acme: req.acme,
            account: Arc::new(account),
        })
    }

    pub async fn request(&self) -> anyhow::Result<AcmeOrder> {
        AcmeOrder::new(self).await
    }

    pub fn id(&self) -> ShortId {
        self.id
    }

    pub fn info(&self, certs: &CertList) -> AcmeInfo {
        AcmeInfo {
            id: self.id,
            config: self.acme.config.clone(),
            identifiers: self
                .acme
                .identifiers
                .iter()
                .map(|id| id.to_string())
                .collect(),
            challenge_type: self.acme.challenge_type.clone(),
            next_renewal: self
                .next_renewal(certs)
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|t| t.as_secs() as i64),
        }
    }

    pub fn last_issued(&self, certs: &CertList) -> Option<SystemTime> {
        certs
            .find_certs_by_acme(self.id)
            .iter()
            .map(|cert| {
                cert.metadata
                    .as_ref()
                    .map(|meta| meta.created_at)
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            })
            .max()
    }

    pub fn next_renewal(&self, certs: &CertList) -> Option<SystemTime> {
        let last_issued = self.last_issued(certs)?;
        let renewal_days = self.acme.config.renewal_days;
        let next_renewal = last_issued + Duration::from_secs(60 * 60 * 24 * renewal_days);
        Some(next_renewal)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeAccount {
    #[serde(flatten)]
    pub acme: Acme,
    pub account: Arc<AccountCredentials>,
}

impl From<AcmeEntry> for (ShortId, AcmeAccount) {
    fn from(entry: AcmeEntry) -> Self {
        (
            entry.id,
            AcmeAccount {
                acme: entry.acme,
                account: entry.account,
            },
        )
    }
}

impl From<(ShortId, AcmeAccount)> for AcmeEntry {
    fn from((id, entry): (ShortId, AcmeAccount)) -> Self {
        Self {
            id,
            acme: entry.acme,
            account: entry.account,
        }
    }
}

pub struct AcmeOrder {
    pub id: ShortId,
    pub challenge_type: ChallengeType,
    pub identifiers: Vec<Identifier>,
    pub http_challenges: HashMap<String, String>,
    pub challenges: Vec<(String, String)>,
    pub order: Order,
}

impl AcmeOrder {
    pub async fn new(entry: &AcmeEntry) -> anyhow::Result<Self> {
        info!("requesting certificate");

        let identifiers = entry
            .acme
            .identifiers
            .iter()
            .filter_map(|id| match id {
                SubjectName::DnsName(domain) => Some(Identifier::Dns(domain.to_string())),
                _ => None,
            })
            .collect::<Vec<_>>();
        let account: AccountCredentials =
            serde_json::from_str(&serde_json::to_string(&entry.account)?)?;
        let account = Account::from_credentials(account).await?;
        let mut order = account
            .new_order(&NewOrder {
                identifiers: &identifiers,
            })
            .await?;
        let authorizations = order.authorizations().await?;

        let mut http_challenges = HashMap::new();
        let mut challenges = Vec::new();

        for authz in &authorizations {
            match authz.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                _ => bail!("authorization status is not valid"),
            }

            let challenge = authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Http01)
                .ok_or_else(|| anyhow::anyhow!("no http01 challenge found"))?;

            let Identifier::Dns(identifier) = &authz.identifier;

            http_challenges.insert(
                challenge.token.to_string(),
                order.key_authorization(challenge).as_str().to_string(),
            );
            challenges.push((identifier.to_string(), challenge.url.to_string()));
        }
        let challenge_type = match entry.acme.challenge_type.as_str() {
            "http-01" => ChallengeType::Http01,
            _ => bail!("challenge type is not supported"),
        };
        Ok(Self {
            id: entry.id,
            challenge_type,
            identifiers,
            http_challenges,
            challenges,
            order,
        })
    }

    pub async fn start_challenge(&mut self) -> anyhow::Result<Cert> {
        for (_, url) in &self.challenges {
            self.order.set_challenge_ready(url).await?;
        }

        let mut backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(HTTP_CHALLENGE_TIMEOUT))
            .build();
        loop {
            let state = self.order.refresh().await?;
            match state.status {
                OrderStatus::Ready => break,
                OrderStatus::Invalid => {
                    bail!("order is invalid");
                }
                _ => (),
            }
            if let Some(next) = backoff.next_backoff() {
                tokio::time::sleep(next).await;
            } else {
                bail!("order is timed-out");
            }
        }

        let san = self
            .identifiers
            .iter()
            .map(|id| {
                let Identifier::Dns(domain) = id;
                domain.clone()
            })
            .collect::<Vec<_>>();

        let mut params = CertificateParams::new(san)?;
        params.distinguished_name = DistinguishedName::new();

        let keypair = KeyPair::generate()?;
        let request = params.serialize_request(&keypair)?;
        let csr = request.der();

        self.order.finalize(csr).await?;
        let cert_chain_pem = loop {
            match self.order.certificate().await? {
                Some(cert_chain_pem) => break cert_chain_pem,
                None => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        };

        let metadata = CertMetadata {
            acme_id: self.id,
            created_at: SystemTime::now(),
        };
        let metadata = serde_qs::to_string(&metadata).unwrap_or_default();
        let cert_chain_pem = format!("# {}\r\n\r\n{}", metadata, cert_chain_pem);

        let cert = Cert::new(
            CertKind::Server,
            cert_chain_pem.into_bytes(),
            Some(keypair.serialize_pem().into_bytes()),
        );

        Ok(cert?)
    }
}
