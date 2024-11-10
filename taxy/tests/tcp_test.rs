use taxy_api::{
    port::{Port, PortEntry, UpstreamServer},
    proxy::{Proxy, ProxyEntry, ProxyKind, TcpProxy},
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod common;
use common::{alloc_port, with_server, TestStorage};

#[tokio::test]
async fn tcp_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_port().await?;
    let proxy_port = alloc_port().await?;

    let listener = TcpListener::bind(listen_port.socket_addr()).await.unwrap();
    let hello = warp::path!("hello").map(|| "Hello".to_string());
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_tcp(),
                opts: Default::default(),
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap()],
                kind: ProxyKind::Tcp(TcpProxy {
                    upstream_servers: vec![UpstreamServer {
                        addr: listen_port.multiaddr_tcp(),
                    }],
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let resp = reqwest::get(proxy_port.http_url("/hello"))
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");
        Ok(())
    })
    .await
}
