use taxy::server::Server;
use taxy_api::port::{Port, PortEntry, PortOptions, UpstreamServer};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod storage;
use storage::TestStorage;

#[tokio::test]
async fn tcp_proxy() {
    let listener = TcpListener::bind("127.0.0.1:50000").await.unwrap();
    let hello = warp::path!("hello").map(|| format!("Hello"));
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let (server, channels) = Server::new(
        TestStorage::builder()
            .ports(vec![PortEntry {
                id: "aaa".into(),
                port: Port {
                    listen: "/ip4/127.0.0.1/tcp/50001".parse().unwrap(),
                    opts: PortOptions {
                        upstream_servers: vec![UpstreamServer {
                            addr: "/ip4/127.0.0.1/tcp/50000".parse().unwrap(),
                        }],
                        ..Default::default()
                    },
                },
            }])
            .build(),
    );

    let task = tokio::spawn(server.start());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let resp = reqwest::get("http://127.0.0.1:50001/hello")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(resp, "Hello");

    channels.shutdown();
    task.await.unwrap().unwrap();
}
