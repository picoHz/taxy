use hyper::StatusCode;
use sailfish::TemplateOnce;
use thiserror::Error;
use tokio_rustls::rustls;

#[derive(Debug, Clone, Error)]
pub enum ProxyError {
    #[error("domain fronting detected")]
    DomainFrontingDetected,

    #[error("no route found")]
    NoRouteFound,
}

impl ProxyError {
    fn code(&self) -> StatusCode {
        match self {
            Self::DomainFrontingDetected => StatusCode::MISDIRECTED_REQUEST,
            Self::NoRouteFound => StatusCode::BAD_GATEWAY,
        }
    }
}

pub fn map_error(err: anyhow::Error) -> StatusCode {
    if let Some(err) = err.downcast_ref::<ProxyError>() {
        return err.code();
    }
    if let Some(err) = err.downcast_ref::<rustls::Error>() {
        if matches!(err, rustls::Error::InvalidCertificate(_)) {
            return StatusCode::from_u16(526).unwrap();
        } else {
            return StatusCode::from_u16(525).unwrap();
        }
    }
    if let Ok(err) = err.downcast::<hyper::Error>() {
        if err.is_timeout() {
            return StatusCode::GATEWAY_TIMEOUT;
        } else {
            return StatusCode::from_u16(523).unwrap();
        }
    }
    StatusCode::BAD_GATEWAY
}

#[derive(TemplateOnce)]
#[template(path = "error.stpl")]
pub struct ErrorTemplate {
    #[allow(unused_variables)]
    pub code: u16,
}
