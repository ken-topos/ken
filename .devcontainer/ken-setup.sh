#!/usr/bin/env bash
# ken dev tooling — the Rust verified-language layer that neither the moot
# template nor the devcontainer features provide. Runs at container onCreate
# (after the `rust`/`github-cli` features installed rustup/cargo/clippy/rustfmt
# and gh; before the moot post-create.sh registers Claude Code + MCP).
#
# What lives where:
#   - Rust toolchain, clippy, rustfmt ........ `rust` devcontainer feature
#   - gh (Integrator) ........................ `github-cli` feature
#   - tmux, Claude Code, mootup, uv, MCP ..... moot post-create.sh
#   - everything below ....................... here
set -euo pipefail

echo "[ken-setup] system build deps + SMT solver..."
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  build-essential pkg-config libssl-dev cmake \
  z3

# sccache — the shared compiler cache (COORDINATION §12 build-slot model). The
# build's .cargo/config or ken-env sets RUSTC_WRAPPER=sccache; we just install
# the binary. cargo install compiles it once at onCreate (a few minutes); swap
# for a pinned prebuilt release if that latency ever matters.
echo "[ken-setup] sccache (shared build cache)..."
if ! command -v sccache >/dev/null 2>&1; then
  cargo install sccache --locked || echo "[ken-setup] WARN sccache install failed (non-fatal)"
fi

# cvc5 — optional second SMT backend for WS-V. The prover targets Z3 directly
# (strategy §WS-V); enable cvc5 here only if a workstream needs it:
#   CVC5_VER=1.2.0
#   curl -fsSL -o /tmp/cvc5 \
#     "https://github.com/cvc5/cvc5/releases/download/cvc5-${CVC5_VER}/cvc5-Linux-x86_64-static" \
#     && sudo install -m755 /tmp/cvc5 /usr/local/bin/cvc5

echo "[ken-setup] versions:"
rustc --version 2>/dev/null || echo "  rustc: MISSING (rust feature?)"
cargo --version 2>/dev/null || true
z3 --version 2>/dev/null || echo "  z3: MISSING"
sccache --version 2>/dev/null || echo "  sccache: not installed"
gh --version 2>/dev/null | head -1 || echo "  gh: MISSING (github-cli feature?)"
echo "[ken-setup] done."
