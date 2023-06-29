use super::{acme, app_info, auth, certs, config, log, ports, sites};
use taxy_api::acme::AcmeInfo;
use taxy_api::acme::{AcmeRequest, ExternalAccountBinding};
use taxy_api::app::{AppConfig, AppInfo};
use taxy_api::auth::LoginRequest;
use taxy_api::cert::{CertInfo, CertKind, CertMetadata, CertPostBody, SelfSignedCertRequest};
use taxy_api::error::{Error, ErrorMessage};
use taxy_api::event::ServerEvent;
use taxy_api::log::SystemLogRow;
use taxy_api::port::{NetworkAddr, NetworkInterface, PortEntry, PortOptions, UpstreamServer};
use taxy_api::port::{PortState, PortStatus, SocketState};
use taxy_api::site::{Route, Server, SiteEntry};
use taxy_api::tls::TlsState;
use taxy_api::tls::TlsTermination;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use warp::filters::BoxedFilter;
use warp::Filter;
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
        certs::list,
        certs::get,
        certs::delete,
        certs::self_sign,
        certs::upload,
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
        CertKind,
        CertInfo,
        CertMetadata,
        AcmeInfo,
        SelfSignedCertRequest,
        AcmeRequest,
        ExternalAccountBinding,
        CertPostBody,
        Error,
        ErrorMessage,
        ServerEvent,
        SiteEntry,
        Route,
        Server,
        LoginRequest,
        SystemLogRow,
        NetworkInterface,
        NetworkAddr
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub fn swagger_ui() -> BoxedFilter<(impl Reply,)> {
    warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and_then(serve_swagger)
        .boxed()
}

#[cfg(not(feature = "utoipa-swagger-ui"))]
async fn serve_swagger(
    _full_path: warp::path::FullPath,
    _tail: warp::path::Tail,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    Err(warp::reject::not_found())
}

#[cfg(feature = "utoipa-swagger-ui")]
async fn serve_swagger(
    full_path: warp::path::FullPath,
    tail: warp::path::Tail,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    use hyper::{Response, StatusCode, Uri};
    use once_cell::sync::OnceCell;
    use std::sync::Arc;

    static CONFIG: OnceCell<Arc<utoipa_swagger_ui::Config<'static>>> = OnceCell::new();
    let config =
        CONFIG.get_or_init(|| Arc::new(utoipa_swagger_ui::Config::from("/api/api-doc.json")));

    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/swagger-ui/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config.clone()) {
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
            "cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("token"))),
        )
    }
}
