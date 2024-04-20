use super::{
    compression::{is_compressed, CompressionStream},
    error::map_response,
};
use crate::proxy::http::{hyper_tls::client::HttpsConnector, HTTP2_MAX_FRAME_SIZE};
use hyper::{
    client::HttpConnector, header::UPGRADE, http::HeaderValue, Body, Client, Request, Response,
};
use std::sync::Arc;
use tokio_rustls::rustls::ClientConfig;
use tracing::error;

pub struct ConnectionPool {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl ConnectionPool {
    pub fn new(tls_client_config: Arc<ClientConfig>) -> Self {
        let https = HttpsConnector::new(tls_client_config.clone());
        let client = Client::builder()
            .http2_max_frame_size(Some(HTTP2_MAX_FRAME_SIZE as u32))
            .build::<_, hyper::Body>(https);
        Self { client }
    }

    pub async fn request(&self, mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
        let upgrading_req = if req.headers().contains_key(UPGRADE) {
            let mut cloned_req = Request::builder().uri(req.uri()).body(Body::empty())?;
            cloned_req.headers_mut().clone_from(req.headers());
            let mut cloned_req = Some(cloned_req);
            req = cloned_req.replace(req).unwrap();
            cloned_req
        } else {
            None
        };

        let accept_brotli = req
            .headers()
            .get(hyper::header::ACCEPT_ENCODING)
            .map(|value| value.to_str().unwrap_or_default().contains("br"))
            .unwrap_or_default();

        *req.version_mut() = hyper::Version::HTTP_11;

        let mut result: Result<_, anyhow::Error> =
            self.client.request(req).await.map_err(|err| err.into());

        match (&result, upgrading_req) {
            (Ok(res), Some(upgrading_req))
                if res.status() == hyper::StatusCode::SWITCHING_PROTOCOLS =>
            {
                let mut cloned_res = Response::builder().status(res.status());
                cloned_res.headers_mut().unwrap().clone_from(res.headers());
                let upgrading_res =
                    std::mem::replace(&mut result, Ok(cloned_res.body(Body::empty())?)).unwrap();
                tokio::spawn(async move {
                    upgrade_connection(upgrading_req, upgrading_res).await;
                });
            }
            _ => (),
        }

        let http2 = result
            .as_ref()
            .map(|res| res.version() == hyper::Version::HTTP_2)
            .unwrap_or_default();

        let accept_brotli = accept_brotli & http2;

        let result = result.map(|res| {
            let (mut parts, body) = res.into_parts();

            let is_compressed = parts
                .headers
                .get(hyper::header::CONTENT_TYPE)
                .map(|value| is_compressed(value.as_bytes()))
                .unwrap_or_default();

            if !is_compressed {
                let encoding = parts.headers.entry(hyper::header::CONTENT_ENCODING);
                if let hyper::header::Entry::Vacant(entry) = encoding {
                    if accept_brotli {
                        entry.insert(HeaderValue::from_static("br"));
                        parts.headers.remove(hyper::header::CONTENT_LENGTH);
                        let stream = CompressionStream::new(body, HTTP2_MAX_FRAME_SIZE);
                        return Response::from_parts(parts, hyper::Body::wrap_stream(stream));
                    }
                }
            }

            Response::from_parts(parts, body)
        });

        if let Err(err) = &result {
            error!(%err);
        }

        map_response(result)
    }
}

async fn upgrade_connection(req: Request<Body>, res: Response<Body>) {
    match tokio::try_join!(hyper::upgrade::on(req), hyper::upgrade::on(res)) {
        Ok((mut req, mut res)) => {
            if let Err(err) = tokio::io::copy_bidirectional(&mut req, &mut res).await {
                error!("upgraded io error: {}", err);
            }
        }
        Err(err) => {
            error!("upgrading io error: {}", err);
        }
    };
}
