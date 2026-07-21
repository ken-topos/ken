---
id: CI-TRACKER-GATE
title: "Wire the issue-tracker schema + regeneration gate into CI"
status: ready
owner: operator
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: publisher push rejection, 2026-07-21 (steward)
---

**Needs the operator — the Steward cannot land this.** The scripted
publisher's GitHub App token has no `workflows` permission, so any change
under `.github/workflows/` is remote-rejected:

```
! [remote rejected] steward/work -> steward/work (refusing to allow a
  GitHub App to create or update workflow `.github/workflows/ci.yml`
  without `workflows` permission)
```

The prepared job is saved as a patch and appends an `issue-tracker` job to
`.github/workflows/ci.yml` running two gates on every PR:

- `scripts/check-issue-schema.sh` — every `docs/program/issues/*.md` has all
  required frontmatter, a valid `status`, an `id` matching its filename, no
  duplicate `id`, and every `depends_on`/`blocks` reference resolving.
- `scripts/gen-progress.sh --check` — `IMPLEMENTATION-PROGRESS.md` is
  current with respect to the issue corpus.

## Why it matters

`IMPLEMENTATION-PROGRESS.md` is **generated**. Without the regeneration gate
nothing prevents a hand edit, or a stale committed copy after an issue
changes — which is the exact failure mode that let the previous tracker grow
to 2.23 MB. The gate is what makes "generated cannot drift" a property
rather than a convention.

## Two ways to close

1. Grant the publisher app `workflows` permission (also unblocks every
   future CI change through the scripted path).
2. Apply the patch by hand, out of band.

Option 1 is the durable one — otherwise every CI change is a manual step
outside the merge path, and the gap will be rediscovered.

Until this lands the gates are **advisory**: run both scripts by hand before
publishing tracker changes.
