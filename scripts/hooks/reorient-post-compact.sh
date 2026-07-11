#!/usr/bin/env bash
# Fleet-wide post-compaction / session-start re-orientation injector.
#
# Purpose: a context compaction (auto or manual /compact) drops an agent's
# role-skill BODY and memory-scope files out of context — only the CLAUDE.md /
# AGENTS.md routing block and the MEMORY.md index are re-injected automatically.
# CLAUDE.md mandates re-orienting "after every context compaction", but that was
# a discipline, not machinery. This hook makes it mechanical: on every fresh
# context (startup | resume | clear | compact) it injects a directive that forces
# the agent to re-run its orientation before doing any work.
#
# Provider-agnostic: emits the `hookSpecificOutput.additionalContext` JSON that
# BOTH Claude Code (SessionStart hook) and Codex CLI (SessionStart hook, >=v0.129)
# honor. Wired from `.claude/settings.json` and `.codex/hooks.json`.
#
# Side-effect-free and fail-safe: it only injects a reminder; it never blocks a
# tool call or a session. If the schema is ever wrong it silently no-ops rather
# than breaking a seat.

# Drain the event payload on stdin (the hosts pass event JSON here) and ignore
# it — the directive is the same regardless of source.
cat >/dev/null 2>&1 || true

cat <<'JSON'
{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"⟳ CONTEXT WAS JUST COMPACTED OR RESET. Before any other work, RE-ORIENT per CLAUDE.md / AGENTS.md — the compaction dropped your role-skill body and memory scopes from context: (1) call orientation() [convo MCP] to reconfirm your role and focus space; (2) read agent/COORDINATION.md and agent/MODELS.md; (3) invoke your role's Skill and follow it as your standing playbook — if the Skill tool does not know it, Read .claude/skills/<skill>/SKILL.md directly; (4) read your role's memory scopes under agent/memory/ (fleet + your path). The role-to-skill routing table is in CLAUDE.md. Do this FIRST."}}
JSON
