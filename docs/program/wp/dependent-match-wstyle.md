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
   not by re-parsing the constructor. **Exact construction, grounded
   line-by-line against `method_type`: see the [§ Enclave elaboration][ee]
   section below** —
   the enclave has front-loaded it, the implementer transcribes it.

[ee]: #enclave-elaboration--the-exact-w-style-ih-slot-construction
2. **Method-lambda wrapping / de Bruijn shifting.** The method body already
   wraps `n` field lambdas then `p = rec.len()` IH lambdas (built innermost-first
   so each `weaken(_, 1)` accumulates the shift — `elab.rs:826-836`). Extend so a
   W-style IH lambda's **domain is the Π-type from step 1** (not the direct
   `subst_var_generalize` form). **The `nb` branch binders are *intra-domain*:**
   the outer reverse-loop wrap stays **`weaken(&method, 1)` — one shift per IH
   slot, NOT `+nb`** (each IH is a single top-level method binder; its `nb`
   branch binders live *inside* its domain type, bound by the `Term::pi`s, and
   never enter the method telescope). The `+nb` enters **only** the domain's own
   construction (weaken the goal by `n+nb`, place `field_var` at `+nb`). The
   enclave supplies and *proves* the exact arithmetic in the [§ Enclave
   elaboration][ee] section — the "later slot's shift accounts for `+nb`"
   reading is **wrong** and would emit a term the kernel rejects; the
   correction is pinned there.
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

## Enclave elaboration — the exact W-style IH-slot construction

*Front-loaded by the spec enclave (the fiddly de Bruijn arithmetic), grounded
line-by-line against `ken_kernel::inductive::method_type` and `spec/14 §3.1`.
The implementer transcribes the loop below; the kernel recheck (AC4) is the
fail-closed backstop for any transcription slip. Kernel line numbers are
perishable — verify the shapes against landed code, the arithmetic is the
invariant.*

### The normative IH (spec `14 §3.1`, kernel `method_type`)

For a recursive field of `cₖ` at position `pos`, `method_type` inserts one IH
binder, whose type depends on the field's branching telescope
`branching_tel = [B₁, …, B_{nb}]` (from `recursive_args`):

- **direct** (`nb = 0`, `r : D Δ_p t̄`) → IH `M t̄ r` — a plain type;
- **W-style** (`nb ≥ 1`, `k : (b₁:B₁)…(b_{nb}:B_{nb}) → D Δ_p t̄[b̄]`) → IH
  `Π(b₁:B₁)…(b_{nb}:B_{nb}). M t̄[b̄] (k b₁ … b_{nb})`.

In scope this WP: the family is **non-indexed** (guaranteed by the
`dependent_eligible` gate, `elab.rs` ~535-553), so every `idxs` is **empty** and
`M t̄[b̄]` collapses to `M`. The elaborator holds the goal as `expected = M scrut`
(motive already applied to the scrutinee), so "`M (k b̄)`" is `expected` with its
scrutinee occurrence rewritten to `(k b̄)` — the same `subst_var_generalize`
move the direct case already uses, extended past the `nb` branch binders.

### Why the elaborator's `weaken`-accumulate idiom reproduces `method_type`

The two producers use **opposite idioms** that compute the **same** term — this
is the equivalence the whole extension rests on:

- **`method_type`** computes each IH domain in its *final* context with an
  explicit offset `(n − pos + j)`, where `j` = the number of *preceding* (outer)
  IH binders, and wraps with `Term::pi(ih_ty, ty)` **without re-shifting `ty`**.
- **`check_match_dependent`** computes each domain in the **bare `[fields]`
  frame** (the `j = 0` slice: offset `n − pos`, goal weakened by `n` direct /
  `n + nb` W-style) and relies on the reverse-build's `weaken(&method, 1)` —
  applied once per *outer* IH lambda — to float the already-built inner domains
  into place.

Applying `weaken(_, 1)` a total of `j` times is `shift(_, j, 0)`: it bumps every
free field/Γ reference by `j` while **preserving** each domain's own `nb`
internal branch binders (their `Term::pi` cutoffs rise under the shift). That is
*exactly* the `+j` the kernel adds explicitly. The **direct** case — landed,
kernel-rechecked (`dependent-match-nonnullary`, #254) — already depends on this
equivalence for its `p ≥ 2` slots (binary `Tree`). The W-style extension changes
**only the per-slot domain shape**, never the accumulation.

### The load-bearing correction (do not get this wrong)

> The outer reverse-loop wrap stays **`weaken(&method, 1)` — one shift per IH
> slot, regardless of `nb`.** The `+nb` is **intra-domain only**.

Each IH is a **single** top-level method binder (`Term::lam(ih_ty, …)`); its
`nb` branch binders live **inside** `ih_ty`, bound by the `Term::pi`s, and never
enter the method telescope. The kernel confirms this directly: its per-slot
offset `(n − pos + j)` counts *preceding IH binders* (`j`, each contributing 1),
**never their `nb`** (`inductive.rs`, the `for j in (0..p).rev()` block). An
inner W-style slot with `nb = 2` still advances the next outer slot's `j` by
exactly **1**.

Reading the frame's Deliverable-2 phrase "later slot's shift accounts for `+nb`"
literally — e.g. changing the wrap to `weaken(&method, 1 + nb)` — over-shifts
every already-built inner domain's field/Γ reference by `nb − 1` per slot,
emitting an `Elim` whose methods no longer match `method_type`. The kernel
recheck rejects it (fail-closed — no unsoundness), but **AC1 would not flip to
PASS**. The `+nb` belongs to the goal-weaken *inside* the domain (`n + nb`) and
the `field_var` position (`+nb`), nowhere else.

### The construction (drop-in replacement for the Gap-B rejection)

Delete the Gap-B **pre-rejection** loop (`elab.rs:819-825`,
`for (_, branching_tel, idxs) in &rec { … return Err("Gap B") … }`) and replace
the direct-only emission loop with this unified loop. Only the `ih_ty`
computation is new; the surrounding `method = Term::lam(ih_ty, weaken(&method,
1))` and the field-lambda wrap that follows are **unchanged**.

```rust
let mut method = body_core;
for (pos, branching_tel, idxs) in rec.iter().rev() {
    let nb = branching_tel.len();

    // Indexed FAMILIES are out of scope (Deliverable 4). The `dependent_eligible`
    // gate restricts to flat non-indexed families, so `idxs` is always empty
    // here; a non-empty `idxs` is a genuinely indexed family — a finding →
    // Steward, never a silent build.
    if !idxs.is_empty() {
        return Err(ElabError::Internal(
            "dependent match: indexed-family recursive field is out of scope \
             (W-style fields of a non-indexed family only) — finding -> Steward"
                .into(),
        ));
    }

    let ih_ty = if nb == 0 {
        // DIRECT case — byte-identical to today.
        let field_var = Term::var(n - 1 - pos);
        subst_var_generalize(&weaken(expected, n as i64), scrut_idx + n, &field_var)
    } else {
        // W-STYLE case: Pi(b1:B1)...(b_nb:B_nb). expected[scrut := field_var b1..b_nb].
        // Built in the bare [fields] frame (j = 0); the outer weaken(&method, 1)
        // per IH slot below accumulates the +j exactly as for the direct case.

        // Scrutinee body under the nb branch binders: `field_var` sits at
        // (n-1-pos) shifted past the nb binders -> var(n-1-pos+nb); applied to
        // b1 = var(nb-1), ..., b_nb = var(0).
        let mut scrut_body = Term::var(n - 1 - pos + nb);
        for bk in 0..nb {
            scrut_body = Term::app(scrut_body, Term::var(nb - 1 - bk));
        }

        // Specialized goal under the nb binders: weaken past n fields + nb branch
        // binders, then rewrite the scrutinee occurrence to (field_var b_bar).
        // (idxs empty -> this IS method_type's `M idxs (a_pos b_bar)`, in the
        // elaborator's already-applied `expected = M scrut` representation.)
        let mut ih_ty = subst_var_generalize(
            &weaken(expected, (n + nb) as i64),
            scrut_idx + n + nb,
            &scrut_body,
        );

        // Wrap the branching-domain Pi-binders, innermost (B_nb) to outermost (B1).
        // B_k mirrors method_type's b_dom with j = 0:
        //   shift(subst_outer(branching_tel[bk], m, params_terms, pos+bk), n-pos, bk)
        // cutoff = bk preserves b1..b_{bk-1}; amount (n-pos) lifts args-after-pos
        // and Γ. NO subst_levels — mirror the direct-case field-domain convention
        // (`level_args: vec![]`); the kernel recheck covers any residual.
        for bk in (0..nb).rev() {
            let b_dom = ken_kernel::subst::shift(
                &subst_outer(&branching_tel[bk], m, &params_terms, pos + bk),
                (n - pos) as i64,
                bk,
            );
            ih_ty = Term::pi(b_dom, ih_ty);
        }
        ih_ty
    };

    method = Term::lam(ih_ty, weaken(&method, 1)); // UNCHANGED: +1 per IH slot.
}
```

Notes for the transcriber:

- `pos` destructures as `&usize`; the arithmetic (`n - 1 - pos`, `pos + bk`,
  `n - pos`) uses the same std ref-forwarding `Sub`/`Add` the existing
  `n - 1 - pos` line already relies on — no new deref needed.
- `ken_kernel::subst::shift(term, amount: i64, cutoff: usize)` is the same
  function `subst_var_generalize` calls internally (`elab.rs:587`); `elab.rs`
  imports only `subst::{subst0, subst_outer, weaken}`, so reference `shift`
  fully-qualified (as shown) or add it to the `use`.
- `params_terms`, `m`, `scrut_idx`, `expected`, `n` are the already-bound
  locals from the match setup (`elab.rs` ~715-760); reuse them.

### Worked example (concrete indices)

A minimal 2-constructor W-style inductive isolates the arithmetic (this is also
the shape AC3's positive test should use, alongside `ITree`):

```
data WTree : Type where
  Leaf : WTree
  Node : (b : Bool) -> (Bool -> WTree) -> WTree
```

`Node` has `n = 2` fields (`b`, then `k : Bool -> WTree`). `recursive_args`
returns `[(pos = 1, branching_tel = [Bool], idxs = [])]`, so `p = 1`, `nb = 1`.
In the field context `[Γ, b, k]`: `k = var(0)`, `b = var(1)`. For
`match t { Leaf => …; Node b k => body }` with goal `expected = M t`:

- `field_var` (k) sits at `n-1-pos = 0`; under the `nb = 1` branch binder it is
  `var(n-1-pos+nb) = var(1)`.
- `scrut_body = (k b')` = `App(var(1), var(0))` (the branch binder `b'` is
  `var(0)`).
- goal under the binder: `subst_var_generalize(weaken(expected, 3),
  scrut_idx+3, k b')` = `M (k b')`.
- `B₁ = shift(subst_outer(Bool, m, params_terms, 1), n-pos = 1, 0) = Bool` (Bool
  is closed — shift is a no-op).
- `ih_ty = Π(Bool). M (k b')` = `(b' : Bool) -> M (k b')` — exactly `14 §3.1`'s
  `(b:B) → M (k b)`. ✔

The full `Node` method is then
`λ(b:Bool). λ(k:Bool→WTree). λ(ih:(b':Bool)→M (k b')). body` — byte-identical to
`method_type ind Node M [] []`, which is what the kernel checks the emitted
`Elim` against.

For the real target, `ITree`'s `Vis op k` with `k : Resp op -> ITree E Resp R`
is the same shape with `nb = 1` and a branch domain `B₁ = Resp op` that
**depends on the earlier field** `op` — handled automatically:
`subst_outer(&branching_tel[0], m, &params_terms, pos+0)` keeps `op`'s reference
at its correct index (`branching_tel[0]` is already in context
`[Δ_p, args_before_pos]`), and `shift(_, n-pos, 0)` lifts it past the remaining
args. No special-casing.

### Soundness posture (why front-loading is safe)

This is a **completeness** fix emitted into the outer ring: the `Term::Elim` the
match compiler produces flows through the normal declaration path and is
**independently rechecked by the kernel** against `method_type` (AC4 pins the
exact recheck site). A mis-built IH slot therefore yields an `Elim` whose
methods do not match `method_type` and the kernel **rejects** it —
over-rejection (completeness), never an admitted unsound term. The enclave
front-loads the
arithmetic so *correct* programs are *admitted* (AC1); the kernel recheck is the
backstop that keeps any residual slip **fail-closed**.

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
