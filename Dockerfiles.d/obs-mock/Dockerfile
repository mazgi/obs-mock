# syntax=docker/dockerfile:1
FROM rust:1.88-bookworm

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
  && rm -rf /var/lib/apt/lists/*

ARG UID=1000
ARG GID=1000
RUN groupadd --force --gid ${GID} developer \
  && useradd --uid ${UID} --gid ${GID} --create-home --non-unique developer

RUN cargo install cargo-watch --locked

USER developer

WORKDIR /workspace
