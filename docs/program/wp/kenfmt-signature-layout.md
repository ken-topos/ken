# WP · kenfmt layout rework — batch 1 (operator directive): prefer horizontal

**Owner:** Language (kenfmt owner — B1–B4 + capstone C) · **Reviewer:** QA
(locked workspace + golden layout) · **Architect:** informational (layout =
tool/style, not kernel semantics) · **Size:** M · **Base:** `origin/main @ 6088e0b8`
at kickoff (strict frozen-corpus gate is paused, task #57 — no CI pressure; rework
one file first, review, then the catalog, then re-arm the gate).
**Status: READY — operator-confirmed 2026-07-13** (arg-order slip = editing error /
formatter is semantics-preserving; width budget = 96; R1 indentation refined:
params +6 / `:` +4 / split-return +6, signature always deeper than body). Kicked to
Language after CP0 close.

## Governing principle (operator, 2026-07-13)

**Most humans read horizontally — prefer to keep things horizontal, not
vertical.** kenfmt currently over-verticalizes. The whole batch is instances of
one rule: **collapse to one line wherever the content fits the width budget;
split only when it doesn't fit, and split minimally.**

## Hard invariant — layout only, semantics-preserving

The formatter changes **whitespace/line-breaks only**. It **never** reorders,
renames, inserts, or drops a token. *(The "small expressions" example below shows
`sub m n` → `sub n m`; that argument-order change is treated as a transcription
slip — the formatter must NOT reorder arguments. The point of that example is the
line-collapse, nothing else.)*

## The rules (each = "prefer horizontal", with the split form when it can't fit)

**R1 · Signature split layout — signature is always deeper than the body.**
The controlling invariant (operator refinement): a split signature must **never
share an indentation level with the body**, so it reads as a distinct indented
preamble. Column scheme, measured from the declaration keyword (`fn`/`def`) at +0:
- **body** — single-indent, **+2**
- **parameters** — triple-indent, **+6**
- **`:`** — double-indent, **+4** (return type inline after the `:` when it fits)
- **vertically-split return type** — triple-indent, **+6** (when the return type
  spills past one line, its lines sit at +6)

So every signature part (params +6, `:` +4, split-return +6) is deeper than the
body (+2) — the signature and body are always visually separable.

- Fits on one line → **one line** (whole `fn/def name (params) : ret =`).
- Doesn't fit → split: the **declaration name stays alone** on the first line;
  **parameters triple-indented** (one per line, or grouped on one line if they
  fit together); the **`:` double-indented** so it delimits the parameter block;
  a return type that splits goes to **triple-indent**. Example (operator, revised
  columns):
  ```
  fn total_leq_nat
        (x : Nat)
        (y : Nat)
      : Or (Equal Bool (leq_nat x y) True) (Equal Bool (leq_nat y x) True) =
    body
  ```
  (params at +6, `:` at +4, body at +2 — signature strictly deeper than body.)
  A long return type that itself splits:
  ```
  fn wide
        (x : Nat)
      : SomeLongReturnConstructor
          argument_one
          argument_two =
    body
  ```
  (`:` at +4; the split return-type lines at +6.)

**R2 · Proof references are atomic.** `proof <name> for <target>` **never
splits** — always one horizontal unit, even inside a λ body:
  ```
  total = λx.λy.proof eq_true_of_or for bool_or
  ```

**R3 · Short parenthesized expressions never split.** `(leq_nat x y)` stays
inline when it fits — do not break a small application across lines:
  ```
  (leq_nat x y)          -- not (leq_nat \n x \n y)
  ```

**R4 · Short data declarations on one line.** When it fits:
  ```
  data OrdResult = Lt | Eq | Gt
  ```
  (not the constructor-per-line vertical form.)

**R5 · Small expressions stay on one line.** Don't vertical-split a small
application / match arm body that fits:
  ```
  Suc m ↦ sub n m        -- not Suc m ↦ \n sub \n m \n n
  ```

These compose: at every construct (signature, data decl, parenthesized expr,
application, match arm), if it fits the width budget keep it horizontal; only the
signature has a structured multi-line fallback (R1); the others simply stay
inline when short.

## Settled parameters (operator-confirmed 2026-07-13)

- **Width budget = 96 columns.** "Fits / short enough" is measured against a
  96-column line budget (operator raised it from the B3 88-col target — "96 is
  acceptable"). Every collapse/split decision keys on 96.
- **Arg-order slip confirmed** (R5 `sub m n`→`sub n m` was an editing error) —
  the formatter is **layout-only, semantics-preserving**; it never reorders,
  renames, inserts, or drops a token (AC4 gates this).

## Acceptance criteria (draft — finalize after readback)

- **AC1** — the layout printer implements R1–R5 under the horizontal-first
  principle; the width budget governs every collapse/split decision.
- **AC2** — **applied to ONE representative catalog file first**; the operator
  reviews that diff before any catalog-wide reformat.
- **AC3** — golden tests pin each rule: R1 signature ladder (fits / params-split),
  R2 proof-atomic, R3 short-paren-inline, R4 short-data-inline, R5 small-expr-
  inline; locked workspace green; `ken fmt` idempotent (format-of-format = format).
- **AC4** — **semantics-preserving:** a round-trip corpus check that the token
  stream is unchanged by formatting (only whitespace/layout differs) — the hard
  invariant, mechanically gated.
- **AC5** — the strict frozen-corpus gate stays **paused** in this WP (re-arm is a
  later step, after the catalog is reworked).

## Sequencing (Steward)

kenfmt is Language's tool; Language is mid-CP0 (§5.2, on the CC2 critical path).
Gate disabled ⇒ no CI pressure. **Recommend: next Language WP after CP0** (frame
is shovel-ready now; Handoff-Gate the Language ring at CP0 close, then kick). If
the operator wants it to preempt CP0, say so.
