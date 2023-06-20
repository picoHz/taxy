use super::IoStream;
use hyper::{client, Body, Request, Response, StatusCode};
use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn connect(
    req: Request<Body>,
    stream: Box<dyn IoStream>,
) -> anyhow::Result<Response<Body>> {
    let mut client_req = Request::builder().uri(req.uri()).body(Body::empty())?;
    *client_req.headers_mut() = req.headers().clone();

    let (mut sender, conn) = client::conn::Builder::new()
        .handshake::<_, Body>(stream)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed: {:?}", err);
        }
    });

    let mut res = sender.send_request(client_req).await?;
    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Ok(res);
    }

    let mut upgraded_client = match hyper::upgrade::on(&mut res).await {
        Ok(upgraded) => upgraded,
        Err(e) => {
            error!("client upgrade error: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap());
        }
    };

    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(mut upgraded) => {
                if let Err(err) =
                    tokio::io::copy_bidirectional(&mut upgraded_client, &mut upgraded).await
                {
                    error!("upgraded io error: {}", err);
                }
                let _ = upgraded.shutdown().await;
                let _ = upgraded_client.shutdown().await;
            }
            Err(e) => error!("server upgrade error: {}", e),
        }
    });

    Ok(res)
}
