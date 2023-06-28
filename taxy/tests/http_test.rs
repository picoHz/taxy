use taxy_api::{
    port::{Port, PortEntry},
    site::{Route, Site, SiteEntry},
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

mod common;
use common::{with_server, TestStorage};

#[tokio::test]
async fn http_proxy() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:52000").await.unwrap();
    let hello = warp::path!("hello").map(|| format!("Hello"));
    tokio::spawn(warp::serve(hello).run_incoming(TcpListenerStream::new(listener)));

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".into(),
            port: Port {
                listen: "/ip4/127.0.0.1/tcp/52001/http".parse().unwrap(),
                opts: Default::default(),
            },
        }])
        .sites(vec![SiteEntry {
            id: "test2".into(),
            site: Site {
                ports: vec!["test".into()],
                vhosts: vec!["localhost:52001".parse().unwrap()],
                routes: vec![Route {
                    path: "/".into(),
                    servers: vec![taxy_api::site::Server {
                        url: "http://127.0.0.1:52000/".parse().unwrap(),
                    }],
                }],
            },
        }])
        .build();

    with_server(config, |_| async move {
        let client = reqwest::Client::builder().build()?;
        let resp = client
            .get("http://localhost:52001/hello")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");
        Ok(())
    })
    .await
}
