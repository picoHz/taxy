use std::{net::ToSocketAddrs, sync::Arc};
use taxy::certs::Cert;
use taxy_api::{
    port::{Port, PortEntry, PortOptions, UpstreamServer},
    tls::TlsTermination,
};
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn tls_proxy() -> anyhow::Result<()> {
    let root = Arc::new(Cert::new_ca().unwrap());
    let cert = Arc::new(Cert::new_self_signed(&["localhost".parse().unwrap()], &root).unwrap());

    let addr = "localhost:51000".to_socket_addrs().unwrap().next().unwrap();
    let hello = warp::path!("hello").map(|| format!("Hello"));
    let (_, server) = warp::serve(hello)
        .tls()
        .cert(&cert.raw_chain)
        .key(&cert.raw_key)
        .bind_ephemeral(addr);
    tokio::spawn(server);

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                name: String::new(),
                listen: "/ip4/127.0.0.1/tcp/51001/tls".parse().unwrap(),
                opts: PortOptions {
                    upstream_servers: vec![UpstreamServer {
                        addr: "/dns/localhost/tcp/51000/tls".parse().unwrap(),
                    }],
                    tls_termination: Some(TlsTermination {
                        server_names: vec!["localhost".into()],
                    }),
                },
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

    let ca = reqwest::Certificate::from_pem(&root.raw_chain)?;

    with_server(config, |_| async move {
        let client = reqwest::Client::builder()
            .add_root_certificate(ca)
            .build()?;
        let resp = client
            .get("https://localhost:51001/hello")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");
        Ok(())
    })
    .await
}
