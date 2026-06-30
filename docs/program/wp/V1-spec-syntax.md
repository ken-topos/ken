# WP V1 — Verification spec syntax + the four-way status

> **Status:** Steward frame — **queued enclave WP** (after K-api; deps met). When
> the enclave frees from K-api, spec-leader elaborates
> `spec/20-verification/21-spec-syntax.md` (DRAFT → implementation-ready), then
> the **Verify team** builds (extending V0's elaborator).
>
> **Team:** Verify · **Deps:** **V0** (done — the elaborator V1 extends) · **Size:**
> M · **Risk:** ★★ (**untrusted** — the verification layer is re-checked by the
> kernel; a bug is a wrong verdict/diagnostic, never unsoundness) · ► On the
> **verification spine** V1→V2→V3→V4 (the differentiator); with L5 it also feeds
> **B1** (the Ward export emitter).

## Objective

Elaborate `21-spec-syntax.md` — **the verification surface**, what makes Ken
"Ken": a programmer/agent states a correctness property and the toolchain returns
an **actionable verdict** without reading the kernel. V1 delivers the *syntax* and
its *elaboration* + the *status model*, not the prover (that's V2/V3):
- **`requires`/`ensures`/refinements/goals** — the spec-annotation syntax, how it
  attaches to surface declarations, and its **elaboration to core** (extending V0:
  spec annotations elaborate alongside the term they annotate; `old` scoped to the
  `ensures` of `space` ops per the DAG).
- **The four-way verification status** — the verdict lattice the toolchain returns
  (`proved` / `disproved`-with-countermodel / `incomplete`-with-typed-hole / the
  fourth per `21`), as a precise, **kernel-re-checkable** model: a `proved` carries
  a certificate the kernel checks (`18 §4`); the layer is never believed on its own
  authority.

## The framing that sets the risk level

The verification layer is **entirely untrusted** — everything it emits is
re-checked by the kernel (the de Bruijn criterion). So V1 is ★★: a bug yields a
wrong/over-conservative verdict or a poor diagnostic, **not** unsoundness (a bogus
`proved` certificate is rejected by the kernel's certificate check). Build it
correct + ergonomic; the soundness backstop is the kernel, not V1. The
load-bearing properties are the **status model's honesty** (an `incomplete` must
never masquerade as `proved`; the typed hole is the operational face of partial
verification) and that the elaboration of spec syntax produces **kernel-checkable**
core.

## Scope

**IN:** the `requires`/`ensures`/refinement/goal **syntax** (lexer/grammar
addenda) + AST; **elaboration** of spec annotations to core (extending V0's
`elaborator`, with `old` scoped to `space`-op `ensures`); the **four-way status**
model + how a `proved` attaches a kernel-checkable certificate (`18 §4`); the
minimal surface needed for V2 (obligation generation) to consume.

**OUT — other WPs:** **obligation generation / VC** (`22`, V2); the **prover**
(`23`, V3); **diagnostics** polish (`24`, later); the certificate *checker* itself
(it's already the kernel's `18 §4` — V1 *emits to* it, doesn't reimplement it).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `21` to builder rigor: the spec-syntax grammar + AST; the elaboration
rules (defensive pseudocode — how each annotation lowers, `old`-capture
semantics); the four-way status as a precise lattice with each verdict's
**carried evidence** (certificate / countermodel / typed hole); the V0→V1
elaborator extension points. **Verify against the landed `ken-elaborator` (V0) +
`18 §4` certificate API, not this frame's prose** (the perishable-frame
discipline). Conformance (`conformance/verify/`): each verdict is a
**discriminating** case (a `proved` with a valid cert accepts + the kernel
re-checks it; an invalid cert → kernel-rejects, verdict can't be `proved`; an
`incomplete` carries a hole and is **not** `proved` — the absence-assertion names
its guard); `requires`/`ensures` elaborate + round-trip; `old` resolves correctly.

## Acceptance (testable)

1. **Syntax + elaboration:** `requires`/`ensures`/refinement annotations parse,
   attach to their declaration, and **elaborate to kernel-checkable core** (V0
   re-checks the term; the annotation lowers consistently).
2. **Four-way status is honest:** `proved` carries a cert the **kernel accepts**;
   a bogus cert makes the verdict **not** `proved` (kernel-rejected — verdict-flip);
   `incomplete` carries a typed hole and is distinguishable from `proved` (the
   absence assertion is guard-gated, not coincidental).
3. **`old` semantics:** `old(x)` in an `ensures` resolves to the pre-state, scoped
   to `space` ops; an out-of-scope `old` is rejected.
4. **V2-ready:** the elaborated form exposes exactly what obligation generation
   (V2) needs to consume — stated as the V1→V2 interface.
5. **No regression:** V0's elaborator behavior is unchanged for non-spec programs.

## Sequencing

Queued after K-api in the enclave (deps met now — V0 done). Releasable to the
enclave the moment it frees from K-api spec work (need not wait for K-api's
build-gated merge). Unblocks **V2** (with K-api) and **B1** (with L5). Build:
runtime/verification semantics → Spec; design → Architect.
