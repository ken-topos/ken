---
id: KERNEL-NESTED-IND
title: "admit nested strictly-positive inductives in the kernel — structural positivity through declared parameter positions, generated and checked dependent eliminators with one lifted IH per contained recursive occurrence, iota, and surface consumability"
status: draft
owner: kernel
size: L
gate: none
depends_on: [SPEC-NESTED-IND]
blocks: [DS-9]
github: null
origin: Architect ruling evt_55k9f9efvd8jk, Decision dec_13af1mercv2m0 resolved. Demand-pulled by DS-9, which blocked at its first deliverable on `JsonArray (List Json)`; fork raised by the Steward as evt_1ykvpj7yvtg18. The five-point prerequisite contract below is the Architect's, transcribed verbatim in substance. Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ▶ THE KERNEL HALF OF A TWO-STAGE PREREQUISITE
>
> **Sequence:** `SPEC-NESTED-IND` → **`KERNEL-NESTED-IND`** → `DS-9`.
>
> ⛔ **`draft` until `SPEC-NESTED-IND` merges.** The rules are stated there; this
> node implements and checks them. Starting here first would implement a contract
> that does not yet exist.
>
> ⚠ **This node changes the TCB.** Read `docs/PRINCIPLES.md` on the small
> auditable trusted base before slicing it.

## Why this exists

`DS-9` blocked at `D1` on `JsonArray (List Json)` — the `List (Rose A)` class that
`spec/10-kernel/14-inductive.md` §8.5 defers. The Architect ruled **B,
nested-only**: preserve DS-9's ordinary six-constructor `Json` and lift the
kernel restriction, rather than re-encode the value model.

⭐ **The rejection being lifted is sound, not broken.** Architect, verbatim: *"The
present rejection is a safe, deliberate completeness/staging boundary, **not an
unsound kernel result**."* This node adds capability; it does not fix a bug.

## ⛔ SCOPE — NESTED ONLY, and the exclusion is load-bearing

**Architect, verbatim:** *"Do **not** bundle mutual inductives. Mutual families are
a distinct extension, are not required by DS-9, and would enlarge the trusted
change without present demand."*

⛔ Mutual is **out**. §8.5 defers nested and mutual in one clause, which makes
"un-defer §8.5" read as both — it is not. ⚠ If a slice finds mutual machinery
falling out for free, that is **not** authorization to land it; bring it back to
the Steward as a separate node.

## ⛔ THE FIVE-POINT CONTRACT — complete only when ALL FIVE hold

Transcribed from `evt_55k9f9efvd8jk`. ⛔ Not a summary — these are the completion
conditions.

1. **Positivity is structural through declared strictly-positive type-parameter
   positions**, sufficient for **both** `List Json` **and**
   `List (Pair String Json)`. Unknown and negative positions **fail closed**.
   ⛔⛔ **There is NO `List` name allow-list.**
2. **The kernel generates AND checks the dependent eliminator**, with **one lifted
   induction hypothesis for every contained recursive `Json`**, and the
   corresponding **iota reductions**. ⛔⛔ *"Merely deleting or relaxing the current
   `occurs` guard is **not delivery**: that would admit the declaration without
   supplying sound recursion/proof machinery."*
3. **Surface matching, elaboration, and structural-recursion/termination checking
   can consume those lifted hypotheses**, so that a theorem over the array and
   object branches is **actually writable**.
4. **Conformance** includes: a **positive** nested `List`/Rose-style declaration
   **with a real recursive computation or proof**; a retained **nested-negative
   rejection**; a retained **rejection through an unknown or non-positive
   parameter**; and evidence that **direct and existing W-style inductives are
   unchanged**.
5. ⛔ **No new axiom, postulate, trusted escape, or library-side representation
   workaround** enters the solution.

## ⭐⭐ The anti-pattern point 2 exists to forbid — read this before slicing

The cheap version of this node is: find the `occurs`-guard (§8.2, cited at
`14-inductive.md:569-570`), relax it so the declaration is admitted, watch
`data Json = ... | JsonArray (List Json)` type-check, and report success.

⛔ **That is explicitly not delivery**, and it is worse than nothing: the
declaration would be admitted with **no sound way to induct over it**, so the
first person to try proving anything about the array branch discovers the gap —
after the TCB already grew.

⭐ **This is structurally the same rule as hard-stop `#11`'s inertness clause**
(`RT-FNSPLIT-C1`): *a prerequisite may be inert only in the sense that production
routing has not switched to it yet; its producer → validator → eliminator edge
must nevertheless be real and executable.* Here the edge is **declaration →
eliminator + IH + iota → a writable theorem**. Point 3 is what makes the far end
of that edge observable.

⇒ **The AC that discharges this node is `AC-K3`**, not `AC-K1`.

## Deliverables

- **`D1`** — structural positivity through declared strictly-positive parameter
  positions, replacing the blanket nested rejection. ⛔ Keyed on **declared
  parameter polarity**, never on a type-constructor name.
- **`D2`** — fail-closed handling for unknown and non-positive parameter
  positions.
- **`D3`** — eliminator generation extended: one lifted IH per contained recursive
  occurrence, extending §3.1's Π-abstracted-IH machinery.
- **`D4`** — the matching iota reductions, and the kernel **checks** the generated
  eliminator rather than trusting it.
- **`D5`** — surface consumability: matching, elaboration, and
  structural-recursion/termination checking accept the lifted hypotheses.
- **`D6`** — the four conformance rows of contract point 4.
- **`D7`** — a **`trusted_base()` delta report**, stated as a number with what
  grew and why. ⚠ This node *does* grow the TCB; the deliverable is an honest
  accounting, ⛔ not a zero.

## Acceptance criteria

Each names its positive control.

| AC | claim | positive control |
|---|---|---|
| `AC-K1` | `data Json = ... \| JsonArray (List Json) \| JsonObject (List (Pair String Json)) \| ...` is **admitted**. | ⚠ **necessary, not sufficient** — a guard-deletion passes this row. It is listed to be discharged, not to be relied on |
| `AC-K2` | Admission is keyed on **declared parameter polarity**, not on a name. | declare a **new** strictly-positive container of your own and nest `Json` in it → must be admitted **with no kernel change**. ⛔ If it needs one, an allow-list is hiding somewhere |
| `AC-K3` | ⭐ **A real theorem over the array branch is written and kernel-checked**, consuming a lifted IH. | delete the lifted IH from the generated eliminator → the theorem must **fail to check**. ⛔ If it still checks, `AC-K3` was never testing the IH |
| `AC-K4` | Iota reduces for nested occurrences; a **recursive computation** over `JsonArray` evaluates. | perturb one iota rule → the computation's result changes or it fails to reduce |
| `AC-K5` | Nested-**negative** rejection retained. | the known-bad `(D → Bool) → D` under a container must still be **rejected**, asserted as the specific rejection |
| `AC-K6` | Rejection through an **unknown** parameter retained. | nest `Json` under a parameter whose polarity is undeclared/unknown → **rejected**, not admitted-by-default |
| `AC-K7` | Rejection through a **non-positive** parameter retained. | as `AC-K6` with a declared-negative position |
| `AC-K8` | Direct and existing **W-style** inductives unchanged. | the K1.5 Π-bound suite (`(Nat → D) → D`, §2.1) runs green **untouched**; ⛔ a diff to those tests is itself a finding |
| `AC-K9` | ⛔ **Zero** new axiom, postulate, trusted escape, or library-side representation workaround. | grep the diff for `Axiom`/`postulate`/`sorry`/`unsafe` additions; a hit fails the row |
| `AC-K10` | `trusted_base()` delta reported **as a number**, with what grew. | ⚠ no mechanical control — discharged by the report. Listed so "grew by 0" and "never measured" cannot read identically |

⛔ **`AC-K3` and `AC-K8` are the pair that matters.** `AC-K3` proves the new
capability is *usable*; `AC-K8` proves the old capability is *undamaged*. A node
that greens one and quietly weakens the other has widened the TCB for nothing.

⚠ **Report `AC-K5`–`AC-K7` as three separate rows.** They are three different
rejection reasons and an aggregate "negatives still rejected" pass would hide one
of them defecting.

## Validation — targeted only

⛔ **NEVER `--workspace`** (operator hard rule, `agent/COORDINATION.md §12`). Scope
to the crates you touch (`-p ken-kernel` and, for `D5`, `-p ken-elaborator`), plus
the kernel conformance suite. **The full build, `--locked`, and conformance run in
CI on GitHub.** "No regression" means **green in CI**.

⚠ **A kernel change is the case where the local/CI split bites hardest** — the
blast radius is every crate. ⛔ Do not conclude "no regression" from a green
targeted run; say what you ran and let CI answer the rest.

## Contention

⚠ **This is the one node in flight with a wide blast radius.** It changes the
kernel's admittance surface, so it is **not** contention-free the way DS-9 and
`ABI-S3` are with each other. ⛔ Re-derive contention at kickoff against whatever
is then active in `crates/` — this section will be stale.

## What this unblocks

`DS-9`, and with it Phase 3 of the catalog data-structures campaign. ⭐ More
broadly: nesting a `List` inside a recursive type is the shape of **every tree
with a list of children** — JSON, XML, S-expressions, ASTs, rose trees. ⚠ That
breadth is the argument for doing it properly, ⛔ not for widening scope past
nested-only.
