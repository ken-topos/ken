#!/bin/bash
set -euo pipefail

# System packages
sudo apt-get update && sudo apt-get install -y tmux curl

# Claude Code CLI — install from npm first (puts `claude` on PATH
# via /usr/local/share/npm-global/bin), use it to register MCP servers,
# then call `claude install` to migrate to the native build at
# ~/.local/bin. The native build is the officially-supported path
# going forward; the TUI nags on first run when it's still npm-based.
npm install -g @anthropic-ai/claude-code

# Codex CLI — standalone installer in non-interactive mode. The installer
# places `codex` in ~/.local/bin by default, which bash -lc picks up through
# the devcontainer user's standard profile.
curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh

# Python tooling
pip install uv

# Install moot package. --upgrade is load-bearing: a bare `pip install mootup`
# is a no-op against an already-satisfied requirement (how the env drifted stale
# at 0.5.10), whereas --upgrade pulls the latest each build — so future moot
# fixes (e.g. the 0.5.12 codex stranded-paste fix) land on rebuild with no
# manual version bump.
pip install --upgrade mootup

# Register MCP servers for Claude Code at user scope so claude finds
# them regardless of cwd (agents launch in worktrees under .worktrees/,
# not the project root). Use absolute paths to the wrapper scripts so
# they resolve from any cwd. The wrappers read CONVO_ROLE at runtime
# to look up the per-role API key from .moot/actors.json.
DEVCONTAINER_DIR="$(realpath .devcontainer)"
PROJECT_ROOT="$(dirname "$DEVCONTAINER_DIR")"
claude mcp add convo "$DEVCONTAINER_DIR/run-moot-mcp.sh" -s user
claude mcp add convo-channel "$DEVCONTAINER_DIR/run-moot-channel.sh" -s user

# Register MCP servers for Codex at user scope so Codex finds them regardless
# of cwd (agents launch in worktrees under .worktrees/, not the project root).
# The wrappers read CONVO_ROLE at runtime to look up the per-role API key from
# .moot/actors.json.
mkdir -p /home/node/.codex
chmod 700 /home/node/.codex
cat > /home/node/.codex/config.toml <<CODEX_CONFIG
approval_policy = "never"
sandbox_mode = "danger-full-access"

# Codex memory feature (off by default upstream). Enabled fleet-wide so Codex
# build seats generate + reuse their own per-seat memories. NOTE: this is
# SUPPLEMENTAL to the curated agent/memory/ corpus, which stays canonical
# (CLAUDE.md). Read at codex startup — a restart is required to take effect.
[features]
memories = true

# Optional: uncomment when using the Ken LLM proxy as Codex model provider.
# The proxy expects LLM_PROXY_SHARED_SECRET in the Codex process environment.
# model_provider = "moot-llm-proxy"
#
# [model_providers.moot-llm-proxy]
# name = "Moot LLM proxy"
# base_url = "http://127.0.0.1:8090/v1"
# env_key = "LLM_PROXY_SHARED_SECRET"

[mcp_servers.convo]
command = "$DEVCONTAINER_DIR/run-moot-mcp.sh"
cwd = "$PROJECT_ROOT"
env_vars = ["CONVO_ROLE", "CONVO_API_URL", "CONVO_WORKTREE"]
startup_timeout_sec = 30

[mcp_servers.convo-channel]
command = "$DEVCONTAINER_DIR/run-moot-channel.sh"
cwd = "$PROJECT_ROOT"
env_vars = ["CONVO_ROLE", "CONVO_API_URL", "CONVO_WORKTREE"]
startup_timeout_sec = 30
CODEX_CONFIG
chmod 600 /home/node/.codex/config.toml

# Migrate from the npm-installed claude to the native build. This runs
# LAST (after `claude mcp add`) because `claude install` deletes the
# npm symlink — anything calling `claude` after this point must rely on
# ~/.local/bin/claude, which `bash -lc` picks up via ~/.profile's
# standard "$HOME/.local/bin" snippet. Agent tmux sessions launch with
# `bash -lc`, so they find the native binary automatically.
claude install

# Rebind tmux prefix to Ctrl-Space. Claude Code intercepts Ctrl-B (the
# default prefix), so the usual `Ctrl-B d` detach never reaches tmux.
# Ctrl-Space is rarely claimed by TUIs and leaves readline-style editing
# bindings (Ctrl-A/E/etc.) untouched inside claude's input line.
cat > /home/node/.tmux.conf <<'TMUX_CONF'
unbind C-b
set -g prefix C-Space
bind C-Space send-prefix

# Mouse on: scroll-wheel scrolls the pane, click selects a pane/window,
# drag copies. Without this, scrollback is only reachable via the
# copy-mode keybind (<prefix> [) which is a tmux-literacy tax users
# shouldn't have to pay just to read recent output.
set -g mouse on
TMUX_CONF

# Register a /detach slash command for claude so the user can leave a
# tmux session without having to fight for the prefix key. The command
# calls `tmux detach-client`, which disconnects the terminal but leaves
# claude running in the session so `moot attach` picks up where it left
# off. User-scope so every worktree sees it.
mkdir -p /home/node/.claude/commands
cat > /home/node/.claude/commands/detach.md <<'DETACH_MD'
---
description: Detach from the tmux session (leaves claude running in the background)
allowed-tools: Bash(bash:*)
---

!bash -c 'SOCK=$(find /tmp /run -maxdepth 3 -name default -type s 2>/dev/null | head -1); if [ -n "$SOCK" ]; then tmux -S "$SOCK" detach-client; else echo "tmux socket not found"; fi'
DETACH_MD

# Context-awareness hooks (self-compact signal). Deploy the in-house scripts and
# register them in the global Claude Code settings: the statusline extracts the
# context-window %, and a PreToolUse hook nudges the self-compacting singletons
# (steward/architect/integrator/librarian) to checkpoint + compact at a clean
# seam. Role-scoped (teams get only the statusline) and fail-safe. Source of
# truth is .devcontainer/hooks/; see its README.md.
HOOKS_SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/hooks"
mkdir -p /home/node/.claude/hooks
cp "$HOOKS_SRC"/ctx-*.sh /home/node/.claude/hooks/
chmod +x /home/node/.claude/hooks/ctx-*.sh
if [ -f /home/node/.claude/settings.json ]; then
  jq -s '.[0] * .[1]' /home/node/.claude/settings.json "$HOOKS_SRC/settings-fragment.json" \
    > /home/node/.claude/settings.json.tmp \
    && mv /home/node/.claude/settings.json.tmp /home/node/.claude/settings.json
else
  cp "$HOOKS_SRC/settings-fragment.json" /home/node/.claude/settings.json
fi
echo "[post-create] context-awareness hooks installed."

echo "Container ready."
