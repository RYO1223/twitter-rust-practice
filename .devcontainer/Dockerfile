FROM mcr.microsoft.com/devcontainers/rust:latest

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

RUN apt update && \
    apt install -y cmake libclang-dev 