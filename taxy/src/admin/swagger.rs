use super::{acme, app_info, auth, config, log, ports, server_certs, sites};
use hyper::{Response, StatusCode, Uri};
use std::sync::Arc;
use taxy_api::acme::AcmeInfo;
use taxy_api::acme::{AcmeRequest, ExternalAccountBinding};
use taxy_api::app::{AppConfig, AppInfo, Source};
use taxy_api::auth::{LoginRequest, LoginResult};
use taxy_api::cert::{CertInfo, CertMetadata, CertPostBody, SelfSignedCertRequest};
use taxy_api::error::Error;
use taxy_api::event::ServerEvent;
use taxy_api::log::SystemLogRow;
use taxy_api::port::{PortEntry, PortOptions, UpstreamServer};
use taxy_api::port::{PortState, PortStatus, SocketState};
use taxy_api::site::{Route, Server, SiteEntry};
use taxy_api::tls::TlsState;
use taxy_api::tls::TlsTermination;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::Config;
use warp::{Rejection, Reply};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::login,
        auth::logout,
        ports::list,
        ports::get,
        ports::status,
        ports::delete,
        ports::post,
        ports::put,
        ports::reset,
        config::get,
        config::put,
        app_info::get,
        acme::list,
        acme::get,
        acme::delete,
        acme::add,
        sites::list,
        sites::get,
        sites::delete,
        sites::post,
        sites::put,
        log::get,
        server_certs::list,
        server_certs::get,
        server_certs::delete,
        server_certs::self_sign,
        server_certs::upload,
    ),
    components(schemas(
        AppInfo,
        AppConfig,
        PortEntry,
        PortOptions,
        UpstreamServer,
        TlsTermination,
        PortStatus,
        PortState,
        SocketState,
        TlsState,
        CertInfo,
        CertMetadata,
        AcmeInfo,
        SelfSignedCertRequest,
        AcmeRequest,
        ExternalAccountBinding,
        CertPostBody,
        Error,
        ServerEvent,
        Source,
        SiteEntry,
        Route,
        Server,
        LoginRequest,
        LoginResult,
        SystemLogRow
    )),
    modifiers(&SecurityAddon)
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

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "authorization",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("authorization"))),
        )
    }
}
