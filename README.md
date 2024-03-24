<div align="center">
<img alt="edition logo" src="https://github.com/picoHz/taxy/blob/main/logo.svg?raw=true" width="150" />

# Taxy

A reverse proxy server with built-in WebUI, supporting TCP/HTTP/TLS/WebSocket, written in Rust.

[![Crates.io](https://img.shields.io/crates/v/taxy.svg)](https://crates.io/crates/taxy)
[![GitHub license](https://img.shields.io/github/license/picoHz/taxy.svg)](https://github.com/picoHz/taxy/blob/main/LICENSE)
[![Rust](https://github.com/picoHz/taxy/actions/workflows/rust.yml/badge.svg)](https://github.com/picoHz/taxy/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/crate/taxy/latest/status.svg)](https://deps.rs/crate/taxy)

</div>

## 🚧 Notice

Taxy is currently in early development. Please be aware that breaking changes may occur frequently, particularly when upgrading between minor versions (e.g., from 0.3.x to 0.4.x).

## Overview

- Built with Rust for optimal performance and safety, powered by tokio and hyper
- Supports TCP, TLS, HTTP1, and HTTP2, including HTTP upgrading and WebSocket functionality
- Easily deployable single binary with a built-in WebUI
- Allows live configuration updates via a REST API without restarting the service
- Imports TLS certificates from the GUI or can generate a self-signed certificate
- Provides Let's Encrypt support (ACME v2, HTTP challenge only) for seamless certificate provisioning
- Supports automatic HTTP Brotli compression

## Screenshot

![Taxy WebUI Screenshot](https://raw.githubusercontent.com/picoHz/taxy/main/screenshot.png)

## Web UI Demo

Visit https://taxy.onrender.com/. (username: `admin`, password: `admin`)

Please note, you can change the configuration freely, but due to the instance being behind a firewall, the configured proxies are not accessible from the outside.

## Installation

There are multiple ways to install Taxy.

### Docker Compose

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

### Cargo binstall

[cargo-binstall](https://github.com/cargo-bins/) automatically downloads and installs pre-built binaries for your platform. If there is no pre-built binary available, it will fall back to `cargo install`.

You need to install [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation) first.

Then you can install Taxy with:

```bash
$ cargo binstall taxy
```

### Cargo install

You need to have the Rust toolchain installed. If you don't, please follow the instructions on [rustup.rs](https://rustup.rs/).

The package on crates.io comes bundled with the WebUI as a static asset. Thus, you don't need to build it yourself (which would require [trunk](https://trunkrs.dev/) and wasm toolchain).

```bash
$ cargo install taxy
```

### Github Releases

Alternatively, you can directly download the latest pre-built binaries from the [releases page](https://github.com/picoHz/taxy/releases).

You simply put the extracted binary somewhere in your `$PATH` and you're good to go.

## Starting the server

First, you need to create a user to access the admin panel. You will be prompted for a password.

```bash
# Create a user
$ taxy add-user admin
$ password?: ******
```

Then, you can start the server.

```bash
$ taxy start
```

Once the server is running, you can access the admin panel at [http://localhost:46492/](http://localhost:46492/).

## Development

To contribute or develop Taxy, follow these steps:

```bash
# Clone the repository
git clone https://github.com/picoHz/taxy

# Start the server
cd taxy
cargo run

# In a separate terminal, start `trunk serve` for the WebUI
cd taxy-webui
trunk serve
```

### Gitpod

You can instantly start developing Taxy in your browser using Gitpod.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/picoHz/taxy)

## Similar projects

HTTP reverse proxies written in Rust:

- [Sōzu](https://github.com/sozu-proxy/sozu)
- [rpxy](https://github.com/junkurihara/rust-rpxy)

## Credit

The social preview image uses the photo by [cal gao](https://unsplash.com/@ginnta?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText) on [Unsplash](https://unsplash.com/photos/MASpFp0X2VU?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText).
