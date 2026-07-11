# WP def-path-constraint-binder-unification — `where` parity for fn/proc/const

**Owner:** Language team (elaborator). **Steward-framed** (2026-07-11). Base:
`origin/main @ 7b8ec13b` (re-verify `file:line` at pickup — the elaborator moves).
**Inner-ring**, `crates/ken-elaborator/src`. **Soundness-adjacent but
fail-closed:** the `where`-bound constraint dictionaries are ordinary implicit
arguments the kernel re-checks at the use site (same discipline as
`constrained-instance-elaboration`); a binding/naming bug **rejects an ill-typed
program, never admits one**. → **@architect soundness-adjacent gate.** Surface
grammar changes (def-path `where`) → **Spec ratifies** the parity clause, CV in
loop.

## Context — completes a deliberately-deferred unification

The constrained-instance arc unified the **instance** path: capability
(`constrained-instance-elaboration`, #478/#480), multi-constraint **naming**
(`da`/`db` auto-name, bare-`d` alias, #479), and **grammar** (comma separator +
explicit `(name : C τ)` binders, #481). Those WPs explicitly left the **def-path**
(`fn`/`proc`/`const`/`view`) `where`-clause **not yet unified** — a known,
tracked divergence, not a new idiom.

**The divergence, grounded (`7b8ec13b`; re-verify at pickup):**

| Axis | def-path (`parse_view_decl`, parser.rs ~:272–286) | instance (`parse_instance_decl`, ~:768–800) |
|---|---|---|
| Separator | `;` (`Token::Semicolon`) | `,` (`Token::Comma`) |
| Explicit binder | **none** — bare `(cname, cty)` only | `(name : C τ)` parenthesized form |
| AST | `Vec<(String, Type)>` tuple | `Vec<InstanceConstraint{class_name, head_type, binder}>` |
| Auto-naming | bare `d` (sole) only | `d` / `da` / `db` (multi), per #479 |

So `fn f (…) where DecEq a ; DecEq b = …` cannot name its two dicts, and the
def-path uses a *different separator* than instance — every author re-learns the
divergence, and the def-path can't express the multi-constraint proofs the
instance path now can.

## Goal

Bring the def-path `where` clause to **full parity** with the instance path:
comma separator, explicit `(name : C τ)` binders, `da`/`db` auto-naming, bare-`d`
sole-constraint alias, explicit binder required for compound/same-var-collision —
**the exact rules #479/#481 pinned for instance**. Achieve it by **reusing the
instance path's machinery** (the `InstanceConstraint` AST, the naming/binding and
use-site-resolution code from `constrained-instance-elaboration`), **not** a
second parallel mechanism — reflect-don't-extend (`docs/PRINCIPLES.md`). After
this WP, `where` reads and elaborates identically on `fn`/`proc`/`const`/`view`
and `instance`.

## Design seams — Architect / Language-leader to settle at pickup (flag, don't guess)

1. **Separator back-compat (`;` → `,`).** The def-path currently *requires* `;`.
   **Audit existing `.ken`/`.ken.md` def-path `where` usages** (grep the catalog
   + prelude) and decide: dual-accept `;` and `,` (transition), or migrate all
   uses to `,` in this WP. Do **not** silently break landed catalog proofs —
   whichever path, existing files must still elaborate green (or migrate
   in-WP). Recommend: accept comma (the unified target) and keep `;` working
   for the current callers, or migrate them here if few.
2. **Reuse vs fork (the reflect-don't-extend AC).** The def-path `ViewDecl`
   constraint storage (`(String, Type)` tuple) must move to the same
   `InstanceConstraint`-shaped representation and share the instance path's
   dict-binding + naming + resolution code. Confirm `ViewDecl` elaboration can
   consume that shared logic; a duplicated second binder/naming implementation
   is a **reject**.
3. **Scoping over contracts + body.** The def-path `where` sits **between the
   `requires`/`ensures` contract clauses and the `=` body** (parser.rs ~:272,
   before `visits` and `=`). Confirm named dicts scope correctly over
   `requires`/`ensures`/`visits`/body/refinement — the same one-declaration
   scope the instance dicts get (never leaking to siblings).

## Scope

- `crates/ken-elaborator/src/parser.rs` — def-path `where`: comma separator +
  `(name : C τ)` explicit binders (mirror `parse_instance_decl` ~:768–800),
  per seam 1's back-compat decision.
- `crates/ken-elaborator/src/ast.rs` — `ViewDecl` constraints → the
  `InstanceConstraint`-shaped representation (or a shared type).
- `crates/ken-elaborator/src/elab.rs` — bind the named/auto-named dicts on the
  def-path, reusing the `constrained-instance-elaboration` binding + naming.
- `crates/ken-elaborator/src/resolve.rs` — use-site resolution parity.
- Tests: a `fn`/`proc`/`const` with (a) multi-constraint **auto-named**
  (`da`/`db`), (b) **explicit-binder** `(name : C τ)`, (c) bare-`d` sole, each
  elaborates + kernel-checks; a **same-var collision** without an explicit binder
  is **rejected** (fail-closed, assert the specific variant); an existing
  `;`-separated def-path `where` still elaborates per seam 1.

### Out of scope

- Any **kernel** change — constraint dicts are implicit args the kernel already
  re-checks; `trusted_base()` delta empty, `ken-kernel`/`Cargo.lock` untouched.
- The instance path (already unified) — this WP only brings def-path to it.
- New class/typeclass machinery beyond the naming/binding already shipped.

## Acceptance criteria

- **AC1 — parity.** def-path `where` accepts comma separator, `(name : C τ)`
  explicit binders, `da`/`db` auto-naming, bare-`d` sole alias — the same rules
  as instance (#479/#481); the two paths are grammar- and binding-identical.
- **AC2 — reuse, not fork.** The naming/binding/resolution is the shared
  `constrained-instance-elaboration` code, not a duplicate. Grep-confirmed: no
  second binder/naming implementation.
- **AC3 — fail-closed, verified.** A same-var collision without an explicit
  binder, and a mistyped dict, are **kernel-rejected** (assert the specific
  variant, not `is_err()`).
- **AC4 — back-compat green.** Every existing def-path `where` usage in the
  catalog/prelude elaborates green (dual-accept) or is migrated in-WP (seam 1);
  no landed proof breaks. Targeted builds only (`-p ken-elaborator <test>`),
  full-suite green in CI at merge.
- **AC5 — zero TCB/kernel delta.** No `ken-kernel`/`Cargo.lock` touch; no new
  `Axiom`/`postulate`/`Decl::Opaque`; `trusted_base()` delta empty. Grep-confirmed.

## Spec

The def-path `where` **grammar parity clause** (`32 §1` grammar + `33 §5.4`
desugaring) — extend the instance clauses #481 pinned to cover fn/proc/const/view,
co-authored with Spec, **CV verifies code↔spec** (the constrained-instance-naming
pattern). May **co-land** with the capability or **fast-follow** (like the
case_eq surface clause #487) — Language-leader/Spec call at pickup.

## Gate

Language ring (language-leader → language-implementer → language-qa) →
**@architect soundness-adjacent gate** (fail-closed dict binding, kernel-recheck,
reflect-don't-extend reuse of the instance machinery) → **Spec/CV** ratify the
grammar parity clause (non-terminal for the spec portion) → `git_request` to
Steward → **CI-gated** merge. Own the retro (terra harness readout). **No
WP-token identifiers in production source** (self-grep the whole diff). Re-verify
`file:line` cites at pickup.
