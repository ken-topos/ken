---
id: SPEC-CLOSURE-BOUNDARY
title: "Revise the runtime value spec to remove the closure-identity inconsistency and state the closure/value boundary with minimum constraints on the implementation"
status: merged
owner: spec
size: M
gate: none
depends_on: []
blocks: [RT-FNSPLIT-B2V]
github: null
origin: Operator directive 2026-07-26 ("revise the spec to remove inconsistencies and to meet the mission with minimum constraints on the implementation"), carried at evt_5qr8c07a1tbc9. Implements Architect ruling evt_2093yfkhgwyez / dec_3b1r19v59v20y (RESOLVED) that persistent content-addressed closure identity is a REVISABLE representation commitment, not intrinsic function semantics. Root cause surfaced by research evt_31rkr18ghy5c4 (operator-prompted) after six consecutive Architect production blocks on RT-FNSPLIT-B2V. Steward-filed per COORDINATION §2.
---

> ## ▶ THE DISPATCH IS THE BRIEF — `evt_5qr8c07a1tbc9`
>
> Read it before this file. The **operator directive is quoted verbatim there**
> and is the standard the work is judged against.

## ⭐⭐ The operator directive, verbatim

> **"For the enclave on this topic: revise the spec to remove inconsistencies
> and to meet the mission with minimum constraints on the implementation."**

⛔ **The third clause is load-bearing.** The ask is the **weakest spec that
still meets the mission** — **not** "resolve the contradiction and keep the
strong reading." Where the spec can say **less** and still meet the mission, it
**should say less**.

**That clause has a measured cost behind it.** A constraint stronger than the
mission required, faithfully implemented, produced **six consecutive Architect
production blocks** on `RT-FNSPLIT-B2V`. Every ruling was correct; the premise
underneath them was never examined until the operator asked.

## The inconsistency — real, and it predates the ruling

`spec/40-runtime/41-values.md` contradicts itself **today**:

| location | says |
|---|---|
| `:35-46`, `:99-105`, `:189` | closures are content-addressed identity-bearing heap values, canonically encoded `(code_id, full captured environment)`, **memcmp-exact** identity |
| `:175-180` | the precise closure boundary is **X2 tuning, not a semantic commitment** |

⛔ **Both cannot be true.** Removing this is inside the directive regardless of
the ruling.

## Scope

- `spec/40-runtime/41-values.md` — compound population, closure tag/encoding,
  equality generalization, **the OQ-7 table**
- `spec/40-runtime/45-native-backend.md` — shared value-model wording
- the coupled **content-addressing design** doc

⛔ **Not** a general value-model redesign. Remove inconsistencies, minimize
constraints — do not re-architect.

## What the Architect ruled (six binding clauses)

Summarized; `evt_2093yfkhgwyez` is authoritative for wording.

1. Ordinary `Closure` is **runtime-local and opaque** — callable, with no
   Ken-visible structural equality, `DecEq`, ordering, canonical hash, slot
   identity or provenance.
2. Closure equality is **absent, not extensional**. An aggregate containing a
   closure does **not** gain structural `DecEq` via a hidden runtime identity.
3. Persistence/serialization is **transitively closure-free** — rejected before
   publication, ⛔ never silently substituted by a pointer/ordinal/digest/handle.
4. Cross-artifact exchange only **within one live runtime domain**, as an
   unforgeable typed opaque handle/trampoline with explicit owner/lifetime.
5. Stable callable identity survives **only** as an explicit
   `StaticCallableRef`-class value with **no** captured environment. ⛔
   Empty-capture optimization must not silently produce a serializable value.
6. A durable higher-order value is a **separate `FrozenClosure` abstraction**.

**Retained:** content-addressed ordinary data, static-code-identity vs
dynamic-activation factoring, verified native equivalence, artifact binding
where observable. Higher-order equivalence is tested by **applying callables and
comparing ground observations**, never by comparing closure slots.

## Acceptance criteria

**`AC-S1` — the inconsistency is GONE, not papered over.** The amended text has
**one** statement about closure identity/equality. ⛔ A clarifying note that
leaves both readings present does not discharge this — the superseded text stays
operative and is the one positioned to be obeyed.

**`AC-S2` — every constraint retained is JUSTIFIED AGAINST THE MISSION.** For
each retained requirement on closures, state **which mission property fails
without it**, citing `docs/PRINCIPLES.md`. ⛔ *"The current spec says so"* is not
a justification — that is the reasoning this node exists to retire.

**`AC-S3` — minimum-constraint statement.** Name explicitly what the spec
**stopped requiring**, so the relaxation is auditable rather than implicit.

**`AC-S4` — conformance blast radius, stated EARLY.** The `conformance-validator`
enumerates every case asserting closure structural equality, closure
serialization, or closure-containing canonical graphs, **before the wording
settles.** ⛔ Not a post-hoc re-derivation.

**`AC-S5` — the OQ-7 table is reconciled**, not left describing the superseded
position.

**`AC-S6` — `45-native-backend.md` agrees with `41-values.md`.** The shared
value-model wording must not re-assert the strong reading from the other side.

**`AC-S7` — a clause the Architect ruled may be challenged.** ⭐ If the enclave
concludes a ruled clause is **still stronger than the mission needs**, it
**says so** rather than implementing it. The directive invites this, and not
saying it is the failure mode that cost six blocks.

## Standing

- ⛔ **Until this lands, the OLD SPEC TEXT IS THE REPOSITORY'S CURRENT
  STATEMENT.** A WP frame amendment **cannot** outrank these files — which is
  why this is a spec node and not a frame fold.
- Wrap markdown at 80 columns. Report an unpushed ref and keep going; the
  Steward pushes.

## ✅ MERGED 2026-07-26 — PR #982, exact `0ccca4c5`

`origin/main` `dd9f4e76` → **`33f0695f`**. Verified by **blob identity**, not
ancestry — this repo **squash-merges**, so the approved SHA is correctly *not* an
ancestor of `main`. `41-values.md` `b2ec8cb8`, `73-conformance.md` `27747cc1`,
`06-execution.md` `6ad4c461`, `SOURCE-ATTESTATIONS` `c03c7fe7`,
`toolchain.md` `f64437b0` all byte-identical on `main`.

**It took three candidates and a fourth descendant.** `10e29f48` was **rejected**
by the Architect; `26cfb5db` was **blocked** by the CV on one normative residual
in `73 §5` item 1; `7bfd744f` was approved and `dec_5dws8kw685gj3` resolved — and
then **could not merge**. The publisher's library-currency gate refused the merge
result because six revised spec files are **cited sources**. `0ccca4c5` is the
Librarian's revalidation descendant, carried by a **fresh** Decision
`dec_44qez0er55ytz` (resolved by the Architect; `7bfd744f`'s approval did not
carry).

⭐ **The corpus was genuinely falsified, not merely stale.** Two derived pages
were asserting things the revision makes false and were repaired
(`06-execution.md` claimed universal *"identical values"* **twice**;
`agents/core/toolchain.md` gave an unqualified direct comparison). Seven further
consumers were opened at their cited anchors and held.

⛔ **`AC-S4` IS THE DEFECT THIS NODE LEAVES BEHIND.** It scoped the blast radius
to **conformance**, and the **library corpus was also a consumer**. Nothing in the
criteria named it, which is why a real falsification surfaced at the publisher
instead of during review — after *two* exhaustive T1 review passes that were each
correct within their scope. **Future spec nodes touching cited sources must name
`library/` in the blast-radius AC.** Promoted to `steward.md` §2c step **7c**.

⚠ **What this does NOT do:** it settles the *contract*, not the implementation.
`crates/` still contradicts it — see [[RT-VALUE-TOTALITY]], which measured that
`canonical.rs:182` still encodes closures memcmp-exact and that `Value`'s derive
list grants `Closure` the structural equality, ordering and hashing this boundary
forbids. **Retros outstanding: spec-leader, spec-author, conformance-validator.**
