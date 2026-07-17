# B1-EXACT — exact-denotation Σ derivation for the assumption-boundary export (general B1 I3 soundness prerequisite)

- **State:** FRAMED — ready to route. A **general soundness correction** to the
  Verify-owned B1 checked-export transaction. Prerequisite of **PX7-T-ERR** (which
  rebases on it) and therefore of **PX8-T** (held behind the corrected B1
  authority). NOT a new capability; does **not** redesign Q/P/T/G.
- **Owner:** **Team Verify** (leader `agt_37reqqf16g800` / implementer
  `agt_37reqfz3jnw00` / qa `agt_37reqtacftr00`) — B1 is Verify-owned (WS-B, 03
  §B1). Grounded by the Architect's mechanism ruling `evt_21ads3za1z4a9`.
- **Size:** L. **Risk:** High (soundness change to the trust-adjacent export
  emitter; shifts export hashes across B1/B2/B3).
- **Branch:** `wp/b1-exact-denotation-alphabet` off `origin/main @ 2e588367`.
  **ONE branch, ONE Decision.**
- **Route:** **Architect §14** (he designed it; independent soundness review) **+
  CV** for any `spec`/`conformance` frozen-hash reconciliation (B2/B3 consumers)
  **+ G-Ward-seam.**

## The defect (Architect `evt_21ads3za1z4a9`, grounded on `2e588367` + red `9eee201c`)

`emit_export` accepts a caller-supplied `EffectRow` and copies it into
`BehavioralExport.alphabet`. Every repository caller is an acceptance/test caller;
`ElabEnv.effect_rows` stores the **declared `visits`** row, and §36 deliberately
checks only `ρ_inf ⊆ ρ_decl` (**declared headroom is legal**). So Σ is a
caller-supplied over-approximation, not the denotation's perform nodes. I3 (§71)
requires Σ to be **exactly** the denotation perform-node alphabet for **every**
assumption-boundary export. Foundation's information-theoretic proof (red
`9eee201c`): a metadata-performing and a non-performing producer hand `emit_export`
an identical input tuple, so no downstream predicate can recover the fact. The
PX7-T resource obligation merely **exposed** a general B1 defect; resources own no
derivation rule.

## Fixed inputs — Architect-ruled, DO NOT REOPEN (cite `evt_21ads3za1z4a9`)

### The sanctioned mechanism — one checked-target export transaction (7 requirements)

Add **one checked-target export transaction** in the B1 producer. Its authority is
the selected, kernel-admitted target and its closed target-neutral denotation —
**not** a caller-supplied row and **not** machine code.

1. **Select one checked target by stable identity** and compute the same
   target-rooted reachable **declaration closure** used to build its denotation.
   Unused admitted siblings are outside the population.
2. **Obtain a finite, backend-neutral denotation graph** for that closure — the
   checked interaction-tree graph directly, **or** the existing target-neutral
   lowered `RuntimeProgram` when available. Must be **before Cranelift/linking**
   and **identity-bound** to the selected checked target.
3. **Traverse the closed graph exhaustively** and collect the **structural static
   alphabet**: exactly the perform-node signatures occurring in the target
   denotation, across every retained case/continuation. This is **static**, not
   dynamic path-feasibility. Follow calls, recursion, higher-order callbacks, and
   selected branches through the closed graph; **exclude** unused declarations and
   declaration-only headroom.
4. **Carry a typed canonical `PerformNodeInventoryV1`** (name illustrative), bound
   at least to the target stable symbol **and** the checked-core/denotation
   semantic identity. Its constructor lives **inside** the producer transaction —
   callers must not be able to hand `emit_export` an arbitrary provenance-free set.
5. **Assemble `BehavioralExport.alphabet` only from that inventory.** The existing
   `BTreeSet<String>` wire projection may remain for byte compatibility, but its
   strings must be an **exhaustive canonical projection of typed perform
   identities**. Host ops use the identity-checked operation decoder + an
   exhaustive `HostOpV1::ALL` mapping; the inventory must **also** represent
   **non-host L5 perform signatures** — this is **not** a resource-only `HostOpV1`
   list.
6. **Keep `ρ_decl` separate.** `ρ_inf ⊆ ρ_decl` is unchanged and may be rechecked
   as a consistency assertion, but `ρ_decl` is **never** a source or fallback for Σ.
7. **Fail closed** before export serialization/hashing if the selected target
   cannot yield a closed exact inventory — unresolved/imported/dynamic nodes may
   **not** be widened to the declared row or to an operation family.

**Public API consequence (intentional):** replace production use of the present
hand-fed `emit_export(..., EffectRow, ...)` with the checked transaction. A
lower-level assembler may remain **module-private/test-only**, taking the
producer-created inventory; it must not remain an authoritative public path that
can mint an unbound Σ. The producer must **validate the inventory's target/semantic
binding before hashing** (a typed set without that binding just moves the
hand-authored assertion one parameter right).

### Scope — general B1 I3 correction

- **B1 owns** the corrected derivation and export hash.
- **B2 and B3 remain consumers** of that one corrected B1 alphabet — they must not
  derive a second set. Their acceptance suites and any frozen hashes must be
  **reconciled** (they serialize/consume B1 exports), but **no independent B2/B3
  alphabet mechanism** is authorized.
- An old export whose declared row had headroom contains **orphan Σ members** and
  was never I3-authoritative — its hash is **allowed and required to change**.
  Exports whose old supplied row already equalled the exact inventory **preserve**
  canonical bytes and hash.
- **Do NOT** opportunistically redesign Q/P/T/G in this prerequisite.

## Mandated deliverables

- **D1** — the checked-target export transaction (mechanism 1–7) in the B1
  producer crate.
- **D2** — `PerformNodeInventoryV1` typed carrier with target+semantic-identity
  binding, constructor private to the transaction.
- **D3** — `emit_export`/`BehavioralExport.alphabet` assembled only from the
  inventory; the hand-fed `EffectRow` public path removed/demoted to private.
- **D4** — the 7 required discriminators (below), each through the **real checked
  transaction**.
- **D5** — B2/B3 acceptance-suite + frozen-hash reconciliation (consumers of the
  one corrected B1 Σ); coordinate any `spec`/`conformance` frozen-hash deltas with
  CV on the same branch.

## Required discriminators (all, through the real checked transaction)

1. **Declared-headroom negative** — keep `ρ_decl` byte-identical, remove the real
   metadata perform → observe `FsHandleMetadata ∉ Σ`; the PX7 resource entry
   rejects before T/hash.
2. **Non-resource generality** — for an ordinary effect, declaration-only headroom
   does not enter Σ, while adding the real perform node does.
3. **Closure** — a perform reachable through a nested checked callee/higher-order
   callback is present; an effect in an unused sibling is absent and does not
   change the hash.
4. **Recursion/cases** — the finite closed graph reaches every retained perform
   case exactly once as a set; no catch-all or whole-family widening.
5. **Binding mutation** — an inventory from another target or semantic identity, or
   an omitted/added node, rejects before hash.
6. **Compatibility** — a no-headroom B1 fixture and the PX7 no-resource fixture
   retain bytes/hash; a headroom fixture changes only because its former orphan
   alphabet member disappears.
7. **Consumer closure** — B2 T symbols and B3 trace events remain subsets/images of
   the corrected B1 Σ, with no second alphabet.

## Acceptance criteria

- **AC1** — `BehavioralExport.alphabet` derives from the exhaustive structural
  perform-node inventory of the checked target's closed denotation graph; `ρ_decl`
  is never a source/fallback; fail-closed on non-closed inventory.
- **AC2** — all 7 discriminators pass through the real checked transaction
  (reaching, non-vacuous — the declared-headroom and generality negatives are the
  load-bearing I3 controls).
- **AC3** — hash migration is correct: headroom exports change (orphan member
  drops), exact-match exports (incl. the PX7 no-resource fixture) preserve bytes.
  B2/B3 frozen hashes reconciled; no second alphabet mechanism.
- **AC4** — no dummy perform, no I3 weakening, no `visits`-made-exact; Q/P/T/G not
  redesigned; `ρ_inf ⊆ ρ_decl` admission law intact.
- **AC5** — green in **CI** (never a local `--workspace` run); the B1/B2/B3
  acceptance suites and conformance frozen-hash controls pass.

## Do-not-reopen guard

- Do **not** derive a second Σ in B2/B3; they consume the one corrected B1 alphabet.
- Do **not** make `visits` exact or use `ρ_decl` as a Σ source/fallback.
- Do **not** widen unresolved/dynamic nodes to the declared row or an op family —
  fail closed.
- Do **not** touch PX7-T-ERR (it rebases after this lands) or PX8-T (held).
- Do **not** redesign Q/P/T/G or the Ward-delegation boundary.

## Sequencing

B1-EXACT lands FIRST → **PX7-T-ERR** rebases: Foundation's resource-export I3 gate
consumes the corrected B1 inventory (its declared-headroom negative now passes
through the real transaction), enclave reconciles the discriminating conformance
negative → §14 → merge → **PX8-T** rebases on the corrected B1 authority + final
hash → pin → PX8-R/F proceed.
