# Yoinked from:
# https://github.com/emk/rust-musl-builder/blob/master/examples/using-diesel/Dockerfile
FROM ekidd/rust-musl-builder:stable AS builder
ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/dicebot \
    /usr/local/bin/
CMD /usr/local/bin/dicebot /config/dicebot-config.toml
