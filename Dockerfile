# Build stage
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .

# Install build dependencies and set optimization flags
RUN apt-get update && \
    apt-get install -y clang cmake libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

ENV RUSTFLAGS="-C target-cpu=native -C lto -C codegen-units=1"
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy kernel optimization script
COPY install.sh /usr/local/bin/

# Copy built binary
COPY --from=builder /app/target/release/server /app/server

# Set kernel parameters and run server
CMD ["sh", "-c", "/usr/local/bin/install.sh && /app/server"]
EXPOSE 7171