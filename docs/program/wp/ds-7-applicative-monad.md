# DS-7 · Export lawful `Applicative` + `Monad` constructor classes

**Owned by the Steward** (frame); **home: Foundation**. Core-toolkit item of
`wp/catalog-data-structures-program.md`, kicked in the operator's autonomous
window (2026-07-10), immediately after DS-2. The **design is settled** — it is a
full spec chapter, not a convo ruling: **`spec/50-stdlib/56-effectful-classes.md`
(CAT-2)** is the build contract. Read it in full first; this frame points at its
sections, it does not restate the design.

## Scope — D1 + D2 only (D3 `Traversable` is DS-8, gated)

Deliver the two effectful constructor classes and their proved instances:

1. **`class Applicative (f : Type → Type)`** — chapter **§3.1** signature
   (`functor : Functor f` wired superclass, `pure`, `ap`) + the **four laws**
   `ap_id`/`ap_hom`/`ap_ich`/`ap_cmp` **§3.2** stated character-for-character +
   the **`map_coh`** coherence equation **§3.2**.
2. **`class Monad (f : Type → Type)`** — chapter **§4.1** signature
   (`applicative : Applicative f` wired superclass, `bind`; `bind`-primary,
   Fork B; `pure := applicative.pure`) + the **three laws**
   `bind_lid`/`bind_rid`/`bind_asc` **§4.2**.
3. **Proved zero-delta instances** — `Applicative List` (cartesian `ap`, **§3.3**)
   + `Applicative Option` (**§3.3**); `Monad List` (`bind = concatMap`, **§4.4**)
   + `Monad Option` (**§4.4**). Every law a real kernel proof by induction /
   finite case-split; `tt`-vs-`Refl` per endpoint (`55 §3.2`).
4. **The ITree bridge is an ATTESTED CORRESPONDENCE, documented in prose — NOT a
   surface instance** (chapter **§4.3**, Fork E). `Monad`'s fields/laws are
   satisfied-by the landed interaction-tree `bind` (`declare_bind`,
   `ken-elaborator/src/effects/state.rs:477`); the entry **attests** this and
   discriminates it — **it mints no second `bind` and writes no `instance Monad
   (ITree e resp)`** (the parametric-instance-head path is the still-open CAT-1
   `55 §6.1` Steward fork; do **not** reopen it here). Zero new code for the
   bridge.

**Out of scope (DS-8, do NOT build here):** `Traversable` (chapter **§5**) — it
is gated on SURF-1 (landed) **+ SURF-2** (`proc` class-field marker). Leave it
out entirely; DS-8 appends it once SURF-2 is verified landed.

## The wired superclass chain (chapter §2 — resolved WIRE)

`Applicative f` carries an explicit `functor : Functor f` field; `Monad f`
carries an explicit `applicative : Applicative f` field. **The wired dict is the
already-built superclass dict, supplied whole — its laws are NOT re-proved**
(chapter §2/§3.3): `instance Applicative List` supplies the landed
`Functor List` dict as its `functor` field; `instance Monad List` supplies the
just-built `Applicative List` dict as its `applicative` field. The whole win of
WIRE over restate is this proof reuse — do not duplicate the Functor/Applicative
law proofs at the deeper instance.

Landed dep: **`class Functor (f : Type → Type)`** at
`catalog/packages/Core/Classes/LawfulFunctors.ken:188` (`map`/`id_law`/`fusion_law`).
Its instances `Functor List`/`Functor Option` are the wired `functor` fields —
**probe they are landed and reusable first** (grep the emission, not a `.ken`
view); if a needed `Functor` instance or a needed collections op
(`concatMap`/`list_append`, chapter §6) is missing, escalate to the leader
before designing around it — do not silently re-derive it.

## Boundary / constraints (the acceptance bar = chapter §7)

- **AC1 — kernel-untouched, extension-reused, zero `trusted_base()` delta.** No
  `ken-kernel` diff; **no new elaborator capability** — the wiring (superclass
  field), the nested `.field` projection (`d.applicative.pure`), and the attested
  bridge all ride CAT-1's `55 §6` extension + existing record/projection
  machinery (`elab_class_decl`, `infer_proj`, `compute_ordered_field_values`).
  **If the build finds a genuinely-required new elaborator capability, STOP and
  hand back to me** (it re-forks to Steward, chapter §2/§6 — it is not smuggled).
  Mirror DS-2's self-enforcing evidence: an executable `trusted_base()`
  before==after set-diff test.
- **AC2–AC4 — laws `Ω`, pointwise, proved, zero `Axiom`.** Every law field is
  `Equal (f _) u v : Ω` (no `‖·‖` truncation); one canonical field per law, no
  point-free duplicate; `List`/`Option` are inductive carriers so every ∀-law is
  a genuine kernel proof — **zero new `Axiom`/postulate/opaque, zero delta**.
- **AC5 — Monad ⇔ ITree.** Attested correspondence per §4.3, no second divergent
  `bind`; discriminated in the acceptance test.
- **AC7 — WIRE applied consistently** across `Functor → Applicative → Monad`.
- **AC8 — discriminators genuinely flip** accept→reject on the wrong witness, at
  the named law field, asserted as the **specific** error variant (not bare
  `is_err()`): e.g. a wired `applicative` field that is a non-cartesian /
  law-breaking `Applicative`; a masked `Axiom` inhabiting `Bottom`; the
  ITree-bridge discriminator (no second divergent `bind`).
- **Explicit wiring only.** Implicit-superclass-coercion sugar (auto-`map` in an
  `Applicative` context) is **deferred** (`OQ-syntax`, chapter §2) — it would
  need a new elaborator capability, which this WP does **not** take. Use sites
  are explicit (`d.functor.map`, not a bare `map`). That verbosity is expected,
  not a defect to fix here.
- **Outer-ring only.** No kernel/elaborator/TCB change is expected or permitted.
- **Format `.ken.md`** per `07-catalog-style-guide.md`, and per **PRINCIPLES
  #14**: the laws (required facts) live in the Ken as `law`/`fn`/`const` proof
  terms, narrative in prose, **no required info in `--` comments**.

## Home / package (judgment call — logged)

New entry **`catalog/packages/Core/Classes/EffectfulClasses.ken.md`** for the CAT-2
effectful-class family: `Applicative` + `Monad` + their instances now; DS-8
appends `Traversable` to the same entry (or a sibling) once SURF-2 lands. It sits
alongside `LawfulFunctors.ken`/`LawfulClasses.ken` in `Core`, reusing
`class Functor` from `LawfulFunctors`. (Chapter §6's perishable build-note names
`catalog/packages/Core/Classes/`; the Steward homes catalog authoring at
Foundation per program-doc `P3`, and the family is `Core`-resident — the
Foundation build may pick the final basename, this is the frame's suggestion, not
a hard pin. Flag the final name in the handback.)

## Gate

Normal ring: Foundation build → foundation-qa independent re-derivation →
**Architect fidelity gate (build vs chapter 56) + soundness gate (zero-new-`Axiom`
/ zero-`trusted_base()`-delta)** → `git_request` to Steward. CI-gated (real
catalog `.ken.md` + acceptance test). Own retro. Resource discipline on
(`CARGO_BUILD_JOBS=2`, scoped `-p` tests, never a bare/`--workspace` local run).
Any judgment call touching surface / elaboration / functionality — flag it
explicitly in the handback so the Steward logs it for the operator.
