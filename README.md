# Taxy

A reverse proxy server with a user-friendly WebUI.

[![Crates.io](https://img.shields.io/crates/v/taxy.svg)](https://crates.io/crates/taxy)
[![GitHub license](https://img.shields.io/github/license/picoHz/taxy.svg)](https://github.com/picoHz/taxy/blob/main/LICENSE)

## Overview

- Built with Rust for optimal performance and safety, powered by tokio and hyper
- Supports TLS, HTTP1, and HTTP2 protocols, including HTTP upgrading and WebSocket.
- Intuitive and user-friendly WebUI for easy configuration
- Live configuration updates without restarting the service
- Import TLS certificates from GUI or generate a self-signed certificate
- Let's Encrypt support (ACME v2) for seamless certificate provisioning

## Web UI Demo

Visit https://taxy.onrender.com/. (username: `admin`, password: `admin`)

You can access the REST API documentation on https://taxy.onrender.com/swagger-ui.

## Installation

To build the Taxy binary, ensure that you have the Rust toolchain installed.

Once you have successfully built and started taxy, you can access the admin panel at http://localhost:46492/.

### From crates.io

The package on crates.io contains the WebUI as a static asset, so you don't need to build it yourself.

Install "Taxy" using Cargo:

```bash
cargo install taxy

# Create admin user
taxy add-user admin
password?: ******

# Start server
taxy start
```

### From git

To build the Web UI, make sure you have [trunk](https://trunkrs.dev/) installed on your system.

Clone the repository and install the package:

```bash
git clone https://github.com/picoHz/taxy
cd taxy/taxy-webui
trunk build --release
cd ..
cargo install --path .
```

## Development

To contribute or develop Taxy, follow these steps:

```bash
# Clone the repository
git clone https://github.com/picoHz/taxy

# Start the server
cd taxy
cargo run

# In a separate terminal, start `trunk serve` for the WebUI
cd webui
trunk serve --proxy-backend=http://localhost:46492/api/
```

## FAQ

### Why don't changes to the configuration take effect immediately?

Updating the configuration solely impacts new connections.
When browsers maintain active TCP streams, subsequent requests will continue to follow the prior configuration.
