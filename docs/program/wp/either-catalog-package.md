# Either · user-level catalog package (NOT prelude)

**Owned by the Steward** (frame); **home: Foundation** (stdlib authoring).
**Operator-ruled** (2026-07-10, Pat): `Either a b = Left a | Right b` is defined
at the **user level as a catalog package**, NOT baked into the prelude — matching
the spec's own model (`50-stdlib/README.md:42`: core data are *packages*, "Ken
`data`/defs over the built-ins, not prelude"). Judgment call **L5** (placement
arm). Splits off the effect-coproduct rename, which is a separate Runtime WP.

## Why catalog-level (not prelude)

`Either` is an ordinary non-dependent sum — nothing in the kernel/elaborator/
effects depends on it, so it needs zero built-in support. The spec explicitly
models core data as **packages** (imported, re-checked), not prelude primitives.
PRINCIPLES agrees (small auditable core, reflect-don't-extend). So `Either` is the
first core sum done per the stated model — its siblings `Option`/`Result` sit in
the prelude today only as a bootstrap shortcut (a spec-vs-impl gap named as a
separate future item below; NOT this WP's concern).

## Scope

1. **`data Either a b = Left a | Right b`** as an ordinary catalog `data` decl —
   home **`catalog/packages/Data/Sums/`** (join DS-3's sum-types package; Option/
   Result/Either are all "sums" — confirm the exact entry with DS-3's landed
   layout, or a sibling `Data/Either` if cleaner). No prelude/`crates` change.
2. **Combinators + laws** (`Ω`/`Prop` proof terms, zero `Axiom`):
   - `either (a b c) (f : a → c) (g : b → c) (x : Either a b) : c` — the eliminator.
   - `mapLeft`/`mapRight` (`map` on each side), `swap : Either a b → Either b a`.
   - Laws that fall out cleanly (e.g. `either f g (Left x) = f x`; `swap (swap e)
     = e`); drop any needing out-of-scope machinery (subsume-don't-proliferate),
     ship the combinator with its definitional equations and say so — the DS-4
     pattern.
3. **Spec reconcile — the L4 subsume erratum is now false and must be corrected
   when `Either` actually lands** (it was correct while `Either` didn't exist):
   - `spec/30-surface/34-data-match.md`: **REWRITE** the subsume note → `Either a
     b = Left a | Right b` is a **distinct declared** value coproduct (a **catalog
     package**, per README:42's "core data are packages" — NOT a prelude decl);
     `Result` a distinct error sum. No "subsumed"/"no first-party Either" survives.
   - **RESTORE `Either`** at the list-sites the erratum dropped (`README.md:42`
     core-data, `34-data-match.md:5` core-sums, `:633`) — framed as a **package**,
     consistent with README:42. **Do NOT** add `Either` to the `:56` "ordinary
     prelude `data` decls" set — it is a catalog package, not a prelude decl (that
     distinction is the whole point of this WP; keep it honest).

## Boundary / constraints

- **Zero `crates/` delta** — pure catalog `.ken` + spec docs. No kernel, no
  prelude, no elaborator change (`Either` needs none). If anything looks like it
  needs a prelude/elaborator touch, STOP and hand back — it shouldn't.
- **Zero `Axiom`/`postulate`; zero `trusted_base()` delta** (executable set-diff,
  DS-2/DS-4 pattern). Ordinary structural recursion/case-split.
- **AC8 discriminators** flip on a wrong witness at a named law (e.g. an `either`
  that swaps the branches, a non-involutive `swap`), specific error variant.
- **Sequencing:** rides after / alongside DS-3 (reuses its `Data/Sums` package).
  Coordinate the entry with DS-3's landed layout. Independent of the Runtime
  `Sum`→`Coproduct` rename (no shared files).

## Named future (NOT this WP)

- **Core-data → packages migration:** `Option`/`Result`/`Nat`/`List`/`Prod`/`Unit`
  are prelude-declared today but the spec models them as packages — a standing
  spec-vs-impl gap. Aligning them (moving prelude core-data to catalog packages)
  is a separate architectural WP to frame on its own once the operator sets
  direction. `Either`-as-package is the first correct instance / precedent.
- **`Data.Functor.Sum f g`** — the higher-kinded functor coproduct (sibling to
  DS-8's `Compose`), owning the `Sum` name freed by the Runtime rename.

## Gate

Ring: Foundation build → foundation-qa independent re-derivation → **@architect**
(fidelity vs frame + zero-`Axiom`/zero-`trusted_base`; the honest "package not
prelude" framing in the spec reconcile) → **Spec vote** (spec/ normative delta,
standing rule) → `git_request` to Steward. CI-gated (catalog `.ken` + acceptance
test + spec). Own retro; flag every judgment call for the operator's log (L5).
