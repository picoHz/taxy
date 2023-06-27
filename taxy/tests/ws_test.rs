use core::panic;
use futures::{FutureExt, SinkExt, StreamExt};
use taxy::server::Server;
use taxy_api::{
    port::{Port, PortEntry, Protocol},
    site::{Route, Site, SiteEntry},
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use warp::Filter;

mod storage;
use storage::TestStorage;

#[tokio::test]
async fn ws_proxy() {
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

    let (server, channels) = Server::new(
        TestStorage::builder()
            .ports(vec![PortEntry {
                id: "test".into(),
                port: Port {
                    protocol: Protocol::Http,
                    bind: vec!["127.0.0.1:54001".parse().unwrap()],
                    opts: Default::default(),
                },
            }])
            .sites(vec![SiteEntry {
                id: "test2".into(),
                site: Site {
                    ports: vec!["test".into()],
                    vhosts: vec!["localhost:54001".parse().unwrap()],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::site::Server {
                            url: "http://127.0.0.1:54000/".parse().unwrap(),
                        }],
                    }],
                },
            }])
            .build(),
    )
    .await;
    let task = tokio::spawn(server.start());

    let url = Url::parse("ws://localhost:54001/ws").unwrap();
    let (mut ws_stream, _) = connect_async(url).await.unwrap();
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
