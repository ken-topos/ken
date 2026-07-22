---
id: CB-HYGIENE
title: "cranelift_backend facade: strip WP-token narration, separate test material from implementation"
status: ready
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: operator directive 2026-07-22
---

**Operator directive (2026-07-22):**

> *"The file is an interleaving of operational comments that contain work
> package identifiers and the actual implementation. Remove the WP identifiers
> and either consolidate the test items, or — better — move them to a separate
> file."*

## ★ The measurement that reframes this

`crates/ken-runtime/src/cranelift_backend.rs` on `origin/main @ e7b2a8a5` is
**492 lines, of which ~366 (74%) are `#[cfg(test)]` fixture builders.**

```
lines 1–127     production facade — imports + re-exports          (~26%)
lines 128–492   4 top-level #[cfg(test)] fns + their fixtures     (~74%)
```

⚠ **This corrects a number I published.** I closed RT-SPLIT on *"22,081 → 492
lines."* The **production** facade is ~127 lines — *better* than advertised —
but 492 conflated the facade with the test material sharing its file, and I
reported the conflated figure as the achievement. The decomposition is not
diminished; the headline was imprecise, and the imprecision is exactly the
interleaving the operator is pointing at.

## Two classes — different rules, do not merge them

| class | where | repo rule | action |
|---|---|---|---|
| **A — process narration in comments** (`RT-SPLIT slice 7`, `RT-SPLIT §10.2`, `slice 5 — …facade wiring`) | production regions | ⛔ **forbidden** in non-test source | **delete or restate as mechanism** |
| **B — WP tokens in test fixture data** (`ctor:fixture::PX8J::Done`, `"PX8-TR test interface is canonical"`) | `#[cfg(test)]` only | ✅ **permitted in test code** | **moves with the test material; do not rename** |

★ **Class B is not a violation and must not be "fixed" by renaming.** The
standing rule (four prior operator flags) is that WP tokens are banned in
*production* source and **allowed in test code**. Those strings are fixture
identifiers; renaming them is a semantic edit to test data with assertion
consumers, i.e. real risk for zero rule-compliance gain. **Relocation alone
discharges them** — once the test items live in a test file, the tokens are in
a place the rule already permits.

⇒ **The structural move is not a nicety, it is what makes the token question
disappear.** That is why the operator's "better" option is better.

## Scope

**In scope — `crates/ken-runtime/src/cranelift_backend.rs` only.**

1. **Class-A removal.** The WP-narration comments in the production head —
   `:11` (`⛔ RT-SPLIT slice 7: once the nine internals moved…`), `:60`
   (`RT-SPLIT slice 5 — #[cfg(test)] facade wiring`), `:68` (`RT-SPLIT slice 7 —
   owner-adjacent test adapters (§10.5a′)`). Each states a **real invariant**
   in **process vocabulary**. Restate the invariant without the WP token, or
   delete it if the invariant is already enforced elsewhere. ⛔ **Do not delete
   the content silently** — a comment that survived a 95% rewrite is usually
   load-bearing.
2. **Structural separation.** The 4 top-level `#[cfg(test)]` fns and their
   fixtures leave the facade. **Placement is the ring's call** — a
   `#[cfg(test)] mod test_support` already exists in this tree; whether these
   join it, get their own module, or move to `tests/` is a design decision for
   @runtime-leader / @architect, not a Steward instruction.

**Out of scope — flagged, not fixed.** The wider tree has **78 WP-token hits
across 16 files**, concentrated in `lowering/core/tests/*` (46 of them). Those
are Class B in test files — **already compliant**. The tree-wide Class-A sweep
(`lowering/mod.rs`, `core.rs`, `planning.rs`, `compiled.rs`, `surface.rs`,
`artifact/*`) is a **separate, larger question** and is deliberately not bundled
here. Filed as a note; route it back to me if the ring judges it inseparable.

## Acceptance — the property, not the mechanism

1. **`cranelift_backend.rs` contains no `#[cfg(test)]` item.** Falsifiable by
   grep on the landed file.
2. **No WP token survives in any non-test region of the file** — emit the
   occurrence set against the **exact candidate**, classify every row as
   removed / restated / relocated-to-test, and make the **leaves as visible as
   the changes**. ⛔ A substitution count is not evidence.
3. **The relocated tests still run and still pass** — same test names, same
   count, no `#[ignore]` introduced. ⛔ **A green suite that lost a test is
   green.** Report the before/after test count explicitly; equality is the
   claim, not "green".
4. **Production behavior is byte-identical.** No non-test code path changes.
   This is a move + comment edit; anything else is out of scope and should come
   back to me rather than ride along.
5. **`cargo test -p ken-runtime` green** (targeted — ⛔ never `--workspace`),
   and CI green on the candidate.

★ **The mutation that would actually bind #3:** delete one relocated test and
confirm the count check fails. Test-count equality asserted without ever being
observed failing is the same unverified-oracle shape as a green-vs-green
comparison.

## Sequencing

**Queued behind the current front, ahead of `KW-THEOREM`.** Small, mechanical,
and it touches the tree Verify is about to enter for BUDGET-EFF's native half —
so it wants to land **either before that WP starts or after it merges**, not
concurrently. The native half edits `lowering/`, this edits the facade; they are
disjoint by path, but @runtime-implementer's RT-SPLIT carry (*"the re-anchor
loop has no natural termination when two contention-free tracks interleave"*) is
exactly about disjoint-but-concurrent candidates, and it applies here.

⇒ **@runtime-leader owns it; @steward sequences the start.**
