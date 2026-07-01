# X3 native backend conformance — differential-equivalence seed cases

Format: `../../README.md`. These pin the **differential-equivalence
discipline** of the native backend (X3,
`spec/40-runtime/45-native-backend.md`, DRAFT v0): the backend is validated by
running the **same** closed ground core term through it **and** the reference
interpreter, requiring **identical K3 values** — the interpreter is the
**oracle**, and on any disagreement the interpreter is **right by definition**
(`42 §5`). The load-bearing posture: the native backend runs
**already-kernel-checked** core, so it is **NOT in the type-soundness TCB**
(the X1 framing extended to codegen) — a codegen bug is a **wrong answer**,
not a soundness hole. This is a **design/discipline** corpus: it pins the
*rules* the X3-build differential harness must implement; the backend itself
is greenfield (the deferred producer).

The layering (one home per property): **kernel ← interpreter (X1) ← backend
(X3)**. Each layer is differential-validated against the layer below. The
interpreter's own **canonicity + determinism** are X1's home
(`../evaluation/seed-evaluation.md`, CAN1/CAN2) — this seed does **not**
re-pin them; it pins the **backend ≡ interpreter** differential on top of that
oracle.

Grounding (landed `§`-bodies + landed code, content-reconciled — not the
plan): `42 §5` (the interpreter as differential oracle; the native backend as
its consumer; X1 **not** in the type-soundness TCB — runs
already-kernel-checked terms), `42 §2` (determinism/canonicity carry;
"different layers, allowed to differ… need only **agree on final values**" —
the observable/unobservable boundary), `42 §1` (values realize the kernel's
reductions). **Landed reference:** the interpreter oracle
`crates/ken-interp/src/lib.rs` (the differential reference; the future backend
runs against **this**). `docs/PRINCIPLES.md` (small-auditable-TCB,
reflect-don't-extend — the not-in-TCB posture). The concrete backend target is
**`OQ-backend-target`** (operator-ratifiable); this corpus is
**target-agnostic** — differential equivalence must hold for **any** target,
so no case names or assumes a target.

## Reading these cases — the X3-backend disciplines

**The observable is the final K3 value; the unobservable is strategy and
representation (`42 §2`).** The differential corpus compares the **content-
addressed K3 value** a term computes to — the observable. What a backend is
free to differ in (relative to the interpreter, or the kernel's WHNF) is
**strategy and internal representation**: reduction/evaluation order
(meaning-preserving in the total core, `42 §2`), closure encoding, how sharing
is realized on the heap — the **unobservable internals**. The `42 §2` rule is
exact: "different layers, allowed to differ… need only **agree on final
values**." So the corpus's discriminating boundary (§BD2) is
**observable-value divergence rejects / an unobservable-internal difference
admits** — a non-degenerate pair on that boundary (COORDINATION §7). (`@ct`
**timing** is order-sensitive, but that is a **separate** discipline delegated
to `Ward` (`61 §5a.6`), not the value differential — this corpus pins
**value** equivalence, not timing.)

**The backend is a TESTED surface, not a TRUSTED one — get the trust level
right (`42 §5`, reflect-don't-extend).** Because the backend runs
already-kernel- checked core, a codegen bug **cannot** produce a false
`proved` or a typing violation — type soundness is the kernel's, and it holds
**regardless** of the backend. So the backend's correctness is **established
by differential agreement over this corpus (tested)**, **never**
kernel-proved: the permanent kernel TCB is **unchanged** (§BD3; the `45 §2`
trust chain — kernel `Q` / interpreter-oracle `tested` / backend `tested`). A
backend corpus that framed codegen correctness as "kernel-guaranteed" would
over-claim — the kernel does not check the backend; the **differential corpus
is the net**. This is the same tested-vs-trusted honesty the security spine
applies to erased-label properties.

**The backend is the deferred producer (`(oracle)`).** The interpreter half of
every differential case is **landed** (`crates/ken-interp`); the **backend
half is `(oracle)` / X3-build-deferred** — the real differential harness
(backend output vs the landed interpreter, no hand-fed value table) is the
**X3-build** producer-grep gate, flagged forward, not exercised here. These
cases pin the **discipline**; the runs are the build ring's.

## BD1 — the interpreter is the oracle (differential agreement) (AC2)

### runtime/backend/backend-agrees-with-interpreter-on-observable-value (oracle)
- spec: `45 §4` (BE-Differential), `42 §5` (interpreter as oracle), `42 §1`
- given: a closed, ground core term `t` (e.g. an arithmetic computation, a
  constructor-producing `elim`, an observational `Eq`/`cast` computation) run
  through **both** the reference interpreter and the native backend
- expect: the two produce the **identical K3 value** (content-addressed,
  `41`); the corpus asserts **value equality** (same content-hash / same
  slot), and on **disagreement the interpreter is right by definition** — the
  backend is wrong
- why: (oracle) BD1, the differential-agreement rule (`42 §5`). The backend
  earns trust by **agreement over the corpus**, not by inspection. The
  interpreter half is landed (`crates/ken-interp`); the **backend half is
  X3-build-deferred** — the run is the build harness's, the **rule** is pinned
  here. Assert the **observable value**, not an internal trace (that is §BD2's
  boundary).

## BD2 — the observable / unobservable boundary (AC2 ★)

> The non-degenerate pair is **{BD2-obs, BD2-int}** on the `42 §2`
> observable/unobservable boundary: a divergence in the **final value**
> rejects; a difference in **unobservable internals** with the same final
> value admits. A corpus comparing *strategy* would over-reject valid
> backends; one ignoring observable divergence would admit miscompiles (the
> omission hole).

### runtime/backend/observable-value-divergence-rejected (oracle)
- spec: `45 §4` (BE-Differential), `42 §2` (agree on final values), `42 §5`
- given: a (hypothetical) backend whose output for a closed ground `t` is a
  **different observable K3 value** than the interpreter's — e.g. a wrong
  arithmetic result, a different constructor head from a mis-lowered `elim`,
  or a different normal form
- expect: the corpus **rejects** it — the content-addressed values differ, so
  the backend disagrees with the oracle and is **wrong** (`42 §5`, interpreter
  right by definition)
- why: (oracle, soundness of the discipline) BD2 — the reject arm, the
  miscompile net. This is the guard: a corpus that did **not** compare the
  observable value would admit a miscompile silently (the untrusted-layer
  **omission** analog — a divergence no one checks reads
  "correct"-by-default). Paired with BD2-int, it pins the boundary at the
  **value**, not the strategy.

### runtime/backend/unobservable-internal-difference-admitted (oracle)
- spec: `45 §4` (BE-Differential), `42 §2` (layers-may-differ bound)
- given: a backend that differs from the interpreter only in **unobservable
  internals** — a different (meaning-preserving) reduction/evaluation order, a
  different closure encoding, or a different internal heap layout — but
  computes the **same** observable K3 value for `t`
- expect: the corpus **admits** it — value equality holds, so the difference
  is **within** the `42 §2` layers-may-differ bound; strategy is not a
  conformance property
- why: (oracle) BD2 — the admit arm. The corpus must **not** over-constrain
  the backend to the interpreter's internal strategy (that would reject a
  valid, faster codegen). The discriminator is **value equality**, keyed on
  the `42 §2` boundary — the dual of BD2-obs; together they are the
  non-degenerate pair. (A single reject-only or admit-only case is
  green-vs-green under a boundary drawn at the wrong layer.)

## BD3 — the backend is not in the type-soundness TCB (AC1) (soundness)

### runtime/backend/codegen-bug-is-wrong-value-not-soundness-hole (soundness)
- spec: `45 §2` (BE-NotInTCB), `42 §5` (already-kernel-checked core),
  PRINCIPLES (reflect-don't-extend)
- given: a core term that the elaborator produced and the **kernel already
  checked** (well-typed), then miscompiled by a buggy backend
- expect: the failure surfaces **only** as a **wrong observable value** the
  differential corpus catches (§BD1/§BD2) — **never** as a typing/soundness
  violation, a false `proved`, or a change to what the kernel admits. Type
  soundness holds **regardless** of the backend
- why: (soundness) BD3, the **reflect-don't-extend anchor** and the not-in-TCB
  posture. The backend runs already-kernel-checked core (`42 §5`), so it adds
  an **execution path, not a typing rule** — the permanent kernel TCB is
  **unchanged**. **Trust level: the backend is a TESTED surface (differential
  agreement is the net), NOT kernel-backed** — a codegen bug is a
  *correctness* (wrong-value) failure, not a *soundness* one; the kernel does
  not check the backend, and this corpus must not frame it as if it did. The
  trust chain (`45 §2`): **kernel `Q` (in the TCB) / interpreter-oracle
  `tested` / backend `tested`** — each labeled at its real level; neither the
  interpreter nor the backend is in the type-soundness TCB. (The structural
  claim, not a value-flip: it pins the **failure mode** — differential
  disagreement, never a soundness breach.)

## BD4 — determinism carries to the backend (AC2)

### runtime/backend/backend-preserves-determinism (oracle)
- spec: `45 §4` (BE-Differential), `42 §3.7` (determinism), `42 §2`
- given: the same closed ground term `t` evaluated by the backend **twice**
  (and against the interpreter)
- expect: the backend yields the **same** observable K3 value on every run
  (intra-backend determinism) **and** agrees with the interpreter
  (inter-layer) — a value depending on allocation addresses, hash-map
  iteration order, or any non-deterministic internal leaking into the
  **observable** is **rejected**
- why: (oracle) BD4 — determinism is a precondition for the backend to be a
  faithful differential consumer (the interpreter's own determinism is X1's
  CAN2, `../evaluation/seed-evaluation.md`; this pins the backend must
  **also** be deterministic and agree). A non-deterministic backend breaks the
  "same term → same value" contract the differential corpus rests on.
  Cross-refs CAN2; does **not** re-pin the interpreter's determinism.

## BD5 — capacity / limits cross-ref (AC5)

### runtime/backend/backend-inherits-capacity-bounds (cross-ref)
- spec: `45 §6` (BE-Capacity), `44` (capacity), WP `X4` (scale/limits)
- given: native execution of a term that approaches the `44` capacity bounds
- expect: the backend **inherits** `44`'s capacity model (the store bounds,
  the loud at-limit failure) — its resource story is `44` + X4's, **not** a
  backend-local mechanism
- why: (cross-ref, contract-posture) BD5. Native execution does not get a
  separate capacity model; it rides `44` (`../capacity/seed-capacity.md`) and
  the backend-specific scale/limits are **X4's lane**. Cross-link only — this
  seed does **not** re-specify `44`. (Doc-posture fidelity, no producer.)

## Coverage map (AC → cases)

- **AC1** (not-in-TCB posture):
  `codegen-bug-is-wrong-value-not-soundness-hole` (soundness).
- **AC2** (differential-equivalence discipline):
  `backend-agrees-with-interpreter-on-observable-value` (oracle),
  `observable-value-divergence-rejected` (oracle),
  `unobservable-internal-difference-admitted` (oracle),
  `backend-preserves-determinism` (oracle).
- **AC5** (capacity cross-ref): `backend-inherits-capacity-bounds`
  (cross-ref).
- **AC3** (codegen model) + **AC4** (`OQ-backend-target`) are `/spec §45`'s
  lane (the lowering model + the tradeoff table) — not conformance cases; this
  corpus is **target-agnostic** and does not encode a lowering shape or a
  target.

## Cross-case consistency sweep

- **The observable is one thing across every case (`42 §2`).** BD1
  (agreement), BD2-obs (divergence rejects), BD2-int (internal difference
  admits), and BD4 (determinism) must **agree**: the conformance observable is
  **always** the final content-addressed K3 value — never an internal trace,
  evaluation order, or heap layout. A case asserting a strategy/representation
  difference is a conformance failure would contradict BD2-int and the `42 §2`
  bound.
- **The backend never touches soundness (`42 §5`).** BD3 (not-in-TCB) and
  BD1/BD2 (differential net) are one story: a backend defect is **always** a
  wrong value caught by the differential corpus, **never** a typing/soundness
  breach. A case implying a codegen bug could change what the kernel admits,
  or that the corpus is "kernel-backed," would contradict the not-in-TCB
  posture and the tested-vs-trusted trust level.
- **Determinism is the interpreter's and the backend's (`42 §3.7`).** BD4 and
  X1's CAN2 (`../evaluation/seed-evaluation.md`) are duals across the
  layering: each layer must be deterministic, and the backend must
  additionally **agree** with the interpreter. A non-deterministic backend
  contradicts both.

## Subsumed / not-duplicated (one home per property)

- **The interpreter's canonicity + determinism + kernel-agreement** are
  **X1's** (`../evaluation/seed-evaluation.md`, CAN1/CAN2). This seed drives
  the interpreter as the **oracle** the backend validates against; it does
  **not** re-pin the interpreter's own correctness.
- **The K3 value model, content addressing, O(1) equality, dedup** are the
  **runtime's** (`../seed-runtime.md`, `../values/`, `../../` `41`). This seed
  observes value equality as the **differential** observable; the value model
  is X1/runtime's home.
- **The capacity model** is **`44`/X4's** (`../capacity/seed-capacity.md`).
  BD5 cross-refs it; it is not re-pinned.
- **The codegen lowering model + the backend target** are **`/spec §45`'s**
  (AC3/AC4). `OQ-backend-target` is **operator-open**; this corpus is
  target-agnostic and encodes neither.

## Build-sequencing note (the X3-build differential harness)

This is a **design/discipline** corpus — spec + conformance only, **no crate**
(there is no backend yet). Every differential case's **interpreter half is
landed** (`crates/ken-interp/src/lib.rs`); the **backend half is `(oracle)` /
X3-build-deferred**. The **producer-grep gate is the X3-build ring's**:
`ken- codegen` output must be validated by the **real differential harness**
running the corpus through the backend **and** the landed interpreter,
requiring identical K3 values — **not** a hand-fed value table, **not** the
interpreter compared against itself (the
`conformance-hand-feeds-the-deliverable` net). Flagged forward into the
X3-build frame; it is **not** exercised here. The build does **not** start
until **`OQ-backend-target`** is operator-ratified (~11:00 UTC); this corpus
is target-agnostic and merges independent of that ratification.
