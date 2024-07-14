FROM rust:1.78-alpine AS builder
RUN apk add --no-cache musl-dev

COPY . /usr/src/qs
WORKDIR /usr/src/qs
RUN cargo build --release

FROM alpine
WORKDIR /usr/share/qs
COPY --from=builder /usr/src/qs/target/release/qs /usr/bin/qs

ENTRYPOINT [ "/usr/bin/qs" ]
