# Use the official Rust image as the base image for the builder stage
FROM rust:latest as builder

# Install trunk
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
# COPY taxy/Cargo.toml ./

# Create a phony main.rs to build dependencies in a separate layer
# RUN mkdir src
# RUN echo "fn main() {}" > src/main.rs

# Build the dependencies
# RUN cargo build --release

# Remove the dummy main.rs
# RUN rm src/main.rs

# Copy the actual source code
COPY Cargo.toml Cargo.lock ./
COPY taxy taxy
COPY taxy-api taxy-api
COPY taxy-webui taxy-webui

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
COPY --from=builder /usr/src/app/target/release/taxy .

# Add admin user
RUN ./taxy add-user admin -p admin

# Set the entrypoint to run the Rust binary
ENTRYPOINT ["./taxy", "start", "--webui", "0.0.0.0:8080"]