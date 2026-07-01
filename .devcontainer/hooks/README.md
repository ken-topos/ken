# Context-awareness hooks (in-house)

Makes a Claude Code agent aware of its own **context-window usage %** so it can
self-decide when to compact — the "checkpoint-and-seam" discipline
(`agent/playbooks/federation/steward.md §2d`, `.../architect.md §3`). Built
in-house (bash + `jq`, **no external dependency**) rather than vendoring a
third-party tool: for a project whose thesis is a small TCB and supply-chain
re-check, the code that runs on every tool call across the fleet must be ours,
reviewed, and version-controlled.

## Mechanism

Claude Code has **no** direct way to hand context usage to a hook. The only
surface that carries it is the **statusLine** stdin JSON
(`.context_window.used_percentage`, CC ≥ 2.1.196). So:

- **`ctx-statusline.sh`** (statusLine command) — extracts `used_percentage`,
  stashes it in a **session-scoped** temp file (`/tmp/cc-ctx-<session_id>.pct`),
  and prints `ctx N% · Model` (a passive readout for the human, too).
- **`ctx-nudge.sh`** (PreToolUse hook) — reads that file and, on crossing **up**
  into the 70% (seam-check) or 85% (imminent) band, injects a one-shot
  checkpoint-and-seam reminder via `hookSpecificOutput.additionalContext`.
  Debounced per band; **resets when usage drops** (post-compaction) so the next
  climb re-nudges.

**Role-scoped:** the nudge fires **only for the self-compacting singletons**
(`steward`, `architect`, `integrator`, `librarian` — they compact via
`request_context_reset`). Team roles compact via `moot` at WP boundaries, so
they are gated out (they still get the harmless statusline readout, no nudge).
Role is read from `CONVO_ROLE`, falling back to the worktree name in `.cwd`.

**Fail-safe:** every path in both scripts exits 0; the hook emits stdout **only**
when actually nudging (empty stdout = the tool proceeds untouched). Malformed or
missing input is a silent no-op — a hook that runs on every tool call must never
wedge one.

## Enabling it for an agent

Point the agent's Claude Code settings at the two scripts. Per-agent (local,
non-committed) via `<worktree>/.claude/settings.local.json`:

```json
{
  "statusLine": { "type": "command",
    "command": "/home/node/.claude/hooks/ctx-statusline.sh" },
  "hooks": { "PreToolUse": [ { "matcher": "*", "hooks": [
    { "type": "command",
      "command": "/home/node/.claude/hooks/ctx-nudge.sh", "timeout": 5000 } ] } ] }
}
```

Claude Code **hot-reloads** this mid-session (no restart needed). The scripts
live at `/home/node/.claude/hooks/` at runtime; this directory is the
version-controlled source — deploy with `cp .devcontainer/hooks/ctx-*.sh
/home/node/.claude/hooks/ && chmod +x /home/node/.claude/hooks/ctx-*.sh`.

## Status / rollout

- **Active (pilot):** the 4 self-compacting singletons, via per-worktree
  `settings.local.json` (hot-reloaded). The role gate means the config is safe
  to widen.
- **Follow-ons:** (1) wire the `cp`-deploy into `post-create.sh` so a rebuilt
  container reprovisions the scripts; (2) after the model transition + a
  validation window, widen the settings to the whole fleet (the role gate
  already makes the nudge correct everywhere — teams get only the statusline).
