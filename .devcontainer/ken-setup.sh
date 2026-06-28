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

# LLM proxy package — the build tiers (GLM/DeepSeek) route through a local
# proxy (mootup_harness_sdk.llm_proxy); see run-llm-proxy.sh. Installed to the
# same system python the moot post-create uses for `pip install mootup`.
echo "[ken-setup] installing mootup-harness-sdk (llm proxy)..."
pip install --quiet mootup-harness-sdk || echo "[ken-setup] WARN harness-sdk install failed (non-fatal)"

# Per-role provider routing (HYBRID), keyed on CONVO_ROLE (set by moot's
# launcher): the build tiers point at the local proxy; the Opus enclave stays
# direct on the Anthropic subscription. Written to a sourced file so each
# agent shell self-configures at startup.
# NOTE: validate on first `moot up` that the agent shell sources this — if
# moot execs `claude` without a login/interactive shell, fall back to a
# per-worktree .claude/settings.json `env` block instead.
cat > /home/node/.ken-agent-routing.sh <<'ROUTING'
# ken hybrid LLM routing — branches on CONVO_ROLE.
case "${CONVO_ROLE:-}" in
  spec-author|conformance-validator|steward|architect)
    # enclave: clean subscription (OAuth) — no proxy, no API key
    unset ANTHROPIC_BASE_URL ANTHROPIC_API_KEY ANTHROPIC_AUTH_TOKEN ;;
  "")
    : ;;  # no role assigned — leave ambient env untouched
  *)
    # build tiers: through the local proxy, presenting the shared secret
    export ANTHROPIC_BASE_URL="http://127.0.0.1:8090"
    [ -r /home/node/.secrets/llm-proxy-secret ] && \
      export ANTHROPIC_API_KEY="$(cat /home/node/.secrets/llm-proxy-secret)" ;;
esac
ROUTING
for rc in /home/node/.bashrc /home/node/.profile; do
  grep -q ken-agent-routing "$rc" 2>/dev/null || \
    echo '[ -r /home/node/.ken-agent-routing.sh ] && . /home/node/.ken-agent-routing.sh' >> "$rc"
done

echo "[ken-setup] versions:"
rustc --version 2>/dev/null || echo "  rustc: MISSING (rust feature?)"
cargo --version 2>/dev/null || true
z3 --version 2>/dev/null || echo "  z3: MISSING"
sccache --version 2>/dev/null || echo "  sccache: not installed"
gh --version 2>/dev/null | head -1 || echo "  gh: MISSING (github-cli feature?)"
echo "[ken-setup] done."
