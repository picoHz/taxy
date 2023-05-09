use super::RpcMethod;
use crate::{error::Error, server::state::ServerState, keyring::{KeyringInfo, KeyringItem}};

pub struct GetKeyringItemList;

impl RpcMethod for GetKeyringItemList {
    const NAME: &'static str = "get_keyring_item_list";
    type Output = Vec<KeyringInfo>;

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_keyring_item_list())
    }
}

pub struct AddKeyringItem {
    pub item: KeyringItem,
}

impl RpcMethod for AddKeyringItem {
    const NAME: &'static str = "add_keyring_item";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_keyring_item(self.item)
    }
}

pub struct DeleteKeyringItem {
    pub id: String,
}

impl RpcMethod for DeleteKeyringItem {
    const NAME: &'static str = "delete_keyring_item";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_keyring_item(&self.id)
    }
}
