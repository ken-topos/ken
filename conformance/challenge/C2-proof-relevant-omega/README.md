# C2 — a proof-relevant inductive declared at `Ω`

**Axis:** the Ω / strict-prop boundary. **Flavor:** A (soundness-boundary — the
unsound arm must REJECT; acceptance = a consistency hole). **Couples with C5**
(C5's `Perm` must live at the universe this exercise establishes).

## Why this is a blind spot

`Ω` (SProp / `Prop`) is **definitionally proof-irrelevant**: any two inhabitants
of an Ω-proposition are equal by fiat. So only **sub-singletons** may enter Ω.
A **proof-relevant** multi-constructor inductive — where distinct derivations
are distinct data (a permutation relation, `∨`, `∃`) — **cannot** be declared
`data P … : Ω` directly: an unrestricted `Type → Ω` admits `Bool`, and Ω-
proof-irrelevance then forces `True ≡ False`, collapsing consistency
(`16 §1.3`). It reaches Ω only via **truncation** `‖P_rel‖` or a natively-Ω
form (a count-equality). VAL2 never touched Ω at all.

## The pair

- **Sound arm — `sound-perm-truncated.ken` — should-PASS.** The permutation
  relation as a proof-*relevant* inductive at **`Type`** (`data Perm_rel … :
  Type`, distinct derivations allowed), then truncated to Ω —
  `Perm := ‖Perm_rel‖ : Ω` (the sub-singleton that is the `∃ := ‖Σ‖` analog,
  `16 §6`). A
  count-equality alternative (`Perm xs ys := Equal Nat (count xs) (count ys)`,
  natively Ω-valued) is included as `sound-perm-count.ken`.
- **Unsound arm — `unsound-perm-omega.ken` — should-REJECT.** The same
  permutation relation as a **4-constructor proof-relevant inductive declared
  directly at Ω**: `data Perm … : Omega = perm_nil | perm_skip … | perm_swap … |
  perm_trans …`. This is the exact construction `16 §1.3` forbids.

## Expected behavior (exact)

- Sound arms: **PASS** — `Perm_rel : Type` elaborates; `‖Perm_rel‖ : Ω` is a
  legal truncation (sub-singleton); the count-equality form is a legal Ω-valued
  proposition (a `Nat` equation). (Grounded: `16 §6` truncation; `37 §6` pins
  `Perm := ‖Perm_rel‖`.)
- Unsound arm: **should-REJECT** — the kernel's Ω sort-check must refuse a
  proof-relevant multi-constructor inductive at Ω (`16 §1.3`: `Type → Ω`
  unrestricted admits `Bool ⇒ True ≡ False`). The reject fires at the `data …
  : Omega` admission (a sort/positivity-into-Ω error), **not** downstream. **If
  it is ACCEPTED, that is a consistency hole** — you can then derive `True ≡
  False` and inhabit `Bottom`.

## Discriminates

Does the kernel enforce the Ω admission gate (sub-singletons only)? `‖Perm_rel‖`
/ count-equality (legal Ω) vs `data Perm : Ω` (4 proof-relevant constructors,
illegal) is the flip. A single "the truncation works" case would be green-vs-
green — the illegal direct-Ω declaration is the guard.

## Surface-expressibility note

`data … : Type`, truncation `‖·‖`, and Ω/`Prop` sort annotations follow the
kernel surface (`11-syntax.md`, `16`). If the surface cannot yet *write* an
explicit Ω sort annotation on a `data`, the probe degrades to "can a proof-
relevant inductive be forced into Ω by any surface path?" — document whichever
path is reachable and its result. This was a live CV-Spec BLOCKING finding on
ES1 (a `Perm` specified as a 4-ctor inductive at Ω), so the boundary is real.
