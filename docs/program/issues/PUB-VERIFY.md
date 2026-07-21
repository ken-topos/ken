---
id: PUB-VERIFY
title: "scripted-pr-automerge.sh exits 0 on a failed push"
status: ready
owner: steward
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: observed 3x, 2026-07-21 (steward)
---

`scripts/scripted-pr-automerge.sh` returns **exit code 0 when nothing was
pushed and nothing merged**. Observed three times on 2026-07-21:

| failure | what the script returned |
|---|---|
| `! [rejected] steward/work (stale info)` | exit 0 |
| `! [remote rejected] ... without 'workflows' permission` | exit 0 |
| earlier: a "merged" line during RT-PARITY that had not merged | exit 0 |

## Why this is more than an annoyance

This script **is** the merge gate. Every WP closure depends on its report,
and the Steward's §14 duty is to verify landings by content precisely
because this signal cannot be trusted. A merge tool that reports success
having done nothing inverts the one guarantee it exists to provide — and it
fails *silently*, so the natural reading is "landed."

Both 2026-07-21 rejections were recoverable and caught only because the
output was read directly rather than the exit code trusted. A Steward under
time pressure who trusted it would have reported merged work that was still
local.

## Fix

1. Propagate the real exit status of `git push` and the merge command —
   do not swallow it.
2. Add a terminal verification step: re-read `origin/main` and confirm the
   intended SHA is an ancestor, or that the expected content is present.
   Exit non-zero if not.
3. On failure print a single unambiguous line (`PUBLISH FAILED: <reason>`)
   so it survives a `tail` or a grep pipe.

Point 2 is the one that matters — it checks the **property** (the content is
on `main`) rather than a proxy (a command ran). Every other signal in this
class has failed the same way.

## Related

Under-buffering caveat: piping this script's output through `grep` block-
buffers its poll lines, so failures can be invisible until it exits. Read
the output file directly.
