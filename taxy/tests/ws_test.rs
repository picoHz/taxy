use futures::{FutureExt, SinkExt, StreamExt};
use taxy_api::{
    port::{Port, PortEntry},
    site::{HttpProxy, Proxy, ProxyEntry, ProxyKind, Route},
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn ws_proxy() -> anyhow::Result<()> {
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

    let listener = TcpListener::bind("127.0.0.1:54000").await.unwrap();
    tokio::spawn(warp::serve(routes).run_incoming(TcpListenerStream::new(listener)));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                name: String::new(),
                listen: "/ip4/127.0.0.1/tcp/54001/http".parse().unwrap(),
                opts: Default::default(),
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".into(),
            proxy: Proxy {
                name: String::new(),
                ports: vec!["test".into()],
                kind: ProxyKind::Http(HttpProxy {
                    vhosts: vec!["localhost:54001".parse().unwrap()],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::site::Server {
                            url: "http://127.0.0.1:54000/".parse().unwrap(),
                        }],
                    }],
                }),
            },
        }])
        .build();

    with_server(config, |_| async move {
        let url = Url::parse("ws://localhost:54001/ws")?;
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
