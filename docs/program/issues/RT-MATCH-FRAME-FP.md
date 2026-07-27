---
id: RT-MATCH-FRAME-FP
title: "match-frame fingerprints must hash a dedicated closure-free header carrier, not a Debug rendering of closure-capable cases"
status: active
owner: runtime
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: Re-sliced out of RT-VALUE-TOTALITY-P2 by the Steward (evt_35eggstm95hk1) as item 1 of the Architect's block on 6f2ca526. Mechanism ruled by the Architect as dec_16n1t4b92463g (route C), which also states "Steward owns framing and scope release". Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ▶ WHY THIS IS ITS OWN NODE
>
> **Re-sliced out of `RT-VALUE-TOTALITY-P2`, deliberately.** `P2`'s `D2` had
> already expanded four times; this is **production mechanism** work, not the
> `cfg(test)` repair `P2`'s remaining scope covers, and it depended on a mechanism
> ruling that did not exist yet. ⛔ Keeping it inside `P2` would have held a green
> 19-file body hostage to an open design question.
>
> ⭐ **`P2` is unaffected.** Its retained fixes are Architect-approved at
> `b7e358fc` and released for `D3`/`D4`. ⛔ Nothing here authorizes an edit in
> `P2`, and nothing in `P2` authorizes an edit here.

## The defect — an equality verdict over a closure-capable carrier

`compiler_private_ordinary_match_frame_fingerprint` and
`compiler_private_computational_match_frame_fingerprint` accept
`RuntimeMatchCase` / `RuntimeComputationalMatchCase`. Their `body` is an
**unrestricted `RuntimeExpr`**, which can contain `Closure`, `LexicalClosure`, or
`Value(ClosureRef)`. Both render the complete cases with `Debug`, hash those
bytes, and **production planner/lowering consumers compare the hashes as frame
identity**.

⛔ That is an equality verdict over a closure-capable carrier under another
spelling, and `spec/40-runtime/41-values.md §2.1` forbids it. ⚠ The
`compiler_private` name is **insufficient**: the binding ruling permits that
route only where the input type **excludes `ClosureRef` by construction**, or the
operation **refuses closure-containing input before any verdict**. These do
neither.

### ⭐⭐ Why the `P2` census could not see it — the reason this node exists

**`runtime-implementer`, verbatim:** *"My census measured **what stopped
compiling** when the derive came off. `format!("{cases:?}")` never stopped
compiling, so a Debug-hash equality route over the same closure-capable carrier
was invisible to it. ⇒ **A derive-removal census measures one spelling of a
capability, not the capability.**"*

⇒ ⛔ **A census keyed on a compile failure can only see capabilities the compiler
withdraws.** `Debug` + hash is the same verdict with no trait bound to break.

⚠ **And the tell, in the implementer's own words:** they saw
`fnv1a_64(format!("computational\0{cases:?}…"))` early, called it *"an
established idiom in this file,"* and did not follow it. ⭐ **A pattern's
familiarity is not evidence that it is permitted.**

## ⛔ THE MECHANISM IS RULED — route C, `dec_16n1t4b92463g`

⛔ **Not reopenable.** Transcribed from the ruling.

**Plain route B is insufficient.** A function whose parameter is
`&[RuntimeMatchCase]` still has a **closure-capable input**. ⭐ *"Merely choosing
not to read `body` today does not close the property: adding `body` to the format
later **compiles silently**. That is the exact class of blind spot this repair
must remove."*

**The required boundary** — the operation that serializes/hashes the frame must
accept a **dedicated closure-free carrier**. In substance:

```rust
struct OrdinaryFrameHeader<'a> {
    constructor: &'a RuntimeSymbol,
    binders: usize,
}

struct ComputationalFrameHeader<'a> {
    constructor: &'a RuntimeSymbol,
    argument_binders: usize,
    recursive_positions: &'a [usize],
}
```

The fingerprint **core** consumes only one of those ordered header sequences plus
the closure-free `RuntimeTrap`, with an **explicit ordinary/computational domain
separator**. ⛔ **No `RuntimeExpr`, `RuntimeValue`, case `body`, full case value,
or `Debug` rendering of any such value may reach that core.**

⭐ **A wrapper keeps the existing signature.** *"An existing-signature wrapper may
accept the full case slice solely to **project** each case into the typed header
carrier and then call the core. The wrapper must not itself serialize, hash, or
compare full cases."* ⇒ The type boundary is real **where the verdict is formed**,
without broad caller churn.

### ⚠ The semantic consequence, stated so nobody re-derives it as a bug

**Body-change staleness is deliberately NOT an invariant of this fingerprint.**
`site_id` plus the closure-free checked occurrence binding remains the
authoritative identity; this fingerprint checks only
**eliminator-family / header / default compatibility**. ⭐ *"A body change at the
same checked occurrence is program semantics, not a license to inspect forbidden
closure structure."*

⭐ **Identity is already bound by transported site identity** — measured by
`runtime-implementer`: in `planned_join_site_for_frame`, both `Computational` and
`Ordinary` **select** the site by `self.active_join_site` (`site.site_id ==
site_id`), **not** by the fingerprint. ⚠ Re-derive that at kickoff; if it holds,
the fingerprint may be **narrowable rather than merely re-carriered**.

## ⛔ SYMPTOM INVENTORY — Architect appends one line per hard-stop; never rewritten

**NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, …**
**NEXT RESEARCH PULL = 3rd hard-stop, then 6th, 9th, …**

⭐ Both counts are **armed lines, not tallies.** Re-read them on every hard-stop.
A deep chain carrying **zero** research advisories is itself the tell that both
the Architect's self-trigger and the Steward's backstop have lapsed
(`steward.md §5a`, measured on `RT-NATIVE-FNSPLIT` at **10**).

| # | date | the wall that was hit |
|---|---|---|
| 1 | 2026-07-27 | ⭐ **`AC-F1` and the inference selector want opposite things.** `AC-F1` requires body-only differences to share a header fingerprint; `lowering/mod.rs:4095–4106` **selects** from `callee_frame_templates` **by that fingerprint** when `checked_frame_id.is_none()`. Measured affirmatively by `runtime-leader`: one callee declaration with two computational eliminators of the same family yields **header-identical** templates (case fields derive from the family; the default trap is family-symbol keyed). Today's Debug hash distinguishes them **by body**; Route C must not. ⇒ Dominant new result is a lowering refusal, but a complete permutation can **silently exchange** header-identical templates, observable through fields absent from the header (`semantic_position`, `output_interface`, `segment_site_id`). Held at `evt_2qaj3kt3dawhr` for an Architect selector-scope ruling. ⚠ The join-site path stays identity-selected and is unaffected |

**Disposition of entry 1 — RULED, chain not continuing.** The Architect ruled
for an **identity-based selector**: eliminate the `checked_frame_id.is_none()`
fingerprint `find`, **preserve the pre-erasure checked ID through all internal
paths**, **reject a missing ID before CFG**, retain the fingerprint as
**compatibility-only**, and close the permutation gap with **exact
occurrence/order validation**. Both helper signatures stay stable, so the
signature-stability licence in `Contention` is unaffected. Released to the ring
at `evt_j1ajtszmhxt1`; the implementer folds on top of preserved `88980012`.

⭐ **Note what the ruling did to `AC-F1`:** it is no longer in tension, because
the fingerprint stopped being a *selector* and became a *compatibility check*.
⛔ That is the shape to remember if a later entry looks similar — the fix was to
narrow what the fingerprint is **used for**, not to weaken what it **hashes**.

⚠ **Entry 1 is a ruling-surface gap in this node's own narrowability premise** —
the node says the fingerprint *"may be narrowable rather than merely
re-carriered"* because `planned_join_site_for_frame` selects by `site_id`. That
is true **for the join-site path** and does **not** cover the inference selector.
⛔ Do not read the narrowability lead as settling the selector question.

## Acceptance criteria

`AC-F1`–`AC-F4` are the Architect's required controls, transcribed.

| AC | claim | control |
|---|---|---|
| `AC-F1` | Frames differing **only** in a closure-bearing body have **identical** header fingerprints. | build two frames whose `body` differs and contains a closure; the fingerprints must be equal. ⭐ This is the property that makes the carrier honest rather than merely renamed |
| `AC-F2` | **Each** load-bearing header field **and** the default trap **independently** changes the fingerprint. | one mutation per field, each fired separately. ⛔ Not an aggregate "the header matters" pass — a single field silently dropped from the hash would survive that |
| `AC-F3` | Ordinary and computational frames stay **domain-separated**. | an ordinary and a computational frame that coincide field-for-field must **not** collide |
| `AC-F4` | ⭐⭐ **The hash core's signature and carrier definitions contain no body-bearing type, and a mutation that feeds a full case/body CANNOT TYPE-CHECK at that boundary.** | attempt to pass a full case into the core → **must fail to compile**. ⛔ A runtime refusal does not discharge this row: the ruling requires the input type to exclude closures **by construction**, so the control is a compile failure, not an assertion |

⛔ **`AC-F4` is what discharges this node.** `AC-F1`–`AC-F3` prove the new
fingerprint is *correct*; `AC-F4` proves the forbidden route is *unreachable*.
⚠ A repair that greens `AC-F1`–`AC-F3` while leaving the core's parameter
closure-capable has rebuilt the same defect behind better tests.

## Scope

**IN:** the two `compiler_private_*_match_frame_fingerprint` helpers, the header
carriers, the core, the projecting wrapper, and their controls.

⛔ **OUT:**
- ⛔ **Any other whole-`RuntimeExpr` encoding, hash, or handwritten comparison** —
  Architect: *"Do not replace them with another whole-`RuntimeExpr` encoding,
  hash, or handwritten comparison."*
- ⛔ **Making lawful closure-bearing case bodies into rejected programs** merely
  to save the fingerprint. That refusal route is ruled out by the measured
  closure-bearing case body.
- ⛔ `b2ac_topology_digest` — Architect-ruled out of scope; it serializes flat
  topology fields, and its `RuntimeExpr` parameter is planner **input**, not the
  value rendered or compared.
- ⛔ `P2`'s `D3`/`D4`, and any public plan-format change.

## Contention — ⚠ the conclusion holds, but the ORIGINAL PREMISE WAS FALSE

> ### ⛔ CORRECTED 2026-07-27 — `ken-elaborator` IS a consumer. Corrected by
> ### `runtime-leader` (`evt_1cm96c2ce6vbn`), re-measured by the Steward.
>
> ⛔ **This section used to assert:** *"every consumer of the two helpers is in
> `ken-runtime` … **No `ken-elaborator` file among them**."* **That is false**,
> and the Steward repeated it in the kickoff (`evt_3hp0wxgtedd36`) labelled
> *"measured rather than assumed"* — which it was not; it was inherited from this
> node without re-derivation.
>
> **Re-measured across the whole workspace:**
>
> ```
> grep -rn 'match_frame_fingerprint' crates/ --include=*.rs
>   5  crates/ken-runtime/src/cranelift_backend/lowering/mod.rs
>   3  crates/ken-elaborator/src/erasure.rs        <-- the missed population
>   2  crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs
>   1  crates/ken-runtime/src/cranelift_backend/test_objects.rs
>   1  crates/ken-runtime/src/cranelift_backend/planning.rs
> ```

⭐ **The conclusion survives, for a reason that is narrower than the old one and
must not be widened back.** Route C keeps the **existing helper signatures**, and
an existing-signature wrapper leaves all three `erasure.rs` consumers **untouched**
— so *this* implementation needs no cross-ring authorization. ⛔ The licence comes
from **signature stability**, not from an absence of elaborator consumers.

⇒ ⛔ **Any variant that changes either helper's signature has three
`ken-elaborator` callers and REQUIRES cross-ring authorization. STOP and re-raise
to the Steward before such an edit** — that is a finding about the ruling's
premise, not a licence.

⚠ Kernel's `KERNEL-NESTED-IND` `D5` will also reach `ken-elaborator/src`, so a
signature-changing variant would contend with live kernel work as well.

### ⚠ Second open question — raised by `runtime-leader`, not yet ruled

`lowering/mod.rs:4095–4106` selects `callee_frame_templates` **by fingerprint**
when `checked_frame_id.is_none()`. ⭐ `AC-F1` **deliberately** makes body-only
differences collide, so if **header-identical callee templates are jointly
reachable**, that selection becomes ambiguous. The implementer is measuring
reachability; ⛔ **if reachable, route the evidence to the Architect for a scope
decision before extending the candidate** — do not resolve it inside this node.

⚠ Kernel's `KERNEL-NESTED-IND` `D5` will reach `ken-elaborator/src`. That is why
this section is measured rather than assumed.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). `-p
ken-runtime`, and `-p ken-verify` as the built-artifact oracle.

⚠ **Read RAW first-run output.** ⛔ A `cargo`/`ken-cargo` **re-run is not
idempotent for error reporting** — a second invocation can report *fewer*
failures than the first while nothing changed, because diagnostics are not
replayed from cache. Measured on `P2`: a re-run reported **2** sites where there
were **4**. ⭐ If you must filter, `tee` the first run and grep the file.

⚠ `ken-cargo` is a **single machine-wide `flock`, slots == 1.** Another ring
building means you wait, legitimately.

## ⛔ Before you start — the frame-authoring question this node exists to answer

⭐ When withdrawing a capability, ask: **what spellings of this capability do NOT
go through the mechanism I am removing?** `Debug`, `Display`, serde, and hash are
the usual escapes, and none of them break when a derive comes off.
