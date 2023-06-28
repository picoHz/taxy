use std::sync::Arc;
use taxy::certs::Cert;
use taxy_api::{
    cert::SelfSignedCertRequest,
    port::{Port, PortEntry, PortOptions, UpstreamServer},
    tls::TlsTermination,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn tls_proxy() -> anyhow::Result<()> {
    let cert = Arc::new(
        Cert::new_self_signed(&SelfSignedCertRequest {
            san: vec!["localhost".parse().unwrap()],
        })
        .unwrap(),
    );

    let listener = TcpListener::bind("127.0.0.1:51000").await.unwrap();
    let hello = warp::path!("hello").map(|| format!("Hello"));
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                listen: "/ip4/127.0.0.1/tcp/51001/tls".parse().unwrap(),
                opts: PortOptions {
                    upstream_servers: vec![UpstreamServer {
                        addr: "/ip4/127.0.0.1/tcp/51000".parse().unwrap(),
                    }],
                    tls_termination: Some(TlsTermination {
                        server_names: vec!["localhost".into()],
                    }),
                },
            },
        }])
        .certs([(cert.id.clone(), cert.clone())].into_iter().collect())
        .build();

    let pem = String::from_utf8_lossy(&cert.raw_chain);
    let root = pem.split("\r\n\r\n").skip(1).next().unwrap();
    let ca = reqwest::Certificate::from_pem(root.as_bytes())?;

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
