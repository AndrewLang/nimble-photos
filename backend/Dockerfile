FROM rust:latest as builder

# Install native deps needed for building postgres/sqlx etc.
RUN apt-get update \
    && apt-get install -y pkg-config libssl-dev libpq-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app/backend

# Copy the whole repository (workspace) so path dependencies build correctly
COPY . ../

# Build the backend with postgres feature
RUN cargo build --release --features postgres

FROM debian:bookworm-slim

# runtime deps
RUN apt-get update \
    && apt-get install -y ca-certificates libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/backend/target/release/nimble-photos /usr/local/bin/nimble-photos

WORKDIR /app

ENV RUST_LOG=info
EXPOSE 8080

CMD ["/usr/local/bin/nimble-photos"]
