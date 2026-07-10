# WP constrained-instance-elaboration — bind `where C A` dicts on the instance path

**Owner:** Language team (elaborator). **Steward-framed** (2026-07-10). Base:
`origin/main @ 306989cf` (re-verify `file:line` cites at pickup — the elaborator
moves fast). **Inner-ring**, `crates/ken-elaborator/src`. Soundness-**adjacent**
but **fail-closed**: the elaborator is untrusted and the kernel re-checks the
elaborated Σ-value, so a resolution bug **rejects an ill-typed dictionary**, it
never admits a bad proof → **@architect soundness-adjacent gate** (confirm the
kernel-recheck backstop holds and resolution fails closed).

This is the **first hard-level inner-ring implementer trial on the Codex-native +
gpt-5.6-terra seat** (language-implementer). Reviewers apply full rigor, no
benefit of the doubt.

## Context — where this sits

`deceq-structural-liftings` (Foundation, HELD @ `306989cf`) is **blocked** on
this capability: its `DecEq (Pair a b) where DecEq a; DecEq b` and `DecEq (List
a) where DecEq a` cannot be written because the elaborator never binds a
constrained instance's prerequisite dictionaries into the field context. This WP
unblocks it — and every future constrained instance (the operator's path needs
`Ord (Pair a b)`/`Ord (List a)` next). Ruling chain: Architect
`evt_13aajtj86jheg` (real prerequisite capability gap, build it first) +
`evt_47rfddxwcdyjj` (B-first two-WP split is the ruling). Not a proof problem —
an **unfinished capability surfacing on its first real consumer**.

## Goal

Complete constrained-instance elaboration so that

```
instance C (F a) where C a { ... }
```

elaborates: bind each `where`-constraint dictionary into the field-elaboration
context, and Pi/lam-abstract the instance value over those dictionaries, so the
elaborated instance has type `(a : Type) → C a → C (F a)` and a use-site
`C (F T)` resolves its constraint `C T` from the ambient environment and applies
it. The spec already mandates this — `33 §5.4`: "Constraint `where C A` → an
implicit instance argument" — **the elaborator is behind its own spec.** This is
*implement the spec'd capability*, not an open design question about what to
build.

**The confirming probe** (from foundation-implementer, `evt_e12c34kc7hab`) must
elaborate after this WP:
```
class Probe a { apply : a → a }
instance Probe (List a) where Probe a { apply = λx. <dict>.apply x }
```
Today it fails `UnresolvedCon { name: "d" }`.

## Background — grounded on `origin/main @ 306989cf`

**The gap (foundation-leader's cite, independently confirmed):**
`elab_instance_decl` (`crates/ken-elaborator/src/elab.rs:3886`) receives
`constraints: &[(String, RType)]` (`:3897`). On the non-self constrained-instance
path (`:4006`–`:4024`) it builds the field context with only
`.with_classes(&*class_env)` + `push_type0_params(head_params.len())`
(`:4013`–`:4014`) — it binds the head **type** params and the class env but
**never binds `constraints` as dictionary locals**. `compute_ordered_field_values`
(`:3748`, called `:4015`) then checks each field body in that context, so any
reference to a prerequisite dict fails `UnresolvedCon`. `closed_instance_ty`
(`:3961`) and the `pair_chain` (`:4025`) are **not** abstracted over the
constraint dicts.

**The existing where-clause mechanism to reflect (do NOT mint a second one —
`reflect-don't-extend`, `PRINCIPLES.md`):** the **View/def** path already
implements `where C T`:
- `elaborate_rdecl_v1_with_effect_rows` (`:3320`+), `RDeclKind::View` arm
  (`:3335`+): resolves each `where C T` constraint via
  `class_env.instance_search(class_name, head_name)` and **supplies the resolved
  dictionary under the fixed surface name `d`** (comment `:3352`+ cites spec
  `51 §4` — "the illustrative name the spec itself uses"), scoped to the decl
  (save/restore around the call so it never leaks to siblings).
- `ProjectionPurityCtx { local_constraints, bound_dict_classes, .. }` (`:3034`)
  and `projected_instance_id` (`:3051`: `RCon("d") ∧ local_constraints.len()==1
  → instance_search(class, head)`) already thread a single named constraint
  through projection/purity checking.

**Why the View mechanism doesn't transfer verbatim (the real substance):** the
View path resolves the constraint **concretely** (`instance_search` at a known
head `T`). A constrained *instance* `instance C (F a) where C a` has an
**abstract** head `a` — `instance_search(C, "a")` has nothing to find. So the
instance path must bind the constraint as an **abstract lambda-bound dictionary
local** (a parameter of the instance value), not resolve it by search; concrete
resolution happens later, at the **use site** (`C (F Int)` resolves `C Int`).
That is the new work: abstract dict binding + Pi/lam-abstraction of the instance
value + use-site recursive resolution.

**Two grounding facts that bound the risk:**
- **Zero constrained instances ship in the catalog** (`instance X where Y` is
  empty across all packages) — this is an unfinished feature's first exercise,
  **not a regression**. Use-site resolution is therefore **unexercised**: treat
  it as verify-and-build, not assumed-present.
- **The termination guard is already wired:** the non-self path already routes
  through `declare_recursive_group` (`:4030`) so `sct_check` runs on the group
  (`:4008`–`:4009` comment; the self-ref path at `:3979`–`:3996` handles the
  recursive-instance case). Confirm it composes with the new binding; don't
  rebuild it.

## Design seams to settle at pickup — **flag, don't guess**

Ground each against spec `33 §5.4` (constraint → implicit instance arg) and
`51 §4` (the `d` surface name). If the spec settles it, follow it; if ambiguous,
surface to the **Architect** for a ruling before building — do not force a shape.

1. **Multi-constraint surface naming (load-bearing for the downstream deceq
   WP).** The existing mechanism binds a **single** dict under the fixed name
   `d` (`51 §4`; `projected_instance_id` hard-codes `local_constraints.len()==1`).
   But `DecEq (Pair a b) where DecEq a; DecEq b` needs **two distinct**
   prerequisite dicts, and the deceq frame's proofs reference them as
   `(DecEq a).eq` / element-instance `.sound`/`.complete`. Settle the surface
   syntax the field bodies use to reference each bound dict — one of:
   fixed-`d`-generalized-to-many, per-constraint names (`da`/`db`), or
   type-directed re-projection `(C a).field`. This choice is a **contract with
   `deceq-structural-liftings`**: whatever you land, the resumed deceq proofs
   must be able to spell it. Name it explicitly in the retro so the deceq
   re-kick matches.
2. **Abstract binding + Pi/lam-abstraction.** After `push_type0_params`, bind
   each `constraints[i] = (name, C a)` as a dict local in the field ctx, and
   abstract `closed_instance_ty` (add `→ C a` Pis) and the `pair_chain` value
   (add `λ` binders) over those dicts, in the head-param-then-constraint order,
   so the elaborated instance is `(a : Type) → C a → C (F a)`.
3. **Use-site resolution (unexercised — verify-and-build).** A use-site
   `C (F T)` resolves the head instance, then **recursively resolves** its
   constraint `C T` from the ambient env and applies it. Verify this path end to
   end with a real test (zero instances have ever resolved through it); do not
   assume the resolver already walks constraints.
4. **Orphan/overlap composition.** Confirm the orphan check (`33 §5.3`) and any
   overlap check compose with the constraint — the constraint dict is resolved,
   not declared, so it must not itself trip orphan/overlap.

## Scope

- `crates/ken-elaborator/src` — complete `elab_instance_decl`'s non-self
  constrained path (and confirm the self-ref path composes): field-ctx dict
  binding, instance-type Pi/lam-abstraction, use-site constraint resolution,
  reusing the existing `where`/`d`/`ProjectionPurityCtx` machinery and the wired
  `declare_recursive_group`/`sct_check` termination guard.
- Elaborator acceptance tests: the `Probe (List a) where Probe a` probe
  elaborates; a real constrained instance **resolves at a use site** and its
  field computes; a **non-terminating** constrained instance is **rejected** by
  the termination guard (assert the specific error variant, not `is_err()`); an
  **ill-typed** field body is rejected by the kernel re-check (fail-closed
  witness).

### Out of scope

- The `DecEq`/`Ord` catalog instances themselves — those are
  `deceq-structural-liftings` (Foundation) and later bricks; this WP ships the
  **capability + elaborator tests only**, no catalog `.ken.md` proof content.
- `crates/ken-kernel` reduction/checking logic — untouched (the kernel already
  re-checks the elaborated dictionary; that backstop is the safety property, not
  a thing to modify).
- spec/conformance semantic changes — the behavior is already spec'd
  (`33 §5.4`/`51 §4`); if the text is found ambiguous on multi-constraint naming,
  route an **erratum to the Spec/CV lane**, don't change semantics here.

## Acceptance criteria

- **AC1 — capability lands.** `instance C (F a) where C a { ... }` elaborates;
  the probe above passes; the elaborated instance has type `(a:Type) → C a →
  C (F a)`; a use-site `C (F T)` resolves `C T` and applies it (a real test, not
  a mock). Multi-constraint (`where C a; D b`) binds all dicts.
- **AC2 — fail-closed, verified.** An ill-typed constrained-instance field body
  is **rejected by the kernel re-check** (resolution produces a candidate the
  kernel refuses — assert the specific rejection), and a non-terminating
  constrained instance is rejected by `sct_check`. Both asserted with the
  **specific error variant**, not bare `is_err()`. This is the soundness
  property the Architect gates.
- **AC3 — one mechanism, not two.** The where-clause binding reflects/extends
  the existing `d`/`51 §4`/`ProjectionPurityCtx` machinery rather than adding a
  parallel one; the View/def where-path still behaves identically (its tests
  stay green).
- **AC4 — zero trust growth.** `trusted_base()` delta empty; no new
  `Decl::Opaque`/`declare_postulate`; `crates/ken-kernel` and `Cargo.lock`
  untouched. Grep-confirmed in the handoff. (The elaborator is untrusted; this
  is a completeness feature, not a trust widening.)
- **AC5 — build.** Workspace-green in CI at merge (QA re-runs the suite
  independently). Local: **targeted builds only** (`-p ken-elaborator <test>`),
  never a full local `cargo build`.

## Gate (soundness-adjacent, inner-ring)

Language ring (language-leader → language-implementer → language-qa) →
**@architect soundness-adjacent gate** — the Architect confirms the kernel-recheck
backstop holds (a resolution bug **rejects, never admits**), the termination
guard composes, and orphan/overlap still hold → `git_request` to Steward →
**CI-gated** merge. No Spec/CV vote unless the multi-constraint-naming seam
surfaces a spec erratum (welcome, not gating on the code).

Own the retro — **record the harness readout** (this is the hard-implementer
terra trial on inner-ring compiler work) **and name the multi-constraint surface
syntax you landed** (the deceq re-kick depends on it). **No WP-token identifiers
in production source** — self-grep the whole diff (incl. `Cargo.toml`) before
handoff. Re-verify every `file:line` cite against the elaborator at pickup.
