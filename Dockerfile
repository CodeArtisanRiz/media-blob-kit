# Builder stage
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev curl

WORKDIR /app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache libgcc openssl ca-certificates dumb-init

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/media-blob-kit .

# Environment setup
ENV RUST_LOG=info
ENV APP_HOST=0.0.0.0
ENV APP_PORT=3000

# Expose port
EXPOSE 3000

# Use dumb-init as entrypoint to handle signals correctly
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["./media-blob-kit"]
