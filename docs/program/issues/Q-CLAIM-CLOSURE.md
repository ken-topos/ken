---
id: Q-CLAIM-CLOSURE
title: "Q-RESIDUE adversary findings — claim-loss in multi-claim test blocks, plus R1/R2/R3"
status: ready
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: adversary report evt_2nwr718cpfbg on Q-RESIDUE @ 64337192 (2026-07-21)
---

The adversary's post-merge hunt on Q-RESIDUE. **Advisory, non-blocking — no
live defect sits behind any of it.** The adversary checked the production
side directly and confirmed the underlying code is correct today. These are
**coverage regressions** that would land green on a future refactor.

Full report: `evt_2nwr718cpfbg` (thread `thr_7htm8zxfn46wh`).

## ★ The generator — the finding that outlives the rows

**Q-RESIDUE's acceptance criteria took the TEST as the unit. The load-bearing
unit was the CLAIM.** AC-1 said "reworked to a durable assertion"; AC-2 said
mutation-proof "**the** claimed mechanism" — singular. At least three of the
ten tests were multi-claim blocks:

| test | claims | outcome |
|---|---|---|
| `resource_table_lifetime_is_owned_by_one_interpreter_invocation` | 3 | 1 behaviorally proved; **2 now checked by nothing** |
| `list_instance_routes_the_canonical_compare_into_raw_list_compare` | 2 | **both dropped**; replacement instantiates only `Bool` |
| `generated_manifest_is_closed_and_probe_comparison_discriminates` | 2 | see R1 |

**A rework that strengthens one claim, mutation-proves that claim, and
silently drops its siblings fully satisfies AC-1 and AC-2 as written.** There
is no closure requirement over the claims a test carried.

This is `ac-naming-a-mechanism-transfers-its-blind-spots` at the granularity
level. **Fix it in the next rework frame, not by re-litigating this one:**
require the rework to *enumerate every claim the old assertion block carried*
and mark each **replaced** or **consciously dropped**. That is the closure the
current AC lacks.

Note the second-order point: `list_...raw_list_compare` kept its **name** while
losing the routing claim the name asserts — a hardcode of `list_compare` to
`Bool` would now pass under it. See `identifiers-are-claim-artifacts`.

## R1 [highest] — the ABI fact inventory has no independent anchor

`crates/ken-host/src/lib.rs:977` dropped `assert_eq!(TARGET_ABI.fact_count,
23)`, keeping `fact_count == facts.len()`.

**★ The frame directed this, and the frame's premise was inverted.** The
inventory row said *"`fact_count==23` duplicates a generated manifest; assert
against the manifest, not a copy of its size."* But at `crates/ken-host/
build.rs:475-476` **both fields come from one generator expression** — `facts`
is interpolated as the array, `fact_count` counts `AbiFact` occurrences in that
same text. The surviving check is near-tautological: it fails only if
build.rs's own substring heuristic miscounts.

The obvious rescue does not work. `TARGET_ABI_MANIFEST_HASH` is itself
generated (`build.rs:474`), as is `TARGET_ABI_CANONICAL` (`:473`). The
adversary grepped `crates/`, `spec/`, `conformance/`, `docs/` — **no pinned
literal hash and no golden copy of the fact inventory exists anywhere.** If
build.rs emitted 22 facts, the hash would be the hash of 22 facts and
everything stays green.

**Being generated is exactly what made the literal load-bearing** — it was the
only assertion in the loop that was *not* generated. "Assert against the
manifest" is unachievable as written: there is no independent manifest.
**The replacement must be an out-of-band anchor** (a pinned inventory the
generator does not produce), not a relational check.

## R2 [high] — a comment cites a sibling test that does not exercise the claim

`cat5_parsing_package.rs:470` says `source_length`'s byte-view computation is
"proven behaviorally by `cat5_d1_concrete_nonempty_source_constructs_and_
projects`". It is not. That test (`:612`) recomputes on the **raw bytes
constant** — never calling `source_length`, never going through
`sample_source` or the Source class instance — yet `:641` asserts on it with
the message *"source_length must execute through the same class-backed Source
instance."*

**No test in the file ever evaluates `source_length` on a concrete Source and
checks the number.** Every other occurrence is in a *type* position, discharged
by `LessEqNat::zero_left`/`::refl` — generic in the Nat. Redefine
`source_length` to return a constant and the file stays green. `PX8-F-PROOF`
shape: a claim whose cited evidence does not carry it.

## R3 [moderate] — assertion messages naming properties the assertions don't check

`pi_arity` (`cat5:175-180`) matches `Term::Pi(_, codomain)` — **the domain is
discarded.** At `:504-513` it backs the message *"SourceId must wrap exactly
one **Nat**"*; change `MkSourceId`'s field to any other single-argument type
and it still passes.

## R4 [low] — undisclosed cat5 narrowings

Each existed before, each now has zero coverage, **none acknowledged**:
`!contains("data SourceId =")` (guarded against local re-declaration vs.
importing `Diagnostics.Core`'s); `erase_spans`/`ValidSyntax` dropped from
signature-pinning to bare `contains_key`, inconsistent with neighbours three
lines down that get a real type-pinning probe; `!contains("compiler")`/
`!contains("AST")` dropped with no replacement; D1's `!contains("String Nat")`
and D2's `decoder_recursive`/`decoder_many` type-arg pins (the D2 one looks
genuinely subsumed by D3's roundtrip — it just isn't cross-referenced the way
the `= Axiom` drops were).

## Acceptance criteria

1. **R1 first**, framed as an out-of-band anchor. Do **not** re-use the
   "assert against the manifest" phrasing — that phrasing produced the defect.
2. R2/R3: either make the cited evidence real, or correct the claim to what is
   actually checked. Both are valid; an honest narrower claim beats a false
   broader one.
3. For each of the three multi-claim tests, **enumerate the claims the original
   block carried** and mark each replaced or consciously dropped, with reason.
4. R4's narrowings: acknowledge in-test or restore. Silence is the defect.
5. Mutation proof per reworked claim — **per claim, not per test.** That is the
   whole point of this item.

## ⚠ Separately: `ken-host` cannot compile on any non-linux target

**Pre-existing, NOT from this merge** — introduced at `049628f8` (PX5).
`crates/ken-host/src/abi_v1.rs:747` uses `?` on an `Option` inside a function
returning `Result<_, ProcessContextInitError>`. The adversary did not infer
this: they extracted the shape into a standalone file and **compiled it**
(`error[E0277]`).

Consequence: ken-host carries **28** `cfg(not(target_os = "linux"))` sites
implementing a deliberate **fail-closed** posture — including
`unavailable_target_manifest_fails_closed` (`lib.rs:1129`) asserting
`backend.starts_with("unavailable-")` and `fact_count == 0`. **None of it has
ever been compiled, let alone run.** The crate is documented Linux-only, so
this is honesty/dead-code tier rather than a runtime bug, but *a fail-closed
safety posture that has never built* is worth a decision. **Not folded into
this item** — needs its own scoping call (fix, or state the non-linux lane as
unsupported and delete the dead posture).

## Correction worth recording

`ken_host_invocation_v1_start` **does not exist anywhere in the repo.** My
merge-notification note said two reviewers checked the SAFETY comment against
`_start`'s shape. The real entrypoints are `_init` (`:539`), `_destroy`
(`:816`), `_finish` (`:824`), `ken_host_dispatch_v1` (`:1142`). The shape they
would have found under `_init` is the right one and the SAFETY claim **holds**
(independently verified: `initialize_process_context` boxes at `:757`,
`Box::into_raw(context).cast()` at `:672`, `finish` guards null + alignment at
`:828`). So this reads as a transcription slip, not a bad review — **but it is
a claim about what was checked**, and it propagated through my summary
unchallenged. `identifiers-are-claim-artifacts` again.
