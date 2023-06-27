use core::panic;
use std::sync::Arc;

use futures::{FutureExt, SinkExt, StreamExt};
use taxy::{certs::Cert, server::Server};
use taxy_api::{
    cert::SelfSignedCertRequest,
    port::{Port, PortEntry, PortOptions},
    site::{Route, Site, SiteEntry},
    tls::TlsTermination,
};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{client::ClientConfig, RootCertStore};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
use url::Url;
use warp::Filter;

mod storage;
use storage::TestStorage;

#[tokio::test]
async fn wss_proxy() {
    let cert = Arc::new(
        Cert::new_self_signed(&SelfSignedCertRequest {
            san: vec!["localhost".parse().unwrap()],
        })
        .unwrap(),
    );

    let routes = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("websocket error: {:?}", e);
                }
            })
        })
    });

    let listener = TcpListener::bind("127.0.0.1:55000").await.unwrap();
    tokio::spawn(warp::serve(routes).run_incoming(TcpListenerStream::new(listener)));

    let (server, channels) = Server::new(
        TestStorage::builder()
            .ports(vec![PortEntry {
                id: "test".into(),
                port: Port {
                    listen: "/ip4/127.0.0.1/tcp/55001/https".parse().unwrap(),
                    opts: PortOptions {
                        tls_termination: Some(TlsTermination {
                            server_names: vec!["localhost".into()],
                        }),
                        ..Default::default()
                    },
                },
            }])
            .sites(vec![SiteEntry {
                id: "test2".into(),
                site: Site {
                    ports: vec!["test".into()],
                    vhosts: vec![],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::site::Server {
                            url: "http://127.0.0.1:55000/".parse().unwrap(),
                        }],
                    }],
                },
            }])
            .certs([(cert.id.clone(), cert.clone())].into_iter().collect())
            .build(),
    )
    .await;
    let task = tokio::spawn(server.start());

    let mut root_certs = RootCertStore::empty();
    for cert in cert.certified().unwrap().cert {
        root_certs.add(&cert).unwrap();
    }

    let client = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();

    let url = Url::parse("wss://localhost:55001/ws").unwrap();
    let (mut ws_stream, _) =
        connect_async_tls_with_config(url, None, false, Some(Connector::Rustls(Arc::new(client))))
            .await
            .unwrap();
    ws_stream
        .send(Message::Text("Hello, server!".to_string()))
        .await
        .unwrap();

    let message = ws_stream.next().await.unwrap();
    match message {
        Ok(msg) => assert_eq!("Hello, server!", msg.into_text().unwrap()),
        Err(e) => panic!("{e}"),
    }

    channels.shutdown();
    task.await.unwrap().unwrap();
}
