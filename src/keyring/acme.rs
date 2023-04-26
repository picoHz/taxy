use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus,
};
use serde_derive::{Deserialize, Serialize};
use std::{fmt, time::Duration};
use tracing::{error, info};

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

    pub async fn update(&mut self) -> Result<(), instant_acme::Error> {
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
        let mut challenges = Vec::with_capacity(authorizations.len());
        for authz in &authorizations {
            match authz.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                _ => todo!(),
            }

            // We'll use the DNS challenges for this example, but you could
            // pick something else to use here.

            let challenge = authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Dns01)
                .ok_or_else(|| anyhow::anyhow!("no dns01 challenge found"))
                .unwrap();

            let Identifier::Dns(identifier) = &authz.identifier;

            println!("Please set the following DNS record then press any key:");
            println!(
                "_acme-challenge.{} IN TXT {}",
                identifier,
                order.key_authorization(challenge).dns_value()
            );
            // io::stdin().read_line(&mut String::new()).unwrap();

            challenges.push((identifier, &challenge.url));
        }

        for (_, url) in &challenges {
            order.set_challenge_ready(url).await.unwrap();
        }

        let mut tries = 1u8;
        let mut delay = Duration::from_millis(250);
        loop {
            tokio::time::sleep(delay).await;
            let state = order.refresh().await.unwrap();
            if let OrderStatus::Ready | OrderStatus::Invalid = state.status {
                info!("order state: {:#?}", state);
                break;
            }

            delay *= 2;
            tries += 1;
            match tries < 5 {
                true => info!(?state, tries, "order is not ready, waiting {delay:?}"),
                false => {
                    error!(?state, tries, "order is not ready");
                }
            }
        }

        let state = order.state();
        println!("order state: {:#?}", state);

        Ok(())
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
