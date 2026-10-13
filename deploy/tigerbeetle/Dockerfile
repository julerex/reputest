FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

ARG TB_VERSION=0.17.9
RUN curl -fsSL -o /tmp/tigerbeetle.zip \
    "https://github.com/tigerbeetle/tigerbeetle/releases/download/${TB_VERSION}/tigerbeetle-x86_64-linux.zip" \
    && unzip /tmp/tigerbeetle.zip -d /usr/local/bin \
    && chmod +x /usr/local/bin/tigerbeetle \
    && rm /tmp/tigerbeetle.zip \
    && tigerbeetle version

COPY start.sh /start.sh
RUN chmod +x /start.sh

VOLUME /data
ENTRYPOINT ["/start.sh"]
