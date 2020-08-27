# Builder image
FROM bougyman/voidlinux:glibc as builder
RUN xbps-install -Sy base-devel libressl-devel wget gnupg rustup
RUN rustup-init -qy

# Install tini for signal processing and zombie killing
ENV TINI_VERSION v0.18.0
ENV TINI_SIGN_KEY 595E85A6B1B4779EA4DAAEC70B588DFF0527A9B7
RUN set -eux; \
  wget -O /usr/local/bin/tini "https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini"; \
  wget -O /usr/local/bin/tini.asc "https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini.asc"; \
  export GNUPGHOME="$(mktemp -d)"; \
  gpg --batch --keyserver ha.pool.sks-keyservers.net --recv-keys "$TINI_SIGN_KEY"; \
  gpg --batch --verify /usr/local/bin/tini.asc /usr/local/bin/tini; \
  command -v gpgconf && gpgconf --kill all || :; \
  rm -r "$GNUPGHOME" /usr/local/bin/tini.asc; \
  chmod +x /usr/local/bin/tini; \
	tini --version

# Build dicebot
RUN mkdir -p /root/src
WORKDIR /root/src
ADD . ./
RUN . /root/.cargo/env && cargo build --release

# Final image
FROM bougyman/voidlinux:latest
RUN xbps-install -Sy ca-certificates libstdc++
COPY --from=builder \
    /root/src/target/release/dicebot \
    /usr/local/bin/
COPY --from=builder \
    /usr/local/bin/tini \
    /usr/local/bin/

ENV XDG_CACHE_HOME "/cache"
ENTRYPOINT [ "/usr/local/bin/tini", "-v", "--", "/usr/local/bin/dicebot", "/config/dicebot-config.toml" ]
