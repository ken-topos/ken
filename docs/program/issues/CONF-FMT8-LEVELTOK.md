---
id: CONF-FMT8-LEVELTOK
title: "FMT8's fixture is unproducible: the row demands a 'genuine level-token fixture' but the lexer has no Level/Label token kind and never will under endpoint (b)"
status: draft
owner: spec-enclave
size: S
gate: none
depends_on: [SPEC-IDENT-BLESSED]
blocks: []
github: null
origin: "Raised by the conformance-validator's block on SPEC-IDENT-BLESSED successor b3468101 (2026-07-27); both carriers independently verified by the Steward at origin/main d6df571e. Ruled out of that WP's scope in evt_7egdvdf68p7a4 and filed here."
---

⛔ **`status: draft` is deliberate.** The enclave is finishing
`SPEC-IDENT-BLESSED`; `depends_on` records a **scheduling** dependency —
the endpoint (b) ruling this node rests on lands with that WP.

## The measurement

`conformance/surface/formatting/seed-canonical-format.md:304-314`, row
`surface/formatting/l-identifier-is-not-a-level-token`:

- **given:** `fn keep_l (l : Nat) : Nat = l` beside *"a **genuine level-token
  fixture** using the canonical level role"*
- **expect:** `RED-UNTIL-BUILT (B2/B3/C)` — *"only the parsed level token
  prints `ℓ`"*

⛔ **The lexer has no Level/Label token kind.** A `Token`-scoped grep for either
in `crates/ken-elaborator/src/lexer.rs` returns **nothing**. Both `ℓ` and ASCII
`l` produce `Token::Ident("level")`; the formatter preserves whichever raw
source lexeme was written.

⇒ **The fixture the row requires cannot be constructed**, and under the ruled
endpoint (b) it never will be — the absence of a distinct level token *is* the
ruling. **FMT8 as written can never go green.**

## ⭐⭐ Why this is a defect class, not a stale line

⚠ **A `RED-UNTIL-BUILT` row whose fixture is unproducible is byte-identical, to
any reader, to a row that simply has not been built yet.** It sits red forever
and reads as *pending*. Nothing in the corpus distinguishes "waiting on work"
from "waiting on something that will never exist."

⭐ Same class as `SEC1-IFC-R3`'s synthetic `Disproved` verdicts: a row whose
evidence can never be real, sitting in a corpus that reports it as merely
outstanding.

⇒ ⭐ **The valuable deliverable is not this one row — it is the sweep.** How
many other rows in the formatting seed (and the other seeds) name a fixture the
landed lexer/formatter cannot produce? That census is the point of this node.

## Scope note for whoever frames this

- ⛔ **Not foldable into `SPEC-IDENT-BLESSED`** — that WP is spec-only and
  `conformance/**` is outside its edit authority. Ruled `evt_7egdvdf68p7a4`.
- ⚠ The `SPEC-IDENT-BLESSED` successor carries a **forward note** naming this
  node, so the contradiction is recorded rather than silent. ⛔ That note is
  not a fix and must not be read as one.
- ⭐ The row's *intent* is sound and worth preserving: it discriminates a
  raw-byte over-firing canonicalizer from a correct one. The repair is to
  re-express that discrimination in terms of operands the lexer **can**
  produce — source `ℓ` vs source `l` vs source `level`, all resolving to one
  binding while each round-trips to its own spelling. ⛔ Do not simply delete
  the row; that discards a real discriminator.
