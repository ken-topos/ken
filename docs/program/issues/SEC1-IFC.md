---
id: SEC1-IFC
title: "Reify the three named Sec1 stubs — two of them are the SOLE NETS for Sec1's two trusted surfaces, and both are placeholders under a green suite"
status: merged
owner: verify
size: M
gate: G-Sec
depends_on: []
blocks: []
github: https://github.com/swe-toolkit/ken/pull/1094
origin: Corrected 2026-07-27 by the Steward after a mis-release. Sec1 is BUILT (crates/ken-elaborator/src/ifc.rs + tests/sec1_acceptance.rs, 20 tests green); the residual is the three reify triggers the source itself names at ifc.rs:361-381. Frame docs/program/wp/Sec1-build.md. Owner operator-decided — WS-Sec build is a scope extension of Team Verify.
---

> ## ⛔⛔ READ FIRST — THIS NODE WAS MIS-RELEASED ONCE. Sec1 IS ALREADY BUILT.
>
> ⛔ **Do not implement IFC-by-typing.** It exists:
> `crates/ken-elaborator/src/ifc.rs` plus
> `crates/ken-elaborator/tests/sec1_acceptance.rs` — **711 lines, 20 tests,
> green** — and **all 16** seed cases in
> `conformance/security/ifc/seed-ifc.md` have implementing tests, name for name.
> Sec1ct (`sec1ct_acceptance.rs`, 9 tests) and Sec2 (`sec2_acceptance.rs`, 12
> tests) are built too.
>
> ⚠ **The first cut of this node asked for all of that to be written.** It was
> released to Team Verify and withdrawn minutes later. The error: I verified the
> frame existed, the node did not, the spec was implementation-ready and the deps
> were on `main` — then probed the crates with the wrong grep key
> (`noninterference|ifc_label`, zero hits) and read that as "unbuilt". The real
> spelling was `sec1_acceptance.rs`, one `find -iname '*sec1*'` away.
>
> ⭐⭐ **The upstream error is the one to keep: "a frame with no tracker node" has
> TWO causes** — never released, **or built and completed while the node was
> never filed.** ⛔ Node absence is *equally consistent with done.* Discriminate
> before releasing anything: **is there a landed acceptance suite whose name
> matches the frame's?**

## Objective

**Reify the three deferred capabilities the Sec1 source names itself** — the
`[Sec1-dual]`, `[Sec1-launder]`, and `[Sec1-reduce]` triggers at
`crates/ken-elaborator/src/ifc.rs:361–381`.

⭐⭐ **Why this is worth a WP rather than cleanup: two of the three are the SOLE
NETS for Sec1's two trusted surfaces, and both are currently placeholders under
a green suite.** The frame states that `N1`'s only protection is the flip cases
`{A1–A4, C1}` and `N2`'s only protection is `D5`. ⛔ **`C1` is label-equality
over hand-assigned literals and `D5` is a verdict-shape predicate over a
synthetic obligation.** ⇒ The guarantees the suite appears to establish are not
established, and nothing about the green run says so.

## ⭐ The unifying property — each stub is its own surface's discriminator

| trigger | what is stubbed | ⛔ what the stub CANNOT SEE |
|---|---|---|
| `[Sec1-dual]` | integrity is a **scalar** `integ: u8` (`ifc.rs:32`), with `UNTRUSTED = Label(2) = SECRET` and `TRUSTED = Label(0) = PUBLIC` — one axis doing two jobs | ⛔ **a bug specific to the `IntegLabel` ordering is indistinguishable from a `ConfLabel` bug.** A2 cannot flip independently of A1 |
| `[Sec1-launder]` | `check_no_laundering` compares **label equality over hand-assigned literals** | ⛔ **the actual trusted surface is never exercised** — `36 §2.2/§2.4`'s `bind (Vis e f) k = Vis e (λr.…)` index preservation. C1 is `N1`'s sole net and it does not route a `Vis` |
| `[Sec1-reduce]` | `check_reduction_faithfulness` is a **verdict-shape predicate** over a synthetic obligation; `product(c, ζ)` does not exist | ⛔ **the `N2` failure mode — a too-weak `Φ_post` — cannot be detected, because nothing constructs `Φ_post`.** D5 is `N2`'s sole net |

⇒ ⭐ **All three are the same defect in three places: a control named for a
property it cannot observe.** That is what makes them reifiable as one unit and
what makes each one's acceptance a *discrimination* requirement, not a feature
requirement.

## Fixed inputs (measured at `origin/main = fe543c93`; ⛔ re-derive at point of use)

| input | pin |
|---|---|
| production | `crates/ken-elaborator/src/ifc.rs` — triggers at **:361–381**, scalar `integ` at **:32**, doc at **:21** |
| tests | `crates/ken-elaborator/tests/sec1_acceptance.rs` (**20** tests, incl. `n1_n2_stub_gaps_carry_reify_triggers` which enumerates these gaps) |
| spec | `spec/60-security/61-information-flow.md` blob **`e6c91f50`** |
| seed | `conformance/security/ifc/seed-ifc.md` blob **`45160418`** (**16** Sec1 cases) |
| ITree contract | `spec/30-surface/36-effects.md §2.2`/`§2.4` (`bind`/`incl` reconstruct the same `Vis e`) |

⭐ **Prior fleet knowledge exists — read it, do not re-derive:**
`agent/memory/build/qa/taint-axis-orientation-needs-distinguishing-pair.md`
(`[Sec1-dual]`) and
`agent/memory/build/qa/composition-wp-real-producer-may-be-deferred-engine.md`
(`[Sec1-reduce]`).

## Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-R1` | `[Sec1-dual]` — integrity is a **separate carrier with a dual `⊑`**, giving a genuine `(Conf × Integ)` product lattice with lattice-**parametric** flow rules. | ⭐⭐ **A2 must flip while A1 stays green**, and vice versa. ⛔ An aggregate "labels still work" pass does not discharge this — the whole point is that the two axes become **independently** falsifiable. Both directions required |
| `AC-R2` | `[Sec1-launder]` — `check_no_laundering` is wired to the **real** `bind`/`incl`/`handler_fold` in `effects::itree`. | ⭐ **A real `Vis`-routed tree is the discriminant**: build a tree, route it through `bind`/`incl`, and show the label index survives. ⛔ Then **redden it** by perturbing the routing to drop the index — a label-equality assertion that passes without a `Vis` in play does not discharge this row |
| `AC-R3` | `[Sec1-reduce]` — `product(c, ζ)` exists (variable renaming, `lowEq_ζ`, the `coterminates_ζ` conjunct) and **D5 is tied to a genuine product-program reduction**. | ⭐⭐ **A too-weak `Φ_post` must be DETECTED.** Construct one deliberately and show D5 reports `disproved`/failure rather than a false `proved`. ⛔ A verdict-shape assertion cannot discharge this — the row exists because nothing currently builds `Φ_post` |
| `AC-R4` | The three triggers are **removed from the deferred set** exactly where they are reified, and any that remain still name a live gap. | `n1_n2_stub_gaps_carry_reify_triggers` (`sec1_acceptance.rs`) must be **updated, not deleted**: a reified capability moves out of the "NOT yet delivered" list into the delivered one. ⛔ Deleting the test would erase the fleet's only inventory of these gaps |
| `AC-R5` | No over-claim: whatever is **not** reified in this WP still carries a named, non-silent trigger (`§H`/LP-2). | ⛔ Silence is the failure mode this whole node exists to correct. If a trigger survives, it must survive **explicitly** |
| `AC-R6` | No regression in the landed Sec1/Sec1ct/Sec2 behaviour, and **no kernel enlargement** — labels stay `Vis` indices. | the three existing suites stay green (**20 + 9 + 12**); "no-regression" means **green in CI**, never a local `--workspace` run |

## Slicing — `AC-R3` is separable and may need `V3`

⭐ `AC-R1` + `AC-R2` are the **`N1`** surface and need **no prover** — they are
elaborator-local and independently releasable. `AC-R3` is the **`N2`** surface
and reaches the product-program/obligation path.

⚠ **`V3`'s delivered state is NOT verified in this node.** ⇒ If `AC-R3` turns
out to need prover work that is not present, **land `AC-R1`+`AC-R2` and re-raise
`AC-R3` to the Steward with what you measured** — ⛔ do not stub it further, and
⛔ do not infer V3's state from a DAG table or a tracker row.

## Scope

**IN:** `crates/ken-elaborator/src/ifc.rs`, its `effects::itree` wiring for
`AC-R2`, and the three suites' controls.

⛔ **OUT:**
- ⛔ **Re-implementing anything already green** — the four flow rules,
  `flows_to`, join/meet, declassification's capability gate / strictly-lower /
  delta-audit, the `@ct` hook. ⚠ They are **delivered**; the suite says so
  explicitly.
- ⛔ **The `@ct` discipline** → Sec1ct, which is **built**
  (`sec1ct_acceptance.rs`). ⚠ Two stale `ct_*` hook tests still sit in
  `sec1_acceptance.rs` although the seed moved F1/F2 to Sec1ct — that is a
  **currency drift, not a gap**; ⛔ do not "fix" it by deleting coverage. Report
  it and leave it.
- ⛔ Heavy value-dependent relational machinery → `[rel-deferred]`.
- ⛔ Sec2/Sec4/Sec5 (`62`/`64`/`65`).
- ⛔ **No kernel enlargement.** If this needs a kernel change, **stop and
  re-raise** — that is a finding about the spec's premise, not a licence.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). `-p
ken-elaborator`, plus `--test sec1_acceptance` / `--test sec1ct_acceptance` /
`--test sec2_acceptance`. Workspace, `--locked` and conformance run **in CI**.

⚠ **Read RAW first-run output** — a re-run is **not idempotent for error
reporting** and can report *fewer* failures than the first while nothing
changed. `tee` the first run and grep the file.

⚠ `ken-cargo` is a **single machine-wide `flock`, slots == 1.** Kernel and
Runtime are both active — coordinate a **seat-to-seat yield in-thread**, ⛔ never
by sampling `ps`.

## Clean-room

⛔ Copyleft security references (**jif, DCC, FaCT**) are **Spec-enclave-only —
never vendored, never consulted by the implementer** (`CLEAN-ROOM.md`).

## Reporting

Return exact SHA/tree/base, per-AC evidence, and specifically: **the
independent-flip evidence for `AC-R1` in both directions**, **the redden result
for `AC-R2`'s perturbed `Vis` routing**, and **the detected too-weak `Φ_post`
for `AC-R3`**. ⚠ Architect review is required regardless of diff scope — the
trust model is load-bearing. Security semantics → Spec; trust-model/TCB →
Architect.
