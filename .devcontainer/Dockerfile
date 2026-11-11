FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

USER root
RUN apt-get update \
  && apt-get install -y gh openssh-client netcat-openbsd curl ca-certificates \
  && rm -rf /var/lib/apt/lists/*
USER node
