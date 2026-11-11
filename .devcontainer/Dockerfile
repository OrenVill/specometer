FROM mcr.microsoft.com/devcontainers/javascript-node:1-22-bookworm

USER root
RUN curl -fsSL https://cursor.com/install | bash \
  && AGENT_DIR="$(ls -d /root/.local/share/cursor-agent/versions/* | head -1)" \
  && mkdir -p /usr/local/share/cursor-agent \
  && cp -a "$AGENT_DIR" /usr/local/share/cursor-agent/current \
  && ln -sf /usr/local/share/cursor-agent/current/cursor-agent /usr/local/bin/agent \
  && apt-get update \
  && apt-get install -y gh openssh-client netcat-openbsd curl ca-certificates \
  && rm -rf /var/lib/apt/lists/*
USER node
RUN bash -lc "source /usr/local/share/nvm/nvm.sh && nvm install 24 && nvm alias default 24"
RUN mkdir -p /home/node/.devbox
