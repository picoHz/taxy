use super::app_info::AppInfo;
use super::{app_info, certs, certs::CertPostBody, config, port};
use crate::config::port::{BackendServer, PortEntry, PortOptions};
use crate::config::tls::TlsTermination;
use crate::config::{AppConfig, Source};
use crate::error::Error;
use crate::event::ServerEvent;
use crate::keyring::acme::AcmeRequest;
use crate::keyring::certs::{CertInfo, SelfSignedCertRequest};
use crate::keyring::KeyringInfo;
use crate::proxy::tls::TlsState;
use crate::proxy::{PortState, PortStatus, SocketState};
use hyper::{Response, StatusCode, Uri};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::{Rejection, Reply};

#[derive(OpenApi)]
#[openapi(
    paths(
        port::list,
        port::status,
        port::delete,
        port::post,
        port::put,
        config::get,
        config::put,
        certs::list,
        certs::delete,
        certs::self_signed,
        certs::upload,
        certs::acme,
        app_info::get,
    ),
    components(schemas(
        AppInfo,
        AppConfig,
        PortEntry,
        PortOptions,
        BackendServer,
        TlsTermination,
        PortStatus,
        PortState,
        SocketState,
        TlsState,
        KeyringInfo,
        CertInfo,
        SelfSignedCertRequest,
        AcmeRequest,
        CertPostBody,
        Error,
        ServerEvent,
        Source,
    ))
)]
pub struct ApiDoc;

pub async fn serve_swagger(
    full_path: warp::path::FullPath,
    tail: warp::path::Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/swagger-ui/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}
