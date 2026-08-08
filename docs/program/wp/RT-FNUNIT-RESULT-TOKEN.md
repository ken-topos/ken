# RT-FNUNIT-RESULT-TOKEN — a broad starter shape fails result-token decode on the functionized lane

**A composite starter program reaching `FunctionizedUnits` fails with `native
result token 265 is not in the result table`. The wall predates the seed-closure
port and was masked by it; retiring `SeedClosureCall` made it reachable. This
node makes the shape work on the functionized lane.**

**Owner:** Team Runtime. **Branch:** `wp/RT-FNUNIT-RESULT-TOKEN`.
**Size:** M — **provisional, and §3 may overturn it before any code is written.**
**Risk:** medium — the failure is in result decoding, which every native return
crosses.

**Read `docs/program/16-recursive-descent-retirement.md` first.** This node exists
because of that campaign's Trap 2, and the frame does not repeat the traps.

---

## 1. Fixed inputs

Measured at `ddddb48d`, **re-verified at `origin/main = 464cb446`
(2026-08-08): all three blobs are unchanged between those two commits.**

| path | blob |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/surface.rs` | `99b9b507` |
| `crates/ken-runtime/src/cranelift_backend/artifact/api/tests.rs` | `f96a0b0b` |
| `crates/ken-runtime/src/cranelift_backend/compiled.rs` | `31e5c149` |

`RT-PRODUCER-MATCH-PORT` was in flight when this frame was written and has
since **merged**; it did not touch any of the three.

**Re-pin at pickup anyway.** These are recorded so the derivation below can be
checked against what changed — not so the numbers can be trusted.

## 2. What is known, and how it was established

- The failing row is `nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes`
  (`artifact/api/tests.rs`), currently `#[ignore]`d with this node named as owner.
- The error is `BackendFailure::NativeResultDecode { token }`, declared in
  `surface.rs` and rendered there by a single `write!`.
- **The port is NOT the cause, and this was measured rather than argued.**
  Flipping `nc22`'s callee from `RuntimeExpr::Closure` to
  `RuntimeExpr::LexicalClosure` — an arm live since [[RT-DECL-CLOSURE-PORT]] and
  untouched by [[RT-SEED-CALL-PORT]]'s `D2`/`D3` — reproduces the **identical**
  error. The shape was already unsupported on the functionized lane.

> #### CORRECTED 2026-08-08 — this frame previously named FIVE producers and the
> #### enumeration was wrong when written, not stale
>
> It said: *"Its producers are five sites in `cranelift_backend/compiled.rs` —
> `:135`, `:168`, `:194`, `:197`, `:200` — each an `ok_or_else` on a failed
> lookup."* **`compiled.rs` raises it at eight sites, and the blob is identical
> at `ddddb48d` and `464cb446`** — nothing moved; the count was simply short.
>
> **The omission is not arbitrary. The one site the error message actually
> names — the `Table` arm's `result_table.get(&token)` miss, the only literal
> result-table lookup of the eight — is among the three that were missing.**
>
> ```sh
> git grep -n 'NativeResultDecode' -- crates/ken-runtime/src/cranelift_backend/compiled.rs
> ```
>
> **The authoritative arm-by-arm table is [[RT-WORKER-FIXTURE-DECODE]] §1e, and
> it is not repeated here** — that frame had all eight correct on 2026-08-07,
> a day before this one contradicted it with five. One table, one place to
> maintain. Read §1e, then come back.
>
> Two consequences, both binding on `D2`:
>
> 1. **"each an `ok_or_else` on a failed lookup" is false for two of the eight.**
>    An unconditional `return Err` on a reached path is not a lookup miss — it
>    says the decoder was *selected* wrongly, or the artifact declared it returns
>    only a trap. Those route to **production**, not registration, so the false
>    uniformity biased `D2`'s own discrimination toward one of its two answers.
> 2. **The message does not localize the fault.** One variant, one `write!`, eight
>    raise sites — *every* one of them prints `native result token N is not in the
>    result table`, and that wording is literally accurate for exactly one. **Do
>    not infer the site from the text.** Discriminate by which `ResultDecoder`
>    arm was selected for `nc22`.

**Discounted evidence, recorded so nobody re-counts it:** an earlier smaller
record-returning probe failed on both arms with a *different* error
(`BoundaryCarrier` unsupported). It does not attribute this stop.

## 3. FIRST DELIVERABLE IS A SCOPING ANSWER, AND IT MAY RESIZE OR RECUT THIS NODE

**Do not start repairing before `D1` answers this.**

**Measured on the closing merge:** `nc22` is a **single composite program**, not a
loop over shapes — one nested `Let` / `Call{callee: Closure}` / `Match` /
`Construct` / `Record` / `If` tree, where "broad starter shapes" names breadth
*within* one program. And **it is the only one of 21 `nc` fixtures carrying a
`Call` whose callee is a `Closure` or `LexicalClosure`.**

⇒ **Two consequences, both binding:**

1. **Family width is UNESTABLISHABLE from this corpus.** It holds exactly one
   instance of the failing shape. Answering "one shape or a family?" **requires
   authoring fixtures that do not exist.**
2. **The corpus currently has ZERO live coverage of this shape in either
   direction.** Nothing in it will observe the wall move — **not a repair, and
   not a regression.** Un-skipping `nc22` is the only thing that restores an
   oracle.

**So the sizing question is scoping, not measurement.** `M` was set for a repair
against one known fixture. **If authoring the missing coverage belongs in this
node, `M` is wrong — report that and it comes back to the Steward for a re-cut.**
Do not silently absorb it.

## 4. Deliverables

- **`D1` — the scoping answer, and it gates everything else.** Establish whether
  the failure is one shape or a family, and state **what authoring that answer
  cost or would cost.** Report before building. **A `D1` that concludes the node
  is mis-sized is a success.**
- **`D2` — locate the gap.** Which of the **eight** `compiled.rs` producers
  raises it for `nc22`, and **whether the gap is the token's PRODUCTION or its
  REGISTRATION** — those route differently and the answer determines `D3`'s
  shape.

  **Establish the site by observation, not by the message** — see the correction
  in §2: all eight render the same text. Name which `ResultDecoder` arm `nc22`
  selects and say how you determined it; [[RT-WORKER-FIXTURE-DECODE]] §1e lists
  the decoder-selection sites to look at. **If the arm is `TrapOnly` or an
  unrecognized `Boundary` tag, the fault is upstream of the decode table
  entirely** and §7's hard stop is likely live.

  **Do not re-derive what 265 "denotes" as an open question.**
  [[RT-WORKER-FIXTURE-DECODE]] §1d settles the general form: `token` is the
  **native return value** — literally what the compiled code returned — not an
  error code, an arm tag, or an index into anything. So `265` carries no
  information until you have named the selected decoder. **An inference drawn
  from the numeral before that point is unfounded**, which is the trap this
  frame walked into by calling the message a result-table fact.
- **`D3` — the repair.** Cut against `D2`'s finding.
- **`D4` — un-skip `nc22` and prove it green on the functionized lane.**
  **This node closes on the row running, not on the skip being tidied.**

## 5. Acceptance criteria

- **`AC-1` — `nc22` runs green on `FunctionizedUnits`**, with its `#[ignore]` and
  the owner reference removed. **Seen to fail before it passes** — this row has
  been dark, so a green with no demonstrated red is not evidence the repair did
  anything.
- **`AC-2` — the coverage gap is closed or explicitly reported.** If `D1` found a
  family, every member is covered or named with its measured cause. **A repair
  that fixes `nc22` alone while a family exists must say so.**
- **`AC-3` (no-regression).** Workspace green **in CI** — never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-4` — the decode surface stays fail-closed, on the arm the repair
  actually touches.** A value the decoder genuinely cannot interpret must still
  raise `NativeResultDecode` rather than being defaulted, silently mapped, or
  widened away. **Making the error disappear is the failure mode, not the fix.**

  **State the guard on the path `nc22`'s value takes**, named by the
  `ResultDecoder` arm `D2` identified — not on the table lookup by default.
  Seven of the eight producers are not table lookups, so an `AC-4` discharged
  against `result_table` **passes vacuously** for a repair that widened the
  `Boundary` `_ =>` arm or the `TrapOnly` arm. Those two are the dangerous ones:
  each is an unconditional refusal on a reached path, and the cheap way to make
  either stop firing is to accept what it was built to reject.

## 6. Banned scope

- **Adjusting `nc22` to pass** — narrowing its assertions, changing its shape, or
  re-routing it off the functionized lane.
- **Weakening the `NativeResultDecode` refusal.** See `AC-4`.
- **Retiring any residual class or touching the selector or the
  `RecursiveDescent` lane** — those are the campaign's nodes.
- **Absorbing a re-scope.** If `D1` says the node is mis-sized, that is a Steward
  recut, not something to work through.

## 7. Hard stop

Stop and return the seam if the repair requires changing what a native result
token *is* for callers other than this shape, or if `D2` finds the gap is in
token **production** inside emitted code rather than in the decode table — that
is a different layer and likely a different node.

## 8. Why this blocks `RT-DESCENT-RETIRE`

[[RT-DESCENT-RETIRE]] **deletes the `RecursiveDescent` emission lane.** This shape
is currently supported only there. If it is still unsupported on the functionized
lane when that deletion lands, **the retirement silently narrows what Ken can
compile** — and with `nc22` skipped, no row in the corpus would report it.
