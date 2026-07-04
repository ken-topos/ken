# effect-composition — execute a program that composes two base effects

**Steward frame → spec enclave (elaborate the design) → build.** The capability
VAL2 surfaced and that `fs-read-file-lines-flip` had to route around: today a
Ken program **cannot compose two distinct base effects in one computation** —
`main` can read via `[FS]` **or** print via `[Console]`, but not both, because
`Vis` forces a single concrete op-family per typed tree and the top-level driver
`run_io` has no coproduct dispatch. `read-file-lines` closed 16/0 via **Option
3** (pure-FS `main` returning its lines, CLI prints them) with a documented
honesty asterisk: *it demonstrates FS-read + pure-parse, not effect
composition.* **This WP builds the real capability and retires that asterisk.**

Owner: **Runtime** builds (the effect substrate lives in
`crates/ken-elaborator/src/effects/` + `crates/ken-interp`; Runtime built FS +
the `run_io` driver + owns the State-effect machinery this generalizes). Design
front-loaded to the **enclave** — **Architect owns the core** (he sized this
capability, `evt_2aj8ybb5b44pf`); spec-author + conformance-validator assist on
surface + conformance. Gate: enclave elaboration → merge → Runtime build →
Architect soundness + Runtime-QA + Verify-QA + CI. Findings → **Steward**.

## Why this is a real capability, not a rosetta fix (Architect sizing)

Grounded by the Architect at the flip's D4 escalation (`evt_2aj8ybb5b44pf`):
building+running a `Sum ConsoleOp (FSOp a)` tree needs **three pieces that do
not exist today**. The general `Sum a b = InL a | InR b` coproduct and
`InL`/`InR` **already exist** and are effect-agnostic (`state.rs:206`
`declare_sum`) — so the gap is not the coproduct type; it is everything that
makes a coproduct of effects *constructible at the surface* and *executable at
the top level*:

1. **A general coproduct response family.** `resp_sum` is hardcoded to `Sum
   (StateOp s) f` with State pinned as the first summand (`state.rs:245`
   `declare_resp_sum`). Composing two *arbitrary* base effects needs a response
   combinator for `Sum g h` given each summand's response family — not a
   State-first special case.
2. **Surface injection / lift into the `Sum`.** `read_bytes` / `print_line`
   produce **un-wrapped** `ITree (FSOp a) …` / `ITree ConsoleOp …`; there is
   **no `inject`/`lift` morphism** into a coproduct tree (the State path
   hand-bakes `InL` directly into `get`/`put`). Without a lift, two effects
   cannot be sequenced in one tree at all.
3. **A coproduct-aware top-level interpreter.** `run_io` matches **raw** op tags
   (`Write`/`ReadFile`) with zero `Sum` awareness; `run_state` (the only
   `Sum`-fold) interprets State and passes the *other* summand through
   **un-run**. **No path executes two base effects at the top level.** This WP
   builds one.

The landed **State-effect machinery**
(`crates/ken-elaborator/src/effects/state.rs` — `declare_sum` /
`declare_resp_sum` / `declare_run_state` / `get` / `put`) is the
**template/pattern** to generalize — **not to copy**. Line numbers perishable;
verify against the landed code at pickup.

## Objective + the load-bearing property (frame by acceptance, not mechanism)

**Objective.** A Ken program can **express and execute** a computation that
performs two distinct base effects (concretely: read a file via `[FS]` *and*
print via `[Console]` in one `main`), producing correct observable output — with
the composition **general**, not a one-off `Sum ConsoleOp (FSOp a)`
special-case.

**The generality property (load-bearing — this is *why* the flip deferred the
bolt-on).** The whole reason Option 1 was not bolted into the flip is
**subsume-don't-proliferate**: a hand-built `Sum ConsoleOp (FSOp a)` dispatcher
to print three lines is the extend-with-a-special-case anti-pattern. This WP
must deliver the **general** mechanism — demonstrated by **either** (a) a
parametric combinator that composes *any* two base effects given their pieces,
**or** (b) at least **two distinct effect pairings** running through the same
machinery — so the acceptance proves generality, not a second special case.

## Settled inputs / locked — DO NOT REOPEN

- **Kernel / `trusted_base` untouched.** The whole delta is outer-ring — the
  effect prelude (`ken-elaborator/src/effects/`) + `ken-interp` — mirroring how
  FS and State stayed kernel-clean. **Zero `ken-kernel/` diff, no new
  `Term`/`Decl` variant, `trusted_base` delta zero** (grep-verified, not a
  test). Hand-built inductives via `declare_inductive` (as FS/ITree/`Sum`
  already are, because surface `data` hardcodes params to `Type0`) are fine —
  that is outer-ring elaborator work, not a kernel change. **If any step seems
  to need a kernel/`Term`/`Decl` change → STOP, route to Steward.**
- **Totality preserved.** Composing effects must not open a non-termination or a
  partiality hole; the composed program stays total, and the interpreter's fold
  is structural. (A general `run` over a coproduct tree is the stress case — the
  enclave must show it terminates.)
- **Do not break State or FS.** The State-effect machinery and the FS driver +
  their conformance/tests stay green. Generalizing `resp_sum`/the interpreter
  must **subsume** State (State becomes an instance of the general mechanism, or
  coexists unbroken) — not fork it. `cargo test --workspace` green.
- **The concrete first consumer is FS + Console.** The acceptance re-authors
  `read-file-lines` (or adds a dedicated example) to **genuinely compose** an FS
  read with a Console print — retiring the Option-3 honesty asterisk.
- **Reflect the effect-row model, don't invent a parallel one.** The surface
  already declares effect **rows** (`[FS, Console]`); the gap is executing a
  composed row. The enclave grounds whether `/spec` already specifies effect-row
  composition (e.g. `62`/the effects chapters) and builds the driver + surface
  to **reflect** that spec — not a bespoke composition calculus. If the spec is
  silent or forks, route to Steward → operator.

## Mandated deliverable outline (each item → a concrete choice)

The enclave elaborates the **how**; this frame fixes **what each piece must
achieve**. Illustrative names are tagged *decide against the landed system.*

### D1 — General coproduct response family
Generalize `resp_sum` from `Sum (StateOp s) f` to a response combinator over
`Sum g h` given each summand's response family (`resp_g`, `resp_h`). Pin its
signature + that it is total/structural. State's `resp_sum` becomes an instance
(or is subsumed).

### D2 — Surface injection / lift into a coproduct effect
The morphism(s) that lift a single-effect computation (`ITree g rg a`) into a
coproduct tree (`ITree (Sum g h) (resp_sum…) a`) — the missing `inject`/`lift`.
Decide the surface form: **how does a program author write a computation that
performs both effects?** (a lift/`inject` applied to each op; a combined-effect
`view`; an effect-row-directed elaboration). This is the
**surface-expressibility** crux — if it needs new surface syntax that itself
forks the design, **route to Steward → operator** (do not invent surface
language unilaterally). Ground whether surface `data`'s `Type0`-param limit
forces hand-built inductives here (as it did for FS).

### D3 — Coproduct-aware top-level interpreter
A `run_io` (or a new top-level `run`) that **executes both base effects** in a
`Sum g h` tree — dispatching `InL`/`InR` to each effect's real handler, resuming
the continuation, looping — rather than folding one and passing the other
un-run. Fail-closed on any degenerate shape. Show it terminates (totality
stress).

### D4 — The composed example(s) + generality demonstration
Re-author `read-file-lines` (or a dedicated example) to genuinely compose
FS-read + Console-print through D1–D3; update its `expected` oracle; **retire /
rewrite the honesty asterisk** in its README. Plus the generality witness (AC3):
a second effect pairing **or** the parametric-combinator demonstration.

### D5 — Conformance / acceptance plan (CV)
The e2e that drives a **real composed program** end-to-end (no hand-fed
coproduct trees at the interpreter — the anti-pattern guard,
[[conformance-hand-feeds-the-deliverable]]); the generality discriminator; the
State-and-FS no-regression face; the totality face.

## Acceptance criteria (testable)

- **AC1 — kernel untouched.** `git diff origin/main -- crates/ken-kernel/`
  empty; no new `Term`/`Decl`; `trusted_base` delta zero. **Grep-verified.**
- **AC2 — composed program runs.** A program performing FS-read **and**
  Console-print in one computation elaborates and executes, producing the
  correct observable output (a byte-exact oracle).
- **AC3 — generality (subsume-don't-proliferate).** Not a one-off `Sum ConsoleOp
  (FSOp a)` special-case: **either** a parametric combinator composing any two
  base effects, **or** ≥2 distinct effect pairings through the same machinery.
  Verified structurally, not by a single example.
- **AC4 — totality preserved.** The composed program is total; the interpreter's
  coproduct fold terminates (the recursive/continuation case stressed).
- **AC5 — State + FS subsumed/unbroken.** State-effect + FS driver + their tests
  stay green; State is an instance of (or coexists with) the general mechanism.
  `cargo test --workspace` green.
- **AC6 — honesty asterisk retired.** `read-file-lines` (or the dedicated
  example) genuinely composes; its README no longer defers Console-composition
  as a gap (or the deferral is rewritten to what actually remains).
- **AC7 — no hand-fed coproduct.** The e2e drives a real composed surface
  program; no test hand-constructs the `Sum`/`InL`/`InR` tree at the interpreter
  to stand in for the surface flow ([[conformance-hand-feeds-the-deliverable]]).

## Guardrails — do not reopen

1. **Kernel untouched** (AC1). Seems-to-need-kernel ⇒ STOP → Steward.
2. **General, not a special-case** (AC3) — the reason the flip deferred this.
3. **Subsume State/FS, don't fork** (AC5).
4. **Reflect the spec's effect-row model** — if surface syntax forks the design,
   route to Steward → operator; don't invent surface language in the build.
5. **Totality is non-negotiable** (AC4).

## Sequencing (§2c)

1. **Steward authors this frame** (done) on `wp/effect-composition` (off
   `origin/main@43e97d02`).
2. **Enclave elaboration** — ⛔ handoff gate first (compact-verify the enclave
   **unconditionally**). Mention **spec-leader**; **Architect owns the core
   design** (D1–D3 soundness + the coproduct/interpreter mechanism), spec-author
   the surface form (D2), conformance-validator D5. If D2's
   surface-expressibility forks the design (new surface syntax) → **route back
   to Steward → operator.**
3. **Elaborated spec merges to `main`** (spec-leader → Integrator).
4. **Runtime build** — ⛔ handoff gate first (compact the team, unconditional).
   Kick off leader-only. One branch, one merge Decision. Gate: Architect
   soundness + Runtime-QA + Verify-QA + CI.

## Size / risk

**L** (Architect: operator-scope-worthy new capability). Three coupled pieces
(response combinator + injection/lift + coproduct interpreter) plus a
surface-form decision (D2) that could itself fork. Design-heavy → front-loaded
to the enclave. Contained by: kernel-clean by construction (outer-ring, like
FS/State), a landed template to generalize (`state.rs`), and a concrete first
consumer (FS+Console). **This retires VAL2 16/0's one honesty asterisk — Ken
programs that compose effects become expressible and runnable.**
