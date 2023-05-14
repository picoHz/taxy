use super::IoStream;
use hyper::{client, upgrade::Upgraded, Body, Request, Response, StatusCode};
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

    let upgraded_client = match hyper::upgrade::on(&mut res).await {
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
            Ok(upgraded) => {
                if let Err(err) = serve_upgraded_io(upgraded, upgraded_client).await {
                    error!("upgraded io error: {}", err);
                }
            }
            Err(e) => error!("server upgrade error: {}", e),
        }
    });

    Ok(res)
}

async fn serve_upgraded_io(
    mut server: Upgraded,
    mut client: Upgraded,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::io::copy_bidirectional(&mut server, &mut client).await?;
    Ok(())
}
