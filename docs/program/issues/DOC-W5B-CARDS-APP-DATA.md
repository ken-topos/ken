---
id: DOC-W5B-CARDS-APP-DATA
title: "Wave 5 slice 2 — apply the settled card format to Application (3) and Data (11): fourteen complete cards"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-W5A-CARD-FORMAT]
blocks: [DOC-W5C-CARDS-CAPABILITY]
github: null
origin: "Steward 2026-08-02 under section 2a-bis — framed as DOC-W5A's immediate successor so the frontier stays one release ahead. Measured at origin/main = a8df4b7b."
---

# Wave 5 slice 2 — Application and Data, against a settled format

Fourteen cards. The format is [[DOC-W5A-CARD-FORMAT]]'s and is not re-designed
here.

## What the frame settles that this node must not be read without

**The format is settled; this slice applies it.** If the format does not fit a
package, that is a finding about the format and it routes — it is not licence to
vary it. A card that quietly departs makes the corpus inconsistent in the way
that stays invisible until someone reads two cards side by side.

**`AC-3` is the row that decays under volume.** `none-declared` may appear only
where the canonical fences are genuinely empty for that class, and each one owes
the fence it was read from. By card ten this is the row most likely to be filled
in from memory, which is why it carries a per-row control rather than a
per-slice one.

**Why these two areas.** Data is the most law- and proof-dense and Application
the most end-user-shaped, so together they stress the `law` and
`effect/capability` rows from opposite directions before Capability's 19 go
through in slice 3.

The frame is `docs/program/wp/DOC-W5B-CARDS-APP-DATA.md`.
