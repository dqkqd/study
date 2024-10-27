FROM rust:1.82-alpine3.19 as builder
RUN apk add --no-cache musl-dev
WORKDIR /server
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y musl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
EXPOSE 3000
CMD ["server"]
