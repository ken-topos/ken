# WP L-resolver-globals — resolve lowercase globals in expression positions

**Owner:** Team Language (elaborator). **Branch:** `wp/L-resolver-globals` (cut
from `origin/main`). **Stream / gate:** L-stream enabler → **unblocks L8**
(stdlib)
+ **FFI integration tests** + cross-declaration references. **Depends on:** L2
(elaborator) — merged. **Crates-only** (`ken-elaborator`).

> **⚠ DIRECT build — NO enclave step.** This is an **elaborator completeness
> fix**, not a new language feature: the intended behavior (a declared global is
> referenceable by name in an expression body) already holds in *type*
> positions;
> the resolver just never threaded it into *expression* positions. So there's no
> spec elaboration — **build from this frame**; the conformance is a cross-decl
> resolution test. Merge Decision is **Architect-only** (crates-only).

## 1. The problem (confirmed twice — L2-build + L7-build both hit it)

`resolve_expr`/`resolve_expr_ctx`
(`crates/ken-elaborator/src/resolve.rs:423/432`)
is **de-Bruijn-only for expression positions**: a bare lowercase name lexes as
`Ident` → `EVar` → a scope/de-Bruijn lookup; the **global-by-name fallback
(`RCon` → `cx.globals`) fires only in TYPE positions**, not expression bodies.
So
a lowercase global referenced in a `view` body fails with **`UnboundName`**:

- **L7-build:** `view use_ffi (b : Bytes) : Int = os_write 0 b` → `UnboundName {
  name: "os_write" }`; B1/B2 had to be tested via `GlobalEnv::add_decl(Decl::
  Transparent)` directly because the surface couldn't express the scenario.
- **L2-build:** the forgetful-coercion AC7(b) cross-decl case hit the same wall;
  had to restructure to a same-declaration negative discriminant.

**Both implementers diagnosed the same fix and named it a blocker for the next
WPs** (foundation-impl evt_5yrmgyxnaq2qp, language-impl L2 retro).

## 2. The fix (their prescription)

Thread `globals: &HashMap<String, GlobalId>` (or the equivalent global table)
through `resolve_expr`/`resolve_expr_ctx` so that a bare lowercase name in an
**expression** body, **not bound in the local de-Bruijn scope**, resolves to the
global (`RExpr::RGlobal(id)` / the appropriate `RExpr` variant) rather than
erroring — **the same fallback the type path already has**, applied to the
expression path. Pin the exact resolution order against the landed `resolve.rs`.

## 3. Do-not-break guardrails

- **Local de-Bruijn resolution is unchanged** — a name bound in the local scope
  still resolves to its de-Bruijn index; the global fallback fires **only** when
  the name is **not locally bound** (locals shadow globals). This is the
  regression risk — assert it.
- **Uppercase `ConId` resolution is unchanged** (it already reaches globals).
- **No new resolution semantics beyond completing the expression-path fallback**
  — this is a completeness fix, not a scoping redesign. If you find the spec
  (`39-elaboration`) is ambiguous about shadowing/order, flag it to the Steward
  (a `Spec` query), don't invent a rule.

## 4. Testable acceptance criteria

- **AC1 (cross-decl lowercase global resolves)** A `view` body referencing a
  **prior-declared lowercase global** (e.g. `foreign os_write …` then `view
  use_ffi … = os_write …`, or a plain `let f = …; view g … = f …`)
  **elaborates**
  — no `UnboundName`. The exact case that failed in L7/L2 now passes (verdict
  flips: pre-fix `UnboundName` / post-fix resolves).
- **AC2 (locals still shadow — the regression guard)** A locally-bound name with
  the same spelling as a global resolves to the **local** (de-Bruijn), not the
  global — assert the resolved index/target, not just "compiles."
- **AC3 (uppercase + type-position unchanged)** Existing `ConId` and
  type-position
  global resolution still works (no regression in the full suite).
- **Conformance:** add the cross-decl resolution case to the elaborator tests
  (`conformance/surface/elaboration/` or the elaborator acceptance suite). **QA
  gate:** AC1 routes a **real** two-declaration program through the **real**
  resolver (not a hand-built `RExpr`); AC2 asserts the resolved target.

## 5. Sequencing notes

- **Unblocks:** L8 (the stdlib needs surface cross-references between
  definitions),
  L7's deferred end-to-end FFI surface tests, and the L2 forgetful-coercion
  AC7(b)
  full case. Land this **before L8 is framed**.
- Small + well-scoped — a focused resolver pass, not a redesign. If it turns out
  larger than expected (e.g. it forces a resolver/elaborator interface change),
  flag the Steward to re-scope.
- Standard ring: implementer → QA (Architect-only crates merge). `merge_ready`
  states `status: resolved` + a real @mention to the Architect
  (`agt_37reqftfe6g00`).
