#!/bin/bash
set -euo pipefail

# System packages
sudo apt-get update && sudo apt-get install -y tmux

# Claude Code CLI — install from npm first (puts `claude` on PATH
# via /usr/local/share/npm-global/bin), use it to register MCP servers,
# then call `claude install` to migrate to the native build at
# ~/.local/bin. The native build is the officially-supported path
# going forward; the TUI nags on first run when it's still npm-based.
npm install -g @anthropic-ai/claude-code

# Python tooling
pip install uv

# Install moot package
pip install mootup

# Register MCP servers for Claude Code at user scope so claude finds
# them regardless of cwd (agents launch in worktrees under .worktrees/,
# not the project root). Use absolute paths to the wrapper scripts so
# they resolve from any cwd. The wrappers read CONVO_ROLE at runtime
# to look up the per-role API key from .moot/actors.json.
DEVCONTAINER_DIR="$(realpath .devcontainer)"
claude mcp add convo "$DEVCONTAINER_DIR/run-moot-mcp.sh" -s user
claude mcp add convo-channel "$DEVCONTAINER_DIR/run-moot-channel.sh" -s user

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

echo "Container ready."
