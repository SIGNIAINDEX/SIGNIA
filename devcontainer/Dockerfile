# syntax=docker/dockerfile:1
FROM mcr.microsoft.com/devcontainers/base:bookworm

ARG USERNAME=vscode

# ---- OS deps ----
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    git \
    unzip \
    jq \
    python3 \
    python3-pip \
    openssh-client \
    && rm -rf /var/lib/apt/lists/*

# ---- Rust toolchain ----
# Keep the install in the image so devcontainer startup is fast.
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup component add rustfmt clippy

# ---- Solana CLI (optional but convenient) ----
# Pin a version so builds are reproducible.
ARG SOLANA_VERSION=1.18.22
RUN curl -sSfL https://release.solana.com/v${SOLANA_VERSION}/install | bash -s -- -u /usr/local/solana -s v${SOLANA_VERSION}
ENV PATH="/usr/local/solana/active_release/bin:${PATH}"

# ---- Anchor (optional) ----
# Anchor version pin is maintained by your repo; this just installs a recent stable anchor.
# If you want strict pinning, set ANCHOR_VERSION.
ARG ANCHOR_VERSION=0.30.1
RUN cargo install --locked anchor-cli --version ${ANCHOR_VERSION} || true

# ---- Node (provided by devcontainer feature) ----
# Ensure npm is available and corepack is enabled.
RUN corepack enable || true

# ---- Non-root user ----
RUN usermod -aG sudo ${USERNAME} || true

# ---- Workspace ----
WORKDIR /workspaces/signia
