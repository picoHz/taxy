use core::panic;
use futures::{FutureExt, SinkExt, StreamExt};
use std::{net::ToSocketAddrs, sync::Arc};
use taxy::certs::Cert;
use taxy_api::{
    port::{Port, PortEntry, PortOptions},
    site::{Route, Site, SiteEntry},
    tls::TlsTermination,
};
use tokio_rustls::rustls::{client::ClientConfig, RootCertStore};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
use url::Url;
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn wss_proxy() -> anyhow::Result<()> {
    let root = Arc::new(Cert::new_ca().unwrap());
    let cert = Arc::new(Cert::new_self_signed(&["localhost".parse().unwrap()], &root).unwrap());

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

    let addr = "localhost:55000".to_socket_addrs().unwrap().next().unwrap();
    let (_, server) = warp::serve(routes)
        .tls()
        .cert(&cert.pem_chain)
        .key(&cert.pem_key)
        .bind_ephemeral(addr);
    tokio::spawn(server);

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                name: String::new(),
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
                name: String::new(),
                ports: vec!["test".into()],
                vhosts: vec![],
                routes: vec![Route {
                    path: "/".into(),
                    servers: vec![taxy_api::site::Server {
                        url: "https://localhost:55000/".parse().unwrap(),
                    }],
                }],
            },
        }])
        .certs(
            [
                (root.id.clone(), root.clone()),
                (cert.id.clone(), cert.clone()),
            ]
            .into_iter()
            .collect(),
        )
        .build();

    let mut root_certs = RootCertStore::empty();
    for cert in root.certified().unwrap().cert {
        root_certs.add(&cert).unwrap();
    }

    let client = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();

    with_server(config, |_| async move {
        let url = Url::parse("wss://localhost:55001/ws")?;
        let (mut ws_stream, _) = connect_async_tls_with_config(
            url,
            None,
            false,
            Some(Connector::Rustls(Arc::new(client))),
        )
        .await?;
        ws_stream
            .send(Message::Text("Hello, server!".to_string()))
            .await?;

        let message = ws_stream.next().await.unwrap();
        match message {
            Ok(msg) => assert_eq!("Hello, server!", msg.into_text()?),
            Err(e) => panic!("{e}"),
        }
        Ok(())
    })
    .await
}
