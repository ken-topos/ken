# LET-1b — a `let` chain is a sequence, not a staircase

# ⛔⛔ HELD — SUPERSEDED BY LET-4. DO NOT BUILD THIS.

> **Superseded by `docs/program/wp/let4-multi-binding-let.md` (Steward,
> 2026-07-14, on an operator-originated WP). LET-1b was never started.**
>
> **LET-1b framed the staircase as a FORMATTING defect** and would have amended
> §31 to render a nested chain flat. **LET-4 makes a chain NOT BE A NEST** — its
> S6 canonicalizes a maximal nested chain into a semicolon-separated binding
> **group**, so after LET-4 **a nested chain never survives formatting** and this
> WP's amended production would be **dead the day LET-4 merged**.
>
> **The staircase was never a formatting bug. It was a SURFACE bug wearing a
> formatting bug's clothes:** Ken had no way to *say* "these bindings are one
> group," so a group had to be spelled as a nest — and we then asked the printer
> to render a nest as though it weren't one. **That is a workaround fossil, and
> the fix is the language, not the printer.**
>
> **What survives, folded into LET-4:**
> - **D1** (a fitting binding's RHS forced onto its own line) → LET-4 **AC5.1 /
>   AC5.4**.
> - **AC4 "a reader looked"** → LET-4 **AC-READER**, strengthened.
> - **The no-re-baseline rule** → LET-4 **AC-DERIVE**.
>
> **The analysis below is RETAINED as the evidence record** — it is why LET-4
> exists and why its layout ACs are shaped as they are. **It is not a build
> order.**

**Owner:** ~~Ergo~~ **HELD** · **Size:** S/M · **Depends on:** LET-1 (merged
`ec980d76`), LET-2 (merged `8853b475`) · **Blocked LET-3** — *that dependency now
routes through **LET-4***

> **Treat every anchor in this frame as PERISHABLE.** If a fixed input below is
> false against the landed code, **say so with exact tree anchors and
> ESCALATE** — do not quietly build around it.

## 1. The defect

LET-1 fixed *shredding* (a `let` chain emitted one token per line). It did not
fix **staircasing**, because **the spec mandates it**. This is LET-1's own
frozen oracle (`crates/ken-elaborator/tests/kenfmt_let_layout.rs`), asserting
the canonical text of a **flat, three-binding chain**:

```
const chars : List Char =
  let left_chars : List Char =
    string_to_list_char left
  in
    let right_chars : List Char =
      string_to_list_char right
    in
      let joined_chars : List Char =
        append Char left_chars right_chars
      in
        joined_chars
```

**Two defects, both mandated by `spec/30-surface/31-lexical.md:216-227`:**

| # | Defect | Why it is wrong |
|---|---|---|
| **D1** | A `let`'s RHS is forced onto its own line whenever the **body** is compound — the binding's own width is never consulted. | `let left_chars : List Char = string_to_list_char left in` is **58 columns**. It fits. Nothing about a compound *body* should break a binding that fits. |
| **D2** | A `let` whose body is a `let` indents that body one level, **recursively** — so a flat chain becomes a right-nested staircase. | `let a = x in let b = y in body` is a **sequence**, not a nest. Six bindings put the final expression **twelve columns** to the right of the first. Every ML-family language renders this flat. |

**⚠ LET-1's implementer did not choose this.** They derived the emitted text
from §31 faithfully and correctly; the derivation is sound and the oracle
records it exactly. **The contract is what is wrong.** Do not read this WP as a
criticism of LET-1 — read it as the reason a derived-from-contract oracle
cannot detect a defective contract.

## 2. Why it is urgent

**LET-2 has landed the convention that tells every agent to stage code with
`let`.** Until LET-1b lands, **`ken fmt` staircases every chain anyone writes** —
and a frozen corpus gate then makes that layout canonical across the catalog.
**LET-3 (the catalog pilot) is blocked on this WP** for exactly that reason.

## 3. Deliverables

### 3.1 Amend `spec/30-surface/31-lexical.md` (the `let` production, ~L216-227)

Two changes, stated as productions:

1. **A `let` binding that FITS stays on one line** — `let x : A = value in` —
   **regardless of whether the body is compound.** The binding's own width
   decides the binding's own layout. A `let` with a genuinely **compound
   VALUE** still breaks structurally, and **that part of §31 is correct and
   stays**:
   ```ken ignore
   let x : A =
     compound_value
   in
   body
   ```
2. **A `let` CHAIN is a FLAT SEQUENCE.** When a `let`'s body is itself a `let`,
   the body does **not** indent. Every binding in the chain, and the final body,
   sit at the **same** indentation:
   ```ken ignore
   const chars : List Char =
     let left_chars : List Char = string_to_list_char left in
     let right_chars : List Char = string_to_list_char right in
     let joined_chars : List Char = append Char left_chars right_chars in
     joined_chars
   ```

**Note the `in` placement changes too:** a fitting binding carries its `in` on
the same line. A structurally-broken binding keeps `in` on its own line at the
binding's indentation, with the body following at the **same** level (not +1).

### 3.2 Fix `kenfmt` (`crates/ken-elaborator/src/layout.rs`)

LET-1 landed the compound test at `layout.rs:1068`
(`matches!(body, Expr::ELet(..))`). That test is what makes a chain "compound";
it is **still correct** — a chain *does* need structural treatment. What is
wrong is the **shape** that treatment emits. Fix the emission, not the
predicate.

### 3.3 Re-derive the oracle — and DELETE the staircase fixtures

`kenfmt_let_layout.rs`'s expected texts were derived from the defective
production. **They are not a baseline to preserve; they are the bug.** Re-derive
each expected text from the **amended** production and replace them.

> **⛔ Do NOT re-baseline by pasting the formatter's new output into the
> expected string.** That is the rubber-stamp failure LET-1's own QA named.
> **Derive the expected text from the amended §31 first, then make the
> formatter match it.** If the amended production leaves two texts equally
> admissible, **STOP and escalate** — that is a spec gap, not a coin flip.

### 3.4 Reformat the two guide strands flat

`catalog/guide/proof-techniques.ken.md` and
`catalog/guide/surface-reference.ken.md` are currently canonical **at the
staircase** (`origin/main @ 62205675`). Re-run `ken fmt` after the formatter is
fixed. **The teaching guide must show the layout the convention actually
wants.**

## 4. Acceptance criteria

- **AC1 — D1 is gone.** A `let` whose binding fits emits on one line with its
  `in`, even when its body is compound. Assert the **exact emitted text**.
- **AC2 — D2 is gone.** In a chain, **every binding and the final body are at
  the same indentation.** Assert the exact emitted text for a **six**-binding
  chain.
- **AC3 — the structural break still works** for a genuinely compound *value*
  (a `match` RHS). This is the LET-1 behavior that is **correct**; prove it did
  not regress. Assert the exact emitted text.
- **AC4 — ★ A READER LOOKED.** Render the six-binding chain and the guide's
  `let_staged_color` example, **paste both into the handoff verbatim**, and
  state that the final body is **not** indented deeper than the first binding.
  *This AC cannot be discharged by a passing test. LET-1 had six green
  mechanical gates and staircased anyway — because **not one of them asked
  whether the output was readable**, which was the property in its own title.*
- **AC5 — LET-1's preservation net is fully retained** and green: AST equality,
  token equality, idempotence, ≤ width ceiling, `trusted_base()` unchanged.
  **None of these substitutes for AC4, and AC4 does not substitute for these.**
- **AC6 — the frozen corpus gate is green** (`-p ken-cli --test ken_fmt`, 5/5)
  **and** the two guide strands are canonical **and** all three guide strands
  still `ken check`.
- **AC7 — ZERO `trusted_base()` delta.** No postulate, no primitive, no `Axiom`.

## 5. Guardrails

- **⛔ Do NOT re-baseline an expected text from the formatter's output.** Derive
  it from the amended spec, then make the code match. (§3.3)
- **⛔ Do NOT touch the compound-VALUE break.** `let x : A =\n  <compound>\nin`
  is correct; only the **body/chain** handling is wrong.
- **⛔ Do NOT delete or weaken LET-1's preservation net** (AC5). The staircase
  passed all of it — that means the net is *insufficient*, **not wrong**.
- **⛔ Build/test TARGETED ONLY** — `scripts/ken-cargo -p ken-elaborator` /
  `-p ken-cli --test ken_fmt`. **Never `--workspace`** (COORDINATION §12,
  operator hard rule). Workspace-green is CI's job, at publish.
- **This WP touches `spec/`.** The merge gate therefore includes the Spec
  enclave (CV) alongside the Architect.
