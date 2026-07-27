---
id: SEC1-IFC
title: "IFC by typing — the label lattice, the flow-typing pass, and no-laundering at the Vis boundary (WS-Sec increment 1)"
status: ready
owner: verify
size: M
gate: G-Sec
depends_on: []
blocks: []
github: null
origin: DAG WP Sec1 (docs/program/05-implementation-dag.md). Frame docs/program/wp/Sec1-build.md, authored by the Steward and re-pinned 2026-07-27 after spec 61 drifted under it. Owner is operator-decided — WS-Sec build is a scope extension of Team Verify, not a new team. Spec 61 elaborated impl-ready by the spec enclave; conformance seed landed.
---

> ## ▶ WHY THIS NODE EXISTS AT ALL — read this, it is the finding
>
> ⛔ **Six WS-Sec frames have existed in `docs/program/wp/` for weeks with zero
> tracker nodes** — `Sec1-build`, `Sec1-ifc-by-typing`, `Sec1ct-constant-time`,
> `Sec2-capabilities`, `Sec4-trust-model`, `Sec5-policy`. The spec was
> implementation-ready, the conformance seed was landed, the owning team was
> operator-decided, and **every dependency was on `main`** — and none of it was
> releasable, because **nothing was `ready`, because there was no node.**
>
> ⭐ **The release mechanism reads `issues/`, not `wp/`.** A frame without a node
> is invisible to it. Team Verify sat idle holding zero ready nodes while its
> next WP was fully written and sitting one directory over.
>
> ⭐ **`blocks:` is empty above for the same reason, and that is a symptom, not a
> choice.** This node's real successors are **Sec1ct** (`@ct` timing) and
> **Sec5** (policy-as-code), whose frames exist at `wp/Sec1ct-constant-time.md`
> and `wp/Sec5-policy.md` — but ⛔ **neither has a node to reference**, so the
> schema check correctly refused the edge. ⇒ The downstream is recorded in prose
> here until those nodes are filed. ⚠ **Do not read `blocks: []` as "nothing
> depends on this"** — two tier-1 WPs do. ⇒ ⛔ **A written
> frame is not a released WP.** File the node when you write the frame.

## Objective

Implement **IFC-by-typing** in `ken-elaborator` per landed `61`: the
lattice-parametric label interface + its DLM instance, the flow-typing pass, and
the **no-laundering** guarantee at the `Vis` boundary — then make the Sec1 half
of `conformance/security/ifc/seed-ifc.md` pass.

⛔ **This node is increment 1 of 2** and carries **`N1` — the by-typing trusted
surface — only.** `N2` (the by-proof product-program reduction) is increment 2
and is **`not-ready`**: see the frame's §Slicing.

## Fixed inputs (measured at `origin/main = 4be827eb`; ⛔ re-derive at point of use)

| input | pin |
|---|---|
| frame | `docs/program/wp/Sec1-build.md` |
| spec | `spec/60-security/61-information-flow.md` blob **`e6c91f50`** |
| conformance seed | `conformance/security/ifc/seed-ifc.md` blob **`45160418`** (**16** Sec1 cases) |
| L5 (ITree denotation) | **built** — `ken-elaborator/src/effects/{lower,extract}.rs`, `capabilities.rs` |
| K1.5 (kernel admission) | `f037451`, **verified ancestor of `origin/main`** |

⚠ **Settled — do not reopen:** `OQ-ifc` DECIDED (lattice-parametric + DLM);
`OQ-relational` DECIDED (by-proof = re-checked product programs,
progress-sensitive; heavy machinery deferred); `@ct` = an opt-in label whose
timing guarantee is delegated to `Ward`.

## ⛔ The trusted surface this node carries — `N1`

**IFC labels are erased before the kernel** (`61 §3`: "at the kernel it *is*
`A`"). ⇒ A flow-typing bug — a wrong `⊑` in `L-SINK`, a dropped `pc`-join, a
label-dropping `bind`/`incl` — emits a **well-typed core term the kernel
accepts** while non-interference is violated. ⛔ **The kernel is blind to it.**

⭐ **The sole net is the `§H` meta-theorem plus the discriminating flip cases
`{A1–A4, C1}`** — five cases, and never the kernel. ⇒ Treat the flow pass with
trust-root discipline: each flip case must genuinely **redden** under its exact
named bug, not pass green-vs-green.

⚠ **`F1` used to be in this net and is not available to it** — the seed moved F1
to Sec1ct. ⛔ Do not substitute a Sec1ct case into a Sec1 trust-root net.

## Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-S1` | The `§2` lattice interface (carrier + `⊑`/`⊔`/`⊓`/`⊥`/`⊤` + laws-as-Ω-obligations) and the **DLM instance** are lattice-**parametric**: confidentiality = reader-sets by reverse inclusion (`⊔ = ∩`), integrity the order-dual, products componentwise. | instantiate the interface **twice** (DLM + one non-DLM lattice) and run the same flow-typing case through both. ⛔ A single-instance pass cannot distinguish parametric from hardcoded |
| `AC-S2` | The four `§3` rules `L-PURE`/`L-COMBINE`/`L-OBSERVE`/`L-SINK` fire, with `L-SINK` joining `pc` (`(ℓ ⊔ pc) ⊑ κ`). | seed A1–A4, B1–B3 accept/reject. ⛔ Test at **non-empty Γ** and **non-degenerate labels** — an all-`⊥`/all-`⊤` fixture makes `⊑` vacuous and passes for any rule |
| `AC-S3` | ⭐⭐ **No laundering through effects:** the label rides the `Vis` op/resp, and `bind`/`incl` reconstructing the **same** `Vis e` node **preserve** the index. | seed C1, which must **redden** under a label-dropping `bind`/`incl`/handler at the `Vis` boundary. ⛔ This is the case `N1` is most blind to — the kernel accepts the laundered term |
| `AC-S4` | Labels are **erased** before the kernel and add **no kernel former and no new level rule**; the `ℓ_carrier ≤ ℓ_ITree` side-condition on the parametric `Lattice` holds. | assert the **erased core shape**, not merely that elaboration succeeded; plus seed E1's **forged-label** half — a forged label is kernel-rejected |
| `AC-S5` | Honest limits are surfaced, not silent: the four-way status shows the termination-(in)sensitivity choice, and the deferred machinery carries its `[rel-deferred]` trigger. | seed G1, G2. ⛔ A deferred capability is **named with its trigger**, never faked and never silently omitted |
| `AC-S6` | The flow-rule and lattice-op dispatch is **exhaustive by construction** (COORDINATION §7) — a single no-`_ =>` match, so a new rule or lattice case is a **compile error**. | add a variant locally and confirm it **fails to compile**. ⛔ A runtime `todo!()`/fallback does not discharge this |

⛔ **`AC4` of the frame is deliberately absent here.** Sec1's `@ct` AC was
elaborated into the Sec1ct discipline; F1/F2 now live in
`conformance/security/ct/seed-ct.md` as `CT-A1/A2/A3`, `CT-A4`, `CT-E1` under
Sec1ct's **own** `AC1`–`AC7` namespace. ⇒ **Discharge nothing `@ct` here.**

## Scope

**IN:** `§2` lattice + DLM · `§3` flow-typing (four rules, explicit `pc`) ·
`§3.2` no-laundering at `Vis` · `§H`/`§9` honest limits · seed E1's
**forged-label** half.

⛔ **OUT:**
- ⛔ **The entire `@ct` discipline — label, `L-CT-SINK`, the sealed `LeakSink`
  set, the CT-promise/`P` export, declassify-ends-span** → `[Sec1ct]`. **None of
  `§5a` is this node's.** ⚠ The frame's §The `@ct` boundary explains why this
  changed; a spec section was re-elaborated under the frame and its **title**
  had been carrying the boundary.
- ⛔ **`§5.3` by-proof / product programs** (seed D1–D5, E1's cert half) →
  increment 2, which needs `V3` and is **`not-ready`**.
- ⛔ Heavy value-dependent relational machinery (seed D3/D4) → `[rel-deferred]`.
- ⛔ Authority/capabilities (`62`, Sec2) · policy-as-code (`65`) · supply-chain
  (`63`).
- ⛔ **No kernel enlargement.** Labels are `Vis` indices; if this node needs a
  kernel change, **stop and re-raise** — that is a finding about the spec's
  premise, not a licence.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). `-p
ken-elaborator`, plus `--test` for the named suites. The full-workspace build,
the `--locked` gate, and the conformance suite run **in CI on GitHub**; the
"no-regression" criterion means **green in CI**, never a local workspace run.

⚠ **Read RAW first-run output** — a `cargo`/`ken-cargo` re-run is **not
idempotent for error reporting**; a second invocation can report *fewer*
failures than the first while nothing changed. `tee` the first run and grep the
file.

⚠ `ken-cargo` is a **single machine-wide `flock`, slots == 1.** Another ring
building means you wait, legitimately — coordinate a **seat-to-seat yield** in
thread rather than sampling `ps` for the lock.

## Clean-room

⛔ Copyleft security references (**jif, DCC, FaCT**) are **Spec-enclave-only —
never vendored, never consulted by the implementer** (`CLEAN-ROOM.md`). Build
from landed `61`/`36` and first principles.

## Reporting

Return exact SHA/tree/base, per-AC evidence, **the redden result for each of the
five `{A1–A4, C1}` flip cases**, the `AC-S6` compile-failure result, and branch
freedom. Security semantics → Spec; trust-model/TCB → Architect. ⚠ The trust
model is load-bearing, so **Architect review is required regardless of diff
scope**.
