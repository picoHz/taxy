+++
title = "Taxy"
sort_by = "weight"
+++

[![Crates.io](https://img.shields.io/crates/v/taxy.svg)](https://crates.io/crates/taxy)
[![GitHub license](https://img.shields.io/github/license/picoHz/taxy.svg)](https://github.com/picoHz/taxy/blob/main/LICENSE)
[![Rust](https://github.com/picoHz/taxy/actions/workflows/rust.yml/badge.svg)](https://github.com/picoHz/taxy/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/crate/taxy/latest/status.svg)](https://deps.rs/crate/taxy)

# Key Features

- Built with Rust for optimal performance and safety, powered by tokio and hyper
- Supports TCP, TLS, HTTP1, and HTTP2, including HTTP upgrading and WebSocket functionality
- Easily deployable single binary with a built-in WebUI
- Allows live configuration updates via a REST API without restarting the service
- Imports TLS certificates from the GUI or can generate a self-signed certificate
- Provides Let's Encrypt support (ACME v2, HTTP challenge only) for seamless certificate provisioning
- Supports automatic HTTP Brotli compression

# Installation

There are multiple ways to install Taxy.

## Cargo binstall (recommended)

[cargo-binstall](https://github.com/cargo-bins/) automatically downloads and installs pre-built binaries for your platform. If there is no pre-built binary available, it will fall back to `cargo install`.

You need to install [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation) first.

```bash
$ curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```

Then you can install Taxy with:

```bash
$ cargo binstall taxy
```

In case it falls back to `cargo install`, you need to have the Rust toolchain installed as explained in the next section.

## Cargo install

You need to have the Rust toolchain installed. If you don't, please follow the instructions on [rustup.rs](https://rustup.rs/).

The package on crates.io comes bundled with the WebUI as a static asset. Thus, you don't need to build it yourself (which would require [trunk](https://trunkrs.dev/) and wasm toolchain).

```bash
$ cargo install taxy
```

## Github Releases

Alternatively, you can directly download the latest pre-built binaries from the [releases page](https://github.com/picoHz/taxy/releases).

You simply put the extracted binary somewhere in your `$PATH` and you're good to go.

# First setup

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

> If you run the server on a remote machine, it is strongly recommended to use SSH port forwarding to access the admin panel instead of exposing the port to the public, as the connection is plain HTTP and not encrypted.
>
> You can serve the admin panel via HTTPS by using Taxy itself later on.

# Tutorial

In this tutorial, we will create a proxy for the admin panel itself.

## 1. Log in

Log in to the admin panel with the user you created earlier.

## 2. Bind a port

Before you can create a proxy, you need to bind a port to listen on. You can do this in the "Ports" section.

1. Click on "New port"
2. Name the port (e.g. "My Website"), you can leave this empty.
3. Select the network interface to listen on.
4. Select the port to listen on. Make sure it is not already in use and that you have the necessary permissions to bind to it.
5. Select the protocol to listen on. In this example, we will use "HTTP".
6. Click on "Create".
7. Make sure the created port appears in the list and the status is "Listening".

## 3. Create a proxy

Now you can create a proxy in the "Proxies" section. You need to specify the port you bound earlier and the target URL.

1. Click on "New proxy".
2. Name the proxy (e.g. "My Website"), you can leave this empty.
3. Select the protocol to use. In this example, we will use "HTTP / HTTPS".
4. Select the port you bound earlier. You can choose multiple ports if you want to.
5. Left the "Virtual Host" field empty to match all hosts.
6. Enter the target URL. In this example, we will use the admin panel "http://localhost:46492". Put the URL in the "Destination" field.
7. Click on "Create".
8. Make sure the created port appears in the list. The proxy is now active and you can access the admin panel via the proxy.

> If you want to expose the proxy to the public, you need to configure the firewall.
> This operation depends on your operating system.
