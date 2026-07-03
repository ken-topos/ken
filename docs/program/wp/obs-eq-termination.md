# obs-eq-termination — congruence-first (lazy-δ) conversion so law-4's `ordBelowL`/`toListOrdered` recheck terminates and decides true (Map capstone)

**Steward frame → Kernel build.** Follow-on to the `conv-eq-congruence`
(Gap-conv, `90f39fe`) arc: that arm was **reverted-to-green** (`1466238`,
net-zero to pre-arm `019b695`) because it **triggered** a pre-existing
**eager-δ conversion non-termination** in `conv_struct`. This WP fixes that
root — a **congruence-first / lazy-δ fast path** — then **re-lands the fixed
`(Eq,Eq)` arm** so law 4's conclusion can build. Owner: **Kernel** (implementer
+ QA). Gate: **Architect** (soundness — vector certified against the hand-built
`ordBelowL`/`allKeys` Term test) + **kernel-qa** + CI. Findings → **Steward**.
No `/spec` change (kernel conv internal). Same reviewer set + no-`/spec` profile
as `conv-eq-congruence` / `sct-completeness`.

> **⚠️ Scope corrected after grounding (2026-07-03).** The original frame
> targeted the **funext** paths (`eq_at_pi`/`cast_at_pi` in `obs.rs`) and an
> occurs-guard/fail-closed envelope. Direct instrumentation **refuted funext
> empirically** (zero hits) and the grounded mechanism is an **eager-δ loop in
> `conv.rs`** whose recurring goal is genuinely **TRUE** (so fail-closed and
> occurs-guard-returns-true are both **out** — they'd re-wall law 4). The
> sections below are the corrected, as-built frame. The `(Eq,Eq)` arm is the
> **trigger**, not the loop body — that part held.

## Why this WP exists

Law 4's proof (`ordBelowL`, then `toListOrdered`) **elaborates + is well-typed**
but its `declare_def` whole-body kernel-recheck **diverges** (unbounded,
~1.1 GB/s, SIGKILL ~12 GB) — a **fatal usability regression in the trusted
checker** (Pat: *"12 GB on such small snippets is a bug, not a resource
restriction; the language is non-functional with that resource usage"*).
Grounded decisively (2026-07-03), with three self-corrections along the way
(all recorded so the reasoning is auditable):

- **Six-round opaque-stub bisection** ruled out every proof term ⇒ **the cost is
  in the kernel conversion machinery on the term shape, function-independent**.
- **Isolation-flip:** reverting *only* the 5-line `(Eq,Eq)` `conv_struct` arm
  (`90f39fe`) turns the divergence into a clean reject ⇒ the arm is the
  **trigger**.
- **Direct instrumentation** of `eq_at_pi`/`cast_at_pi`/`eq_at_sigma`/
  `cast_at_sigma`/the `(Eq,Eq)` arm → **ZERO hits**: **funext is refuted**, the
  loop is not in the obs function-type paths at all.
- **Id-table dump** at the panic point overturned the term-reads: the recurring
  `whnf_eq :: ty=Bool … y=True` goal is `Equal Bool (leq k2 key) True` (**not**
  an `Eq` at `Tree`), and the real recursive driver is **`allKeys`**, not the
  non-recursive `leBelow` view.
- **Single-view narrowing:** the divergence is inside **`declare_def`'s check of
  `ordBelowL`** — **upstream of `toListOrdered`, before any stubbing**. The
  round-6 stubs were downstream of the real hang; `ordBelowL`'s conversion is
  the minimal repro.

## The bug (root cause — grounded on real ids, hand-built repro)

An **eager-δ conversion non-termination** in `conv_struct`
(`crates/ken-kernel/src/conv.rs`).

`ordBelowL` converts `h : Ordered leq (Node l key val r)` against `andFst`'s
expected `And A B`. `Ordered (Node …)` def-unfolds to exactly
`And (allKeys (leBelow key) l) (And (allKeys (leAbove key) r) (And (Ordered l)
(Ordered r)))` — so the goal is a **valid, genuinely-convertible (TRUE)** one.
The two predicate spellings differ only syntactically: the **inlined**
`\k2. Eq Bool (leq k2 key) True` vs the **named** `leBelow key` (convertible via
one δ-unfold of the *non-recursive* `leBelow`).

The loop: `conv_struct` begins with an **unconditional** `let a = whnf(a);
let b = whnf(b);` **before any congruence dispatch**. `whnf` on a bare
transparent `Const` unconditionally δ-unfolds it — so `allKeys P1 l` /
`allKeys P2 l` are each δ-unfolded + ι-stuck into `Elim{fam=Tree,
methods=[…P embedded…], scrut=l}` (`l` free ⇒ stuck) **before** the `match` ever
sees "same head `Const`, same spine — just compare the args." The `Elim~Elim`
congruence arm then compares the two stuck eliminators' **methods**, where `P1`
vs `P2` differ; the Node-method body itself contains the recursive
`allKeys P child` (with `child` a fresh bound var, still free/neutral) → the
same "δ-unfold `allKeys` to a stuck `Elim`, compare methods" fires **one binder
deeper, forever**. `l`/its descendants never become a literal `Node`/`Leaf`, so
ι never bottoms it out, and nothing recognises the in-progress shape.

**Trigger vs. root.** The `(Eq,Eq)` arm is the **messenger**: pre-arm,
`(Eq,Eq) → _ => false` cut the path at lap 0; the arm makes the outer `Eq`
crack into componentwise `conv_struct`, exposing the `allKeys`-vs-`allKeys`
comparison. **Divergence ≠ unsoundness**: a hung checker admits *nothing*, the
trust root is intact — this is completeness/usability, not a soundness hole
([[gate-widening-exposes-latent-bugs-in-newly-reachable-code]]). The fix belongs
at the **root (the eager-δ in `conv_struct`)**, not the messenger arm.

## The fix (congruence-first / lazy-δ — Architect-certified vector)

Insert a **congruence-first fast path** at the top of `conv_struct`, **before**
the `whnf` calls: peel both **pre-whnf** terms; if both are the **same head
`Const`** (same `id` + equal `level_args`) applied to an **equal number of
args**, compare the **args pairwise via `conv_struct` first**, with **no
δ-unfold of the head**. If every arg converts ⇒ the applications are convertible
⇒ **return `true`**. Otherwise **fall through unchanged** to the existing
whnf-based path.

```rust
if let (Term::Const { id: id1, level_args: la1 }, args1) = peel_app(a) {
    if let (Term::Const { id: id2, level_args: la2 }, args2) = peel_app(b) {
        if id1 == id2
            && level_args_eq(&la1, &la2)
            && args1.len() == args2.len()
            && args1.iter().zip(args2.iter()).all(|(x, y)| conv_struct(env, ctx, x, y))
        {
            return true;
        }
    }
}
// falls through unchanged to the existing whnf-based path
```

For `allKeys k v P1 l` vs `allKeys k v P2 l`: `k`,`v`,`l` match syntactically,
`P1 ≟ P2` resolves in one trivial non-recursive step (`leBelow`-app δ-unfolds to
the inline λ) — congruence succeeds **without ever δ-unfolding `allKeys`**, so
the `Elim`-manufacturing loop never starts. Reaches **TRUE** on
`Ordered(Node) ≟ And A B` → `ordBelowL` checks → law 4 builds.

**Why sound (application congruence).** Same `Const` `id` + equal `level_args`
+ equal arg count + all args convert ⟹ the applications convert: a constant
denotes a fixed function, and functions respect conversion (holds for
transparent *and* opaque consts). Args go through the **same `conv_struct`
machinery** the fallback would use, so a function-typed arg (the predicate)
keeps whatever η/typed-conv the normal path gives it — no completeness
regression on the arg comparison. Decides **identical** convertibility;
recognises *more* equalities efficiently, never *fewer*.

**Two guards (both hard, both baked in):**
1. **Fall-through on arg-mismatch — NEVER return `false`.** Congruence-**first**,
   not congruence-**only**. The "constant ignores/absorbs an arg so results are
   equal despite differing args" case is still caught by the fallback unfold.
   This is the load-bearing completeness property.
2. **Guard = same const `id` + equal `level_args` + equal arg count** (mirrors
   the existing `Const` arm's `id == id && level_args_eq`), args compared via
   `conv_struct` (the fallback's machinery).

**Envelope note — occurs-guard / fail-closed are OUT.** The recurring goal is
genuinely **TRUE**, so returning `false` (fail-closed / stick-neutral) or
short-circuiting on it would **re-wall law 4** (fail AC4). The only admissible
vector is one that **reaches the true answer without looping** — lazy-delta does
exactly that.

## Deliverables (mandated outline)

1. **Hand-built kernel-Term repro (FIRST deliverable, the regression test) —**
   `crates/ken-kernel/tests/obs_eq_termination_congruence.rs`, k7/
   `conv_eq_congruence`-style, elaborator-independent (`Tree`/`allKeys`/`And`
   hand-built via `Term`/`GlobalEnv`). It reproduces `ordBelowL`'s conversion
   (`allKeys (leBelow key) l ≟ allKeys (\k2. Eq Bool (leq k2 key) True) l`)
   directly, not resting on the `toListOrdered` report.
2. **The congruence-first fast path** in `conv_struct`, both guards baked in,
   deciding identical convertibility.
3. **Re-land the fixed `(Eq,Eq)` `conv_struct` arm** (the reverted `90f39fe`
   congruence closure) **+ its `conv_eq_congruence` test** — safe *because* the
   fast path stops `allKeys P1 l ≟ allKeys P2 l` from unfolding into the
   regenerating `Elim`. (Permanent-revert-B — reserve law 4 without the arm —
   stays **RULED OUT**.)
4. **Law 4's conclusion** (`ordBelowL` → `consSorted` / `isSortedAppend` /
   `toListOrdered`) **kernel-checks clean and cheap** on the fixed base — the
   whole point (validated end-to-end by Foundation on the resumed capstone
   branch).

## Acceptance criteria (all testable)

- **AC1 — repro flips under ONLY the fix (isolation-flip).** The hand-built test
  **diverges/traps pre-fix** (depth-guard trap, same `whnf_eq :: ty=Bool … y=
  True` signature) and **decides TRUE + bounded post-fix**, under **only** this
  change. Land as a kernel regression test with a bounded guard.
- **AC2 — decides TRUE, not merely terminates (deliverable-4-critical).**
  `allkeys_two_predicate_spellings_converts` decides **true** (reaches the
  answer law 4 needs), and a **discriminating** `allkeys_distinct_predicate_
  stays_rejected` (genuinely different bound var) stays **REJECTED** — the fast
  path recognises more, never fewer, and non-convertible pairs still fall
  through and fail.
- **AC3 — the recheck COMPLETES, never skipped.** `ordBelowL`'s (and downstream
  `toListOrdered`'s) `declare_def` whole-body recheck **runs to a PASS** cheaply
  (bounded memory, no SIGKILL). "Assembles + well-typed" is **not** acceptance.
- **AC4 — workspace-wide green (blast radius).** `cargo test --workspace` green —
  re-landing the arm is a conv **completeness** change (K7 blast-radius lesson).
  Validate the **whole workspace**, not just `ken-kernel`.
- **AC5 — `trusted_base` unchanged; diff `conv.rs`-local.** No new `declare_*` /
  `Term` variant / kernel decl; `trusted_base()` byte-unchanged; the diff is
  `conv.rs` (fast path + re-landed arm) + the two new kernel test files only;
  `obs.rs` byte-identical to `origin/main`. TCB **power** does not grow — only
  conv **termination + completeness**. Grep-verify.

## Guardrails (do-not-reopen)

- **Root, not messenger.** Fix the eager-δ in `conv_struct`; do **not** "fix" by
  gutting the `(Eq,Eq)` arm again — the arm re-lands with the fix.
- **Congruence-first, not congruence-only.** The fast path must **fall through**
  (never return `false`) on arg-mismatch/inapplicability; the fallback unfold
  preserves completeness. A congruence-only version over-rejects the
  constant-absorbs-an-arg case.
- **Not funext, not the Cast-telescope.** Both refuted empirically. The site is
  `conv_struct`'s App handling.
- **Completeness/termination, not soundness-loosening.** If any change would make
  conversion accept a genuinely non-convertible pair, STOP → raise to Steward.
  Only sanctioned changes decide **identical** convertibility.

## Residual / follow-on (NOT blocking law 4)

The **fallback** unfold could still loop on *genuinely-non-convertible*
same-const applications (heads match but args don't converge, so the fast path
falls through and the old whnf path runs). **No valid proof exercises that
path**, so law 4 and this WP are unaffected — but Pat's "checker never loops on
any input" bar wants a scoped **occurs-guard / fuel backstop** on the fallback.
Filed as a **separate follow-on WP** (conv robustness backstop), sequenced after
the Map capstone lands. Core fix = lazy-delta (this WP); robustness backstop =
separate.

## Sequencing

- **Kick off Kernel after the revert (`1466238`) merges**; the fix branch cuts
  off termination-guaranteed post-revert `main` and re-adds the fixed arm.
- **Gate:** Architect (soundness — certifies the vector against the hand-built
  `ordBelowL`/`allKeys` Term test: reaches **true + bounded**, decides identical
  convertibility on the discriminating pair) + kernel-qa + CI. **Parallel-gate**
  once the diff is confirmed `conv.rs`-local + tests.
- **On merge:** Steward signals Foundation to resume `map-verified-laws` on the
  held branch (rebased onto the fixed `main`) and build **law 4's conclusion**
  end-to-end; the **whole capstone** (law 4 + laws 1/2/3/5 via Tree-Σ v2) lands
  on the fixed base as one unit.
- **Lane:** Kernel (`ken-kernel/conv.rs`).
