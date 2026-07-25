---
id: DOC-GATE-NEEDLE
title: "schema-gate controls assert on a needle the test itself supplied, so one constraint class is fully vacuous"
status: ready
owner: verify
size: S
gate: none
depends_on: [DOC-W2]
blocks: []
github: null
origin: adversary finding L1+L2 on DOC-W2 (d3b9f36c), side thread thr_2seh2bm1kr5mh evt_2sk7s27prrhdn, 2026-07-25. Steward-filed (agents cannot create tracked work per COORDINATION §2); Steward triage = CONFIRMED DEFECT, independently re-grounded at the two cited lines. The under-scoped blast-radius claim in the merge notification is a Steward defect, recorded below.
---

> ## ⛔ OPERATOR RULING 2026-07-25 — HELD. Fleet stays FNSPLIT-only. SETTLED.
>
> I put the concurrency fork to the operator with this WP as the strongest case
> for widening (a **live** false-green in a shipped gate, an **idle** Verify ring
> with retros already in, and file sets verified disjoint from everything in
> flight). **Ruling: hold. The fleet remains strictly single-threaded on
> `RT-NATIVE-FNSPLIT`.**
>
> ★ **This also settles the standing ambiguity, and that is the durable part:**
> the doc-track concurrency exception is **DOC-ONLY**. Its stated basis —
> contention-free-ness — explains *why doc got the exception*; it is **not** a
> general licence for any contention-free WP to run concurrently. ⇒ **Proving
> your file sets disjoint does NOT earn a slot.**
>
> ⛔ **Do not re-ask.** A settled operator ruling is a fixed input. This WP stays
> `ready` until the FNSPLIT chain closes and a slot genuinely opens.
>
> ⚠ **The frame below is complete and shovel-ready** — nothing about it is
> pending. The only thing missing is a slot.

> ## The assertion cannot fail, because the needle is the label the test passed in
>
> `DOC-W2` shipped a schema gate whose headline claim is that **a
> declared-but-unenforced constraint cannot stay green**. That claim is
> **false for one of six constraint classes today, and structurally
> unsupported for all six.**
>
> `library_documentation_gates.rs:3583-3591` passes `constraint` — the
> constraint's own *name* — as the `location` argument to
> `schema_violations`. Every violation message is formatted
> `"{location}: <kind> violation: …"`, so every message produced by that call
> **begins with the needle**. The assertion is then
> `violations.iter().any(|message| message.contains(constraint))`.
>
> ⇒ **`contains(constraint)` is true for any violation whatsoever.** The test
> asks "did *something* fail?", not "did *this constraint* fail?"

## Why this is a confirmed defect and not a style objection

The Adversary measured it with a **discriminating pair** from a clean tree, each
mutation restored byte-identically:

| mutation to the validator | full suite |
|---|---|
| delete the `type` block from `schema_violations_with_refs` | **31 passed; 0 failed** |
| delete the `const` block | **FAILED** (`planted const violation did not fail…`) |

The pair is what makes this precise rather than a blanket "the test is weak":
the suite genuinely catches **five of six**. It was also proved independently of
the `type` case — relabelling the `minItems` row to plant an
`additionalProperties` violation instead, keeping the label, still passed.

**Steward re-grounding (independent, not taken on report):** confirmed
`message.contains(constraint)` at `:3589` with `constraint` passed as the
4th/`location` argument at `:3587`, and confirmed the second occurrence at
`:3617`, whose location `"module minimum"` **literally contains its own needle**
`"minimum"`. Both are on `origin/main` at `d3b9f36c`.

**Two compounding causes — fix both, because either alone would be survivable:**

1. **Systemic:** the needle is the caller-supplied label, so the assertion has
   no discriminating power at any row.
2. **What makes it bite today:** `schema_version = "1"` violates **both**
   `const: 1` and `type: integer`. With `type` enforcement deleted, the `const`
   violation still fires and satisfies the prefix match.

⚠ **The other five rows are sound only because their mutations happen to isolate
a single constraint** — not because the assertion discriminates. **A future
constraint whose natural mutation also trips a neighbour inherits this hole
silently.** That is the part worth fixing; the vacuous `type` row is only the
instance that surfaced.

## Deliverables

**D1 — make the assertion test the message BODY, not the label.** Both
occurrences (`:3583-3591` and `:3610-3617`). Use a neutral location and require
the constraint *kind*, so the matched substring cannot be something the caller
supplied:

```rust
let violations = schema_violations(&pack_schema, &value, &pack_schema, "pack fixture");
assert!(
    violations.iter().any(|m| m.contains(&format!("{constraint} violation"))),
    "planted {constraint} violation did not fail at that constraint: {violations:?}"
);
```

**D2 — close the `additionalProperties` gaps (adversary L2, verified latent not
live).** `:3083` nests the check inside `if let Some(properties) =
schema.get("properties")`, so a subschema declaring `additionalProperties: false`
with **no** sibling `properties` accepts every field. All four current
occurrences in both shipped schemas do have a sibling `properties` — hence
latent — but the nesting is wrong regardless. Separately: `additionalProperties`
is on the `SUPPORTED` list (`:3113`), so the legal
`"additionalProperties": {"type": "string"}` form is **neither enforced nor
rejected** — the code only matches `Bool(false)`.

⇒ **State the true guarantee in the code, precisely:** fail-closed covers
*unknown keywords*, **not** *supported keywords in unsupported forms*. Either
extend enforcement to the schema form, or reject the unsupported form loudly.
**Do not leave a third, silent outcome.**

**D3 — a regression control for the systemic cause, not just the instance.**
D1 fixes two call sites by hand; nothing stops the shape returning at the 3rd.
There are **14 `schema_violations` call sites** in this file. Add a check that
fails if an assertion's needle is derivable from the `location` that call
passed — or restructure so `location` is not caller-chosen at the assertion
sites at all. **A hand-enumerated fix to a category needs a structural
closure**; grep the shared tell and derive the count rather than listing two
lines.

## Acceptance criteria

- **AC-1 — the currently-passing mutation must REDDEN.** Delete the `type`
  enforcement block from `schema_violations_with_refs`; the suite **must fail**.
  This is the exact discriminator that returns `31 passed; 0 failed` today, so
  it is the one criterion that proves the fix. Restore byte-identically and
  verify with `git diff --quiet`.
- **AC-2 — positive control retained.** `:3553`'s assertion of **zero**
  violations on a valid fixture before any mutation still holds. *A negative
  check passes for any reason; without this control a "reddens correctly" claim
  is worthless.* Do not weaken it to make AC-1 pass.
- **AC-3 — the label-swap attack fails.** Reproduce the Adversary's independent
  probe: relabel one row to plant a *different* constraint's violation while
  keeping the original label. After the fix that row **must fail**. Currently it
  passes.
- **AC-4 — all six constraint classes still redden individually**, each mutation
  applied at its **natural production site** in the validator (not at an
  artificial injection point), each restored byte-identically.
- **AC-5 — D3's closure is structural.** Name the predicate and derive the
  affected count from a grep over all 14 call sites. "We fixed the two we knew
  about" does not discharge this.
- **AC-6 — no-regression in CI.** The workspace build, `--locked`, and the
  conformance suite are **CI's** job on GitHub. ⛔ Do **not** run
  `--workspace`/`--locked` locally (COORDINATION §12); test with
  `scripts/ken-cargo` scoped to `-p ken-cli --test library_documentation_gates`.

## Guardrails — do not reopen

- ⛔ **Do not touch the mutation table itself.** The six planted violations are
  correct; the *assertion* is the defect. Rewriting the table would hide the
  bug rather than fix it.
- ⛔ **Do not weaken AC-1 to a smaller mutation.** Deleting the whole `type`
  enforcement block is the measured discriminator; a narrower mutation may
  redden for an unrelated reason.
- The recursion, `$ref` cycle detection, keyword-audit descent, and
  unknown-keyword fail-closed behaviour were **attacked and cleared** — the
  Adversary confirmed enforcement genuinely descends through `properties`,
  `items`, and `$ref`, and that every constraint keyword in both shipped schemas
  has a sibling `type` guard. **This WP does not revisit any of that.** It is
  narrowly the assertion's discriminating power plus the two
  `additionalProperties` gaps.

## ⚠ A Steward defect recorded here, because it changes how this file is bounded

My merge notification described
`crates/ken-cli/tests/library_documentation_gates.rs` as **NEW at 4007 lines**.
**It is MODIFIED, +1356/−1** (2652 → 4007). Verified: the blob exists at
`5015bc71` at 2652 lines.

⇒ Calling it new **scopes the blast radius too narrowly** — the 24 pre-existing
tests live in that same file, including the `VALIDATION_GATES` registry, the
`document-kind` row, and the status-population invariant from the
`DOC-GATE-*` lineage. Anyone reviewing this WP must treat the file as
**shared, long-lived gate infrastructure with four merged WPs of history**, not
as a fresh DOC-W2 artifact. The line count came from `wc -l` on the merged blob,
which answers "how big is it now" and says **nothing** about whether it is new —
I reported a derived measurement as a provenance claim.
