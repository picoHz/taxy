use hyper::{Body, Response, StatusCode};
use sailfish::TemplateOnce;
use thiserror::Error;
use tokio_rustls::rustls;

#[derive(Debug, Clone, Error)]
pub enum ProxyError {
    #[error("invalid hostname")]
    InvalidHostName,

    #[error("dns lookup failed")]
    DnsLookupFailed,
}

impl ProxyError {
    fn code(&self) -> StatusCode {
        match self {
            Self::InvalidHostName => StatusCode::BAD_GATEWAY,
            Self::DnsLookupFailed => StatusCode::from_u16(523).unwrap(),
        }
    }
}

pub fn map_response(
    res: Result<Response<Body>, anyhow::Error>,
) -> Result<Response<Body>, anyhow::Error> {
    match res {
        Ok(res) => Ok(res),
        Err(err) => {
            let code = map_error(err);
            let ctx = ErrorTemplate {
                code: code.as_u16(),
            };
            let mut res = Response::new(Body::from(ctx.render_once().unwrap()));
            *res.status_mut() = code;
            Ok(res)
        }
    }
}

fn map_error(err: anyhow::Error) -> StatusCode {
    if let Some(err) = err.downcast_ref::<ProxyError>() {
        return err.code();
    }
    if let Ok(err) = err.downcast::<std::io::Error>() {
        if err.kind() == std::io::ErrorKind::TimedOut {
            return StatusCode::GATEWAY_TIMEOUT;
        }
        if let Some(inner) = err.into_inner() {
            if let Some(err) = inner.downcast_ref::<rustls::Error>() {
                if matches!(*err, rustls::Error::InvalidCertificate(_)) {
                    return StatusCode::from_u16(526).unwrap();
                } else {
                    return StatusCode::from_u16(525).unwrap();
                }
            }
        }
    }
    StatusCode::BAD_GATEWAY
}

#[derive(TemplateOnce)]
#[template(path = "error.stpl")]
struct ErrorTemplate {
    #[allow(unused_variables)]
    code: u16,
}
