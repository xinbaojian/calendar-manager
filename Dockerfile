FROM rust:alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf

COPY Cargo.toml Cargo.lock ./
COPY askama.toml ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs

# 缓存 cargo registry 和编译产物
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && rm -rf src

COPY src ./src
COPY templates ./templates
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    touch src/main.rs && cargo build --release && cp target/release/calendarsync /usr/local/bin/

FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata
COPY --from=builder /usr/local/bin/calendarsync /usr/local/bin/
EXPOSE 8080
CMD ["calendarsync"]
