use crate::keyring::certs::Cert;
use anyhow::bail;
use backoff::{backoff::Backoff, ExponentialBackoff};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, ExternalAccountKey,
    Identifier, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, SystemTime},
};
use taxy_api::{acme::Acme, cert::CertMetadata};
use taxy_api::{acme::AcmeInfo, subject_name::SubjectName};
use taxy_api::{acme::AcmeRequest, error::Error};
use tracing::{error, info};

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: String,
    #[serde(flatten)]
    pub acme: Acme,
    #[serde(
        serialize_with = "serialize_account",
        deserialize_with = "deserialize_account"
    )]
    pub account: Account,
}

impl fmt::Debug for AcmeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcmeEntry")
            .field("provider", &self.acme.provider)
            .field("identifiers", &self.acme.identifiers)
            .finish()
    }
}

impl AcmeEntry {
    pub async fn new(id: String, req: AcmeRequest) -> Result<Self, Error> {
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

        let account = match account {
            Ok(account) => account,
            Err(e) => {
                error!("failed to create account: {}", e);
                return Err(Error::AcmeAccountCreationFailed);
            }
        };

        Ok(Self {
            id,
            acme: req.acme,
            account,
        })
    }

    pub async fn request(&self) -> anyhow::Result<AcmeOrder> {
        AcmeOrder::new(self).await
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn info(&self) -> AcmeInfo {
        AcmeInfo {
            id: self.id.to_string(),
            provider: self.acme.provider.to_string(),
            identifiers: self
                .acme
                .identifiers
                .iter()
                .map(|id| id.to_string())
                .collect(),
            challenge_type: self.acme.challenge_type.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeAccount {
    #[serde(flatten)]
    pub acme: Acme,
    #[serde(
        serialize_with = "serialize_account",
        deserialize_with = "deserialize_account"
    )]
    pub account: Account,
}

impl From<AcmeEntry> for (String, AcmeAccount) {
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

impl From<(String, AcmeAccount)> for AcmeEntry {
    fn from((id, entry): (String, AcmeAccount)) -> Self {
        Self {
            id,
            acme: entry.acme,
            account: entry.account,
        }
    }
}

pub struct AcmeOrder {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub identifiers: Vec<Identifier>,
    pub http_challenges: HashMap<String, String>,
    pub challenges: Vec<(String, String)>,
    pub order: Order,
    pub is_trusted: bool,
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
        let mut order = entry
            .account
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
            id: entry.id.clone(),
            challenge_type,
            identifiers,
            http_challenges,
            challenges,
            order,
            is_trusted: entry.acme.is_trusted,
        })
    }

    pub async fn start_challenge(&mut self) -> anyhow::Result<Cert> {
        for (_, url) in &self.challenges {
            self.order.set_challenge_ready(url).await?;
        }

        let mut backoff = ExponentialBackoff::default();
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

        let mut params = CertificateParams::new(san);
        params.distinguished_name = DistinguishedName::new();
        let cert = Certificate::from_params(params)?;
        let csr = cert.serialize_request_der()?;

        self.order.finalize(&csr).await?;
        let cert_chain_pem = loop {
            match self.order.certificate().await? {
                Some(cert_chain_pem) => break cert_chain_pem,
                None => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        };

        let metadata = CertMetadata {
            acme_id: self.id.clone(),
            created_at: SystemTime::now(),
            is_trusted: self.is_trusted,
        };
        let metadata = serde_qs::to_string(&metadata).unwrap_or_default();
        let cert_chain_pem = format!("# {}\r\n\r\n{}", metadata, cert_chain_pem);

        let cert = Cert::new(
            cert_chain_pem.into_bytes(),
            cert.serialize_private_key_pem().into_bytes(),
        );

        Ok(cert?)
    }
}

fn serialize_account<S>(account: &Account, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::Serialize;
    account.credentials().serialize(serializer)
}

fn deserialize_account<'de, D>(deserializer: D) -> Result<Account, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let creds = AccountCredentials::deserialize(deserializer)?;
    Account::from_credentials(creds).map_err(serde::de::Error::custom)
}
