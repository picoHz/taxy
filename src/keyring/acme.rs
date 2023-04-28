use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

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

        let mut request = AcmeRequest::default();
        let authorizations = order.authorizations().await.unwrap();

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

            request.http_challenges.insert(
                challenge.token.to_string(),
                order.key_authorization(challenge).as_str().to_string(),
            );
            request
                .challenges
                .push((identifier.to_string(), challenge.url.to_string()));
        }
        Ok(request)
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

#[derive(Debug, Default)]
pub struct AcmeRequest {
    pub http_challenges: HashMap<String, String>,
    pub challenges: Vec<(String, String)>,
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
