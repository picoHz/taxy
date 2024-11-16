use super::error::map_response;
use crate::proxy::http::{hyper_tls::client::HttpsConnector, HTTP2_MAX_FRAME_SIZE};
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{header::UPGRADE, Request, Response};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
};
use std::sync::Arc;
use tokio_rustls::rustls::ClientConfig;
use tracing::error;

pub struct ConnectionPool {
    client: Client<HttpsConnector<HttpConnector>, BoxBody<Bytes, anyhow::Error>>,
}

impl ConnectionPool {
    pub fn new(tls_client_config: Arc<ClientConfig>) -> Self {
        let https = HttpsConnector::new(tls_client_config.clone());
        let client = Client::builder(TokioExecutor::new())
            .http2_max_frame_size(Some(HTTP2_MAX_FRAME_SIZE as u32))
            .build(https);
        Self { client }
    }

    pub async fn request(
        &self,
        mut req: Request<BoxBody<Bytes, anyhow::Error>>,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>, anyhow::Error> {
        let upgrading_req = if req.headers().contains_key(UPGRADE) {
            let mut cloned_req = Request::builder().uri(req.uri()).body(BoxBody::<
                Bytes,
                anyhow::Error,
            >::new(
                Full::new(Bytes::new()).map_err(Into::into),
            ))?;
            cloned_req.headers_mut().clone_from(req.headers());
            let mut cloned_req = Some(cloned_req);
            req = cloned_req.replace(req).unwrap();
            cloned_req
        } else {
            None
        };

        *req.version_mut() = hyper::Version::HTTP_11;

        let mut result: Result<_, anyhow::Error> = self
            .client
            .request(req)
            .await
            .map_err(|err| err.into())
            .map(|res| res.map(|body| BoxBody::new(body.map_err(|err| err.into()))));

        match (&result, upgrading_req) {
            (Ok(res), Some(upgrading_req))
                if res.status() == hyper::StatusCode::SWITCHING_PROTOCOLS =>
            {
                let mut cloned_res = Response::builder().status(res.status());
                cloned_res.headers_mut().unwrap().clone_from(res.headers());

                let upgrading_res = std::mem::replace(
                    &mut result,
                    Ok(cloned_res
                        .body(BoxBody::new(Full::new(Bytes::new()).map_err(Into::into)))?),
                )
                .unwrap();
                tokio::spawn(async move {
                    upgrade_connection(upgrading_req, upgrading_res).await;
                });
            }
            _ => (),
        }

        if let Err(err) = &result {
            error!(%err);
        }

        map_response(result)
    }
}

async fn upgrade_connection(
    req: Request<BoxBody<Bytes, anyhow::Error>>,
    res: Response<BoxBody<Bytes, anyhow::Error>>,
) {
    match tokio::try_join!(hyper::upgrade::on(req), hyper::upgrade::on(res)) {
        Ok((req, res)) => {
            let mut req = TokioIo::new(req);
            let mut res = TokioIo::new(res);
            if let Err(err) = tokio::io::copy_bidirectional(&mut req, &mut res).await {
                error!("upgraded io error: {}", err);
            }
        }
        Err(err) => {
            error!("upgrading io error: {}", err);
        }
    };
}
