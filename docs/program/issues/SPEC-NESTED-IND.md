---
id: SPEC-NESTED-IND
title: "un-defer nested strictly-positive inductives in 14 §8.5 — state structural positivity through declared strictly-positive type-parameter positions, the lifted induction hypotheses, and the iota rules, WITHOUT mutual families"
status: ready
owner: spec-enclave
size: M
gate: none
depends_on: []
blocks: [KERNEL-NESTED-IND]
github: null
origin: Architect ruling evt_55k9f9efvd8jk, Decision dec_13af1mercv2m0 resolved — the fork was raised by the Steward as evt_1ykvpj7yvtg18 after DS-9 blocked at its first deliverable on `JsonArray (List Json)`. Demand-pulled by DS-9, which stays blocked until KERNEL-NESTED-IND lands. Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ▶ THE SPEC HALF OF A TWO-STAGE PREREQUISITE
>
> **Sequence:** **`SPEC-NESTED-IND`** → `KERNEL-NESTED-IND` → `DS-9`.
>
> This node states the rules. `KERNEL-NESTED-IND` implements and checks them.
> ⛔ Neither is a licence to relax positivity — see "what this is NOT" below.

## Why this exists — a real consumer walked into a deferral

`DS-9` (lawful JSON codec) blocked at **`D1`, its first deliverable**. The
ordinary spelling

```ken
data Json = ... | JsonArray (List Json) | JsonObject (List (Pair String Json)) | ...
```

is rejected: `spec/10-kernel/14-inductive.md:126-128` records **nested**
occurrences under another type former (`List (Rose A)`, §8.5) as *"still
rejected … nested/mutual remain a later extension"*, and `:709` titles §8.5
*"Nested and mutually-defined inductives — still deferred."*

⭐ **The deferral is correct and this node does not dispute it.** The Architect's
ruling is explicit: *"The present rejection is a safe, deliberate
completeness/staging boundary, **not an unsound kernel result**."* What changed is
only that a first-party consumer now puts concrete demand behind lifting it.

## ⛔ SCOPE — NESTED ONLY. MUTUAL IS EXCLUDED.

**Architect, verbatim:** *"Do **not** bundle mutual inductives. Mutual families are
a distinct extension, are not required by DS-9, and would enlarge the trusted
change without present demand."*

⚠ §8.5 defers nested **and** mutual in one clause, so the tempting reading is that
un-deferring §8.5 means un-deferring both. ⛔ It does not. The spec edit must
**split** §8.5 — lift the nested half, leave the mutual half deferred and say so
explicitly, so the next reader cannot mistake the remaining deferral for an
oversight.

## What the spec must state

The five-point contract below is the Architect's, transcribed. Points 1–3 are
**spec obligations** — this node. Points 2–5 are also **checkable obligations** on
`KERNEL-NESTED-IND`; they appear in both because the spec must *say* what the
kernel must *do*.

1. **Positivity is structural through declared strictly-positive type-parameter
   positions**, sufficient for both `List Json` and `List (Pair String Json)`.
   ⛔ **Unknown and negative positions fail closed.** ⛔⛔ **There is NO `List`
   name allow-list** — the rule is about *declared parameter polarity*, never
   about which type constructor is being nested. A spec that says "nesting under
   `List` is permitted" has stated the wrong rule.
2. **The eliminator story, stated normatively:** one **lifted induction
   hypothesis for every contained recursive occurrence**, plus the corresponding
   **iota reductions**. §3.1's Π-abstracted-IH machinery is the precedent to
   extend, not to re-derive.
3. **Consumability:** surface matching, elaboration, and
   structural-recursion/termination checking must be able to **consume** those
   lifted hypotheses, so that a theorem over the array and object branches is
   *actually writable*. ⛔ A rule that admits the declaration but leaves no way to
   induct over it states half a contract.

## ⛔ What this is NOT — the three misreadings to close in the text

1. ⛔ **NOT "delete the `occurs` guard."** `:569-570` and §8.2's `occurs`-guard are
   what currently reject nesting, and the Architect names removing them as
   **insufficient**: *"Merely deleting or relaxing the current `occurs` guard is
   not delivery: that would admit the declaration without supplying sound
   recursion/proof machinery."* ⭐ **Structurally the same rule as hard-stop `#11`'s
   inertness clause** — a representation admitted with its consumers deferred
   discharges nothing. Write §8.5's successor so that reading is unavailable.
2. ⛔ **NOT a widening of positivity itself.** Negative occurrences
   (`(D → Bool) → D`, §8.3) stay rejected, unchanged. Nesting under an *unknown*
   or *non-positive* parameter stays rejected. This node narrows nothing and
   permits nothing new about polarity.
3. ⛔ **NOT a change to the admitted W-style class.** `(Nat → D) → D` and the
   K1.5 Π-bound class (§2.1, `:118-124`) must behave **exactly** as they do today.

## Deliverables

- **`D1`** — §8.5 split: nested lifted, mutual explicitly retained as deferred
  with its own reason. ⛔ Do not leave a single sentence covering both.
- **`D2`** — the structural positivity rule through declared strictly-positive
  parameter positions, with the fail-closed cases (unknown, non-positive) stated
  as rules rather than examples, and ⛔ **no type-constructor name anywhere in the
  rule**.
- **`D3`** — the eliminator/IH/iota statement for nested occurrences, extending
  §3.1 rather than duplicating it.
- **`D4`** — the consumability requirement (§8.5 successor + wherever
  termination/elaboration obligations live), so `D3`'s hypotheses are reachable
  from the surface.
- **`D5`** — the conformance obligations named, so `KERNEL-NESTED-IND` inherits
  rows rather than inventing them. Four are required by the ruling and are listed
  in that node.
- **`D6`** — currency sweep: every consumer that cites §8.5's deferral or repeats
  *"nested/mutual remain a later extension"* is reconciled. ⚠ `:126-128`,
  `:569-570` and `:709` are three **known** sites; ⛔ that list is a starting
  point, not the census — a stale sentence elsewhere silently contradicts the new
  rule.

## Acceptance criteria

| AC | claim | positive control |
|---|---|---|
| `AC-S1` | The nested rule is stated **without naming any type constructor**. | grep the new rule text for `List` — a hit in the *rule* (as opposed to an illustrative example) fails the AC |
| `AC-S2` | Unknown-parameter and non-positive-parameter nesting are stated as **fail-closed rules**, each with its own clause. | a reader-level check: remove either clause and the rule admits something the ruling forbids |
| `AC-S3` | Mutual families remain deferred, in **their own** sentence with their own stated reason. | the split is visible: §8.5's successor cannot be read as covering mutual |
| `AC-S4` | The eliminator/IH/iota obligation is stated, so "admit the declaration" alone cannot satisfy the chapter. | delete the IH clause → the chapter would permit the inert outcome the ruling rejects |
| `AC-S5` | The W-style/K1.5 Π-bound class and negative rejection are **textually unchanged**. | diff §2, §2.1, §8.1–§8.3: any change outside the nested clauses is out of scope |
| `AC-S6` | `D6`'s sweep is **entailment-closed**, not a three-site patch. | ⚠ named control: the `SPEC-STORE-SPLIT` retro's carry — *"a property-removal census must be entailment-closed"*; a repeated expansion signals a faulty enumeration method, not isolated misses |

⛔ `AC-S6` is the one most likely to be under-served. The three known sites were
found by grepping two phrasings; a consumer that *implies* the deferral without
quoting it will not appear in that grep.

## Contention

**None.** Spec-only (`spec/10-kernel/`, plus wherever `D4`'s consumability
obligation lands). Runtime is in `crates/`; Foundation is parked. ⚠ The kernel
implementation is a **separate node** — ⛔ this node ships no `crates/` change.

## What this unblocks

`KERNEL-NESTED-IND` → `DS-9` → the rest of the catalog data-structures tier's
Phase 3.
