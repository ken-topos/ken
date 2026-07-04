# surface-arrow-and-infix — arrow-type in `Expr` + `-`/`*` infix (VAL2 #4 / #11)

**Steward frame → Team Language** (enclave-light; spec-leader decides if the
grammar touch needs elaboration or goes straight to Language like #5). Small
surface-syntax additions surfaced by VAL2 (#4 arrow-type in expression position;
#11 `rpn-calculator`'s missing `-`/`*` infix, worked around via `sub_int`/
`mul_int` prefix). Owner: **Language**. Gate: **Architect approach-review**
(soundness-adjacent) + Language QA + CI; **spec-leader** confirms whether the
surface-grammar `/spec` touch warrants enclave elaboration. Findings → **Steward**.

## Settled inputs — DO NOT REOPEN

- **The kernel already has `Pi`** — arrow types are a **surface/parser** gap, not
  a kernel feature. This WP adds the surface production(s) + elaboration to the
  existing kernel `Pi`; it does **not** add a kernel variant.
- **`-`/`*` infix** desugar to the existing `sub_int`/`mul_int` (and the lawful
  numeric ops) — surface sugar, no new semantics.
- **`div` is NOT in scope** — the `div_int` primitive is separately ruled out
  (Architect); RTP1 fixed the perf that was its only motivation. Do not add a
  division primitive here.
- **Soundness-adjacent, not soundness-critical** — Architect approach-reviews the
  parser/elaborator change; kernel/`trusted_base` untouched.
- **The `#5` sequencing dependency is cleared** — `L-match-ih-fix` merged
  (`07d167f`, PR #236); this WP no longer waits on it. Released **bundled with
  `mutual-recursion-surface.md`** as one Language-lane surface-syntax wave (same
  parser/elaborator surface → one branch + one gate).

## Deliverable outline

1. **Arrow-type production.** Add `->`/arrow in expression position to the surface
   grammar + elaborate it to kernel `Pi`. (Enclave/Architect confirm exact
   grammar precedence + whether it's a `/spec` grammar edit or an
   elaborator-only completeness add à la #5.)
2. **`-`/`*` infix.** Add the infix operators desugaring to the existing numeric
   ops, with the conventional precedence.
3. **Conformance.** The `rpn-calculator` shape written with infix `-`/`*` (not
   prefix) elaborates + runs correctly; an arrow-type annotation elaborates to the
   right `Pi`.

## Acceptance criteria

- **AC1 — Kernel untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  `trusted_base()` unchanged; no new kernel variant (reuses `Pi` + existing ops).
- **AC2 — Surface works.** An arrow-type expression elaborates to the correct
  `Pi`; `-`/`*` infix parse + elaborate + evaluate to the same values as the
  prefix `sub_int`/`mul_int` forms (a discriminating parse+eval test).
- **AC3 — No regression.** `cargo test --workspace` green; existing prefix forms
  unaffected.

## Sequencing

- **Gate:** Architect approach-review + Language QA + CI; spec-leader confirms the
  grammar-spec path (elaborate vs. straight-to-Language).
- **Lane:** Language. Branch off `origin/main`. **After #5 lands** (both touch the
  parser/elaborator surface — avoid contending on the same files).
