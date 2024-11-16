use reqwest::{header::HOST, redirect::Policy, Body};
use serde_json::json;
use taxy_api::{
    port::{Port, PortEntry, PortOptions},
    proxy::{HttpProxy, Proxy, ProxyEntry, ProxyKind, Route},
    tls::TlsTermination,
};

mod common;
use common::{alloc_tcp_port, with_server, TestStorage};

#[tokio::test]
async fn http_proxy() -> anyhow::Result<()> {
    let proxy_port = alloc_tcp_port().await?;
    let mut server = mockito::Server::new_async().await;

    let mock_get = server
        .mock("GET", "/hello?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("accept-encoding", "gzip, br")
        .with_body("Hello")
        .create_async()
        .await;

    let mock_get_host = server
        .mock("GET", "/hello?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("x-forwarded-host", "sphinx.of.black.quartz.judge.my.vow")
        .match_header("accept-encoding", "gzip, br")
        .with_body("Hello")
        .create_async()
        .await;

    let mock_get_with_path = server
        .mock("GET", "/Hello?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("accept-encoding", "gzip, br")
        .with_body("Hello")
        .create_async()
        .await;

    let mock_get_trailing_slash = server
        .mock("GET", "/hello/?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("accept-encoding", "gzip, br")
        .with_body("Hello")
        .create_async()
        .await;

    let mock_post = server
        .mock("POST", "/hello?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("accept-encoding", "gzip, br")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::Json(json!({"hello": "world"})))
        .with_body("Hello")
        .create_async()
        .await;

    let chunks = vec!["hello"; 1024];
    let mock_stream = server
        .mock("POST", "/hello?world=1")
        .match_header("via", "taxy")
        .match_header("x-real-ip", mockito::Matcher::Missing)
        .match_header("x-forwarded-proto", "http")
        .match_header("accept-encoding", "gzip, br")
        .match_header("transfer-encoding", "chunked")
        .match_body(chunks.concat().as_str())
        .with_body("Hello")
        .create_async()
        .await;

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_http(),
                opts: Default::default(),
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap()],
                kind: ProxyKind::Http(HttpProxy {
                    vhosts: vec![
                        "localhost".parse().unwrap(),
                        "^([a-z]+\\.)+my\\.vow$".parse().unwrap(),
                    ],
                    routes: vec![
                        Route {
                            path: "/was/ist/passiert".into(),
                            servers: vec![taxy_api::proxy::Server {
                                url: server.url().parse().unwrap(),
                            }],
                        },
                        Route {
                            path: "/".into(),
                            servers: vec![taxy_api::proxy::Server {
                                url: server.url().parse().unwrap(),
                            }],
                        },
                    ],
                    upgrade_insecure: false,
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let client = reqwest::Client::builder().build()?;
        let resp = client
            .get(proxy_port.http_url("/hello?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        let resp = client
            .get(proxy_port.http_url("/was/ist/passiert/Hello?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        let resp = client
            .get(proxy_port.http_url("/hello/?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        let resp = client
            .post(proxy_port.http_url("/hello?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .json(&json!({"hello": "world"}))
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        let stream = tokio_stream::iter(chunks.into_iter().map(Ok::<_, ::std::io::Error>));
        let body = Body::wrap_stream(stream);

        let resp = client
            .post(proxy_port.http_url("/hello?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        let client = reqwest::Client::builder().build()?;
        let resp = client
            .get(proxy_port.http_url("/hello?world=1"))
            .header("x-forwarded-for", "0.0.0.0")
            .header("x-real-ip", "0.0.0.0")
            .header(HOST, "sphinx.of.black.quartz.judge.my.vow")
            .send()
            .await?
            .text()
            .await?;
        assert_eq!(resp, "Hello");

        Ok(())
    })
    .await?;

    mock_get.assert_async().await;
    mock_get_host.assert_async().await;
    mock_get_with_path.assert_async().await;
    mock_get_trailing_slash.assert_async().await;
    mock_post.assert_async().await;
    mock_stream.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn http_proxy_upgrade_insecure() -> anyhow::Result<()> {
    let proxy_port = alloc_tcp_port().await?;
    let proxy_port2 = alloc_tcp_port().await?;

    let config = TestStorage::builder()
        .ports(vec![
            PortEntry {
                id: "test".parse().unwrap(),
                port: Port {
                    active: true,
                    name: String::new(),
                    listen: proxy_port.multiaddr_http(),
                    opts: Default::default(),
                },
            },
            PortEntry {
                id: "secure".parse().unwrap(),
                port: Port {
                    active: true,
                    name: String::new(),
                    listen: proxy_port2.multiaddr_https(),
                    opts: PortOptions {
                        tls_termination: Some(TlsTermination {
                            server_names: vec!["localhost".into()],
                        }),
                    },
                },
            },
        ])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap(), "secure".parse().unwrap()],
                kind: ProxyKind::Http(HttpProxy {
                    vhosts: vec!["localhost".parse().unwrap()],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::proxy::Server {
                            url: "https://httpbin.org/".parse().unwrap(),
                        }],
                    }],
                    upgrade_insecure: true,
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()?;
        let url = proxy_port.http_url("/hello");
        let resp = client
            .get(url.clone())
            .header(HOST, url.host_str().unwrap())
            .send()
            .await?;
        assert_eq!(resp.status(), 301);
        assert_eq!(
            resp.headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            proxy_port2.https_url("/hello").to_string()
        );
        Ok(())
    })
    .await?;

    Ok(())
}

#[tokio::test]
async fn http_proxy_dns_error() -> anyhow::Result<()> {
    let proxy_port = alloc_tcp_port().await?;

    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_http(),
                opts: Default::default(),
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap()],
                kind: ProxyKind::Http(HttpProxy {
                    vhosts: vec!["localhost".parse().unwrap()],
                    routes: vec![Route {
                        path: "/".into(),
                        servers: vec![taxy_api::proxy::Server {
                            url: "https://example.nodomain/".parse().unwrap(),
                        }],
                    }],
                    upgrade_insecure: false,
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let client = reqwest::Client::builder().build()?;
        let resp = client.get(proxy_port.http_url("/hello")).send().await?;
        assert_eq!(resp.status(), 502);
        Ok(())
    })
    .await?;

    Ok(())
}
