# X1 effect evaluation conformance — seed cases

Format: `../../README.md`. These pin **effect evaluation** in the **reference
interpreter** — the interaction-tree **driver** X1 delivers
(`docs/program/wp/X1-effects-elab.md`, `spec/40-runtime/42-evaluation.md §6`):
run the **pure** denotation `⟦e⟧ : ITree ⟦ρ⟧ R` (L5, `36 §2.4`) by forcing its
`Vis` nodes against a real-world handler, performing each world-interaction and
resuming with the response. They **extend — and must not regress** — the
pure-core `runtime/evaluation/` corpus (`../evaluation/seed-evaluation.md`),
which deliberately scoped effects **out** (its `42 §6` seam).

**The one mechanism (everything hangs off it, `42 §6.2`).** Effect eval adds a
single driver loop over the pure tree:

```
H : (e : ⟦ρ_open⟧.Op) → ⟦ρ_open⟧.Resp e     -- the IMPURE world boundary (36 §7.2)
drive_H : ITree ⟦ρ_open⟧ R → R
drive_H t = case whnf t of                   -- whnf = pure §3 eval to a head ctor
  Ret r    → r                               -- finished
  Vis e k  → drive_H (apply k (H e))         -- perform+observe (H e), resume (β), loop
  unknown  → unknown                         -- §4: an open hole in the tree is strict
```

`H` is **parametric**; conformance plugs a **deterministic mock** `H` (a mock
world) so a trace is reproducible. perform→observe→resume = `H e` then `apply k`
(β, §3.2). Pure handlers (`space` via `runState`, user `handle`) discharge **in
§3** as `elim_ITree` folds — only the **open row** `ρ_open` reaches the driver.

**Trust posture.** X1 is **not in the TCB for type soundness** — it runs
already-kernel-checked core terms (`42 §5`/§7). A bug here is a **wrong
answer**, silently propagated to every backend judged against the oracle (★★).
So correctness is **agreement with L5's denotation** (`42 §6.6`) — and that
agreement is **definitional**: X1 runs the *very term* `⟦e⟧` L5 denotes, by the
same reductions the kernel uses for conversion (§1). The reconciliation cases
(EFF5) are the load-bearing oracle anchor.

**Two-soundnesses (the completeness linchpin, `42 §6.5`).** A **wrong** response
is detectable against L5 (EFF5). But a **missing** effect rule supplies **no**
world step **and no error**, and the kernel re-checks **types, not traces** — so
nothing downstream catches a silently-dropped interaction. **Completeness of the
driver's dispatch over `ρ_open` is the *sole* backstop** (EFF7), enforced
structurally (exhaustive-by-construction, no catch-all). Split, every time,
**supplied** (re-checked vs L5) from **omitted** (NOT backstopped).

**Tags.** `(oracle)` — confirmed at build time against Ken's interpreter (safe:
X1 not in the type-soundness TCB): the per-class `Op`/`Resp` **signatures** (the
op-tag column is illustrative — `38`/stdlib fix the concrete signatures, `42
§6.3`); the **mock `H`** responses; and interpreter-internal **trace**
specifics. `(property)` — an invariant over a corpus, not a single trace.
Op-tags below (`Console.Write`, `Clock.now`, …) are **illustrative** for the
mechanism; nothing locks a concrete syscall signature here.

**Reconcile note (content-verified against `42 §6`, `f59cff0`, and landed
`36`).** Authored Ring-2 against spec-author's locked `42 §6`; each expected
trace **re-derived independently** from the L5 denotation `36 §2.4` (not
transcribed — the `conformance-reconcile-inherits-spec-metatheory-bugs` gate).
Confirmed: the trace is the **`Vis`-tag spine order** (not a response-stream);
handled/unhandled **is** the pure-fold-vs-driver line (`42 §6.1`/§6.2);
row-bounding is **type-level**, caught at elaboration (`42 §6.5`, `36 §1.4`),
**no** runtime check; X1↔L5 is **definitional** (`42 §6.6`).
Mechanism-consistency checked by **grouping cases by shared mechanism** (the
driver `drive_H`; the handler fold; the escape check) and verifying the shape
agrees across the varying parameter (effect class, handled/unhandled, sequencing
depth) — the V2 carry.

**Citations.** `42-evaluation.md` §6.1 (pure-handler discharge in §3), §6.2 (the
driver `drive_H`, perform→observe→resume, tail-resumptive→loop), §6.3
(per-effect uniform rule; op-tags `(oracle)`), §6.4 (sequencing = `bind`-spine
order), §6.5 (row-bounding type-level + exhaustive-over-`ρ_open`,
two-soundnesses), §6.6 (X1 == L5, definitional), §6.7 (`unknown` strict through
the driver), §6.8 (effect determinism / oracle; pure fragment unchanged); §3.2
(`eval`/`apply` β), §3.5 (WHNF/closure), §3.6/§3.7 (canonicity/determinism, must
not regress), §4 (`unknown`). L5 `36-effects.md` §1.4 (escape check — the row
system's one soundness gate), §2.1 (`ITree` Ret/Vis), §2.2 (`perform`/`bind`),
§2.4 (the denotation `⟦·⟧`; `ITree 𝟘 R ≅ R`), §2.5 (`ρ_open`), §4.1/§4.2/§4.3
(`space` → `State`, `runState`, `old`), §5.1 (handler fold: `inl` interpret /
`inr` pass-through), §5.2 (tail-resumptive), §7.0 (K1.5-admitted), §7.2 (the
real-world handler hook). V2 anchor:
`verify/obligations/exhaustive-traversal-no-silent-skip` (the structural/absence
model for EFF7). Pure base: `../evaluation/seed-evaluation.md`.

---

## EFF1 — per-effect perform → observe → resume (the driver, frame AC1)

The uniform driver rule (`42 §6.2`/§6.3): `drive_H` forces a `Vis`, the handler
`H` performs the op and observes its response, `apply k` resumes. Per effect
class the **only** difference is which world-interaction `H` performs and the
response type it observes (`36 §2.1`'s `Op`/`Resp`). The op-tag/signature per
class is `(oracle)` (`38`/stdlib's, `42 §6.3`).

### runtime/effects/perform-observe-resume-console (oracle)
- spec: `42 §6.2` (`drive_H`), `§6.3`; `36 §2.1`/`§2.2` (`Vis`/`perform`),
  `§2.4`
- given: a single-effect program `perform Console (Write "hi")`, denoting to
  `Vis (Write "hi") (λ r. Ret r)` (`36 §2.4`); run by `drive_H` under a
  deterministic mock `H` with `H (Write "hi") = tt`.
- expect: the driver forces the `Vis` (op `Write "hi"` performed/observed by
  `H`), then `apply k tt` resumes (β, §3.2) to `Ret tt`; `drive_H` returns `tt`.
  **Trace** = `[Console.Write "hi"]`, **leaf** `tt`. (Op signature + mock value
  `(oracle)`.)
- why: the perform→observe→resume mechanism (`42 §6.2`). **Two bugs flip:** (i)
  dropping the perform (treating the effect as a pure no-op) gives an **empty**
  trace — the structural trace assertion flips; (ii) failing to resume (yielding
  the `Vis` unforced) leaves a stuck `Vis`, not `Ret tt` — flips value-vs-stuck.
  (oracle; verdict-flip.)

### runtime/effects/perform-rule-uniform-across-classes (oracle, property)
- spec: `42 §6.3` (uniform rule; op-tags illustrative/`(oracle)`); `36 §2.1`
- given: one single-`perform` program per **open-row** class —
  `Console`/`Clock`/`FS`/`Net`/`Rand` — each denoting to `Vis op (λ r. Ret r)`,
  each run under a deterministic mock `H`.
- expect: an **identical mechanism shape** for every class — `drive_H` forces
  the `Vis`, `H` performs+observes (a response at `⟦ρ_open⟧.Resp op`), `apply k
  resp` resumes; **only** which world-interaction `H` performs and the response
  **type** vary (Console:`Unit`, Clock:`Instant`, FS:`Bytes`/`Unit`,
  Net:`Unit`/`Bytes`, Rand:drawn value — all `(oracle)`). **No** class is
  special-cased: none skips the resume, none short-circuits, none is treated as
  pure.
- why: **mechanism-consistency** (V2 carry) — the per-class rule is **one**
  mechanism, not five. A bug that handles one class differently (a class whose
  perform doesn't feed `k`, or is silently treated as pure) breaks the shared
  shape; asserting the uniform shape **across the varying parameter** (effect
  class / `Resp` type) catches a per-class divergence a single-class case would
  miss. (oracle; property; mechanism-consistency.)

---

## EFF2 — effect sequencing = the `Vis`-spine order (frame AC1)

The observable trace is the sequence of `Vis` op-tags along the tree's spine,
and that spine is fixed by `bind`'s grafting (`36 §2.2`, `42 §6.4`): CBV builds
it left-to-right and single tail-resumption (`42 §6.2`) keeps the run linear, so
the **performed** order **is** the spine order. A reorder/drop is a **different
trace** (a different tree).

### runtime/effects/sequencing-trace-is-spine-order (oracle)
- spec: `42 §6.4`; `36 §2.2` (`bind` grafting), `§2.4`
- given: a two-effect program in a **known** order, `let _ = perform Console
  (Write "1") in perform Clock now` — denoting (via `bind`, `36 §2.4`) to `Vis
  (Write "1") (λ _. Vis now (λ r. Ret r))`; run under a deterministic mock `H`.
- expect: the driver consumes the spine **linearly**, performing `Console.Write
  "1"` **before** `Clock.now`. **Trace** = `[Console.Write "1", Clock.now]`.
  (Leaf = the resumed result; mock `(oracle)`.)
- why: the observable order **is** the spine order (`42 §6.4`). A bug that
  **reorders** (performs `Clock.now` first) or **drops** one interaction gives a
  **different trace** (`[Clock.now, Console.Write "1"]` or `[Console.Write
  "1"]`) — the trace-sequence assertion **flips**. ≥2 distinct effects, known
  order. (oracle; verdict-flip.)

### runtime/effects/bind-graft-threads-response (oracle)
- spec: `42 §6.4`, `§6.2` (`apply k resp`); `36 §2.2` (`bind`)
- given: a program whose **second** effect depends on the **first**'s response —
  `let x = perform Clock now in perform Console (Write (show x))`, denoting to
  `Vis now (λ x. Vis (Write (show x)) (λ _. Ret tt))`; mock `H` with `H now =
  t0`.
- expect: the driver resumes the first `Vis` with `t0` (`apply k t0`, §3.2), so
  the second op is `Console.Write (show t0)` — the response **fed forward**
  through `bind`'s graft. **Trace** = `[Clock.now, Console.Write (show t0)]`.
  (Mock `t0` and the `show` form `(oracle)`.)
- why: `bind` grafts `k` onto the first tree's leaf (`36 §2.2`), so the
  resume-response threads into the continuation (`42 §6.2`). A bug that fails to
  thread the response (resumes with a stale/default value, or evaluates the
  continuation **before** resuming) yields a **different second op** — the trace
  flips. This pins the **resume** data-flow of perform→observe→resume, not just
  its order. (oracle; verdict-flip.)

---

## EFF3 — row-bounding is type-level (frame AC2)

An out-of-row effect is a **type-level impossibility**, not a runtime check: the
escape check (`36 §1.4`) rejects, **at elaboration**, any function whose
inferred row exceeds its declaration, so the denotation `⟦e⟧ : ITree ⟦ρ⟧ R` is
built over **exactly** `⟦ρ⟧` (`36 §2.3`) and an op outside `ρ` is **not
constructible** in the term that reaches X1 (`42 §6.5`). The driver performs
**no** runtime row-membership check.

### runtime/effects/row-bounding-escape-rejects-at-elaboration (verdict-flip)
- spec: `42 §6.5`; `36 §1.4` (escape check — the row system's one soundness
  gate), `§7.5.1`
- given: **one** body performing two distinct effects, `perform FS read p ;
  perform Console (Write s)` (inferred row `{FS, Console}`), under two
  declarations: (a) `visits [FS, Console]`; (b) `visits [FS]` (drops `Console`).
- expect: (a) **accepts** — `ρ_inf ⊆ ρ_decl`; the denotation is built over
  `⟦{FS, Console}⟧` and the driver may perform both. (b) **rejects at
  elaboration** — `ρ_inf ⊄ ρ_decl` raises **EFFECT-ESCAPE**, naming `Console` as
  the escaping effect with a `perform`-witness (`36 §1.4`). In (b) the driver
  **never runs**: the out-of-row `Console.Write` is **not constructible** in a
  term that reaches X1 (`42 §6.5`).
- why: row-bounding is a **type-level** fact (the kernel-checked type `ITree ⟦ρ⟧
  R` already witnesses it, `42 §6.5`), not a runtime check. **Verdict-flip** on
  the single soundness-relevant gate of the row system (`36 §1.4`): the same
  body **accepts** under the correct row, **rejects** when one effect is dropped
  — ≥2 distinct effects. **Absence-gate:** the reject is **guard-gated** by
  `ρ_inf ⊄ ρ_decl` (named, not coincidental); under the precise bug it targets —
  the escape check fails to fire, a leak goes unbounded — case (b) would instead
  **accept** and the out-of-row op **would** reach the driver, so the case flips
  exactly on that bug. (verdict-flip.)

---

## EFF4 — handlers: pure-fold discharge vs the driver (frame AC3)

The crisp split (`42 §6.1`/§6.2): `space`/user handlers are pure `elim_ITree`
folds (`36 §5.1`) that discharge **in §3** — assert the reduced value, **no**
residual `Vis`, no driver. Only the **open row** `ρ_open` (`36 §2.5`) reaches
the driver. Handled-vs-unhandled **is** exactly this pure-fold-vs-driver line —
the **same** fold, `inl` interpreted, `inr` passed through (`36 §5.1`).

### runtime/effects/runstate-discharges-in-pure-section3 (verdict-flip)
- spec: `42 §6.1` (pure-handler discharge); `36 §4.2` (`runState`), `§4.3`
  (`old`), `§4.1` (`space` → `State`)
- given: a `space`-only program `runState 0 (inc; inc; get)` — `inc` is
  `Get`-then-`Put (s+1)` (`36 §4.1`), `get` is `Get`; the whole effect is the
  space's `State`.
- expect: `runState` is an `elim_ITree` fold (`36 §4.2`) that ι-reduces
  **entirely in pure §3** — **no** residual `Vis`, **no** driver. It reduces to
  `(r, s_final) = (2, 2) : Int × S` (state `0 → 1 → 2`; `get` observes `2`).
  `inc`'s `ensures n == old(n) + 1` **discharges**: the transformer `λ s. (tt, s
  with .n := s.n + 1)` gives `(s with .n := s.n + 1).n == s.n + 1`, computing by
  record-β/η (`13 §3`) to `refl` (`36 §4.3`).
- why: a `space`/user handler is a **pure fold** that discharges in §3 (`42
  §6.1`) — the handled effect produces **no** world interaction and never
  reaches the driver. A bug that leaves a `State` `Vis` un-discharged (sends it
  to the driver — there is **no** real-world `State` handler) **flips**
  value-vs-stuck (the program would stick, not reduce to `(2, 2)`); a bug in the
  fold's state-threading flips the **final-state** value. Assert the final-state
  value **and** no residual `Vis`. (verdict-flip; the `old(n)` discharge is the
  `36 §4.3` anchor.)

### runtime/effects/handled-discharges-unhandled-reaches-driver (verdict-flip)
- spec: `42 §6.1`/§6.2 (the split); `36 §5.1` (handler fold: `inl` interpret /
  `inr` pass-through), `§2.5` (`ρ_open`)
- given: a program with **two** effects — a `space` `State` (handled by an
  enclosing `runState`) **and** a `Console` (open, `ρ_open`, no enclosing
  handler): `runState 0 (inc; perform Console (Write "x"))`.
- expect: the split — `runState` discharges `State` in pure §3 (`inl`
  interpreted, `36 §5.1`), leaving an `ITree` over **`Console` only**; the
  `Console` `Vis` (the `inr` pass-through, `36 §5.1`) **survives** to the
  driver, which performs it. **Residual trace** = `[Console.Write "x"]` with
  **no** `State` `Vis`; the final state is threaded to `1`. So handled `State` →
  **discharged in §3** (no `Vis`); unhandled `Console` → **reaches the driver**
  (`Vis` performed).
- why: handled-vs-unhandled **is** the pure-fold-vs-driver line (`42 §6.1`/§6.2)
  — the **same** `elim_ITree` fold, `inl` discharges, `inr` passes through (`36
  §5.1`). **Mechanism-consistency** across the handled/unhandled parameter: a
  bug that discharges *both* (drops `Console`) gives an **empty** residual
  trace; a bug that passes *both* through (sends `State` to the driver)
  **sticks**. Either flips the residual trace/value. Pins that the split is
  keyed on **`inl`-vs-`inr`** (enclosing-handler vs open-row), nothing else.
  (verdict-flip; mechanism-consistency.)

---

## EFF5 — X1 == L5 ITree (definitional reconciliation, frame AC4)

The ★★ load-bearing obligation, holding **definitionally** (`42 §6.6`): X1
evaluates the **same kernel term** `⟦e⟧` that `36 §2.4` produces, by the **same
reductions** (§1). There is no second effect semantics — the run **is** the
denotation, with the world's responses substituted at the `Vis` nodes via `H`.

### runtime/effects/x1-trace-equals-l5-itree-denotation (property)
- spec: `42 §6.6` (definitional reconciliation), `§6.8` (oracle); `36 §2.4` (the
  denotation `⟦·⟧`)
- given: a shared corpus of effectful programs; for each, **independently**
  compute `⟦e⟧` (the ITree, from the `36 §2.4` denotation) and **run** `e` under
  a **fixed** deterministic mock `H`. Representative: `let x = perform Clock now
  in perform Console (Write (show x))`, with `⟦e⟧ = Vis now (λ x. Vis (Write
  (show x)) (λ _. Ret tt))` and `H now = t0`.
- expect: X1's **performed `Vis`-tag sequence** and **`Ret` leaf** are
  **exactly** the **spine** and **leaf** of `⟦e⟧` instantiated at `H`'s
  responses — here `[Clock.now, Console.Write (show t0)]`, leaf `tt`. The
  agreement is **definitional** (`42 §6.6`): X1 runs the very term `⟦e⟧`, so the
  run **is** the denotation with responses substituted at the `Vis` nodes.
- why: the **★★ reference-correctness** property — X1 is the oracle for
  effectful programs (`42 §6.8`), so its trace **must** realize L5's denotation.
  **Structural, handler-independent** (the `Vis`-tag sequence + `Ret` leaf — a
  structural output the bug changes regardless of downstream typing): any driver
  bug (reorder, drop, mis-resume, mis-tag) makes the run's trace **diverge
  from** `⟦e⟧`'s spine — the structural identity **flips**. The load-bearing
  property of `42 §6.9`. (property; structural; flagship.)

---

## EFF6 — `unknown` strict through the driver (frame AC4)

`unknown` (§4) propagates through effect evaluation by the **same strict rule**,
since the driver's scrutinee **is** the tree (`42 §6.7`): an open hole in the
tree, or an `unknown` **op**, yields `unknown` and performs **nothing**. A
**hole-free** effectful program **never** yields an `unknown` tree (`42 §6.7`,
`43 §2.1`). Extends `../evaluation/seed-evaluation.md`'s CAN4.

### runtime/effects/unknown-strict-through-driver (oracle)
- spec: `42 §6.7` (`unknown` through effects); `§4` (strict propagation), `41
  §6`
- given: **one** effectful program shape, two instantiations: (a) the effect's
  **op** depends on an **open hole** — `perform Console (Write h)`, `h` an open
  verification hole (`24 §2`), so the op (the driver's dispatch scrutinee) is
  `unknown`; (b) the **same** program with `h` **discharged** to a concrete
  `"ok"`.
- expect: (a) `drive_H` on a tree whose op is `unknown` yields **`unknown`** —
  **no** effect performed (no determinate interaction; the op is the scrutinee,
  strict, `42 §6.7`) → **empty** trace. (b) performs `Console.Write "ok"` →
  trace `[Console.Write "ok"]`, leaf `tt`. A hole-free effectful program
  **never** yields an `unknown` tree (`42 §6.7`, `43 §2.1`).
- why: the operational face of partial verification, **through** effects (`42
  §6.7`) — the driver's scrutinee is the tree, so `unknown` propagates by the §4
  strict rule. **Verdict-flip** on hole-present → `unknown`/no-perform vs
  hole-free → a real trace, catching **both** directions: a bug performing a
  real (wrong) interaction for the holed op (a) is caught by (a)'s `unknown`; a
  bug yielding `unknown` for the hole-free (b) is caught by (b)'s real trace.
  (oracle; verdict-flip.)

---

## EFF7 — exhaustive driver dispatch, no silent skip (two-soundnesses)

The completeness linchpin (`42 §6.5`). X1 is untrusted-but-reference: a
**wrong** response is detectable against L5 (EFF5), but a **missing** effect
rule supplies **no** world step **and no error** — and the kernel re-checks
**types, not traces** — so nothing downstream catches a silently-dropped
interaction. **Completeness of the driver's dispatch over `ρ_open` is the *sole*
backstop.** Structural/absence (no value flip — asserts the dispatch **shape**),
modeled on V2's `verify/obligations/exhaustive-traversal-no-silent-skip`.

### runtime/effects/exhaustive-driver-dispatch-no-silent-skip (property)
- spec: `42 §6.5` (exhaustive over `ρ_open`, no catch-all), `§6.3`; V2 anchor
  `verify/obligations/exhaustive-traversal-no-silent-skip`
- given: the driver `H` / `drive_H` dispatch over the open row `⟦ρ_open⟧.Op`.
- expect (**structural — no value flip**; asserts the dispatch **shape**): the
  dispatch is **exhaustive-by-construction** — a rule for **every** op-tag the
  open row admits, with **no** catch-all `_ → skip` / `_ → no-op`. An open-row
  op with **no** rule is a **build error**, **never** a silent skip. An
  **unhandled** effect is **not** silently dropped: by the EFF4 split, an effect
  with no enclosing pure handler is in `ρ_open` and **must** be performed by the
  driver (an observable `Vis`) — "unhandled" means **performed**, never
  **vanished**.
- why: the **two-soundnesses linchpin** (`42 §6.5`; memory
  `untrusted-layer-backstop-hole-for-omissions`). Split **supplied** (the trace,
  re-checked vs L5 — EFF5) from **omitted** (a never-performed effect — NOT
  backstopped). **Absence-gate:** the safeguard is the **structural shape**
  (dispatch on the finite `⟦ρ_open⟧.Op`, no catch-all) — **not** a coincidental
  "every current op-tag happens to be handled." Disconfirming check: *would a
  newly-added open-row op-tag be a **build error** (genuine) or **silently
  skipped** (vacuous)?* — only exhaustive-by-construction makes it the former.
  No program exhibits the bug today; the property is on the driver's **shape**.
  (property; structural/absence; two-soundnesses.)

---

## Regression — pure programs never reach the driver (frame AC5)

### runtime/effects/pure-program-never-reaches-driver (property)
- spec: `42 §6.1`/§6.8 (pure stays in §3; no regression); `36 §2.4` (`ITree 𝟘 R
  ≅ R`)
- given: a **pure** (effect-free, `ρ = ∅`) program — e.g. `(\ x. add x 1) 2`, a
  `../evaluation/seed-evaluation.md` anchor.
- expect: denotes to `ITree 𝟘 R ≅ R` (`36 §2.4`) — **no** `Vis` is constructible
  (`𝟘.Op = Empty`), the elaborator collapses it to the plain term, and §3
  evaluates it **unchanged** to the value `3`. The program **never** reaches the
  driver; pure determinism + canonicity (`42 §3.6`/§3.7) hold verbatim.
- why: effect evaluation **wraps** the driver *around* the pure core; it does
  **not** alter pure reduction (`42 §6.8`, **no regression**, acceptance 5). A
  bug that routes pure programs through the driver (or otherwise perturbs §3)
  flips this value/behavior. Pins that the `../evaluation/` corpus stays green —
  X1 effect conformance is **additive** over the pure-core anchors. (property;
  regression guard.)
