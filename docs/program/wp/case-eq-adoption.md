# WP case-eq-adoption — migrate the 3 hand-rolled dispatch sites to `eqn:`

**Owner:** Ergo team (catalog). **Steward-framed** (2026-07-11). Base:
`origin/main @ b7c0de73` (re-verify `file:line` cites at pickup — catalog moves).
**Outer-ring, behavior-preserving.** No `crates/**/src`, no `ken-kernel`, no
`Cargo.lock`, no `/spec`, no `/conformance`. This is the **fast-follow adoption**
the capability WP (`case-eq-dependent-match-sugar.md`, seam 5) deferred: the
`match e eqn: h` modifier landed (capability #485 `main @ 9ad154e8`, surface
clause #487 `main @ b7c0de73`); now the sites that hand-rolled the idiom migrate
to it. **Purely ergonomic — every proof term is behaviorally identical**, because
the modifier desugars to the exact `J`-transport the hand idiom builds.

## Context — why now, and the subsume trigger

The dependent-match-on-a-stuck-comparison idiom (`bool_dichotomy e` → `Or (Equal
Bool e True) (Equal Bool e False)` → a **named dispatch helper** whose return
type pins the goal → an **explicit fresh-`Bool` `J`/`subst` motive**) was
hand-rolled at three sites before the modifier subsumed it. With the modifier
landed, each site collapses to a direct `match e eqn: h { <ctor> ⇒ … }`. This is
**subsume-don't-proliferate** (`docs/PRINCIPLES.md`) realized: the tax is paid
once in the elaborator, removed everywhere in the catalog.

## The three sites (grounded on `b7c0de73` — re-verify at pickup)

1. **`catalog/packages/Core/EmptyDec.ken.md`** — `dec_eq_decides` (~:172–190):
   `match bool_dichotomy (d.eq x y) { Inl p ⇒ … ; Inr q ⇒ … }`. 4 `bool_dichotomy`
   uses; the inlined `bool_dichotomy` helper (~:140) is a candidate for removal
   once no site references it.
2. **`catalog/packages/Data/Collections/Map.ken.md`** — the BST-ordering dispatch
   helpers: `dispatch_on_q1`/`dispatch_on_q2`, `insert_case_transport_dispatch`
   (~:1001/:1009/:1018), `lookup_found_dispatch_q1`/`q2`/`lookup_found_dispatch`
   (~:1185/:1206/:1225/:1232), and the top-level splits (~:1317/:1329). **79
   `bool_dichotomy` uses — the bulk of the WP.** All dispatch on a stuck `leq`
   application (`bool_dichotomy (leq k k')`).
3. **`catalog/packages/Core/LawfulClasses.ken.md`** — `list_deceq_sound_cons_true`
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

- Rewrite the three sites above to the modifier; delete subsumed scaffolding.
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

- **AC1 — all three sites migrated.** Every `bool_dichotomy`-plus-dispatch idiom
  at the three sites is expressed via `match e eqn: h`; no site still constructs
  the `Or (Equal Bool e True) (Equal Bool e False)` dichotomy by hand.
- **AC2 — scaffolding removed.** The inlined `bool_dichotomy` helpers and the
  dispatch/`J`-motive helpers subsumed by the modifier are deleted where
  unreferenced; **byte-fewer** overall (the payoff). A helper kept because it is
  still referenced is `log`-noted, not silently left.
- **AC3 — behavior-preserving, green.** Every existing acceptance suite for
  EmptyDec / Map / LawfulClasses passes unchanged; targeted local builds only
  (`-p <crate> <test>`), full-suite green proven in CI at merge.
- **AC4 — zero trust/interface delta.** `trusted_base()` delta empty; no new
  `Axiom`/`postulate`/`Decl::Opaque`/`sorry`; no `crates/**/src`, `ken-kernel`,
  `Cargo.lock`, `/spec`, `/conformance` touch. Grep-confirmed on the whole diff.

## Sequencing (Steward-owned)

- **Only dependency: the landed modifier** (`b7c0de73`) — buildable immediately.
- **File-collision watch:** Track A (Foundation, compare/`Ord`) also edits
  `Collections` and may add `Ord` instances near `LawfulClasses`; Map's dispatch
  is `leq`-based and adjacent to that path. The two WPs touch **different files**
  (adoption: EmptyDec/Map/LawfulClasses proof-internals; Track A: Collections
  `compare` + new `Ord (Pair)/(List)`), but both touch `LawfulClasses`. **The
  Steward routes merge order** — expect adoption (mechanical, quick) to land
  first; if Track A lands first, rebase the LawfulClasses hunk. Flag any real
  conflict to the Steward rather than resolving cross-WP.

## Gate

Ergo ring (ergo-leader → ergo-implementer → ergo-qa) → **@architect gate**
(behavior-preserving: the modifier desugars to the same `J`-transport; zero-TCB;
scaffolding-removal is subsumption not loss — spot-check a migrated site
elaborates to the same checked term class) → `git_request` to Steward →
**CI-gated** merge (real catalog code, not doc-only). Outer-ring, no soundness
urgency. Own the retro (terra harness readout). **No WP-token identifiers in
production/tangled source** (self-grep the whole diff). Re-verify `file:line`
cites at pickup.
