# C6 — a law-carrying `Ord` vs a law-less `Axiom` stub

**Axis:** lawful typeclasses carrying *real* law proofs. **Flavor:** B
(capability-depth — the discriminator is trust-level, not accept/reject).

## Why this is a blind spot

VAL2's `merge-sort` used `Ord` as a **comparator** — it never asked whether the
`Ord` dictionary's laws (`refl`/`antisym`/`trans`/`total`) are **proved** or
merely **postulated**. `§51 §5` ("laws PROVED, not postulated") is the gate that
makes an instance genuinely *lawful*. The distinction is invisible at the value
level — an `Axiom` inhabits a law's type just as a real proof does — so it only
shows up in the **trust budget**: a proved law adds **zero** `trusted_base()`;
an `Axiom` law is a visible `Decl::Opaque` **postulate** that grows it. VAL2
never distinguished them.

## The pair

- **Sound arm — `sound-ord-proved.ken` — should-PASS, zero-delta.** `Ord` over a
  **real inductive** carrier (the `Bool` zero-delta exemplar, `§51 §6`): the
  laws are **proved by case-split** (real proof terms, `tt`/`absurd`), so the
  instance adds **no** `trusted_base()` entry.
- **Stub arm — `stub-ord-axiom.ken` — elaborates, but grows the trust base.**
  The **same** carrier with `refl/antisym/trans/total = Axiom` — a law-less
  stub. It type-checks (an `Axiom` inhabits the law type), but each law is a
  visible postulate.

## Expected behavior (exact)

- Sound arm: **PASS with zero `trusted_base()` delta** — the laws are discharged
  by case-split over the finite inductive carrier (the `Bool` exemplar shows
  this is possible; `§51 §6`).
- Stub arm: **elaborates, but adds `trusted_base()` entries** — each `Axiom` law
  is an honest **visible** `Decl::Opaque` postulate. This is *not* rejected
  today: the machinery accepts honest visible `Axiom`s (all landed illustrative
  instances, incl. `Ord Int`, use `Axiom`). So the enforcement "a law that
  **could** be proved over a provable carrier must not be postulated" is a
  **known-gap** — the deferred lawful-classes work.

## Discriminates

**Laws-proved vs laws-postulated**, read off the **`trusted_base()` delta**: the
proved instance adds zero; the stub adds a postulate per law. The discriminating
consumer (per the lawful-classes discipline: "the case must FAIL against a
law-less instance") is the trust-budget check — a "the instance type-checks"
test is green-vs-green (the stub passes it too). Grep the emitted decls for
`Decl::Opaque`/`Axiom` law fields.

## Surface-expressibility note

Writing real case-split proofs for `antisym`/`trans`/`total` needs `tt`/`absurd`
(K5, landed) and observational collapse to `Top`/`Bottom` at concrete
constructor pairs (K7, landed) — the `es4_classes_acceptance.rs` machinery. If a
particular law's proof term isn't cleanly surface-writable yet, record which
laws are proved-reachable vs still need `Axiom` — that *is* the depth result
(exactly where the lawful-class frontier sits). The `Bool` exemplar is the
existence proof that zero-delta lawful instances are reachable.
