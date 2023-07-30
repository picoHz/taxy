# Use the official Rust image as the base image for the builder stage
FROM rust:latest as builder

# Install trunk
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Set the working directory
WORKDIR /usr/src/app

# Copy the actual source code
COPY Cargo.toml Cargo.lock ./
COPY taxy taxy
COPY taxy-api taxy-api
COPY taxy-webui taxy-webui

# Build the web UI
WORKDIR /usr/src/app/taxy-webui
RUN trunk build --release
WORKDIR /usr/src/app

# Build the Rust project
RUN cargo build --release

# Prepare the final image
FROM debian:bookworm-slim as runtime

# Install dependencies for the Rust binary
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Rust binary from the builder stage
COPY --from=builder /usr/src/app/target/release/taxy /usr/bin

# Set the entrypoint to run the Rust binary
ENTRYPOINT ["taxy", "start"]
