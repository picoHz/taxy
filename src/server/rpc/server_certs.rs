use super::RpcMethod;
use crate::{
    error::Error,
    keyring::certs::{Cert, CertInfo},
    server::state::ServerState,
};

pub struct GetServerCertList;

impl RpcMethod for GetServerCertList {
    const NAME: &'static str = "get_server_cert_list";
    type Output = Vec<CertInfo>;

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_server_cert_list())
    }
}

pub struct AddServerCert {
    pub cert: Cert,
}

impl RpcMethod for AddServerCert {
    const NAME: &'static str = "add_server_cert";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_server_cert(self.cert)
    }
}

pub struct DeleteServerCert {
    pub id: String,
}

impl RpcMethod for DeleteServerCert {
    const NAME: &'static str = "delete_server_cert";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_server_cert(&self.id)
    }
}
