use crate::keyring::certs::Cert;
use anyhow::bail;
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, time::Duration};
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: String,
    pub provider: String,
    pub identifier: String,
    #[serde(
        serialize_with = "serialize_account",
        deserialize_with = "deserialize_account"
    )]
    pub account: Account,
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
            identifier: identifier.to_string(),
            account,
        })
    }

    pub async fn request_challenge(&self) -> Result<AcmeRequest, instant_acme::Error> {
        let identifier = Identifier::Dns(self.identifier.clone());
        let mut order = self
            .account
            .new_order(&NewOrder {
                identifiers: &[identifier],
            })
            .await
            .unwrap();
        let state = order.state();
        println!("order state: {:#?}", state);

        let authorizations = order.authorizations().await.unwrap();

        let mut http_challenges = HashMap::new();
        let mut challenges = Vec::new();

        for authz in &authorizations {
            match authz.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                _ => todo!(),
            }

            let challenge = authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Http01)
                .ok_or_else(|| anyhow::anyhow!("no http01 challenge found"))
                .unwrap();

            let Identifier::Dns(identifier) = &authz.identifier;

            http_challenges.insert(
                challenge.token.to_string(),
                order.key_authorization(challenge).as_str().to_string(),
            );
            challenges.push((identifier.to_string(), challenge.url.to_string()));
        }
        Ok(AcmeRequest {
            identifier: self.identifier.clone(),
            http_challenges,
            challenges,
            order,
        })
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
    pub async fn start_challenge(&mut self) -> anyhow::Result<Cert> {
        for (_, url) in &self.challenges {
            self.order.set_challenge_ready(url).await?;
        }

        let mut tries = 1u8;
        let mut delay = Duration::from_millis(250);
        let state = loop {
            tokio::time::sleep(delay).await;
            let state = self.order.refresh().await?;
            if let OrderStatus::Ready | OrderStatus::Invalid = state.status {
                info!("order state: {:#?}", state);
                break state;
            }

            delay *= 2;
            tries += 1;
            match tries < 50 {
                true => info!("order is not ready, waiting {delay:?} {state:?} {tries}"),
                false => {
                    bail!("order is not ready {state:?} {tries}");
                }
            }
        };

        println!("order state: {:#?}", state);
        if state.status == OrderStatus::Invalid {
            bail!("order is invalid");
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
