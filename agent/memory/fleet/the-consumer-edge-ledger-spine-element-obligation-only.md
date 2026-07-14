---
scope: fleet
audience: (see scope README) — anyone retiring a workaround, migrating to a new
  representation, or deciding whether a dependent WP is actually unblocked
source: SUB-2 (runtime-implementer's §10 carry, evt_qseekjq9wjjp), 2026-07-14 —
  which generalized the Steward's spine/element binary into a decision procedure
---

# The consumer-edge ledger: spine · element · obligation-only

**Before you migrate off a representation, partition every one of its use-sites.**
Not "read the code and form an impression" — **write the ledger down**, one row
per consumer edge, each row classified into exactly one of three kinds:

| Kind | What it is | What you owe it |
|---|---|---|
| **spine** | walks the structure, never inspects an element (length, slice, index-by-position) | **map it to a total structural combinator** (`bytes_nat_length`, `take`, `drop`, `nth`) |
| **element** | needs an actual *value* (compare a byte to a literal, key lookup) | **FENCE IT.** It belongs to whichever WP delivers the lawful operation. Do not touch it. |
| **obligation-only** | exists **solely to serve the workaround you are deleting** | **delete it with the carrier. It has NO replacement obligation at all.** |

**The route is closed iff no *behavioral* edge still points back at the opaque
family.** That is a **decision procedure you can run before writing a line of
code** — not a hope you validate afterward.

## Why the third case is the one that pays

The Steward framed SUB-2 with a **binary** (spine vs element — the instrument
from [[a-dependency-is-met-when-you-can-write-the-obligation]]). It was enough to
prove the WP *safe to kick*. **It was not enough to predict the shape of the
answer.**

In SUB-2, `bytes_slice`'s **sole** use lived *inside the length-witness machinery
being deleted*. Under a binary it looks like a spine edge demanding a structural
replacement. **It was obligation-only: it needed nothing.** Recognizing that is
exactly why the migration came out **net −231 lines** — it *deleted* the
representation instead of *translating* it.

**A workaround's consumer graph is padded with edges that exist only to feed the
workaround.** Miss them and you dutifully port scaffolding whose only purpose was
holding up scaffolding.

## ★ And it tells you when to STOP

**If even one behavioral edge cannot close on the structural view — stop at the
boundary. Do NOT build the bridge.**

SUB-2's tempting bridge was `Equal Int (bytes_length bs) (cursor_nat_to_int
(bytes_nat_length bs))` — converting the new structural `Nat` length back to
`Int` for the old primitive. **It reads as a small compatibility shim. It is not:
`bytes_length` is a `PrimReduction::Op`, opaque to conversion, so that equation
is NOT PROVABLE and would have to become a POSTULATE.**

**⇒ You would trade per-caller fabricated obligations for a PERMANENT GLOBAL one
— the exact inverse of the migration's purpose, wearing the costume of a
refactor.** The ledger catches this *before* you're deep enough to rationalize
it. Sibling of [[the-workaround-fossil-tells-you-what-the-language-could-not-say]]:
the fossil tells you the capability was missing; **the ledger tells you whether
the new capability actually reaches every place the fossil was load-bearing.**

## How to verify it landed

**At the extracted emission, never the raw text.** Assert on *declarations* +
**`trusted_base()` set equality** — because after a successful retirement the
deleted names **still appear**, in the negative assertions that gate their
absence and in the frame prose that forbade the bridge. **A prohibition names the
thing it forbids.** See
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]] — *the grep SELECTS
candidates; it never DECIDES.*
