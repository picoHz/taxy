use std::sync::Arc;
use taxy::certs::Cert;
use taxy_api::{
    port::{Port, PortEntry, PortOptions, UpstreamServer},
    proxy::{Proxy, ProxyEntry, ProxyKind, TcpProxy},
    tls::TlsTermination,
};
use warp::Filter;

mod common;
use common::{alloc_port, with_server, TestStorage};

#[tokio::test]
async fn tls_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_port().await?;
    let proxy_port = alloc_port().await?;

    let root = Arc::new(Cert::new_ca().unwrap());
    let cert = Arc::new(Cert::new_self_signed(&["localhost".parse().unwrap()], &root).unwrap());

    let addr = listen_port.socket_addr();
    let hello = warp::path!("hello").map(|| "Hello".to_string());
    let (_, server) = warp::serve(hello)
        .tls()
        .cert(&cert.pem_chain)
        .key(cert.pem_key.as_ref().unwrap())
        .bind_ephemeral(addr);
    tokio::spawn(server);

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_tls(),
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
                kind: ProxyKind::Tcp(TcpProxy {
                    upstream_servers: vec![UpstreamServer {
                        addr: format!(
                            "/dns/localhost/tcp/{}/tls",
                            listen_port.socket_addr().port()
                        )
                        .parse()
                        .unwrap(),
                    }],
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

    let ca = reqwest::Certificate::from_pem(&root.pem_chain)?;

    with_server(config, |_| async move {
        let client = reqwest::Client::builder()
            .add_root_certificate(ca)
            .build()?;
        let resp = client
            .get(proxy_port.https_url("/hello"))
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");
        Ok(())
    })
    .await
}
