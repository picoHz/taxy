FROM gitpod/workspace-full:latest

RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown