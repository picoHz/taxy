use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use futures::{SinkExt, StreamExt};
use taxy_api::{
    port::{Port, PortEntry},
    proxy::{HttpProxy, Proxy, ProxyEntry, ProxyKind, Route},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

mod common;
use common::{alloc_tcp_port, with_server, TestStorage};

#[tokio::test]
async fn ws_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_tcp_port().await?;
    let proxy_port = alloc_tcp_port().await?;

    let app = Router::new().route("/ws", any(ws_handler));
    let addr = listen_port.socket_addr();
    tokio::spawn(axum_server::bind(addr).serve(app.into_make_service()));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_http(),
                opts: Default::default(),
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
                            url: listen_port.http_url("/").try_into().unwrap(),
                        }],
                    }],
                    upgrade_insecure: false,
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let url = Url::parse(&format!(
            "ws://localhost:{}/ws",
            proxy_port.socket_addr().port()
        ))?;
        let (mut ws_stream, _) = connect_async(url).await?;
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
