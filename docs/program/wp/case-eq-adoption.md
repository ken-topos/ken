# WP case-eq-adoption — migrate the 2 small dispatch sites to `eqn:`

**Owner:** Ergo team (catalog). **Steward-framed** (2026-07-11). Base:
`origin/main @ b7c0de73` (re-verify `file:line` cites at pickup — catalog moves).
**Outer-ring, behavior-preserving.** No `crates/**/src`, no `ken-kernel`, no
`Cargo.lock`, no `/spec`, no `/conformance`. This is the **fast-follow adoption**
the capability WP (`case-eq-dependent-match-sugar.md`, seam 5) deferred: the
`match e eqn: h` modifier landed (capability #485 `main @ 9ad154e8`, surface
clause #487 `main @ b7c0de73`); now the sites that hand-rolled the idiom migrate
to it. **Purely ergonomic — every proof term is behaviorally identical**, because
the modifier desugars to the exact `J`-transport the hand idiom builds.

## ⚠ RE-SCOPED — Map DROPPED; ship the two small sites (Architect `evt_6bk169gj8d0kz`, 2026-07-11)

On grounded pickup the Map bulk (the 64 dispatchers originally counted as "79
uses — the bulk") turned out **not** to be the inline-explicit-`J` idiom this
frame assumed. Map dispatchers are a **structurally different, legitimate
idiom**: dispatch on a **precomputed `Or`** (`bool_dichotomy (cmp)` builds `Or
(Equal cmp True) (Equal cmp False)` **at the caller** and threads it as a param)
→ a **plain non-dependent `match`** → a **named helper** that carries its own
explicit motive / fixed goal and does the per-branch transport. The modifier
synthesizes no motive that reproduces this, and the two Map sub-families fail in
**opposite** directions (the tell it is idiom-mismatch, not a syntactic-reach
gap): insert/lookup (scrutinee not syntactic in the goal → **constant** motive →
subsumes nothing) vs `set_intersection_member` (scrutinee syntactic → modifier
**over-transports** to `bool_and … False` while the retained fixed-goal helper
supplies the original → fail-closed kernel reject). **Ruling: DROP Map from this
WP — leave every Map dispatcher on its existing `bool_dichotomy`/`Or`/named-helper
form (correct coexistence, NOT a regression).** case-eq subsumes **only**
inline-transport of a **syntactically-present** scrutinee proved **in-place** —
which is exactly the two small sites (EmptyDec + LawfulClasses `list_deceq`), and
they succeed. **Not** a modifier defect, **not** an alternate-adoption-form to
find, **not** a modifier-fix prerequisite to wait on; no modifier/kernel work is
warranted for Map (reflect-don't-extend). The WP stays **standalone** (its
LawfulClasses `list_deceq` edit is the Track-A merge anchor); site count and the
byte-reduction claim re-scope honestly to the two small sites below.

## Context — why now, and the subsume trigger

The dependent-match-on-a-stuck-comparison idiom (`bool_dichotomy e` → `Or (Equal
Bool e True) (Equal Bool e False)` → a **named dispatch helper** whose return
type pins the goal → an **explicit fresh-`Bool` `J`/`subst` motive**) was
hand-rolled at three sites before the modifier subsumed it. With the modifier
landed, each site collapses to a direct `match e eqn: h { <ctor> ⇒ … }`. This is
**subsume-don't-proliferate** (`docs/PRINCIPLES.md`) realized: the tax is paid
once in the elaborator, removed everywhere in the catalog.

## The two sites (grounded on `b7c0de73` — re-verify at pickup)

1. **`catalog/packages/Core/Logic/EmptyDec.ken.md`** — `dec_eq_decides` (~:172–190):
   `match bool_dichotomy (d.eq x y) { Inl p ⇒ … ; Inr q ⇒ … }`. 4 `bool_dichotomy`
   uses; the inlined `bool_dichotomy` helper (~:140) is a candidate for removal
   once no site references it.
2. **`catalog/packages/Data/Collections/Map.ken.md` — DROPPED (Architect
   `evt_6bk169gj8d0kz`; see RE-SCOPED banner above).** The Map dispatchers are the
   distinct precomputed-`Or` → plain-`match` → named-helper idiom, which case-eq
   neither cleanly replaces (insert/lookup: constant motive) nor safely rewrites
   (`set_intersection_member`: over-transport → reject). **Leave them on their
   existing form** — not migrated, not a regression, no modifier work. NOT in this
   WP.
3. **`catalog/packages/Core/Classes/LawfulClasses.ken.md`** — `list_deceq_sound_cons_true`
   /`_false` (~:551–566) with the **explicit fresh-`Bool` `J` motive**
   `J (λb _. IsTrue (match b { True ⇒ list_eq a da.eq xs ys ; False ⇒ False })) h
   peq` (:560), plus the inlined `bool_dichotomy` (:488, 2 uses). This is the
   **"revert the hypothesis" exemplar** the capability WP's normative example
   targets (largest per-site token payoff).

## Goal

Rewrite each site's `bool_dichotomy`-plus-named-dispatch-plus-explicit-`J` idiom
as the direct `match e eqn: h { True ⇒ … ; False ⇒ … }` modifier, and **remove
the now-redundant scaffolding** (the inlined `bool_dichotomy` helpers and the
dispatch/`J`-motive helpers the modifier subsumes) wherever nothing else
references them. The transform is mechanical and local:

- `match bool_dichotomy e { Inl p ⇒ body_T[p] ; Inr q ⇒ body_F[q] }`
  becomes `match e eqn: h { True ⇒ body_T[h] ; False ⇒ body_F[h] }` — the branch
  hypothesis `h : Equal Bool e True` / `Equal Bool e False` replaces `p`/`q`.
- A dispatch **helper** that existed only to pin the goal and thread the
  equation folds into the caller's direct `match … eqn:` (delete the helper) —
  but only when it has no other caller; keep any helper still referenced.

## Scope

- Rewrite the two sites above to the modifier; delete subsumed scaffolding.
- Keep every **public API** and every **proof's statement** identical — this is
  an internal proof-term rewrite, not an interface or lemma change.
- All existing acceptance suites for these packages stay **green** — the
  desugaring *is* the idiom, so behavior is preserved by construction.

### Out of scope

- Any behavior/interface/lemma change; any new proof obligation.
- Touching the modifier itself (`crates/ken-elaborator`) — capability is landed;
  this WP only *uses* it.
- The `compare`/`Ord` and Map-`leq` **lemmas** themselves — only their internal
  dispatch spelling changes.
- Multi-scrutinee `eqn:` (not shipped — single-scrutinee only; do not attempt
  the inherited `match e1, e2` form).

## Acceptance criteria

- **AC1 — both small sites migrated.** Every `bool_dichotomy`-plus-dispatch idiom
  at the two sites (EmptyDec + LawfulClasses `list_deceq`) is expressed via `match
  e eqn: h`; neither site still constructs the `Or (Equal Bool e True) (Equal Bool
  e False)` dichotomy by hand. **Map is out of scope** (RE-SCOPED banner) — its
  dispatchers stay on their existing form and are not counted.
- **AC2 — scaffolding removed.** The inlined `bool_dichotomy` helpers and the
  dispatch/`J`-motive helpers subsumed by the modifier are deleted where
  unreferenced; **byte-fewer** overall (the payoff). A helper kept because it is
  still referenced is `log`-noted, not silently left.
- **AC3 — behavior-preserving, green.** Every existing acceptance suite for
  EmptyDec / LawfulClasses (and Map, unchanged) passes; targeted local builds only
  (`-p <crate> <test>`), full-suite green proven in CI at merge.
- **AC4 — zero trust/interface delta.** `trusted_base()` delta empty; no new
  `Axiom`/`postulate`/`Decl::Opaque`/`sorry`; no `crates/**/src`, `ken-kernel`,
  `Cargo.lock`, `/spec`, `/conformance` touch. Grep-confirmed on the whole diff.

## Sequencing (Steward-owned)

- **Only dependency: the landed modifier** (`b7c0de73`) — buildable immediately.
- **File-collision watch:** with Map dropped, this WP now touches only EmptyDec +
  `LawfulClasses` proof-internals. Track A (Foundation, compare/`Ord`) **adds** a
  new `Ord (Pair)/(List)` instances section to `LawfulClasses` — a
  **non-overlapping region** from this WP's `list_deceq_sound_cons_*` rewrite
  (Architect confirmed). **The Steward routes merge order** — this WP (the
  Track-A merge anchor) lands **first**; Track A rebases its `Ord`-instances hunk
  onto it (clean, different region). Flag any real conflict to the Steward rather
  than resolving cross-WP.

## Gate

Ergo ring (ergo-leader → ergo-implementer → ergo-qa) → **@architect gate**
(behavior-preserving on the two small sites: the modifier desugars to the same
`J`-transport; zero-TCB; scaffolding-removal is subsumption not loss — spot-check
a migrated site elaborates to the same checked term class; Map stays on its
existing form) → `git_request` to Steward →
**CI-gated** merge (real catalog code, not doc-only). Outer-ring, no soundness
urgency. Own the retro (terra harness readout). **No WP-token identifiers in
production/tangled source** (self-grep the whole diff). Re-verify `file:line`
cites at pickup.
