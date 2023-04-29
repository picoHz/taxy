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

#[derive(Clone, Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: String,
    pub provider: String,
    pub identifier: String,

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
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl AcmeEntry {
    pub async fn new(
        provider: &str,
        server_url: &str,
        identifier: &str,
    ) -> Result<Self, instant_acme::Error> {
        let account = Account::create(
            &NewAccount {
                contact: &[],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            server_url,
        )
        .await?;
        Ok(Self {
            id: cuid2::create_id(),
            provider: provider.to_string(),
            last_updated: SystemTime::UNIX_EPOCH,
            identifier: identifier.to_string(),
            account,
        })
    }

    pub async fn request(&self) -> anyhow::Result<AcmeRequest> {
        AcmeRequest::new(&self.account, &self.identifier).await
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn info(&self) -> AcmeInfo {
        AcmeInfo {
            id: self.id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AcmeInfo {
    pub id: String,
}

pub struct AcmeRequest {
    pub identifier: String,
    pub http_challenges: HashMap<String, String>,
    pub challenges: Vec<(String, String)>,
    pub order: Order,
}

impl AcmeRequest {
    pub async fn new(account: &Account, name: &str) -> anyhow::Result<Self> {
        let identifier = Identifier::Dns(name.to_string());
        let mut order = account
            .new_order(&NewOrder {
                identifiers: &[identifier],
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
            identifier: name.to_string(),
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

        let mut params = CertificateParams::new(vec![self.identifier.clone()]);
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
