# DS-8 · Export a lawful `Traversable` constructor class

**Owned by the Steward** (frame); **home: Foundation**. The **last Core-toolkit
item** of `wp/catalog-data-structures-program.md`, completing the constructor-
class chain `Functor → Applicative → Monad → Traversable`. Kicked in the
operator's autonomous window (2026-07-10), after DS-7 landed. Design contract:
**`spec/50-stdlib/56-effectful-classes.md §5` (CAT-2)** — read §5 in full; this
frame points at it, it does not restate it.

## Prerequisites — all landed (verified), plus one law-instrument question

- **DS-7 `Applicative`/`Monad`** — ✅ landed `origin/main @ 88dce79`
  (`Core/EffectfulClasses.ken.md`). The wired chain and `Applicative` are in.
- **`class Functor` + `class Foldable`** — landed (`LawfulFunctors.ken:188`,
  `:274`); **instances `Functor List`/`Functor Option` and `Foldable List`/
  `Foldable Option`** all landed (`LawfulFunctors.ken:358`/`:366`) — these are
  the wired superclass fields, supplied whole (do **not** re-prove them).
- **SURF-2** (`proc` class-field marker) — landed; `parse_class_decl` accepts it
  (`ClassField.purity`). **SURF-1** (row-variable) — landed (`main@ef791a3`).
- **The one open axis — the §5.3 law instruments:** the traversal **composition**
  law is stated over the `Compose g h` applicative, and the **identity** law over
  the identity applicative (`g := λa. a`). The identity applicative is trivial;
  **`Compose g h` is NOT landed** (grep-confirmed — only unrelated
  `setUnionIdentityLaw` hits). See "Composition-law scope" below — this is the
  one place DS-8 is not purely mechanical.

## Scope

Append to the existing **`catalog/packages/Core/EffectfulClasses.ken.md`** (the
effectful-class family entry — per judgment call `L1`; keep the family in one
entry), reusing everything DS-7 landed there.

1. **`class Traversable (f : Type → Type)`** — §5.1: wires `functor : Functor f`
   + `foldable : Foldable f`; `proc traverse : (g : Type → Type) → Applicative g
   → (a b : Type) → (a → g b) → f a → g (f b)`; `sequence` = `traverse` at the
   identity action. **`Applicative g` is an EXPLICIT (unbundled) dictionary
   parameter** (Fork C, §5.1) — NOT an implicit `where` constraint (an abstract
   `g` has no concrete head for implicit search; `infer_proj` projects
   `ap_g.ap`/`ap_g.pure` off the opaque bound dict fine, so the explicit form is
   buildable today and the implicit form is not — do not attempt the implicit
   form).
2. **`Traversable List` + `Traversable Option`** instances (§5.4), proved
   zero-delta — the effect-sequencing fold, effect-polymorphic in `g`, laws by
   induction on the carrier + the applicative laws of `g` (`§3.2`).
3. **The three coherence laws** (§5.3), pointwise `Ω`-clean value equations —
   see the composition-law scope note.

## Composition-law scope (the one real decision — probe, then honest split)

Build the **buildable core first, unconditionally:** the two classes, both
instances, `sequence`, and the **identity** + **naturality** laws (identity needs
only the trivial identity applicative; naturality is the parametricity face,
discharged structurally per §5.3). Then handle the **composition** law:

- **Probe building the `Compose g h` applicative in-scope.** It is the
  instrument the composition law is stated over (`traverse` at `Compose g h`). If
  it is a clean, small **derived** applicative instance (subsume-don't-proliferate
  — one `instance Applicative (Compose g h)` from two `Applicative` dicts, its
  four laws by the component laws), **build it and prove composition** — that is
  the complete Traversable showcase, preferred.
- **If `Compose g h` turns out to need machinery not landed** (it is flagged
  "CAT-2/CAT-3 derived" in §5.3 — if it genuinely needs a CAT-3 wall like a
  multi-parameter/higher-order instance head that does not elaborate today),
  **do NOT force it.** Ship DS-8 with `class`/instances/`identity`/`naturality`
  landed and the **composition law GATED on a named `Compose`-applicative
  follow-up WP**, with an honest landed/gated split in the entry (the DS-5
  chapter's pattern — say plainly what builds and what waits, never imply the
  gated law is proved). Escalate the boundary call to @architect through the
  ring; he rules whether `Compose` builds in-scope or is a follow-up.

Either way the **core Traversable lands**; the only question is whether the
composition law rides this WP or a scoped follow-up. Decide it by a build-probe,
not by assumption (buildability ruling must ground every axis).

## Boundary / constraints (acceptance = chapter §7)

- **AC1 — kernel-untouched, zero new elaborator capability, zero `trusted_base()`
  delta.** `traverse`'s abstract `g` + explicit dict + `proc` marker all ride
  landed machinery (`infer_proj` on an opaque dict, SURF-1 row-var, SURF-2
  marker). **If a genuinely-new elaborator capability looks required, STOP and
  hand back** (re-forks to me). Mirror DS-2/DS-7's executable before==after
  `trusted_base()` set-diff test.
- **AC2–AC4 — laws `Ω`, pointwise, proved, zero `Axiom`** over the inductive
  carriers (List/Option); zero delta.
- **AC6 — Traversable ⇔ SURF-1/SURF-2.** `traverse` classifies **`proc`** under
  SURF-1's bidirectional check via the row-variable mechanism (its action's
  abstract codomain head `g` → `Unknown` → fail-closed → `RowVar`); the class
  field carries SURF-2's `proc` marker; `d.traverse` classifies `proc` at
  projection/use. Confirm the marker is erased before the kernel and does **not**
  affect the class record's Type/Ω sort discriminant (computed from field types).
- **AC7 — WIRE** applied consistently (`functor`/`foldable` supplied whole).
- **AC8 — discriminators flip** accept→reject on the wrong witness at the named
  law field, asserted as the **specific** variant (e.g. a `traverse` that is not
  effect-sequencing / breaks the identity law; a wired `foldable` inconsistent
  with `traverse`/`toList`).
- **Dot-projection / `λ`-in-type-position workaround will recur** (DS-7's Finding
  1): where `.field`/bare `λ` don't parse in type-annotation position, use a
  named total `fn` that δ/η-reduces to the exact spelling, and file it as an Ergo
  Finding — **do not** smuggle a new capability or escalate it as a blocker.
- **Outer-ring only.** Format `.ken.md`, **PRINCIPLES #14** (laws as proof terms,
  narrative in prose, no required info in `--` comments).

## Gate

Normal ring: Foundation build → foundation-qa independent re-derivation →
**@architect fidelity gate (build vs chapter 56 §5) + soundness gate
(zero-new-`Axiom`) + the composition-law-scope boundary ruling** → `git_request`
to Steward. CI-gated (real catalog `.ken.md` + acceptance test). Own retro.
Resource discipline (`CARGO_BUILD_JOBS=2`, scoped `-p` tests). Flag every
surface/elaboration/functionality judgment call in the handback for the operator's
log — especially the `Compose`-applicative build-vs-gate decision.
