use taxy_api::port::{Port, PortEntry, PortOptions, UpstreamServer};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn tcp_proxy() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50000").await.unwrap();
    let hello = warp::path!("hello").map(|| "Hello".to_string());
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                name: String::new(),
                listen: "/ip4/127.0.0.1/tcp/50001".parse().unwrap(),
                opts: PortOptions {
                    upstream_servers: vec![UpstreamServer {
                        addr: "/ip4/127.0.0.1/tcp/50000".parse().unwrap(),
                    }],
                    ..Default::default()
                },
            },
        }])
        .build();

    with_server(config, |_| async move {
        let resp = reqwest::get("http://127.0.0.1:50001/hello")
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");
        Ok(())
    })
    .await
}
