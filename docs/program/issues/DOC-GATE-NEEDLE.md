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

> ## ✅ RELEASED BY THE OPERATOR 2026-07-26 — build it now.
>
> **Operator, 2026-07-26 ~11:4xZ, verbatim:** *"the implementation teams are
> quiescent. fix the DOC-GATE-NEEDLE issue. It doesn't matter to me how."*
>
> The slot the 2026-07-25 hold was waiting for is open: `RT-FNSPLIT-B2V` merged
> (#1014) and **every implementation ring is idle with its retros in.** ⇒ This WP
> is released to Team Verify. `RT-FNSPLIT-B2F` is the FNSPLIT frontier and is
> **not** yet kicked, so this is not competing with a live lane.
>
> ### ⚠ WHAT THE SUPERSEDED HOLD SAID, AND WHICH HALF SURVIVES
>
> The 2026-07-25 ruling **held** this WP and, in doing so, settled a standing
> ambiguity. **The hold is spent; the ambiguity ruling is NOT.** They must not be
> discarded together:
>
> - ⛔ **STILL BINDING:** the doc-track concurrency exception is **DOC-ONLY**. Its
>   basis — contention-free-ness — explains *why doc got the exception*; it is
>   **not** a general licence for any contention-free WP to run concurrently. ⇒
>   **Proving your file sets disjoint does NOT earn a slot.** This WP did not earn
>   its slot by being disjoint; it was **granted** one by the operator when the
>   fleet went quiescent.
> - ✅ **SPENT:** *"stays `ready` until the FNSPLIT chain closes"* and *"do not
>   re-ask"*. The operator reopened it directly. ⛔ Nobody needs to re-ask, and
>   nobody should cite the old hold to refuse this kickoff.
>
> ⭐ **Why this block was EDITED rather than annotated:** the previous text said
> *"do not re-ask, this WP stays `ready`"*, positioned at the top of the frame.
> A reader who found that and stopped would have refused the kickoff on the
> authority of a superseded ruling. **A later note saying a deliverable is
> obsolete does not replace the deliverable — the superseded text stays operative
> and is the one positioned to be obeyed.**
>
> ⚠ **The frame below is complete and shovel-ready**, and its locators were
> **re-verified against current `origin/main` on release**: the blob for
> `crates/ken-cli/tests/library_documentation_gates.rs` is `7415e7b2` at **both**
> `d3b9f36c` (where the finding was measured) and current `main` — **byte-identical,
> so every line number cited below is still exact.** One count was wrong and is
> corrected in `D3`/`AC-5`.

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
2. **What makes it bite today:** the `type` row's planted value trips a
   **neighbouring** constraint as well, so the prefix match is satisfied by the
   neighbour even when `type` enforcement is gone.

   > ⛔ **CORRECTED BY `RULING R1` — the original wording of this clause was
   > FALSE.** It said `schema_version = "1"` violates **both** `const: 1` and
   > `type: integer`. Verified on base `57a05000`:
   > `library/agents/schemas/pack.schema.json` declares
   > `"schema_version": {"const": 1}` with **no sibling `type` at all**, so there
   > was never a `type` constraint on that field to violate. ⇒ The row could not
   > discriminate `type` under **any** assertion, and no fix to the assertion
   > could have made it. See `RULING R1` for the replacement operand.

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
Add a check that fails if an assertion's needle is derivable from the `location`
that call passed — or restructure so `location` is not caller-chosen at the
assertion sites at all. **A hand-enumerated fix to a category needs a structural
closure**; grep the shared tell and derive the count rather than listing two
lines.

> ⛔ **CORRECTED ON RELEASE 2026-07-26 — this frame previously asserted "there are
> 14 `schema_violations` call sites" and that number is NOT REPRODUCIBLE.**
> Re-measured on the byte-identical blob `7415e7b2`, and the decomposition is the
> point, because "call site" was never defined:
>
> ```
> 19  occurrences of the string `schema_violations`
>  2  fn DEFINITIONS                          :2883, :2956
>  4  internal/recursive calls INSIDE them     :2889, :2975, :3052, :3074
> 13  call sites in TEST functions             :3359 :3418 :3554 :3582 :3610
>                                              :3622 :3639 :3655 :3673 :3694
>                                              :3711 :3728 :3751
> ```
>
> ⇒ **13, not 14, under the only reading that matters for `D3`** (a site where a
> caller chooses a `location` and an assertion then picks a needle). ⚠ **Do not
> take 13 on this frame's authority either** — it is stated here so it is
> *checkable*, not so it can be inherited. `AC-5` requires you to derive it.
>
> ⭐ **Why a wrong count in an AC is worse than no count:** an implementer who
> reads *"a grep over all 14 call sites"* has been handed the answer, so the grep
> becomes a formality that confirms a number rather than a measurement that
> produces one. **That is the exact defect this WP exists to fix, in the frame
> that describes it** — an assertion whose needle was supplied by the caller.

## ⭐ `RULING R1` — the `type` row's operand moves (Architect, 2026-07-26)

**Decision `dec_7s4xdhrkg8prp`, `resolved`, `resolved_by=agt_37reqftfe6g00`** —
verified from the object, not from the channel reporting it. Raised by
@verify-implementer (`evt_6p0r46xnwkmg2`) as an implementation fork; ruled at
`evt_4gyta88b9y8k9`. Transcribed here **because an in-thread ruling is not a
durable deliverable** — terminal QA must bind a fetchable governing operand
rather than an exception living in a thread.

**THE RULING — option (b). Change only the `type` row's mutation:**

```
purpose = "Write Ken"   →   purpose = 1
```

**Steward re-derivation on base `57a05000`, independently and not on report:**

| operand | measured | verdict |
|---|---|---|
| `pack.schema.json` `.properties.schema_version` | `{"const": 1}` | ⛔ **no sibling `type`** |
| `pack.schema.json` `.properties.purpose` | `{"type":"string","minLength":1}` | ✅ declares a type |
| `schema_violations_with_refs` order | `:44` reads `type` → `:57` emits `type violation` → **`:59` `return`**, before `:68`'s `minLength` | ✅ isolates `type` |

⇒ **The frame's premise was false, not merely awkward.** `schema_version` had no
`type` constraint to violate, so **no** repair to the assertion could ever have
made that row discriminate `type`. The row was untestable by construction.

**Rejected alternatives, and why they matter beyond this WP:**

- ⛔ **(a) add `"type":"integer"` to the shipped schema** — changes the shipped
  library contract to repair a test oracle, with no independent product
  requirement, and widens a test-only WP into production reach.
- ⛔ **(c) infer type-from-`const`** — JSON Schema declares no such rule.
  Inventing validator semantics would make diagnostic attribution disagree with
  the schema.

**Scope of the exception is NARROW: only the `type` row moves.** The six-class
table, `D1`'s body match, `D3`'s closure, and `AC-1`'s whole-type-block deletion
are all unchanged.

⚠ **`AC-1`'s "31 passed / 0 failed" is HISTORY, not a prediction.** That figure
was measured against the **old** operand. Under `R1` the `purpose` row is exactly
what makes `AC-1` satisfiable: with the `type` block deleted, no `type violation`
is emitted, so a body-matching assertion fails as intended.

⭐ **What this cost, and where the defect was positioned.** Two of the frame's
false claims — the `const`+`type` premise and the "every keyword has a sibling
`type` guard" clause — were **mine**, and the second sat inside a bullet whose
whole function was to tell the reader *this was attacked and cleared, do not
revisit*. **A false premise carried by a sentence instructing the reader not to
check it is the most durable kind.** The implementer escalated instead of
inventing a rule, which is the correct behaviour and the reason this was caught
in minutes rather than at QA.

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
- **AC-5 — D3's closure is structural, and the count is DERIVED not quoted.**
  Name the predicate, then derive the affected count yourself from a grep over
  every `schema_violations` call site, **stating the reading of "call site" you
  used** (this frame's own count was wrong until release because that reading was
  never written down — see `D3`). ⛔ "We fixed the two we knew about" does not
  discharge this, and ⛔ neither does reproducing the number in `D3`: report what
  your grep returns, and if it disagrees with `D3`'s 13, **your measurement wins
  and you say so.**
- **AC-6 — no-regression in CI.** The workspace build, `--locked`, and the
  conformance suite are **CI's** job on GitHub. ⛔ Do **not** run
  `--workspace`/`--locked` locally (COORDINATION §12); test with
  `scripts/ken-cargo` scoped to `-p ken-cli --test library_documentation_gates`.

## Guardrails — do not reopen

- ⛔ **Do not touch the mutation table itself — WITH ONE NARROW EXCEPTION granted
  by `RULING R1`.** The six planted violations are correct; the *assertion* is
  the defect. Rewriting the table would hide the bug rather than fix it.
  **EXCEPTION: the `type` row's operand moves** from `schema_version = "1"` to
  `purpose = "Write Ken" → purpose = 1`, because the original operand declared no
  `type` at all. ⛔ **Only that one row moves.** The six-class table, `D1`'s body
  match, `D3`'s closure and `AC-1`'s whole-type-block deletion are unchanged.
- ⛔ **Do not weaken AC-1 to a smaller mutation.** Deleting the whole `type`
  enforcement block is the measured discriminator; a narrower mutation may
  redden for an unrelated reason.
- The recursion, `$ref` cycle detection, keyword-audit descent, and
  unknown-keyword fail-closed behaviour were **attacked and cleared** — the
  Adversary confirmed enforcement genuinely descends through `properties`,
  `items`, and `$ref`. **This WP does not revisit any of that.** It is narrowly
  the assertion's discriminating power plus the two `additionalProperties` gaps.

  > ⛔ **ONE CLAUSE OF THIS BULLET WAS FALSE AND `RULING R1` STRUCK IT.** It also
  > asserted that *"every constraint keyword in both shipped schemas has a sibling
  > `type` guard."* Verified false on base `57a05000`:
  > `"schema_version": {"const": 1}` has **no sibling `type`**. ⚠ The clause sat
  > inside a bullet whose subject is *"attacked and cleared … does not revisit"* —
  > so a false premise was carried by a sentence telling the reader **not to
  > check it.** That is the most effective possible position for a wrong claim,
  > and it is why the implementer had to escalate rather than being able to read
  > the answer here.

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
