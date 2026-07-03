# C3 — codata / `cofix` under the totality guarantee

**Axis:** coinduction under totality. **Flavor:** A (soundness-boundary — the
unsound arm must REJECT; acceptance = a totality hole).

## Why this is a blind spot

Ken serves genuine infinitude with **inductive, total** idioms — a fuel-bounded
unfold produces a finite prefix and terminates by structural descent on the
fuel. It has **no coinductive type former**: no `codata` declaration, no
greatest-fixpoint former, no `cofix`/copattern term former, and no productivity/
guardedness checker. The **only** structural admission gates are strict
positivity (for inductives) and the SCT termination measure (for recursion)
(`37 §7`, `14 §8`, `17 §4`). VAL2 *parked* the codata-vs-totality probe
(`continued-fraction`) — this reaches it.

## The pair

- **Sound arm — `sound-fuel-unfold.ken` — should-PASS.** `unfoldUpTo`, the
  fuel-bounded unfold (recurses on the `Nat` fuel, SCT-accepted), applied to a
  concrete step and fuel `n`, reduces to a finite `List` prefix of length ≤ `n`.
  An ordinary total `List`-producing function; no coinductive value, no `Lazy`,
  no effect. (Grounded: landed in `l3a_acceptance.rs`.)
- **Unsound arm — `unsound-codata-stream.ken` — should-REJECT.** A `codata
  Stream a = SCons a (Stream a)` former with a `cofix`/self-referential
  `nats = SCons 0 (map suc nats)` productive-non-terminating definition — an
  infinite value that never bottoms out on an inductive measure.

## Expected behavior (exact)

- Sound arm: **PASS** — `unfoldUpTo step n seed` elaborates, SCT-accepts (fuel
  descent), and reduces to a concrete finite prefix.
- Unsound arm: **should-REJECT** — Ken has no `codata`/`cofix` former, so the
  expected result is a **parse / elaboration rejection** ("unknown
  construct" / no such former). The reject is *by absence of the construct*,
  which is the correct enforcement of "infinitude without coinduction". **If a
  `codata`/`cofix` former is accepted and admits a non-guarded, non-fuelled
  self-reference, that is a totality hole** (a productive-corecursive value with
  no termination/guardedness gate breaks the total-by-construction guarantee).

## Discriminates

Is there **any** surface path to an unguarded/unfuelled infinite value? Fuel-
bounded unfold (total, PASS) vs `codata`/`cofix` (no former, must REJECT) is the
flip. A single "the unfold works" case is green-vs-green; the codata attempt is
the guard that pins the *absence* of a coinductive former.

## Surface-expressibility note

Because Ken has no `codata`/`cofix` keyword, the unsound arm is expected to fail
at **parse** ("unknown keyword `codata`") rather than a semantic check — that is
still the confirming result (no coinductive former exists). If a future surface
adds any greatest-fixpoint/guarded-recursion form, this exercise is the tripwire
that it did **not** silently weaken totality.
