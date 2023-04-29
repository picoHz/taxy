use super::subject_name::SubjectName;
use crate::keyring::certs::Cert;
use anyhow::bail;
use backoff::{backoff::Backoff, ExponentialBackoff};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, SystemTime},
};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub struct AcmeRequest {
    #[schema(example = "Let's Encrypt")]
    pub provider: String,
    #[schema(example = "https://acme-staging-v02.api.letsencrypt.org/directory")]
    pub server_url: String,
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub identifiers: Vec<SubjectName>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: String,
    pub provider: String,
    pub identifiers: Vec<String>,

    #[serde(
        serialize_with = "serialize_system_time",
        deserialize_with = "deserialize_system_time"
    )]
    pub last_updated: SystemTime,

    #[serde(
        serialize_with = "serialize_account",
        deserialize_with = "deserialize_account"
    )]
    pub account: Account,
}

fn serialize_system_time<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(
        time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    )
}

fn deserialize_system_time<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let secs = i64::deserialize(deserializer)?;
    Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64))
}

impl fmt::Debug for AcmeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcmeEntry")
            .field("provider", &self.provider)
            .field("identifiers", &self.identifiers)
            .finish()
    }
}

impl AcmeEntry {
    pub async fn new(req: AcmeRequest) -> Result<Self, instant_acme::Error> {
        let account = Account::create(
            &NewAccount {
                contact: &[],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            &req.server_url,
        )
        .await?;
        Ok(Self {
            id: cuid2::create_id(),
            provider: req.provider,
            last_updated: SystemTime::UNIX_EPOCH,
            identifiers: req.identifiers.into_iter().map(|i| i.to_string()).collect(),
            account,
        })
    }

    pub async fn request(&self) -> anyhow::Result<AcmeOrder> {
        AcmeOrder::new(&self.account, &self.identifiers).await
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn info(&self) -> AcmeInfo {
        AcmeInfo {
            id: self.id.to_string(),
            provider: self.provider.to_string(),
            identifiers: self.identifiers.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AcmeInfo {
    pub id: String,
    pub provider: String,
    pub identifiers: Vec<String>,
}

pub struct AcmeOrder {
    pub identifiers: Vec<Identifier>,
    pub http_challenges: HashMap<String, String>,
    pub challenges: Vec<(String, String)>,
    pub order: Order,
}

impl AcmeOrder {
    pub async fn new(account: &Account, identifiers: &[String]) -> anyhow::Result<Self> {
        let identifiers = identifiers
            .iter()
            .cloned()
            .map(Identifier::Dns)
            .collect::<Vec<_>>();
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
        Ok(Self {
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
