use std::fmt;

use instant_acme::{Account, AccountCredentials, NewAccount};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AcmeEntry {
    pub id: String,
    pub provider: String,
    pub identifiers: Vec<String>,
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
            .field("identifiers", &self.identifiers)
            .finish()
    }
}

impl AcmeEntry {
    pub async fn new(provider: &str, server_url: &str) -> Result<Self, instant_acme::Error> {
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
            identifiers: Vec::new(),
            account,
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
