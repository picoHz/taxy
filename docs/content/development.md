+++
title = "Development"
description = "Development"
weight = 0
+++

Our project's source code is available on [GitHub](https://github.com/picoHz/taxy).

# Prerequisites

Before getting started, make sure to install the following prerequisites:

- Rust toolchain: You can install it using [rustup.rs](https://rustup.rs/)
- WASM toolchain: After installing the Rust toolchain, add the WASM target by executing `rustup target add wasm32-unknown-unknown` in your terminal
- [Trunk](https://trunkrs.dev/): Visit the website for installation instructions

# Development Setup

To set up the development environment, follow these steps:

1. Clone the repository: `git clone https://github.com/picoHz/taxy`
2. Start the server:

   ```bash
   cd taxy
   cargo run
   ```

3. In a separate terminal, start `trunk serve` for the WebUI:

   ```bash
   cd webui
   trunk serve
   ```

# Building for Release

To build the project for a release, execute the following steps:

1. Build the WebUI:

   ```bash
   cd taxy/taxy-webui
   trunk build --release
   ```

2. In a separate terminal, build the project:

   ```bash
   cd ..
   cargo build --release
   ```

3. Start the server:

   ```bash
   target/release/taxy start
   ```

# Gitpod

You can instantly start developing Taxy in your browser using Gitpod.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/picoHz/taxy)
