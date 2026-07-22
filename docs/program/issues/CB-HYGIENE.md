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

## ⛔ AMENDMENT — AC #2 vs AC #4 were CONTRADICTORY. Adversary `evt_3jx7dwph09qcg`

**My acceptance criteria as first written were unsatisfiable.** AC #2 said *"no
WP token in any non-test region"*; AC #4 said *"move + comment edit only,
production behavior byte-identical."* Four rows in `:1-127` are WP-tokened
**production** and dischargeable only by a **rename** — so the implementer had
to violate one AC or the other. Every claim below re-derived by the Steward on
`origin/main @ eba56ab3`, cfg-aware, not on report.

### ✅ NEW CLASS C — "left in place, must not change." PRE-AUTHORIZED

**These four rows stay exactly as they are. Renaming them is out of scope and
is a rejectable change.** They are recorded here so the leave is *pre-approved
in the brief*, not argued for by the implementer against an AC that reads
absolute.

| line | row | why it stays |
|---|---|---|
| `:51` | `run_nc6_seed_examples` (unconditional `pub use`) | production API rename — forbidden by AC #4 |
| `:52` | `run_nc8_validated_seed_examples` (unconditional `pub use`) | production API rename — forbidden by AC #4 |
| `:90` | `#[cfg(feature = "px8-ds-test-support")]` | **Cargo feature** rename — cross-crate contract |
| `:91` | `pub use lowering::with_px8ds_retired_flat_order;` | cross-crate consumer, see below |

⇒ **AC #2 is hereby scoped to Class A (comments) only.** Identifiers, exported
names, and feature names in production are **Class C** and out of scope.

### ★★ `:90-91` IS THE DANGEROUS ROW — and AC #5's gate is blind to it

```
crates/ken-runtime/Cargo.toml:13   px8-ds-test-support = []     <- default OFF, no `default` key
crates/ken-cli/Cargo.toml:25       features = ["px8-ds-test-support"]
sole consumer                      ken-cli/tests/px8ta_oriented_subcontinuation.rs:272
```

**`ken-cargo test -p ken-runtime` — AC #5's named gate — never compiles
`:90-91` at all.** A change there is **green locally** and surfaces only in CI,
in a **different crate**, in a **test**. ⛔ **The implementer's entire iteration
loop is blind to the one row most able to break something.**

★ `:90-91` is also **neither Class A nor Class B** — it is production code,
under a feature literally named `test-support`, whose only consumer is a test in
another crate. It sits on the **cfg-condition axis**, which my two-class
taxonomy did not enumerate over. A careful implementer could have classified it
either way and **one of those ways is destructive**.

### ⛔ `:86-89` IS LOAD-BEARING — OUT OF SCOPE FOR TRIMMING

The comment at `:86-89` explains precisely this blind spot — that
`with_px8ds_retired_flat_order` is reached cross-crate and *"neither
`-p ken-runtime` build config can observe the break — only the consumer can."*

**It sits four lines below Class-A narration at `:68` that this WP authorizes
removing.** ⇒ **The most dangerous row in this WP was guarded only by prose the
same WP was licensed to delete.** It stays. This is @adversary's slice-5
principle at its terminus — *"a fix protects one commit; that comment protects
the next four"* — and it already survived one 95% rewrite.

### Corrected classification vocabulary

Rows classify as **removed / restated / relocated / left-in-place-with-reason**.
The fourth category was missing, so the honest answer was unrepresentable and a
reviewer would have seen a classification that *looked* complete.

### ⚠ Correction to my own "~127 lines" figure

That number is **itself cfg-conflated** — a substantial share of `:1-127` is
`#[cfg(test)] use`. The real production surface is **smaller still**. An error
in the safe direction, but the same conflation AC #2 would have inherited.
**Define "non-test region" cfg-aware, never by line range.**
