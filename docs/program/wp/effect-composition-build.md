# effect-composition — BUILD (execution wrapper)

**Steward build frame → Runtime team.** The enclave elaboration (D1–D5) merged
on `main` (**PR #285, `e9ebda49`**): `docs/program/wp/effect-composition.md`
(D1/D2/D3 + AC1/AC4 certs) and `docs/program/wp/effect-composition-conformance.md`
(D5). **That doc is the canonical design — read it first and build to it;** this
frame is only the cross-crate execution checklist + the hard build-verification
ACs (BV1–BV7) that gate the merge. Nothing here reopens a design decision.

Owner: **Runtime** (the effect substrate is `crates/ken-elaborator/src/effects/`
+ `crates/ken-interp`; Runtime built FS, the `run_io` driver, and the State
machinery this generalizes). Gate: **Architect re-certifies AC1/AC4 on the built
diff** (the fs-flip pattern — design-time certs become build-time re-checks) +
Runtime-QA + Verify-QA + CI. Findings / any forced deviation → **Steward**
(surface it, don't smuggle it — the fs-flip precedent).

## What lands (D1–D5 → code)

Build strictly to the merged doc's pinned mechanisms. Section anchors below point
into `effect-composition.md` / `-conformance.md` on `e9ebda49`.

- **D1 — general `resp_sum`** (`effects/state.rs`, generalize `declare_resp_sum`;
  doc §D1.1–D1.3). Replace the `Sum (StateOp s) f`-hardcoded response combinator
  with the general family:
  ```
  resp_sum : (g h : Type) -> (rg : g -> Type) -> (rh : h -> Type) -> Sum g h -> Type
  resp_sum g h rg rh (InL x) ≡ rg x   ;   (InR y) ≡ rh y     -- per-tag ι
  ```
  One **non-recursive** `Term::Elim` over `Sum`, large-elim motive
  (`λ_:Sum g h. Type0` ascribed `Π_. Type1`), `method_inl = λx. rg x`,
  `method_inr = λy. rh y`. **Reducing `declare_def`, NEVER a postulate** (BV2).
  State becomes the literal instance `resp_sum (StateOp s) f (resp_state s) RespF`
  — update `get`/`put`/`runState`'s `resp_sum`-apps to the 4-arg form (the
  `resp_sum_app` helper); State's structure is otherwise unchanged (§D1.3, AC5).
- **D2 — `injectL` / `injectR` morphism** (`effects/state.rs` region, hand-built
  `declare_def` + `elim_ITree` tag-map, **same technique/file as `bind`**; doc
  §D2.1–D2.2). `injectL/injectR : ITree g rg a -> ITree (Sum g h) (resp_sum g h
  rg rh) a`, `Vis op k ↦ Vis (InL op) k'` (resp. `InR`). Coercion-free by D1's
  reduction (the `k'` domain `resp_sum … (InL op)` ι-reduces to `rg op` = `k`'s
  domain — no transport term). No new surface `data`, **no new grammar**: the
  surface form is ordinary application `bind (injectL c) (λx. injectR c')`
  (§D2.3 — STOP correctly not triggered). `get`/`put`'s hand-baked `InL` is now
  an `injectL` instance (State subsumed, not forked).
- **D3 — coproduct-aware terminal `run_io`** (`crates/ken-interp/src/eval.rs`;
  doc §D3.2–D3.4). At each `Vis op k`: **recursively strip `InL`/`InR`** (payload
  at `op_args[2]` = Sum's 2 params + slot 0) to the innermost non-`Sum` op →
  match that base tag against the **existing** base table (`Write`→`println`,
  `ReadFile`→cap-gated `fs::read`) → `apply(k, resp)` → loop. **Effect-blind: no
  `ConsoleOp`/`FSOp` literal in the peel** (BV5). Fail-closed EFF7 on unknown
  tag / wrong-arity `InL`/`InR` / `Unknown` ⇒ `UnknownEffect` (never a catch-all
  skip). Wiring: driver gains `SumIds { inl_id, inr_id }` from `GlobalEnv`, like
  the existing `ConsoleIds`/`FSIds` (§D3.4) — thread it in `ken-cli` too.
- **D4 — the composed example** (`examples/rosetta/…`). A **real `.ken` program**
  that reads a file via `[FS]` **and** prints via `[Console]` **in one
  computation** (`bind (injectL (read_bytes …)) (λbytes. injectR (print_line …))`
  or the doc's worked form §D2.4), byte-exact `expected`. This is what **retires
  read-file-lines' Option-3 honesty asterisk** — either re-home `read-file-lines`
  onto real composition or add a sibling composed example and update its README
  (AC6 — retire the asterisk **iff** the residual claim matches the landed
  capability, doc §5 / `-conformance.md`). Plus the **generality demonstration**
  (AC3): the {FS,Console} pairing through `run_io` **and** a State-bearing pairing
  (e.g. `runState s₀ (…)` with a Console op) through the general `resp_sum`.
- **D5 — conformance** (`-conformance.md`; tests under the crates). AC2 composed
  program + byte-exact oracle via `run_file`; AC3 generality (structural
  effect-blind-peel grep + executable pairings + `resp_sum`-reduces + faithful
  retag); AC5 State+FS unbroken (`cargo test --workspace` green); AC6 asterisk
  retirement; AC7 no hand-fed coproduct; the **authority-non-laundering** pair;
  the totality face; and the **synthetic-third-op white-box peel probe** (§3 of
  `-conformance.md`, AC7-exempt — it is a mechanism unit test, not the acceptance
  path).

## Build-verification ACs (BV1–BV7) — HARD merge gates

These fold the enclave's soundness certs into build-checkable gates. Each states
what the reviewer re-derives on the **built diff**, not the prose.

- **BV1 — AC1 kernel-untouched (grep-verified).** `git diff --name-only
  e9ebda49..<build>` touches **no `crates/ken-kernel/`**; no new `Term`/`Decl`
  variant; `trusted_base()` delta **zero**. D1 is a `declare_def` (real kernel
  term, re-checked — a def, not a postulate); D3 is pure Rust in `ken-interp`;
  `Sum`/`ITree` already `declare_inductive`-built. Architect re-certifies.
- **BV2 — `resp_sum` is a REDUCING `declare_def`, never a postulate (the hinge).**
  Verify the per-tag ι actually fires: elaborate a term forcing `resp_sum g h rg
  rh (InL x)` and confirm it reduces to `rg x` (not stuck). A postulate here
  silently breaks `inject`'s coercion-free typing — **discriminating test
  required**, not just "it compiles." (doc §D1.1)
- **BV3 — cap-gate stays downstream of the peel (THE load-bearing soundness AC).**
  `authorizes` (the sole runtime FS net) sits **unconditionally inside the
  `ReadFile` arm**, reached **only after** the peel lands on `readfile_id`; the
  peel adds **no** `std::fs::read` path that skips it, and does **not** hoist /
  cache / short-circuit the gate. Prove with CV's **executable pair**: a composed
  program with a sufficient cap reads+prints; the **same** program with `Cap
  ANone` is denied at `authorizes` → `CapabilityDenied`, **right-reason** (not
  `NotFound`, not a bind error). Green-vs-green forbidden. (doc §D3.3)
- **BV4 — COEXIST preserved (TCB non-regression).** `run_io` does **NOT** subsume
  `run_state`; `run_state` stays a **separate kernel-re-checked `declare_def`
  fold**. Do **not** fuse State's discharge into the trusted-Rust `run_io` — that
  moves a kernel-verified fold into the trusted surface (a TCB regression the
  Architect rejected on soundness). Verify `run_state` is untouched as a distinct
  fold. (doc §D3.1)
- **BV5 — effect-blind peel (AC3 structural).** Grep the peel + the general
  `resp_sum`/`run` for **no `ConsoleOp`/`FSOp` literal** — dispatch is only on the
  innermost base tag, `InL`/`InR`-generic. (doc §D3.2, `-conformance.md` §3)
- **BV6 — totality + no regression.** No unbounded recursion added (the peel is a
  bounded descent on a finite op value; the outer loop still strictly descends the
  finite `ITree` via `apply(k, resp)`). `cargo test --workspace` green — the
  substrate generalization breaks **no** existing consumer; **State + FS
  single-effect trees unchanged** (zero wrappers ⇒ peel is a no-op). (doc §AC4)
- **BV7 — no hand-fed coproduct (AC7).** The **acceptance** e2e enters through
  `ken-cli`'s `run_file` over a real `.ken`; `Sum`/`InL`/`InR`/`inject` are
  **elaborated from surface**, never Rust-built and fed to `run_io`. (The
  synthetic-peel white-box probe is exempt — it is a labeled mechanism unit test,
  not the acceptance path.) This is the trap the flip's `Result` field-order lie
  rode in on ([[conformance-hand-feeds-the-deliverable]]). (`-conformance.md` §2)

## Locked-carried (do not reopen — pinned by the merged elaboration)

- **GENERAL, not a `Sum ConsoleOp (FSOp a)` special-case** — the whole point;
  deliver the parametric machinery (≥2 pairings / a combinator), not a hardcoded
  pair.
- **Kernel-untouched** (BV1); **totality** (BV6); **State + FS subsumed unbroken**
  (instances of the general family, not forks); **reflect the spec's effect-row
  model** (`36 §2.3`/`§2.4`/`§4.5.4`/`§5.1`) — no parallel calculus.
- **The COEXIST ruling (BV4) and the cap-gate constraint (BV3) are
  non-negotiable soundness pins** — a "cleanup" that fuses `run_state` into
  `run_io`, or hoists the `authorizes` gate, is an automatic gate fail.
- **Surface form is `bind (injectL …) (λ. injectR …)`** — ordinary application,
  no new grammar (STOP not triggered; do not invent surface syntax).

## Files (expected touch set)

- `crates/ken-elaborator/src/effects/state.rs` — D1 (`declare_resp_sum` →
  general), D2 (`injectL`/`injectR` `declare_def`), `get`/`put`/`runState` → 4-arg
  `resp_sum`.
- `crates/ken-interp/src/eval.rs` — D3 (`run_io` Sum-peel; `authorizes`
  downstream; `SumIds`). `run_state` **untouched** (BV4).
- `crates/ken-cli/src/main.rs` (+ `repl.rs` if needed) — `SumIds` from
  `GlobalEnv` wiring; composed-example entry.
- `examples/rosetta/…` — D4 composed FS+Console example + `expected`; retire
  read-file-lines' asterisk (README/KNOWN-GAP reconcile, AC6).
- crate tests — D5 conformance (composed e2e, authority-non-laundering pair,
  synthetic-peel probe, faithful-retag, `--workspace` green).

## Sequencing / gate

1. Runtime builds to the merged doc; `run_state` stays a distinct fold (BV4).
2. Runtime-QA independent pass (re-derive BV1–BV7 from the built diff; the
   `op_args[2]` peel index and the `authorizes`-downstream placement are the
   ctor-arity / sole-net checks — trace them, don't trust the report).
3. **Architect re-certifies AC1/AC4 on the built diff** (BV1/BV6) + the cap-gate
   placement (BV3) + COEXIST (BV4).
4. Verify-QA + CI (`cargo test --workspace`); merge Decision → Integrator.
5. On merge: retires read-file-lines' honesty asterisk; rosetta stays 16/0 (or
   +1 if a new composed example is added). Effect-composition capability closed.
