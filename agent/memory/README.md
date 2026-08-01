# Agent memory — the fleet's curated lessons corpus

Durable, **checked-in, human-legible** operational lessons for the agent
federation. This is the harness-agnostic home for knowledge that used to live in
Claude Code's private per-project memory store (invisible outside the tool and,
as it turned out, **shared** across every worktree agent). Migrating it here
makes it reviewable in git and readable by any harness — Claude Code today,
Codex next (which loads `AGENTS.md` + skills, not a tool-specific memory file).

The migration from the private store is complete; `MIGRATION-LOG.md` is the
coverage audit — every source lesson from the old store appears there exactly
once, with its disposition (kept, merged into a kept file, dropped, or excluded
as personal) and the reasoning.

## What belongs here

- **Operational lessons** — the "gotcha → why → how-to-apply" genre: anti-
  footguns, hard-won discriminators, coordination rules. (Was `type: feedback`
  in the old store — the bulk and the point of this corpus.)
- **Live references** — pointers to durable external/internal resources, kept
  only while current.

## What does NOT belong here

- **Stale/transient state** — campaign status, in-flight WP notes, "what's
  active now." That lives in the tracker
  (`docs/program/IMPLEMENTATION-PROGRESS.md`) and moot handoff/checkpoint docs,
  not here.
- **Operator/personal identity** — who the operator is, their timezone, personal
  preferences. This stays **out of git** (a clone shouldn't carry it); it lives
  in the operator's personal, non-repo memory (`~/.codex/AGENTS.md` / Claude
  Code's private store), never in the tracked corpus.
- **Short always-applies *rules*** — those fold directly into `AGENTS.md` or the
  relevant skill prose; only the indexed *lessons* live here.

## Scope hierarchy (how an agent finds its slice)

Roles are separated by **worktree**, not by subdirectory — so scoping rides the
**skill/playbook architecture** (which every harness loads via role identity),
not directory-nested `AGENTS.md`. Each scope is a folder here, and the
`AGENTS.md` / skill that a role already loads carries a one-line reference into
the scopes that role reads.

```
agent/memory/
  fleet/            every agent (coordination law, mention discipline,
                    closure-verification, clean-room, compaction)
  enclave/          T1 enclave: steward, architect, spec-author,
                    spec-leader, conformance-validator
  build/
    leaders/        all team leaders
    qa/             all QA
    implementers/   all implementers
  teams/
    kernel/  verify/  language/  runtime/  ergo/  foundation/  doc/
  roles/
    steward/  architect/  librarian/  …
```

An agent loads **its path + ancestors**. Examples:

| Role | Loads |
|---|---|
| `kernel-qa` | `fleet` + `build` + `build/qa` + `teams/kernel` |
| `steward` | `fleet` + `enclave` + `roles/steward` |
| `librarian` | `fleet` + `roles/librarian` + **`teams/doc`** (it is the doc team's QA) |

**Build-team seats have no `roles/<team>-<seat>` scope** — a build seat's
identity is covered by the function scope (`build/qa`) crossed with the team
scope (`teams/kernel`), which is the whole point of the two parallel branches.
`roles/` is for the **singleton** roles that no function scope describes.
**`CLAUDE.md`'s routing table is the authority** if this document and it ever
disagree.

**An absent or empty scope is a normal state, not a defect or a missing
read.** It means that scope has not yet paid for a lesson. Every scope named in
the tree above exists as a folder with a `README.md` stating its charter, so a
role that follows its routing never lands on a path that isn't there — but
finding only the `README.md` is the expected outcome for a young scope, and it
is not a signal to go looking elsewhere.

Function is the primary axis; `teams/` is a parallel branch a role also pulls,
so "all leaders" (`build/leaders`) and "all kernel" (`teams/kernel`) coexist
with no duplication. **A lesson lives at the broadest scope where every reader
must apply it.**

**Directory placement is authoritative — it IS the routing.** A lesson's
audience is exactly the directory it sits in, so a role that reads its path +
ancestors has read everything applicable to it. **No reader scans the rest of
the corpus**, and no reader inspects frontmatter to discover lessons filed
elsewhere.

**The `scope:` frontmatter key is redundant metadata, not routing.** It
appears on most files and every occurrence restates that file's own directory.
It confers nothing. **To widen a lesson's audience, move the file** — there
is no tag-based alternative.

## Wiring (how it gets loaded)

- **fleet** → referenced from root `AGENTS.md` (every agent reads it).
- **enclave / build / build-* / teams/* / roles/*** → referenced from the
  matching skill (`ken-build-leader`, the team overlay `agent/teams/<team>/`,
  the role skill) that the role already loads at orientation.

Nothing depends on a harness-specific auto-memory feature. Codex's generated
`~/.codex/memories/` (off by default, thread-generated, local recall only) is
**supplemental**, never the source of truth — this corpus is.
