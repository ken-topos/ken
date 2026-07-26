---
scope: fleet
audience: (see scope README) — anyone writing or reviewing evidence that a
  consumer really obeys a table, relation, plan, or registry; anyone authoring a
  ruling that prescribes perturbations
source: RT-FNSPLIT-B2V, 2026-07-26 — runtime-implementer retro, promoted on the
  explicit ground that TWO senior seats said independently they would have made
  the same misreading (Architect: "I would have read them as alternatives";
  Steward: "closes a hole I did not see in the ruling")
---

# Withdrawing a cell and RELOCATING it test different properties

`RULING R5` clause 5 required *remap **and** drop*. The implementer built only
drop, reading the two as **alternatives** — and so did the Architect and the
Steward on first reading. They are not alternatives:

| perturbation | what it proves | what it misses |
|---|---|---|
| **DROP** a cell | the authority can take a cell **away** | a consumer computing `hardcoded ∩ authority` passes **every** drop — the intersection just shrinks |
| **REMAP** a cell | the authority decides **where** cells are | — |
| **remove a whole ROW** | the fold's **seed** is reachable | both of the above leave the row present, so no fold ever reads its default |

★ **The mechanism: acceptance must MOVE to the recipient row, not merely
shrink.** A consumer that intersects the authority with its own hardcoded table
is *correct-looking and defective* — it honours every removal and ignores every
relocation. Drop cannot see that. Only remap can.

## ★★ And a third property nobody had named: the SEED

Dropping **cells** leaves the row present, so the fold still finds a row and
never evaluates its initial value. ⇒ **A fail-closed default that no perturbation
reaches is not a default — it is unexecuted code with a reassuring name.** Only
removing a row **entirely** reaches it.

That is what turns *"seeded with the empty mask, so an absent row fails closed"*
from an untestable claim about a branch into a **testable claim with a control**.

## The rule

⇒ **When you perturb an authority to prove a consumer honours it, enumerate the
perturbation KINDS, not the cells.** Three distinct ones, and coverage of one
says nothing about the others:

1. **withdraw** — can the authority take a cell away?
2. **relocate** — does the authority decide where cells are?
3. **remove the container** — is the fail-closed default reachable at all?

⛔ **And when a ruling says "X and Y", do not collapse it into "X or Y" because
they look like two ways of saying one thing.** Ask what a defective consumer
would do under each. If the answers differ, the ruling meant the conjunction.
Here three seats collapsed it independently, which makes it a **property of how
the pair reads**, not a lapse by the one who acted on it.

Sibling of [[a-pin-cannot-disagree-with-its-own-source]] — that entry is about a
pin whose operands are not independent; this one is about a pin whose operands
*are* independent but whose **perturbation set is too narrow to distinguish the
consumer that cheats.** Also
[[a-mutation-that-passes-when-it-should-fail-means-a-stale-input]] and
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]].
