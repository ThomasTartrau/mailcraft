FROM rust:1.91.0-bookworm AS builder

RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

RUN cargo binstall --no-confirm \
    cargo-nextest@0.9.122 \
    cargo-llvm-cov@0.6.23 \
    sccache@0.9.1

RUN cargo install cargo-audit --locked

FROM rust:1.91.0-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    lld \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt llvm-tools-preview

COPY --from=builder /usr/local/cargo/bin/cargo-nextest /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/cargo-llvm-cov /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/sccache /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/cargo-audit /usr/local/cargo/bin/

ENV RUSTC_WRAPPER=sccache
