# ADR 0008 — Typeclass/instance coherence (canonical, with an explicit escape)

- **Status:** Accepted
- **Date:** 2026-06-27
- **Deciders:** the operator

## Context

Ken's constraint mechanism is typeclasses-as-subobjects (`30-surface/33 §5`): a
class elaborates to a **record** of operations *and their law proofs*; an
instance is a value of that record; a constraint `where C A` is an implicit
instance argument the elaborator discharges by **proof search**. The design was
fixed; the **coherence policy** — what a reader and the prover may assume about
*which* instance resolution finds — was left open as `OQ-classes`.

It matters more in Ken than in an ordinary language because **the resolved
dictionary is semantically load-bearing**: it carries law proofs the prover
*uses*. If `Monoid A` could denote different dictionaries at different sites, a
lemma proved about "the `Monoid A`" could be unsoundly combined with data built
from a *different* `Monoid A`. Coherence is therefore a soundness-adjacent
property of *client* reasoning, not merely an ergonomic preference.

## Decision

**Split classes by where the dictionary lives; make implicit search canonical;
give an explicit escape hatch for everything else.**

1. **Property classes** (Ω-valued: `Decidable p`, `IsHom f`). Proof-irrelevant
   (`10-kernel/16 §1`) ⇒ any two instances are definitionally equal. **Coherence
   is free** — the kernel guarantees it; no resolver convention applies. This is
   a direct dividend of the strict-prop Ω from ADR 0005.
2. **Structure classes** (`Type`-valued, dictionary with computational content:
   `DecEq`, `Monoid`, `Ord`). Genuinely many can exist on one carrier, so
   coherence is a **resolver convention**:
   - **One canonical instance per (class, head-type)** participates in implicit
     search — resolution is a *function of the type*, stable program-wide, which
     the law-carrying prover relies on.
   - **Orphan instances are a hard error**: an instance MUST be declared with
     its class or with its head-type. Keeps canonicity decidable and
     per-module-checkable; canonicity you cannot accidentally break.
   - **No overlapping instances** in search; **ambiguity is a compile error**
     naming both candidates, never a silent pick.
   - **Search terminates** — bounded by a structural-decrease rule on the
     instance graph (the SCT family, `10-kernel/17 §4`).
3. **Named instances are first-class values, passed explicitly.** Because an
   instance *is* a record value, you define `byLength : Ord String` and pass it
   (`sortBy byLength xs`) — the dependent-types escape hatch Haskell lacks (no
   `newtype` gymnastics). *Implicit* search stays canonical and predictable;
   *explicit* passing is unrestricted. The resolver may pick only one canonical
   thing silently; you may deliberately use any value.

## Consequences

- **Human-read:** "the `Monoid A`" denotes one thing; a proof citing it is
  stable — no "which instance was in scope *here*?" archaeology, the worst
  legibility tax of local-instance systems.
- **Prove-what-can-be-proven:** canonical structure-instances make client lemmas
  soundly composable; property-instances get coherence gratis from Ω. The
  verification story is airtight on both halves.
- **Agent-written / human-read asymmetry:** agents do not need the ergonomic
  shortcuts (overlap, orphans, ambient local instances) that human *convenience*
  historically demanded — they can afford to be explicit. The cost of
  explicitness lands on the cheap side (writing); the benefit (legible canonical
  resolution) accrues to the dear side (reading and proving). The trade the
  software-engineering-language philosophy prescribes.
- **`derive`** stays untrusted-but-kernel-checked (`33 §5`); a generated
  instance is just a candidate the kernel re-checks.

## Revisit if

- A real workflow needs *implicit* selection among several structure-class
  dictionaries that the explicit-value escape cannot serve ergonomically — weigh
  against the canonicity guarantee, which is the whole value.
- Cross-package instance governance needs more than orphan rules (e.g. a
  registry-level canonical-instance ownership check) — that is a supply-chain /
  registry concern (`60-security/63 §6`), not a language change.
