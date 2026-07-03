# KNOWN-GAP: GAP-no-mutable-state — no `[State]` effect or mutable
# reference primitive exists

## What's missing

`Accumulator factory` needs a closure that holds a running total as
HIDDEN mutable state, persisting and mutating across independent calls
with no explicit threading by the caller. Ken has neither a mutable
reference primitive nor a `[State]` effect anywhere — confirmed by
inspection (no `Ref`/`StateOp`/`"State"` effect-row entry in
`ken-elaborator`, unlike the landed `Console`/`[FS]`/`[Net]` effect rows).

## What's NOT the gap

A pure, EXPLICITLY-THREADED accumulator (`next : Nat -> Acc -> (Nat,
Acc)`, the caller passing the updated `Acc` back each call) is entirely
expressible today with existing machinery (`Prod`, structural recursion).
That is a genuinely different program from what this task asks for —
explicit threading is precisely the "no hidden state" alternative this
task is meant to contrast against, not a workaround for it. Building the
threaded version and calling it "an accumulator factory" would
misrepresent what the task probes (QA's "idiomatic" concern) — a real
factory returns closures that behave identically to any caller with no
visible state parameter, which is exactly what's missing.

## Impact

Any task needing genuine mutable/hidden state (closures over a mutable
cell, memoization with a shared cache, an object-like counter) has no
path today.

## Fix needed (capability, not a Language-lane workaround)

A `[State]` effect (or a primitive mutable-reference type) — a
significant capability addition, not a small primitive-wiring gap like
`Map`'s missing ops or `read_bytes`'s missing reduction. Routed to
language-leader / Steward as its own capability WP; likely a larger
design question (effect system extension) than the `Map`/`FS` gaps.

## Intended program (once resolved)

```
view mkAccumulator (init : Nat) : <closure over a State cell> =
  \n . <read cell, add n, write cell, return new total>
```
The exact shape depends on the eventual `[State]` effect's design (not
pinned here).
