# SURF-SPACE-CELLS P1 — build the `space` block and `becomes` onto the built `State`

**Node:** [`SURF-SPACE-CELLS`](../issues/SURF-SPACE-CELLS.md) · **Owner:**
Language · **Size:** M–L · **Gate:** none

**Fixed inputs, measured at `origin/main = aea07d62`. ⛔ Current-state claims —
re-derive at point of use.**

| input | pin |
|---|---|
| the surface | `spec/30-surface/36-effects.md §4` — the `space Counter { mut n : Int = 0 ; proc … }` block |
| ⭐ the desugaring, **given verbatim** | `36 §4.1` — state type, effect label, read rule, write rule |
| the target, **already built** | `crates/ken-elaborator/src/effects/state.rs` — `StateOp s = Get \| Put s`, `resp_state`, `get`, `put`, `run_state` |
| the fold | `36 §4.2`; `§4.5.3` says `run_state` **is** that fold at `F = 𝟘` — ⛔ do not re-specify it |
| the record update | `13 §3` η — `s with .i := v` reuses **every other component** |
| the error class | `36 §7.3` class 4 — `becomes` on a non-cell, or `mut` outside a space |
| what `space` does today | `parser.rs:355` — a **modifier on `proc`**; the block form does not parse |
| ⛔ out of scope, stays as-is | `OldPreStateUnsupported` (`elab.rs`, blob `648df173`) |

## 1. What this WP is

**Make `36 §4`'s `space` block parse and elaborate, desugaring cells and
`becomes` onto the `State` effect that is already built.**

⭐ **The hard half exists.** You are not building an effect, a response family, a
handler, or a fold — all four are live in `effects/state.rs`. You are building
**a surface and the translation onto it**.

⛔ **`becomes` is not kernel mutation.** `§4.1`, verbatim: *"`becomes` is **not**
a kernel mutation — it is a `Get`-then-`Put` on the pure tree."* **Zero kernel,
zero trusted-base delta.** If you find yourself putting a mutable cell in the
TCB, stop and report — the premise has failed.

## 2. Deliverable

1. **Parse** `space Name { mut c : T = e … proc … }`. Today `space` only prefixes
   a `proc`; the block form is a parse error.
2. **Build the state type** `S = T₁ × … × T_m` — right-nested Σ/record
   (`13 §3`) with η, so cell update reconstructs definitionally.
3. **One effect label per space.** Every operation `visits [<space>]` uses
   `State S`.
4. **Elaborate cell access** exactly as `§4.1` writes it:

```
cᵢ            (read)   ⤳  bind (perform Get) (λ s. Ret (s.i))
cᵢ becomes e  (write)  ⤳  bind (perform Get) (λ s. perform (Put (s with .i := ⟦e⟧)))
```

5. **The `§7.3` class-4 error** — `becomes` on a non-cell, or `mut` outside a
   space — with a span.

⛔ **Do not invent the desugaring.** It is quoted above and in the node. Reuse
`effects/state.rs`; do not declare a second `State`.

## 3. Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-S1` | The `§4` block parses, including the spec's own `Counter` example verbatim. | ⭐ Use the spec's example **as written** (`36 §4`). A spec example that its own implementation cannot parse is the cheapest possible defect and it is worth one test on its own |
| `AC-S2` | ⭐⭐ **A write updates the target cell and preserves every other cell.** | **This is the load-bearing AC.** ⛔ **A one-cell space proves nothing** — with `m = 1`, `s with .1 := v` *is* `v`, so an implementation that discards the record and returns the new value passes. ⛔ **A two-cell space of equal starting values proves nothing** — an off-by-one that writes the wrong component is invisible. **Use ≥3 cells with pairwise-distinguishable values, write to the MIDDLE one, and assert all three components afterwards**: the written one changed, and both neighbours are byte-for-byte unchanged. The η/"reuses every other component" clause is exactly what a shortcut implementation breaks and this is the only fixture shape that can see it |
| `AC-S3` | Reads resolve to the right component. | Same ≥3-cell fixture: read each cell and assert each returns its own value, not a neighbour's. ⛔ Reading only one cell cannot detect an index error |
| `AC-S4` | The desugaring is the `§4.1` one, not an equivalent-looking shortcut. | ⭐ Assert on the **elaborated term** — a `Get`-then-`Put` structure — not only on the answer a program computes. A state-threading implementation that never emits `perform Get` produces the same final number and is not what the spec specifies. ⚠ If your harness cannot observe the term, say so in the report and state what you asserted instead; do not claim the structure was checked when the value was |
| `AC-S5` | The space's effect label appears in the row, and its absence is caught. | An op that `visits [<space>]` carries the label; one that performs a cell access **without** declaring it fails the existing escape check. ⭐ The escape machinery is already built and already tested — reuse it, and make the negative case a real elaboration, not a synthetic row |
| `AC-S6` | `§7.3` class 4 rejects with a span. | `becomes` on a non-cell **and** `mut` outside a space — both, separately. ⛔ One rejection standing in for two rules is one control, not two |
| `AC-S7` | ⛔ **`old` is untouched and still fails closed.** | `OldPreStateUnsupported` still fires for `old` in a space `ensures`; a pure `fn` still gives exact `UnboundName(old)`. ⭐ **This AC is a fence, not a feature.** Making `old` work here is out of scope — it is the successor WP, and folding it in leaves that WP's controls unwritten |
| `AC-S8` | Each control is causal. | Per control, one compile-preserving mutation at the natural site; show it reddens **that named test**; restore byte-identically (`git diff --exit-code`). ⚠ An unexpectedly wide redden usually means the build broke — report test names |
| `AC-S9` | **Zero kernel, zero trusted-base, zero spec, zero conformance delta.** | `crates/ken-elaborator/` only |

## 4. Scope

**IN:** `crates/ken-elaborator/` — lexer (a `becomes`/`mut` token), parser, the
space desugaring, and tests.

⛔ **OUT:** `old`/pre-state (`AC-S7`) · a second `State` · kernel or
trusted-base · `§4.4` concurrency and isolation (`OQ-Space`) · re-specifying
`run_state` · `spec/` and `conformance/` edits.

## 5. Contention check

**Measured at `aea07d62`.** Live `ken-elaborator` work at measurement time:
`V4-RESIDUAL` (`diagnostics.rs`, `v4_acceptance.rs`) and the queued
`SURF-IDENT-TR39-R1` for Ergo (`lexer.rs`, `surface_unicode.rs`).

⚠ **`lexer.rs` is a real overlap with `SURF-IDENT-TR39-R1`** — you need a
`becomes` (and `mut`) keyword token, they touch the identifier rule. The edits
are in different functions and should merge cleanly, but ⛔ **do not assume it:**
re-derive the intersection at handoff and coordinate through me if both are in
flight. I will sequence rather than let two rings collide on one file.

## 6. Validation

⛔ **Targeted only — never `--workspace`.** `scripts/ken-cargo test -p
ken-elaborator`, and `--test <name>` for individual suites. Workspace-green and
the `--locked` gate are **CI's** job, not this box's. Ask for the shared build
slot; do not take it.

## 7. Reporting

Return: the exact SHA and tree · the ≥3-cell fixture and `AC-S2`'s **actual
assertion output**, since that is the control the rest leans on · your `AC-S4`
position (did you assert on the elaborated term, or on the value — say which) ·
the `AC-S8` mutations with restore proof · and the measured-vs-claimed boundary,
naming plainly what `36 §4` still does not deliver.
