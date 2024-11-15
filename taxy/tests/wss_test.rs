use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use core::panic;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use taxy::certs::Cert;
use taxy_api::{
    port::{Port, PortEntry, PortOptions},
    proxy::{HttpProxy, Proxy, ProxyEntry, ProxyKind, Route},
    tls::TlsTermination,
};
use tokio_rustls::rustls::{client::ClientConfig, RootCertStore};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
use url::Url;

mod common;
use common::{alloc_tcp_port, with_server, TestStorage};

#[tokio::test]
async fn wss_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_tcp_port().await?;
    let proxy_port = alloc_tcp_port().await?;

    let root = Arc::new(Cert::new_ca().unwrap());
    let cert = Arc::new(Cert::new_self_signed(&["localhost".parse().unwrap()], &root).unwrap());

    let config = RustlsConfig::from_pem(
        cert.pem_chain.to_vec(),
        cert.pem_key.as_ref().unwrap().to_vec(),
    )
    .await
    .unwrap();
    let app = Router::new().route("/ws", any(ws_handler));
    let addr = listen_port.socket_addr();
    tokio::spawn(axum_server::bind_rustls(addr, config).serve(app.into_make_service()));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_https(),
                opts: PortOptions {
                    tls_termination: Some(TlsTermination {
                        server_names: vec!["localhost".into()],
                    }),
                },
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap()],
                kind: ProxyKind::Http(HttpProxy {
                    vhosts: vec!["localhost".parse().unwrap()],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::proxy::Server {
                            url: listen_port.https_url("/").try_into().unwrap(),
                        }],
                    }],
                    upgrade_insecure: false,
                }),
                ..Default::default()
            },
        }])
        .certs(
            [(root.id, root.clone()), (cert.id, cert.clone())]
                .into_iter()
                .collect(),
        )
        .build();

    let mut root_certs = RootCertStore::empty();
    for cert in root.certified_key().unwrap().cert {
        root_certs.add(cert).unwrap();
    }

    let client = ClientConfig::builder()
        .with_root_certificates(root_certs)
        .with_no_client_auth();

    with_server(config, |_| async move {
        let url = Url::parse(&format!(
            "wss://localhost:{}/ws",
            proxy_port.socket_addr().port()
        ))?;
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

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            socket.send(msg).await.unwrap();
        } else {
            return;
        }
    }
}
