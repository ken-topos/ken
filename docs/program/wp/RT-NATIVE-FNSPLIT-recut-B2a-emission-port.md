# RT-NATIVE-FNSPLIT — Boundary B2a: make the semantic plane load-bearing for emission

**WP frame (Steward). Owning team: Runtime. Size: L. One branch, one merge
Decision.** Parent: `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md`. Predecessor:
`RT-NATIVE-FNSPLIT-recut-B1-semantic-ir.md` (**landed `5554b33f`**).

> ## ⛔ THIS IS A BEHAVIOUR-PRESERVING PORT. IT IS NOT THE CENSUS.
>
> **B2 is split at the same kind of seam the Architect used to split Boundary B
> into B1/B2** — and for the reason your own B1 retro gave: *"keep
> representation checkpoints separate from a retained emission port."*
>
> - **B2a (this WP)** — emission consumes the semantic plane. The observable
>   behaviour of the compiler does **not** change. The differential suite is the
>   proof.
> - **B2b (next, framed separately)** — the full emission census, the finite
>   differences, and the explicit growth verdict that answers the operator's
>   scaling gate.
>
> ⛔ **Do not report growth metrics as an acceptance argument here, and do not
> tune for them.** A census taken while the emitter is still moving measures a
> moving target, and a reviewer cannot isolate a regression from a redesign.
> **If you find yourself optimizing, you are in B2b's scope — stop and say so.**

## Objective

Make the closed semantic-IR plane that B1 built **load-bearing**: body emission
reads the plane as its source of truth instead of the existing
whole-configuration path. **Behaviour is preserved exactly** — the set of
programs that compile, what they compute, trap identity and order, and every
diagnostic are unchanged.

## The seam — precise, and verified on `main` at `5015bc71`

`lowering/core.rs:33-35` builds the plan and **drops it**:

```rust
// Boundary A of RT-NATIVE-FNSPLIT: close and validate the factored static
// graph before Cranelift sees any semantic body. Phase 2 will consume this
// plan for emission; until then the existing emitter remains unchanged.
let static_transition_plan = plan_static_transition_graph(expr, &declarations)?;
```

⇒ **That comment is this WP's charter.** The plan is built, validated
(`static_transition.rs:947-948`, reached from `finish()`), and then ignored.
B2a makes `static_transition_plan.semantic` the thing emission walks.

⚠ **`lowering/core.rs` is 6201 lines.** This is the largest single surface in
the recut. Read the escalation section before you begin: the expected failure
mode here is *not* a wrong answer, it is an unbounded diff.

## Fixed inputs — SETTLED, do not reopen

Each is decided. **If one is false against the landed code, say so and
escalate — do not build around it** (the recut's Perishability clause; it is
what caught the `VReg::MAX` staleness).

1. **B1's plane is the representation.** Six-opcode source/control grammar,
   exhaustive, **no wildcard arm**; positional origins/descriptors/programs/
   captures with body-free edge ranges; activation frames never enter the
   builder. ⛔ Do not extend the grammar to make a port easier — a new opcode is
   a hard-stop, not a judgement call.
2. **`build_semantic_plane` stays the sole builder** and stays exhaustive.
3. **The outer helper inventory is FULL.** `fixed_k = 8` against cap `8`, zero
   headroom. ⛔ **If this port needs one more outer helper on any static source,
   STOP — that is a hard-stop, not a cap to raise.**
4. **#34 rides in the plane, not as a patch.** The **source-return-owned resume
   edge/node** is present in B1's shape and must stay explicit. ⛔ `Terminal`
   stays un-overloaded — it means *no continuation*; this state has a live
   continuation owned indirectly by an exact source-return descriptor. Option
   2's duplicate direct W is **rejected**.
5. **`RT-PLANNER-ATTRIB-K` landed** (`5015bc71`): a K-exceeded trip is now
   `planner_error` → `PlannerInvariant`, not `Unsupported`. Emission-side
   failures you add follow the same discipline — **invariant violations are
   compiler bugs; only genuine finite representation limits are capacity.**

### Retain (do not rebuild, do not regress)

Exported root + bounded deferred Cranelift functions · exact normal/abrupt edges
· trap sequencing and exact trap identity/order · affine reservation/bind/spend
authority · graph sealing · completion witnesses · the **W/T** producer-wrapper
vs ultimate-tail distinction · linked cleanup/source topology.

### Replace

Whole-configuration specialization · vector-shaped and flattened residual keys ·
recursive `Debug` serialization as identity · helper identity coupled to
environment / control / layout **contents**.

★ **All four of those are one predicate — *a dynamic property naming static
code*.** That predicate is the whole reason the recut exists. **Every emission
key you write is checked against it.**

## Deliverables

> ### ⭐ D0 — THE ORIGIN CARRIER, MOVED HERE FROM B1R (re-slice, 2026-07-25)
>
> **B1R hard-stopped (#5, `evt_3sx56kzx7z9q`; Architect confirmed
> `evt_37sc5gv2yfxr8`) because the carrier cannot be added without editing
> `lowering/core.rs`.** That edit is licensed *here* and nowhere else, so the
> carrier lands in the same diff as the removal of the old authority — which is
> better than the two-step I originally framed: **one authority replaces another
> in one reviewable change, instead of two authorities coexisting across two WPs.**
>
> **Retained closures, declaration bodies, source-machine work items, and
> deferred emission records carry the fixed-width origin** for static body
> identity, alongside the already-ruled dynamic environment/store handles.
>
> **Measured blast radius — nine carriers, compiler-enumerated, not grepped:**
>
> | file | construction (E0063) | pattern (E0027) | total |
> |---|---|---|---|
> | `lowering/core.rs` | 13 | 16 | **29** |
> | `lowering/mod.rs` | 14 | 14 | **28** |
>
> `core.rs` **construction** sites: `1983 2009 2028 2041 2059 2283 2327 2366 2802
> 2834 4211 4226 5626` · **pattern** sites: `265 517 532 638 2195 2346 2380 2500
> 2783 2816 3551 3575 3691 4246 4261 4343`
>
> ★ **The 13 constructions are the expensive half** — a pattern absorbs a field
> with `..`, a construction must *produce* a real origin. ✅ **`SourcePrefixTemplate`
> (all 14 deferred-emission variants) is `mod.rs`-only, zero `core.rs` sites** — the
> cheap end, if you want a first increment.
>
> ### ⚠ THREE FINDINGS THAT WERE IN NEITHER FRAME — each changes the work
>
> 1. **There is NO existing seam to thread an origin through.** `core.rs:35`
>    builds the plan; **`core.rs:204` unconditionally `drop(static_transition_plan)`**;
>    nothing between them consumes it. ⇒ `core.rs:35` is the **only** point where
>    an origin is obtainable. "Built and dropped" is a **mechanism**, not a
>    characterization.
> 2. **`StaticOriginId` is `pub(super)` in `semantic_ir` and re-exported
>    nowhere** (`semantic_ir.rs:15`). ⇒ A lowering-side carrier **requires widening
>    a planning-internal identity out to `crate::cranelift_backend`** — a
>    **visibility/boundary change**, not a field addition. Treat it as such and
>    say what the new boundary is.
> 3. ⛔ **`lowering` ALREADY has a different `origin`.** `RecursorProducerOriginId`
>    is spelled `origin:` on records in these same families — **86 occurrences in
>    `mod.rs`, 44 in `core.rs`.** A second `origin` on the same records is a
>    **same-word/two-concepts trap on a chain whose entire predicate is about
>    identity provenance.** ⇒ **Give the carrier an unambiguous name.** Do not
>    call it `origin`.
>
> ### ✅ ARCHITECT-SPECIFIED at the B2a seam (`evt_37sc5gv2yfxr8`) — settled, do not re-decide
>
> - **Widen `StaticOriginId` ONLY to `pub(in crate::cranelift_backend)`** — not
>   beyond the backend. A wider visibility is a hard-stop, not a convenience.
> - **Name the field `static_origin`.** ⛔ Never bare `origin` beside
>   `RecursorProducerOriginId`.
> - **Each construction obtains the origin from the plane-driven source
>   walk/seam** — ⛔ never from pointer, content, clone order, or activation.
> - ⛔ **The 29 compiler-enumerated sites are a FINITE SURFACE INVENTORY, not
>   permission to absorb unrelated lowering changes.** The unreviewable-diff stop
>   still applies.
> - ⛔ **Do NOT keep a `mod.rs`-only carrier subset as a first increment.** Ruled
>   against explicitly: an unused partial carrier buys no representation closure,
>   creates a transitional second-authority surface, and splits one invariant
>   across two reviews. **Carrier and removal land together.**

Body emission walks `static_transition_plan.semantic`. The old
whole-configuration path is **removed, not left dormant beside the new one** —
two live paths is the condition under which the next reader cannot tell which is
authoritative.

### D2 — one static transition, one emitted unit

Each emitted body unit corresponds to exactly one static transition in the
plane. State the correspondence explicitly and make it checkable.

### D3 — the four width invariants, as assertions in the tree

⛔ **Corrected metric — the original frame's version was WRONG and would have
rejected a correct design.** Assert exactly these:

1. **No flattened env / pending / path member in helper identity.**
2. **Constant ID / node payload width.**
3. **Affine total persistent nodes.**
4. **At most affine logical chain depth.**

⛔ **Do NOT require the logical chain length itself to be constant.** Logical
persistent-chain **depth** for environment / pending / path **may grow Θ(n) and
that is SOUND** — the helper/frame carries one constant-width ID into the
persistent store rather than the chain itself.

### D4 — the differential suite is the equivalence proof

Exact **normal / abrupt / trap / join / affine** differential coverage,
interpreter vs native, asserting identical observable behaviour across the port.
⭐ **This is the acceptance argument for B2a.** Not a census, not a timing — a
demonstration that the port changed nothing observable.

### D5 — negative controls, each red at a NAMED artifact

Carry B1's standard, which your ring reported returned more than it cost: small
reversible mutations, each failing at its **exact named invariant**, each
restored **byte-identically** and verified with `git diff --quiet`. Cover at
minimum: a mis-keyed emitted unit, a dropped source-return resume edge, a body
emitted from the old path, and an identity that reads dynamic content.

⭐ **PLUS the two carrier controls moved here from B1R with D0** — they are
defined *on* the carrier, so they were not constructible in a plane-only slice:

- **Cross-wire one retained closure body origin.**
- **Replace a fixed-width origin carrier with a `RuntimeExpr`/pointer lookup.**

⛔ **The second one is the chain's whole predicate as an executable test** — it
asserts that recovering static identity from a runtime pointer *fails loudly*
rather than working. If it does not redden, the carrier is not authoritative.

## Acceptance criteria

- **AC-1 — behaviour is unchanged, demonstrated not asserted.** D4's
  differential suite green across all five categories. Trap identity **and
  order** preserved. State what you ran and the counts.
- **AC-2 — the old path is gone.** Grep and show zero live callers of the
  removed whole-configuration emission path. ⭐ **State your window** — which
  files, and whether it includes `cfg(test)` and definitions. *Two of my own
  counts on `ATTRIB-K` were wrong because the window silently included a `fn`
  definition and a doc file; your QA's carry says enumerate the full live
  population first, and it applies to me as much as to you.*
- **AC-3 — the four D3 invariants hold, each with its own assertion.** One
  assertion per invariant, each independently falsifiable. ⛔ A single composite
  check discharges none of them.
- **AC-4 — `fixed_k` is still `8,8,8,8,8` against cap `8`,** the pairwise-equal
  census row survives, and `MAX_HELPERS_PER_STATIC_SOURCE` is unchanged.
- **AC-5 — no new opcode, no wildcard arm.** `semantic_ir.rs`'s grammar is
  unchanged and still exhaustive; show it.
- **AC-6 — D5's controls each reddened at a named artifact,** each restored
  byte-identically.
- **AC-7 — no regression.** `scripts/ken-cargo test -p ken-runtime` — the
  **full** crate suite, no filter (a reifier/minted-shape change ripples to
  sibling observation tests that a targeted run cannot see). ⛔ **Workspace,
  `--locked`, and conformance are CI's job, never local** (COORDINATION §12).
- **AC-8 — ⛔ NO growth claim.** Do **not** state an exponent, a ratio, or a
  fitted curve. If you measured something incidentally, report it as an
  observation explicitly labelled *not an acceptance argument*. **The verdict is
  B2b's.**

## ⭐ SYMPTOM INVENTORY — armed (operator-directed, 2026-07-24)

**The Architect appends one line per hard-stop, before it rules; at the 3rd
entry it must answer whether the entries share a predicate** (architect playbook
§1b, steward §5a-ii).

```text
SYMPTOM INVENTORY (append only; never rewritten)
NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, …
1. retained body selection — keyed on cloned RuntimeExpr pointer identity
```

⛔ **Seeded with the held chain's entries deliberately** — compare a new entry
against these rather than discovering it fresh:

```text
HELD CHAIN (closed, retained as the worked example)
1. whole-configuration specialization        — keyed on runtime configuration
2. vector-shaped / flattened residual keys   — keyed on residual contents
3. recursive Debug serialization as identity — keyed on serialized state
4. helper identity coupled to env/control/layout contents — keyed on contents
PREDICATE (named at the recut) = a dynamic property must not name static code
```

★ **A new entry that reduces to that SAME predicate is not a new defect — it is
evidence this port is incomplete.** Say so rather than ruling it.

## Escalation — when to hard-stop rather than improvise

**Hard-stop, do not improvise**, if: the port needs a **new opcode** or a
wildcard arm · it needs a **9th outer helper** on any static source · a fixed
input above is **false against the landed code** · the plane cannot express a
retained semantic and you are tempted to widen a key with dynamic content ·
`Terminal` looks like the natural home for the source-return resume state.

**Cadence: the recut chain's hard-stop count is 3; the next Research pull is
#6** (architect §1a). **A review fold is not a hard-stop.** The Steward holds the
count of record.

⚠ **The diff-size failure mode is the likely one here.** 6201 lines of emitter
and a behaviour-preserving mandate is exactly the shape that produces an
unreviewable candidate. If the diff outgrows what a reviewer can isolate,
**stop and tell me** — splitting again is cheap and my framing is what would
have been wrong, not your execution.

## Contention

**None at kickoff.** Scope is `crates/ken-runtime/**` only. The doc ring's
`DOC-W2` is live on `library/`, `agent/`, and `crates/ken-cli/tests` — disjoint,
and the `library/SOURCE-ATTESTATIONS` ledger attests
`crates/ken-runtime/src/cranelift_backend.rs` (blob `8508a01c`), **not** the
planning or lowering files. ⚠ **If this port edits `cranelift_backend.rs`
itself, tell me** — that crosses the ledger axis and I re-derive the consumer
population before you land.

## Perishability

Every current-state claim here — line numbers, sizes, which paths are live —
was written against **`5015bc71`** on 2026-07-25 and is **perishable**.
**Re-verify each at pickup; anchor on predicates and symbol names, not line
numbers.** My anchors were stale in four places on the previous WP and the ring
caught it before cutting a branch; that is the expected outcome, not an
exception.
