# Builder image with development dependencies.
FROM ghcr.io/void-linux/void-linux:latest-mini-x86_64 as builder
RUN xbps-install -Syu
RUN xbps-install -Sy base-devel rustup cmake wget gnupg
RUN xbps-install -Sy openssl-devel libstdc++-devel
RUN rustup-init -qy

# Install tini for signal processing and zombie killing
ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /usr/local/bin/tini
RUN chmod +x /usr/local/bin/tini

# Build dicebot
RUN mkdir -p /root/src
WORKDIR /root/src
ADD . ./
RUN . /root/.cargo/env && cargo build --release

# Final image
FROM ghcr.io/void-linux/void-linux:latest-mini-x86_64
RUN xbps-install -Sy ca-certificates libstdc++
COPY --from=builder \
    /root/src/target/release/dicebot \
    /usr/local/bin/
COPY --from=builder \
    /usr/local/bin/tini \
    /usr/local/bin/

ENV XDG_CACHE_HOME "/cache"
ENV DATABASE_PATH "/cache/bot-db"
ENTRYPOINT [ "/usr/local/bin/tini", "-v", "--", "/usr/local/bin/dicebot", "/config/dicebot-config.toml" ]
