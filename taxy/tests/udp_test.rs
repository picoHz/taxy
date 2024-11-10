use taxy_api::{
    port::{Port, PortEntry, UpstreamServer},
    proxy::{Proxy, ProxyEntry, ProxyKind, UdpProxy},
};
use tokio::net::UdpSocket;
mod common;
use common::{alloc_udp_port, with_server, TestStorage};

#[tokio::test]
async fn udp_proxy() -> anyhow::Result<()> {
    let listen_port = alloc_udp_port().await?;
    let proxy_port = alloc_udp_port().await?;

    let listener = UdpSocket::bind(listen_port.socket_addr()).await?;
    let config = TestStorage::builder()
        .ports(vec![PortEntry {
            id: "test".parse().unwrap(),
            port: Port {
                active: true,
                name: String::new(),
                listen: proxy_port.multiaddr_udp(),
                opts: Default::default(),
            },
        }])
        .proxies(vec![ProxyEntry {
            id: "test2".parse().unwrap(),
            proxy: Proxy {
                ports: vec!["test".parse().unwrap()],
                kind: ProxyKind::Udp(UdpProxy {
                    upstream_servers: vec![UpstreamServer {
                        addr: listen_port.multiaddr_udp(),
                    }],
                }),
                ..Default::default()
            },
        }])
        .build();

    with_server(config, |_| async move {
        let data = b"Hello";
        listener
            .send_to(&data[..], proxy_port.socket_addr())
            .await?;
        let mut buf = [0; 1024];
        let (size, addr) = listener.recv_from(&mut buf).await?;
        assert_eq!(&buf[..size], data);
        assert_eq!(addr, proxy_port.socket_addr());
        Ok(())
    })
    .await
}
