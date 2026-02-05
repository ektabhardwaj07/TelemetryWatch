# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock* ./

# Create a dummy source file to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/telemetrywatch /usr/local/bin/telemetrywatch

# Copy static files
COPY static /app/static

# Create non-root user
RUN useradd -m -u 1000 telemetrywatch && \
    chown -R telemetrywatch:telemetrywatch /app

USER telemetrywatch

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/telemetrywatch"]

