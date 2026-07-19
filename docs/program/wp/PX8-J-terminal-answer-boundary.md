# PX8-J — continuation terminal-answer boundary (native lowering)

> **Runtime-owned prerequisite for PX8-F, opened by Architect ruling
> `evt_2gs0zfg66x9qk` (2026-07-19 ~03:00 — PX8-I's third/downstream hard-stop).**
> PX8-I's exact-Int mechanism was banked at candidate `a02a4967` when this
> ruling was made, then landed — after two §14 respins (real production
> repairs, then a frame-only fix) — **byte-identical at
> `origin/main = 38ed82231f36c6b19e0682956a741700a7a3cc26`**, the immutable
> PX8-J base. On that base the unchanged PX8-F
> `c8b8cdb7` overlay clears the old dynamic-`add_int` boundary and now stops
> earlier, at `ComputationalMatch: scrutinee is not a constructor value after
> ordinary expression lowering`. The Architect classified this as a **distinct
> Runtime continuation prerequisite**, not an exact-Int closure: the
> process-object scalarization fallback in `merge_scalar_branch` mints a
> `ProcessExitStatus`/`ExitCode` **while a computational eliminator is still
> live** — process-object mode is being treated as evidence of a terminal
> answer, which it is not. PX8-J fixes exactly that: `ProcessExitStatus` may be
> minted **only at a proven terminal process-answer boundary**.
>
> Chain position: **PX8-L ✅ → PX8-H ✅ → PX8-I ✅ (landed `38ed8223`) →
> PX8-J (this) → PX8-F.**

## Objective

`ProcessExitStatus`/`ExitCode` is minted **only when no ordinary, computational,
active, or source continuation frame can still consume the current Ken value**.
Every scalar join scalarizes **only** under the PX8-H **preplanned checked
answer-kind cut** — never merely because a process object is being emitted.
Aggregate `Constructor` values remain inside **branch-local prefix P** until
their eliminators drain; they never cross the scalar join. With that boundary
enforced, the unchanged PX8-F `c8b8cdb7` overlay clears the `ComputationalMatch`
stop, reaches a **real `FsWriteAt`**, and the native run **agrees with the
interpreter** — the overlay AC deferred out of PX8-I lands here.

Owner: **Runtime** (L/I/Q `agt_37reqrd72cg00` / `agt_37reqg3nync00` /
`agt_37reqvb6ce400`). Size **S–M**, Risk **Medium** (a join/answer-boundary
correctness fix on the PX8-H scalar-cut surface; the discipline is
tighten-the-precondition + a complete exhaustive-consumer ledger, never widen a
match arm). Route **Architect §14 + Runtime QA**; **no CV lane** (no
spec/conformance movement).

## Fixed inputs — DO NOT REOPEN (Architect ruling `evt_2gs0zfg66x9qk`; Research advisory `evt_6pktdhbkrskbm`; settled)

These are the **contract**, ruled by the Architect (and grounded by the Research
advisory). They are not open for the implementer to relitigate; a mechanism
question *beyond* them is a hard-stop to the Architect, never an implementer
choice.

1. **Terminal-answer boundary.** `ProcessExitStatus`/`ExitCode` may be minted
   **only when no ordinary, computational, active, or source continuation** can
   still consume the value. A `Constructor` with a **live eliminator** is on the
   aggregate-consuming side of the cut — it is **not** a terminal answer, and it
   is **not** an aggregate-join payload.
2. **Preplanned checked answer-kind cut.** At **every** scalar join, the value
   that crosses is decided by the **PX8-H checked join plan** (the answer-kind
   cut resolved before predecessor emission — only `Int | Bool | StructuralNat |
   ExitCode` crosses). **Process-object mode alone is never evidence of
   `ExitCode`.** The erased graph **validates** the plan; it never invents it.
3. **Aggregates stay in prefix P.** Aggregate `Constructor` values live **and
   are consumed** inside branch-local prefix P until their eliminators drain. Do
   **not** carry an aggregate across the scalar join.
4. **Preserve the fail-closed refusal.** The existing `ComputationalMatch`
   refusal (`lower_computational_match_value_composed`, the "scrutinee is not a
   constructor value…" fail-close) stays **unchanged**. Do **not** relax it.
5. **No special-casing, no weakening.** Do **not** add a `writeAll` special
   case, and do **not** weaken the join-kind contract. The fix is to establish
   the terminal-answer **precondition** before any `ExitCode` mint — not to
   broaden a match arm or invent a bypass.
6. **Authorization boundary.** **No** new kernel rule, **no** public surface op,
   **no** host-ABI/wire change, **no** trusted primitive, **no** `writeAll`
   intrinsic, **no** spec/conformance movement, **no** Cargo dependency.
   Runtime-owned **native lowering** layer only. The external process result and
   `ken_host_dispatch_v1` remain unchanged.
7. **PX8-I is immutable.** PX8-I is landed and **byte-unchanged** on
   `origin/main = 38ed8223` (the ruling named the then-banked candidate
   `a02a4967`; PX8-I landed at `38ed8223` after two §14 respins). PX8-J is
   **additive on top** of it; it does not rebase, amend, or re-open the
   exact-Int mechanism or its gates.

## Landed anchors (at PX8-I `origin/main = 38ed8223`; verify before editing; do not trust frozen line numbers)

Re-grep each on the PX8-J base (= `origin/main = 38ed8223`); the line numbers
below are orientation refreshed against that base, not contract.

- **The defect — process-object scalar fallback:**
  `crates/ken-runtime/src/cranelift_backend.rs` — `merge_scalar_branch`
  (≈`:4399`). The offending arm is `lowered if self.process_object => …
  emit_process_exit_status(…) … ScalarMergeKind::ExitCode` (≈`:4458-4464`); a
  **sibling** process-object fallback (in the dynamic-native-arms scalarizer)
  sits at ≈`:4385-4391`. Both mint an `ExitCode`/exit-status payload from
  **any** otherwise-unmatched `Lowered` whenever `self.process_object`, with
  **no** proof that the continuation is terminal. This is the site the
  terminal-answer predicate must gate. (The adjacent arm at ≈`:4451-4457`,
  `Lowered::ProcessExitStatus => … ScalarMergeKind::ExitCode`, is the
  **legitimate** already-terminal path — do not gate it away.)
- **The precondition to preserve (do not touch its refusal):**
  `lower_computational_match_value_composed` (≈`:3785`); its fail-close
  `"scrutinee is not a constructor value after ordinary expression lowering"`
  (≈`:3814`). It requires a `Lowered::Constructor` semantic input while an
  eliminator is live and fail-closes otherwise. Keep it exactly.
- **Terminal decoder (recognizes only the process constructors):**
  `emit_process_exit_status` (≈`:9663`) — maps the process `Success`/`Failure`
  constructors to a status and other constructors to failure status. It is a
  **terminal observation decoder**; applying it before a computational
  eliminator consumes the constructor is an answer-boundary/phase error, not a
  missing value.
- **The answer-kind cut to route through:** `ScalarMergeKind` and the PX8-H join
  plan mapping — `NativeJoinAnswerKindV1::ExitCode => ScalarMergeKind::ExitCode`
  (≈`:4697`); `ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus`
  (≈`:4510`). The planned cut is the **only** authority for which kind crosses a
  join.
- **PX8-H contract (the invariant PX8-J restores):**
  `docs/program/wp/PX8-H-heterogeneous-continuation-composition.md` (≈`:110-137`)
  and Architect ruling `evt_7sdmvyme8qy50` — a checked join plan is resolved
  before predecessor emission; only the planned scalar kind crosses; aggregates
  remain in branch-local P.

## Mechanism as ruled (authoritative; by citation)

Ruling `evt_2gs0zfg66x9qk` fixes the **contract** in "Fixed inputs" above — the
terminal-answer boundary, the preplanned answer-kind cut, prefix-P retention,
the preserved `ComputationalMatch` fail-close, and the authorization boundary.
The Research advisory `evt_6pktdhbkrskbm` grounds it: standard handler semantics
apply a return clause only when the handled computation returns its **final
value**, while an operation result is first supplied to its captured
continuation (Kammar & Pretnar 2016; Koka handler/resumption account); Cranelift
branch-argument arity supports a **planned** cut but never justifies changing a
value's **semantic kind** merely because a process object is being emitted. The
ruling deliberately **does not** dictate the exact predicate representation or
where it is threaded — that is **implementer execution under Architect §14**.
Restate the contract; **do not design past it.** Any mechanism question the
contract does not settle (e.g. how "no live consumer" is proven at a given join,
how prefix P is represented at a specific site) is a **hard-stop to the
Architect** (`mentions` architect + leader) with the exact value-kind/site —
never an implementer guess, never a widened/weakened match.

## Mandated deliverables (each ends in a concrete implementable choice)

1. **Terminal-answer predicate.** Introduce a proven check that **no** ordinary,
   computational, active, or source continuation frame remains live for the
   value being scalarized to `ProcessExitStatus`/`ExitCode`. **Replace** the
   unconditional `lowered if self.process_object` fallbacks (`:4461-4466` and the
   `:4388-4391` sibling) with mints **gated on that predicate**. **State** how
   the predicate is derived (from the checked join plan / the live-frame set) and
   why it is complete.
2. **Route every scalar join through the preplanned cut.** A join scalarizes
   **only** the answer-kind the PX8-H `NativeJoinPlan` classifies as crossing
   (`Int | Bool | StructuralNat | ExitCode`). A `Constructor` with a live
   eliminator is **not** a join payload — it stays on the aggregate side.
   **State** the exact join sites touched and the plan field consulted.
3. **Prefix-P retention.** Aggregate `Constructor` values live and are consumed
   inside branch-local prefix P until their eliminators drain. **State** how P is
   kept intact across the process-object lowering path (no aggregate crosses the
   scalar join).
4. **Preserve fail-closed behavior verbatim.** The `ComputationalMatch` refusal
   (`:3817`) and every existing fail-close stay unchanged; **no** `writeAll`
   special case; **no** join-kind-contract weakening. Confirm by diff that these
   arms are untouched.
5. **Join/scalarization site ledger.** Enumerate **every** site that can mint
   `ProcessExitStatus`/`ExitCode` (the two fallbacks + any planned-cut mint) and
   show each is now either (a) a planned answer-kind cut or (b) gated by the
   terminal-answer predicate. No process-object-mode-only mint survives.
6. **Cross-crate exhaustive-consumer closure (the PX8-H/PX8-L CI-red lesson).**
   Derive the **whole-test-tree** exhaustive-matcher ledger for the join/scalar
   surface (`git grep -nE 'ScalarMergeKind|ProcessExitStatus|process_object'
   crates/*/tests crates/ken-cli/tests` and the lib crates) and gate every
   classified consumer. A targeted `--lib` + one-suite gate **misses** a second
   exhaustive integration consumer, and **only full-workspace CI catches it**
   (this cost PX8-H and PX8-L CI-red respins; see the nc17 integration-contract
   miss). **No broad `_` arms** that re-admit the process-object bypass.

## Required proofs / discriminators (each independently reaching)

- **The blocked overlay now clears the stop (the deferred PX8-I AC):** the
  unchanged PX8-F `c8b8cdb7` overlay passes the `ComputationalMatch` boundary,
  reaches a **real `FsWriteAt`**, and the native run **agrees with the
  interpreter** (differential). Run it as a **throwaway overlay** — do **not**
  edit `c8b8cdb7` or the fixture.
- **A live computational eliminator is not prematurely scalarized:** the overlay
  failure (reproduces on the `38ed8223` base) — a `Constructor` presented to a
  live computational eliminator after a process-object scalar join — now lowers
  correctly (the aggregate stays in P; no `ExitCode` mint).
- **Mutation control (predicate is load-bearing):** reverting to the
  unconditional `process_object` fallback re-introduces `ComputationalMatch:
  scrutinee is not a constructor value…` — proving the terminal-answer gate is
  what fixes it.
- **No regression on the genuine terminal path:** where the value **is** a real
  terminal process answer (`Success`/`Failure`), the correct `ExitCode` is still
  minted — the boundary tightens the precondition without dropping legitimate
  terminals.
- **Live source/active continuation preserves its aggregate:** a `Constructor`
  flowing through a process-object join **with** a live source/active
  continuation is **not** scalarized to `ExitCode` (a distinct reaching test
  from the computational case).
- **Fail-close intact:** the `ComputationalMatch` refusal still rejects a genuine
  non-constructor scrutinee (unchanged behavior).

## Acceptance criteria (testable)

- The unchanged PX8-F `c8b8cdb7` `writeAll` overlay (throwaway; **do not** edit
  `c8b8cdb7` or the fixture) compiles/links/runs, reaches a **real `FsWriteAt`**,
  and the native run **agrees with the interpreter** (differential). **This is
  the overlay AC deferred out of PX8-I; PX8-J owns it.**
- Every discriminator above is green; the `ComputationalMatch` refusal and all
  fail-closes are **unchanged**; **no** `writeAll` special case; **no**
  join-kind-contract weakening.
- **No process-object-mode-only `ExitCode` mint survives:** every
  `ProcessExitStatus`/`ExitCode` mint is either a planned answer-kind cut or
  gated by the terminal-answer predicate (the site ledger proves it).
- **No forbidden movement:** no kernel / host-ABI / wire / public-surface /
  trusted-primitive / `writeAll`-intrinsic / spec / conformance / Cargo change.
  Any mechanism gap **beyond** the contract is a **hard-stop to the Architect**,
  not an implementer choice.
- **Whole-test-tree exhaustive-consumer ledger complete**; every classified
  integration suite gated. **Targeted local builds only** (`scripts/ken-cargo -p
  <crate>`; never `--workspace`); "workspace-green" / no-regression means
  **green in CI**.
- **PX8-I landed SHA `38ed8223` is byte-unchanged** — PX8-J is additive; it does
  not amend PX8-I or its exact-Int gates.

## Do-not-reopen guard

- The **terminal-answer boundary**, the **preplanned checked answer-kind cut**,
  **prefix-P retention**, and the **preserved `ComputationalMatch` fail-close**
  are Architect-ruled fixed inputs. Do **not** carry aggregates across the scalar
  join, add a `writeAll` special case, weaken the join-kind contract, or relax
  the refusal.
- Do **not** re-open PX8-I's exact-Int mechanism — it is landed immutable at
  `origin/main = 38ed8223`; PX8-J builds on top of it.
- Any mechanism gap **beyond** the contract → **hard-stop to the Architect**
  (`mentions` architect + leader) with the exact value-kind/site — never an
  implementer guess, never a widened/weakened match.

## Sequencing

1. **PX8-I has landed** at `origin/main = 38ed8223` (includes PX8-H and PX8-L).
   Branch `wp/px8j-terminal-answer-boundary` off **`origin/main = 38ed8223`**
   (the fetched ref — verify `git merge-base` == `origin/main`, and re-grep
   every anchor above on that base).
2. **Handoff-gate the Runtime team** first (§2c): all PX8-I retros in → compact
   L/I/Q → verify the drops WIDE → kick with this brief cited → confirm each seat
   goes `Working`.
3. Implementer builds → Runtime QA → Runtime leader creates **one Decision** →
   **Architect §14** (resolve-on-cast). No CV lane.
4. On resolved APPROVE → **publisher non-doc-only** (crates → CI must go green,
   **including full-workspace exhaustiveness**) → verify byte-identical landing +
   **PIN** → chase retros → close the WP.
5. **Downstream:** PX8-J landed → **unblock PX8-F** (rebase `c8b8cdb7` onto the
   combined `main`; the unchanged `writeAll` fixture now performs **real writes**
   in both interpreter and native lanes = PX8-F native evidence). PX8-F is the
   Linux ABI capstone; PX8-J is its last Runtime prerequisite.
