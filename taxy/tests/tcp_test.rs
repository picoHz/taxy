use axum::{routing::get, Router};
use taxy_api::{
    port::{Port, PortEntry, UpstreamServer},
    proxy::{Proxy, ProxyEntry, ProxyKind, TcpProxy},
};
mod common;
use common::{alloc_tcp_port, with_server, TestStorage};

#[tokio::test]
async fn tcp_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_tcp_port().await?;
    let proxy_port = alloc_tcp_port().await?;

    async fn handler() -> &'static str {
        "Hello"
    }
    let app = Router::new().route("/hello", get(handler));

    let addr = listen_port.socket_addr();
    tokio::spawn(axum_server::bind(addr).serve(app.into_make_service()));

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
