# Use the official Rust image as the base image
FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    protobuf-compiler \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Install the nightly toolchain and set it as default
RUN rustup toolchain install nightly
RUN rustup default nightly

# Install the wasm32-unknown-unknown target and rust-src component for nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN rustup component add rust-src --toolchain nightly

# Set the working directory
WORKDIR /usr/src/app

# Clone the repository
RUN git clone https://github.com/paraspell-research/polkadot-sdk.git


# Configure Cargo to use Git CLI
RUN mkdir -p ~/.cargo && echo "[net]\ngit-fetch-with-cli = true" > ~/.cargo/config

ENV CARGO_HOME=~/.cargo

# Change directory to Polkadot
WORKDIR /usr/src/app/polkadot-sdk

# Build the project
RUN cargo b -r -p polkadot

# Build parachain-template-node-two
RUN cargo build --release -p parachain-template-node-two 

# Build parachain-template-node
RUN cargo build --release -p parachain-template-node

# Change directory to binaries
WORKDIR /usr/src/app/polkadot-sdk/binaries

# Download the latest zombienet image
RUN wget https://github.com/paritytech/zombienet/releases/download/v1.3.116/zombienet-linux-x64 \
    && chmod +x zombienet-linux-x64
    
# Set environment variable for interface binding
ENV BIND_INTERFACE=127.0.0.1

# Launch zombienet
CMD ["./zombienet-linux-x64", "-p", "native", "-c", "1", "spawn", "config-both.toml"]


