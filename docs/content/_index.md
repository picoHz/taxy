+++
title = "Taxy"
sort_by = "weight"
+++

[![Crates.io](https://img.shields.io/crates/v/taxy.svg)](https://crates.io/crates/taxy)
[![GitHub license](https://img.shields.io/github/license/picoHz/taxy.svg)](https://github.com/picoHz/taxy/blob/main/LICENSE)
[![Rust](https://github.com/picoHz/taxy/actions/workflows/rust.yml/badge.svg)](https://github.com/picoHz/taxy/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/crate/taxy/latest/status.svg)](https://deps.rs/crate/taxy)

# Key Features

- Built with Rust for optimal performance and safety, powered by [tokio](https://tokio.rs/) and [hyper](https://hyper.rs/)
- Supports TCP, UDP, TLS, HTTP1, and HTTP2, including HTTP upgrading and WebSocket functionality
- Easily deployable single binary with a built-in WebUI
- Allows live configuration updates via a REST API without restarting the service
- Imports TLS certificates from the GUI or can generate a self-signed certificate
- Provides Let's Encrypt support (ACME v2, HTTP challenge only) for seamless certificate provisioning
- Supports automatic HTTP Brotli compression

# Installation

There are multiple ways to install Taxy.

## Docker

Run the following command to start Taxy using Docker:

```bash
docker run -d \
  -v taxy-config:/root/.config/taxy \
  -p 80:80 \
  -p 443:443 \
  -p 127.0.0.1:46492:46492 \
  --restart unless-stopped \
  --name taxy \
  ghcr.io/picohz/taxy:latest
```

To log in to the admin panel, you'll first need to create a user. Follow the steps below to create an admin user:

```bash
docker exec -t -i taxy taxy add-user admin
password?: ******
```

## Docker Compose

Create a file named `docker-compose.yml` with the following content:

```yaml
version: "3"
services:
  taxy:
    image: ghcr.io/picohz/taxy:latest
    container_name: taxy
    volumes:
      - taxy-config:/root/.config/taxy
    ports:
      # Add ports here if you want to expose them to the host
      - 80:80
      - 443:443
      - 127.0.0.1:46492:46492 # Admin panel
    restart: unless-stopped

volumes:
  taxy-config:
```

Run the following command to start Taxy:

```bash
$ docker-compose up -d
```

To log in to the admin panel, you'll first need to create a user. Follow the steps below to create an admin user:

```bash
$ docker-compose exec taxy taxy add-user admin
password?: ******
```

Then, you can access the admin panel at [http://localhost:46492/](http://localhost:46492/).

## Cargo binstall

[cargo-binstall](https://github.com/cargo-bins/) automatically downloads and installs pre-built binaries for your platform. If there is no pre-built binary available, it will fall back to `cargo install`.

You need to install [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation) first.

Then you can install Taxy with:

```bash
$ cargo binstall taxy
```

## Cargo install

You need to have the Rust toolchain installed. If you don't, please follow the instructions on [rustup.rs](https://rustup.rs/).

The package on crates.io comes bundled with the WebUI as a static asset. Thus, you don't need to build it yourself (which would require [trunk](https://trunkrs.dev/) and wasm toolchain).

```bash
$ cargo install taxy
```

## Github Releases

Alternatively, you can directly download the latest pre-built binaries from the [releases page](https://github.com/picoHz/taxy/releases).

You simply put the extracted binary somewhere in your `$PATH` and you're good to go.

# Development

Please refer to the [Development](/development) section for details.

# First setup

First, you need to create a user to access the admin panel. You will be prompted for a password.

```bash
# Create a user
$ taxy add-user admin
$ password?: ******
```

If you want to use TOTP for two-factor authentication, you can enable it with the `--totp` flag.

```bash
# Create a user with TOTP enabled
$ taxy add-user admin --totp
$ password?: ******

Use this code to setup your TOTP client:
EXAMPLECODEEXAMPLECODE
```

Then, you can start the server.

```bash
$ taxy start
```

Once the server is running, you can access the admin panel at [http://localhost:46492/](http://localhost:46492/).

> To ensure the security of your server's admin panel, it is highly recommended to employ SSH port forwarding when running the server on a remote machine. This practice prevents exposing the admin panel's port to the public, as the connection is in plain HTTP and lacks encryption. However, you have the option to serve the admin panel via HTTPS using Taxy later on.

# Tutorial

In this tutorial, we will create a proxy for the admin panel itself.

## 1. Log in

Log in to the admin panel with the user you created earlier.

## 2. Bind a port

Before you can create a proxy, you need to bind a port to listen on. You can do this in the "Ports" section.

1. Click on "Ports" in the menu.
2. Click on "Add"
3. Name the port (e.g. "My Website"), you can leave this empty.
4. Select the network interface to listen on.
5. Select the port to listen on. Make sure it is not already in use and that you have the necessary permissions to bind to it.
6. Select the protocol to listen on. In this example, we will use "HTTP".
7. Click on "Create".
8. Make sure the created port appears in the list and the status is "Listening".

## 3. Create a proxy

Now you can create a proxy in the "Proxies" section. You need to specify the port you bound earlier and the target URL.

1. Click on "Proxies" in the menu.
2. Click on "Add".
3. Name the proxy (e.g. "My Website"), you can leave this empty.
4. Select the protocol to use. In this example, we will use "HTTP / HTTPS".
5. Select the port you bound earlier. You can choose multiple ports if you want to.
6. Left the "Virtual Host" field empty to match all hosts.
7. Enter the target URL. In this example, we will use the admin panel "http://localhost:46492". Put the URL in the "Target" field.
8. Click on "Create".
9. Make sure the created port appears in the list. The proxy is now active and you can access the admin panel via the proxy.

> If you want to expose the proxy to the public, you may need to configure the firewall.
