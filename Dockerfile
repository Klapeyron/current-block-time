FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin current-block-time

WORKDIR ./current-block-time

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY src src

RUN cargo build --release

CMD ["/home/rust/src/current-block-time/target/x86_64-unknown-linux-musl/release/current-block-time"]

FROM alpine:latest

EXPOSE 2137

COPY --from=builder /home/rust/src/current-block-time/target/x86_64-unknown-linux-musl/release/current-block-time /usr/local/bin/current-block-time

CMD ["current-block-time"]
