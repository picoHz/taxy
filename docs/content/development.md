+++
title = "Development"
description = "Development"
weight = 0
+++

The source code is available on [GitHub](https://github.com/picoHz/taxy).

# Prerequisites

- Stable Rust toolchain
- WASM toolchain
- [Trunk](https://trunkrs.dev/)

# Install dependencies

## Rust Toolchain

See [rustup.rs](https://rustup.rs/).

## WASM Toolchain

```bash
$ rustup target add wasm32-unknown-unknown
```

## Trunk

See [trunkrs.dev](https://trunkrs.dev/).

# Debug Build

```bash
$ git clone https://github.com/picoHz/taxy

# Start the server
$ cd taxy
$ cargo run

# In a separate terminal, start `trunk serve` for the WebUI
$ cd webui
$ trunk serve --proxy-backend=http://localhost:46492/api/
```

# Release Build

```bash
# Build the WebUI
$ cd taxy/taxy-webui
$ trunk build --release

# In a separate terminal, start `trunk serve` for the WebUI
$ cd ..
$ cargo build --release

# Start the server
$ target/release/taxy start
```