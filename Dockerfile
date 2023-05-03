# Use the official Rust image as the base image for the builder stage
FROM rust:latest as builder

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get install -y nodejs

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock build.rs ./

# Create a phony main.rs to build dependencies in a separate layer
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

# Build the dependencies
RUN cargo build --release

# Remove the dummy main.rs
RUN rm src/main.rs

# Copy the actual source code
COPY src src
COPY webui webui

WORKDIR /usr/src/app/webui
RUN npm install
RUN npm run build
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