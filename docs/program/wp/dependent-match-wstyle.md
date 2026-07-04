# dependent-match-wstyle — surface `match` over W-style (Π-bound) recursive fields

**Steward frame → spec enclave (elaborate) → Team Language (build).** Full
`§2c` pipeline: spec-leader elaborates this brief to team-ready rigor, the
Integrator merges the elaborated brief to `main`, then Team Language builds
under the **Architect + CV gate** (soundness-adjacent — IH construction). Owner
of the build: **language-implementer** → **language-qa**. Findings → **Steward**.

## Objective (one line)

Extend the elaborator's dependent-`match` compiler so a `match` on an inductive
whose constructor has a **W-style (Π-bound) recursive field** — e.g. `ITree`'s
`Vis` continuation `k : Resp op -> ITree E Resp R` — compiles to a well-typed
`Term::Elim` instead of being rejected, by emitting the **Π-wrapped
induction-hypothesis slot** the kernel's own `method_type` already prescribes.

## Why (the gap this closes)

VAL2 rosetta `accumulator-factory` (`examples/rosetta/accumulator-factory/`,
`KNOWN-GAP.md` = **`GAP-itree-w-style-match`**) is the last-but-one corpus gap.
The `[State]` effect (VAL2 #10, `5626038`) is **fully landed and usable** —
`get`/`put`/`bind`/`runState` type-check and compose; the example builds a
genuine hidden-state accumulator up through `finalResult : ITree … (Pair Bool
Nat)`. What is missing is any **surface way to *observe*** that `ITree`: the
only surface destructuring form is `match`, and `match` on `ITree` — **any**
instantiation, even a non-dependent motive, even when the `Vis` arm is
unreachable at runtime — is unconditionally rejected with:

```
Internal("dependent match (Gap B): W-style or indexed recursive fields are
out of scope for this WP")
```

The cause is a **deliberate, documented scope limit** in the match compiler's
IH-slot emission, not a bug: `Vis`'s field `k : Resp op -> ITree E Resp R` is a
recursive occurrence **under a Π** (W-style), and the loop that builds IH slots
(`elab.rs:819-825` on `origin/main@e9f0804` — *verify against landed code, this
line is perishable*) rejects any recursive field with a non-empty
`branching_tel` or `idxs` rather than risk mis-building the hypothesis. This
blocks **every** `.ken` program that constructs an effectful/`ITree` value and
needs to consume its result.

**This is not a `[State]`-specific gap and not a kernel gap.** It is the
narrower, previously-latent match-compiler limitation that VAL2 re-authoring
first surfaced (the existing `state_effect_build_*` suite drives everything via
hand-built Rust `Term::Elim`, never surface `match`).

## Fixed inputs — settled, do not reopen

These are **grounded on `origin/main@e9f0804`**; the build re-verifies them
against the landed code at pickup (frames are perishable), but they are **not**
open questions:

1. **The kernel already fully supports W-style eliminators — nothing in
   `ken-kernel` changes.** `ken_kernel::inductive::method_type`
   (`inductive.rs:202-300`) already produces the correct per-method type for a
   W-style recursive arg: for `(pos, branching_tel, idxs)` with
   `nb = branching_tel.len() ≥ 1`, the IH is
   `Π(b₁:B₁)…(b_{nb}:B_{nb}). M idxs (a_pos b₁ … b_{nb})` (see the in-file
   comment at `inductive.rs:250`). The matching ι-reduction
   (`inductive.rs:377-384`) and `elim_reduce` W-style handling landed as **K1.5**
   (`5c8dac0`, #240). So the elaborator has (a) an **exact reference to mirror**
   and (b) a **kernel recheck backstop** (below). **The kernel is out of scope —
   grep must show zero `ken-kernel/` delta.**

2. **This is the direct sibling of `dependent-match-nonnullary` (#250 frame
   `4487a6e` + #254 build `282856c`).** That WP built the IH-slot emission
   machinery at this same site (`elab.rs:806-836`) for **direct** (non-Π-bound)
   recursive fields — `List`'s `Cons x xs2`, where the IH is `P xs2`. This WP
   **extends the same loop** to the W-style case. Reuse that machinery and its
   ratified decomposition; do not re-architect it.

3. **Reuse the kernel's own producer, do not re-derive.** The IH shape comes
   from `ken_kernel::inductive::recursive_args` (`inductive.rs:174`), which
   returns `(arg_position, branching_tel, idxs)` — the **same** function
   `method_type` consumes. The elaborator already calls it (`elab.rs:15` import,
   `elab.rs` use-site). Build the IH slot from that triple; do not write a
   second recursive-field detector.

4. **Completeness fix, kernel-rechecked, fail-closed — this bounds the
   soundness risk.** The match compiler emits a `Term::Elim`; every top-level
   declaration the elaborator produces is **independently re-checked by the
   kernel** (the outer-ring/kernel split — Ken's core discipline). A
   *mis-built* IH slot therefore produces an `Elim` whose methods do **not**
   match `method_type`, and the kernel **rejects it** (a `TypeMismatch`/
   `KernelRejected`, i.e. over-rejection = completeness, **not** an admitted
   unsound term). The goal is to build the IH slot **correctly** so valid
   programs are admitted; the backstop is that any residual error fails closed.
   **This posture is only valid if the emitted `Elim` actually flows through the
   kernel checker** — an AC pins that (see AC4), so "kernel-rechecked" is a
   grepped fact, not a label.

## Mandated deliverable outline

Each section ends in a concrete, implementable choice — not a survey. The spec
enclave sharpens the de Bruijn detail against `spec/14 §3.1` and
`inductive.rs::method_type`; the implementer executes mechanically.

1. **IH-slot type for a W-style field.** In the IH-emission loop
   (`elab.rs:806-836`), for a recursive field `(pos, branching_tel, idxs)` with
   `nb = branching_tel.len() ≥ 1` (and/or non-empty `idxs`): replace the
   `return Err(… "Gap B" …)` with construction of the Π-wrapped IH type
   **mirroring `method_type`'s W-style branch** (`inductive.rs:250-300`):
   `Π(b₁:B₁)…(b_{nb}:B_{nb}). expected[idxs] (field_var b₁ … b_{nb})`, where
   `expected` is the match motive specialized as it already is for the direct
   case, `field_var` is the recursive field's variable, and the `Bₖ` domains and
   `idxs` are `subst_outer`'d into the match's param context exactly as
   `method_type` does. **Mandate:** derive the domains/indices via
   `subst_outer(&branching_tel[bk], …)` / the `idxs` terms from `recursive_args`,
   not by re-parsing the constructor.
2. **Method-lambda wrapping / de Bruijn shifting.** The method body already
   wraps `n` field lambdas then `p = rec.len()` IH lambdas (built innermost-first
   so each `weaken(_, 1)` accumulates the shift — `elab.rs:826-836`). Extend so a
   W-style IH lambda's **domain is the Π-type from step 1** (not the direct
   `subst_var_generalize` form), and the **`nb` inner binders `b₁…b_{nb}` are
   accounted for in the shift** of any later IH slot. **Mandate:** keep the
   existing reverse-build + `weaken` technique; the only change is the per-slot
   domain and the `+nb` it contributes. The spec enclave supplies the exact shift
   arithmetic (this is the fiddly part — front-loaded by the enclave, not left to
   the build model to derive).
3. **Reachability / dead-IH handling stays as-is.** The IH binders are
   dead (never surface-referenced) — the surface arm body cannot name them, same
   as the direct case today. **Mandate:** do not add surface syntax for binding
   the IH; the arm binds only the constructor's own fields (`Vis op k` binds `op`
   and `k`), and the emitted method has the extra dead IH binders as
   `method_type` requires. No grammar change.
4. **`indices: vec![]` on the `Elim`.** The in-scope motive stays index-free at
   the match level (the family's own indices are handled inside the IH via
   `idxs`); confirm the emitted `Term::Elim { indices, … }` matches what
   `method_type`/the kernel expect for a non-indexed *family* with W-style
   *fields*. If a genuinely **indexed family** (not just a W-style field of a
   non-indexed family) is needed by any in-scope example, that is a **finding →
   Steward** (its own WP), not silently in scope — `accumulator-factory`'s
   `ITree` is a non-indexed family with a W-style field, which is the target.

## Acceptance criteria (all testable)

- **AC1 — `accumulator-factory` flips to PASS.** Uncomment the intended
  `unwrapRet` (`match finalResult { Ret v => v ; Vis op k => <dummy> }`) →
  `pairFst` → `"PASS"`/`"FAIL"` oracle (the target is in the current file /
  `KNOWN-GAP.md`); add `expected`; **delete `KNOWN-GAP.md`**; the differential
  runner `cargo test -p ken-cli --test rosetta` passes it. **Idiomatic, not
  faked** (AC of the corpus): it observes the real `ITree`, not a threaded
  workaround.
- **AC2 — corpus + workspace green.** `cargo test -p ken-cli --test rosetta`
  = whole corpus green except the single remaining `read-file-lines` gap;
  `cargo test --workspace` stays green (the kernel-reduction blast-radius
  lesson — validate the **workspace**, never just the touched crate).
- **AC3 — discriminating positive + negative, isolation-flipped.** A **new**
  elaborator/kernel test that (a) a well-typed W-style `match` (e.g. over a
  minimal 2-ctor W-style inductive, or `ITree`) **elaborates and kernel-checks**,
  and (b) an **ill-typed** arm body is still **rejected** (the checker is live,
  not rubber-stamping). Prove the fix is **load-bearing**: on unmodified
  `origin/main` the positive case hits the "Gap B" `Internal` error — i.e. the
  test flips red→green with the fix, not green-vs-green. (Memory:
  green-vs-green does not confirm a fix; assert the concrete pass, not `is_ok()`
  alone.)
- **AC4 — kernel-recheck path is grepped, not assumed.** Show — by pointing at
  the producer path, not a test name — that the emitted `Term::Elim` for the
  W-style match **is** re-checked by the kernel (the decl-check path), so the
  fail-closed posture in Fixed-Input 4 is real. State precisely where the
  recheck happens. (Memory: kernel-backed claim → grep the emission, not the
  name.)
- **AC5 — zero kernel / zero `trusted_base` delta.** `git diff origin/main`
  touches **no `ken-kernel/` file** and adds **no new `Term`/`Decl` variant**;
  `trusted_base` is byte-unchanged. The change is `ken-elaborator`-only (plus the
  example + its test). Grep-verified, not asserted.

## Do-not-reopen guardrails

- **No new surface syntax.** No `mutual`-style keyword, no IH-binding syntax;
  the arm binds constructor fields only. Pure elaborator completeness.
- **No kernel change.** If the build believes the kernel needs a change to admit
  a *correctly-built* W-style `Elim`, **stop and route to Steward** — that would
  contradict Fixed-Input 1 (K1.5 already landed) and is a finding, not in-scope
  scope-creep.
- **Indexed *families* are out of scope** (Deliverable 4) — W-style *fields of a
  non-indexed family* are the target. A real indexed-family need is a finding.
- **Do not re-derive recursive-field detection** — reuse
  `inductive::recursive_args` (Fixed-Input 3).
- **Lane:** Language. Branch off `origin/main`. **After** VAL2 Phase-2 revalidate
  (`e9f0804`) — done.

## Sequencing

Unblocks `accumulator-factory` → corpus reaches **15 PASS / 1 gap** (only
`read-file-lines` / FS-driver remains, held for the operator). Feeds the VAL2
"repeat until zero KNOWN-GAP" close-out.
