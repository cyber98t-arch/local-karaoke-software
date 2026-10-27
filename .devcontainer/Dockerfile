FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

ARG NODE_VERSION=22
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
       nodejs python3 python3-venv python3-pip pkg-config libssl-dev \
       libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf \
       libasound2-dev ffmpeg git-lfs \
    && npm install --global @openai/codex \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

