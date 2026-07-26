---
id: SPEC-ALIGN-A1
title: "Scope the landed-code authority convention out of the normative status blocks, and census every private-mechanism constraint against its conformance consumers before relaxing any of them"
status: ready
owner: spec
size: M
gate: none
depends_on: []
blocks: [SPEC-ALIGN-B1]
github: null
origin: Operator dispatch 2026-07-26 on the conformance-validator's mission/spec overspecification advisory with research's prior-art addendum — "category 1 (spec edits) should be dispatched to the enclave for cleanup". Advisory captured verbatim at docs/program/spec-mission-overspecification-advisory.md; program dispositions at docs/program/14-spec-mission-alignment-campaign.md. This node is Track A of the campaign whose umbrella is SPEC-MISSION-GROUNDING (now active — its AC-M1 is discharged by docs/MISSION.md and its first pass produced the advisory; its AC-M2/AC-M3/AC-M6 remain open and are NOT subsumed here). Steward-filed per COORDINATION §2; Steward owns the frame and AC/control placement. Generalizes SPEC-CLOSURE-BOUNDARY (merged 2026-07-26) from closures to the rest of the spec.
---

> ## ⛔ READ THIS FIRST — the operator called this "cleanup, low risk, mostly
> ## mechanical". **The measurement says otherwise, and that changed the node.**
>
> Every private-mechanism family the advisory nominates for relaxation is
> **asserted by at least one conformance row** — FNV-1a addressing, the 0.70
> load factor, same-slot dedup, bignum tag `0x01`, minimal-limb encoding,
> canonical indentation, the 4 MiB page size. Measured on `9410d7b8`; the table
> is in the campaign doc §2.
>
> ⇒ **A relaxation is a COUPLED `spec/` + `conformance/` change**, and relaxing
> the spec alone is *worse* than the over-specification: the two then disagree
> and the suite wins in practice.
>
> ⇒ **So A1's deliverable is the census and an honest stop list, not an edit
> count.** ⛔ **If the census returns an empty edit set, A1 is COMPLETE and
> successful.** Do not manufacture relaxations to look productive.

## ⭐ The one item that needs no census, and it is the most valuable

`spec/40-runtime/44-capacity.md:20` — *"Where the F4 design and the landed code
diverge, the **landed code is normative**."*

⚠ **Read the scope before repairing it.** The advisory renders this as the spec
declaring implementation normative. Literally accurate, **but** the sentence
sits inside an X2 grounding block labelled *"perishable-frame, K2c-s2 rule"*,
and what it arbitrates is **two internal drafts** — F4 design prose versus the
landed K3 store. It is **not** a global declaration that implementation outranks
specification.

⇒ **The defect is a drafting-convention leak, not an inverted principle.** A
perishable draft-reconciliation rule is sitting untimed and unscoped in a
normative chapter's status block — the exact place an independent implementer
reads for the authority rule. Research's objection stands **at that reading**:

> never make "landed code is normative" the authority rule. That reverses the
> spec/implementation relationship and excludes an independent conforming
> implementation by construction.

⇒ **Repair = scope or retire the convention** (name the two drafts it
arbitrates, and its expiry). ⛔ **Not** "reverse a principle" — the spec does not
assert that principle globally, and a repair written as if it did will overshoot.

**Second site, same shape:** `conformance/runtime/capacity/seed-capacity.md:44`
closes a divergence note with *"conformance follows the landed code"* — the
artifact whose job is to be able to **fail** the implementation deferring to it.

⛔ **Do not lose the information while fixing the sentence.** That block records
three real divergences (per-`space` bare-hash index rather than a process-wide
`(root, hash)` index; reclamation drops page buffers rather than `madvise`;
single-writer resize rather than lock-free). Those are facts worth keeping — the
defect is the *authority claim* wrapped around them, not the divergence record.

## The classification that decides everything else

Research's four classes. Only class 4 is straightforward over-specification:

| class | what it fixes | relaxable from mission text alone? |
|---|---|---|
| 1. language semantics | evaluation order, equality, accepted recursion, effects | ⛔ no |
| 2. interoperability protocol | exact bytes/versions so independent producers and consumers agree | ⛔ no — versioned profile |
| 3. security binding | a repeated identity field defeating substitution/rollback/confusion | ⛔ no — per-edge threat argument |
| 4. **private mechanism** | hash policy, page size, probing, copy-vs-share | ✅ **yes — A1's whole scope** |

⛔ **A1 touches class 4 ONLY.** A candidate that turns out to be class 1, 2, or
3 is a **stop**, routed to the campaign's Track B or Track C — not an edit, and
not a judgement call to make inside this node.

## Scope

✅ **In:** the authority repair at both sites; the constraint → consumer census
over the class-4 candidates; relaxation of **only** those constraints whose
consumer set is empty; the stop list for the rest.

⛔ **Out:** every Track C fork (runtime `unknown`, `Ord`/`Map` key equality,
capability revocation, SCT termination, instance coherence, prover portfolio,
logical `space`, purity reverse errors) and every Track B protocol profile
(Ward/ITF, package + executable envelopes, named supply-chain products). ⛔ **No
`crates/` change. No conformance row moved.** Moving a row is a
conformance-granularity decision that belongs to the Architect.

## ⛔ DO NOT RELAX — carried, not cited

Small auditable kernel; kernel rechecking of every claimed certificate;
totality and predictability by default; explicit partial and foreign
boundaries; exhaustive obligation extraction; honest
`proved`/`tested`/`delegated`/`unknown`; explicit effects, capabilities, IFC,
provenance, trust; loud failure over silent weakening; no promotion of Ward,
test, or monitor results to `proved`.

**Mechanisms may be simplified. These guarantees may not.** A simplification
campaign is exactly when they get shaved by accident.

## Frame

**`docs/program/wp/SPEC-ALIGN-A1-private-mechanism-census.md`** — carries the
census method, the two measured control cases, the deliverables, and
`AC-A1`–`AC-A7`.

## Concurrency

`RT-FNSPLIT-B2E` is live in `crates/ken-runtime/`. **Contention-free by path**
(A1 is `spec/` + `docs/`), ⚠ **not semantically free** — the store family is
what `B2E`/`B2F` build against, which is a further reason it lands in the stop
list rather than the edit list.
