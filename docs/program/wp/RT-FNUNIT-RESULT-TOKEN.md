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

Measured at `origin/main = ddddb48d`.

| path | blob |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/surface.rs` | `99b9b5070e5c2780e73bfec6c4bac2a55764af40` |
| `crates/ken-runtime/src/cranelift_backend/artifact/api/tests.rs` | `f96a0b0bd1d7ed21ce228b9157346f89c1bb7f01` |

**Re-pin at pickup.** `RT-PRODUCER-MATCH-PORT` is in flight and rewrites this
region's neighbours. These are recorded so the derivation below can be checked
against what changed, not so the numbers can be trusted.

## 2. What is known, and how it was established

- The failing row is `nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes`
  (`artifact/api/tests.rs`), currently `#[ignore]`d with this node named as owner.
- The error is `BackendFailure::NativeResultDecode { token }`
  (`surface.rs:192`, rendered at `:251`/`:315`). **Its producers are five sites in
  `cranelift_backend/compiled.rs`** — `:135`, `:168`, `:194`, `:197`, `:200` —
  each an `ok_or_else` on a failed lookup.
- **The port is NOT the cause, and this was measured rather than argued.**
  Flipping `nc22`'s callee from `RuntimeExpr::Closure` to
  `RuntimeExpr::LexicalClosure` — an arm live since [[RT-DECL-CLOSURE-PORT]] and
  untouched by [[RT-SEED-CALL-PORT]]'s `D2`/`D3` — reproduces the **identical**
  error. The shape was already unsupported on the functionized lane.

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
- **`D2` — locate the gap.** Which of the five `compiled.rs` producers raises it
  for `nc22`, what token 265 denotes, and **whether the gap is the token's
  PRODUCTION or its REGISTRATION** — those route differently and the answer
  determines `D3`'s shape.
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
- **`AC-4` — the decode surface stays fail-closed.** A token genuinely absent
  from the table must still raise `NativeResultDecode` rather than being
  defaulted, silently mapped, or widened away. **Making the error disappear is
  the failure mode, not the fix.**

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
