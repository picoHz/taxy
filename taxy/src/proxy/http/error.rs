use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Body, Response, StatusCode};
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

pub fn map_response<B>(
    res: Result<Response<B>, anyhow::Error>,
) -> Result<Response<BoxBody<Bytes, anyhow::Error>>, anyhow::Error>
where
    B: Body<Data = Bytes, Error = anyhow::Error> + Send + Sync + 'static,
{
    match res {
        Ok(res) => Ok(res.map(|body| BoxBody::new(body))),
        Err(err) => {
            let code = map_error(err);
            let ctx = ErrorTemplate {
                code: code.as_u16(),
            };
            let mut res = Response::new(BoxBody::new(
                Full::new(Bytes::from(ctx.render_once().unwrap())).map_err(Into::into),
            ));
            *res.status_mut() = code;
            Ok(res)
        }
    }
}

fn map_error(err: anyhow::Error) -> StatusCode {
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
struct ErrorTemplate {
    #[allow(unused_variables)]
    code: u16,
}
