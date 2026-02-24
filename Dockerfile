FROM node:22-bookworm-slim AS frontend-builder

WORKDIR /src/nimble-photos

COPY nimble-photos/package.json nimble-photos/package-lock.json ./
RUN npm ci

COPY nimble-photos/ ./
RUN npm run build

FROM rust:bookworm AS backend-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libpq-dev ca-certificates cmake make g++ nasm \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src/backend
COPY . /src
RUN CMAKE_GENERATOR="Unix Makefiles" cargo build --release --manifest-path /src/backend/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /src/backend/target/release/nimble-photos /usr/local/bin/nimble-photos
COPY --from=backend-builder /src/backend/src/web.config.json /usr/local/bin/web.config.json
COPY --from=frontend-builder /src/backend/www /app/www
COPY --from=backend-builder /src/backend/bins /src/backend/bins

ENV RUST_LOG=info
EXPOSE 5151

CMD ["/usr/local/bin/nimble-photos"]
