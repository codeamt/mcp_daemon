FROM mcr.microsoft.com/devcontainers/rust:1-ubuntu

# Install protobuf compiler
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends protobuf-compiler libssl-dev pkg-config cmake

# Install Rust tools
RUN rustup component add rustfmt clippy \
    && cargo install cargo-watch cargo-audit cargo-tarpaulin grcov

# Install gRPC health probe
RUN curl -L https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.4.24/grpc_health_probe-linux-amd64 \
    -o /usr/local/bin/grpc_health_probe \
    && chmod +x /usr/local/bin/grpc_health_probe

# Clean up
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

