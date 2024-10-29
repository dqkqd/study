ARG RUST_VERSION=1.82
ARG APP_NAME=ratelimiter

FROM rust:${RUST_VERSION}-alpine3.19 AS build
ARG APP_NAME
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF

set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/app
EOF

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y musl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build /bin/app /bin/
EXPOSE 3001
CMD ["app"]
