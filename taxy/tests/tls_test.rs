use std::sync::Arc;
use taxy::{certs::Cert, server::Server};
use taxy_api::{
    cert::SelfSignedCertRequest,
    port::{Port, PortEntry, PortOptions, UpstreamServer},
    tls::TlsTermination,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod storage;
use storage::TestStorage;

#[tokio::test]
async fn tls_proxy() {
    let cert = Arc::new(
        Cert::new_self_signed(&SelfSignedCertRequest {
            san: vec!["localhost".parse().unwrap()],
        })
        .unwrap(),
    );

    let listener = TcpListener::bind("127.0.0.1:51000").await.unwrap();
    let hello = warp::path!("hello").map(|| format!("Hello"));
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let (server, channels) = Server::new(
        TestStorage::builder()
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
            .build(),
    )
    .await;
    let task = tokio::spawn(server.start());

    let pem = String::from_utf8_lossy(&cert.raw_chain);
    let root = pem.split("\r\n\r\n").skip(1).next().unwrap();
    let ca = reqwest::Certificate::from_pem(root.as_bytes()).unwrap();
    let client = reqwest::Client::builder()
        .add_root_certificate(ca)
        .build()
        .unwrap();
    let resp = client
        .get("https://localhost:51001/hello")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(resp, "Hello");

    channels.shutdown();
    task.await.unwrap().unwrap();
}
