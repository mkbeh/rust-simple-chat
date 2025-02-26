ARG RUST_IMAGE=rust
ARG ALPINE_VER=3.21
ARG RUST_VER=1.85.0-alpine${ALPINE_VER}

FROM ${RUST_IMAGE}:${RUST_VER} as builder

ARG TARGET="x86_64-unknown-linux-musl"

ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

WORKDIR /src
COPY . .

RUN apk add --no-cache musl-dev

RUN rustup target add ${TARGET}
RUN cargo build --release --target=${TARGET} --bins

FROM alpine:${ALPINE_VER} as runtime

RUN addgroup -g 101 app && \
    adduser -H -u 101 -G app -s /bin/sh -D app && \
    apk update --no-cache -X alpine/v${ALPINE_VER}/main && \
    apk upgrade --no-cache -X alpine/v${ALPINE_VER}/main

WORKDIR /app/

COPY --from=builder --chown=app:app /src/target/x86_64-unknown-linux-musl/release/chat .
COPY --chown=app:app migrations migrations

USER app

CMD ["/app/chat"]