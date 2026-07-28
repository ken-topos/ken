---
scope: fleet
audience: (see scope README) — anyone whose WP's acceptance rests on a
  property run over a corpus (parse-preservation, idempotence, round-trip,
  "no catalog regressions")
source: kenfmt capstone C (B3 layout printer), 2026-07-13 — Architect
  terminal BLOCK after every automated gate had gone green
---

# A corpus-run property gate is only as strong as the corpus

A property gate run **over a corpus** (e.g. whole-catalog
parse-preservation for a formatter) is only as strong as the corpus — a
construct absent from the catalog leaves the gate green while a real
violation ships. Token-content-changing axes (parenthesization) need
**adversarial** fixtures targeting the shapes the corpus lacks, and each
confirmed counterexample must become a permanent regression fixture so the
blind spot closes.

**2026-07-13 — kenfmt capstone C (layout printer) terminal BLOCK.** The
build passed **whole-catalog parse-preservation + idempotence + 88-col**
*and* the literal locked workspace CI — QA approved it — yet the
Architect's terminal review found **two confirmed meaning-changing
parse-preservation violations** in the parenthesization printer on `old`'s
operand:

```
(old x).field  →  old x.field    EProj(EOld x) ⇒ EOld(EProj x)   (meaning changed)
old (f x)      →  old f x         EOld(EApp f x) ⇒ EApp(EOld f) x (meaning changed)
```

**Why every green gate missed it:** the whole-catalog parse-preservation
gate passed **only because the catalog contains no `old`-with-compound-
operand program** — the corpus simply lacks the counterexample. `old` is
contract-only but the forms are legal and meaningful (`ensures (old
buf).len == n`), so a live formatter run would have **silently changed
contract meaning.** A property verified by running over a corpus is **only
as strong as that corpus's coverage** — a gate that iterates the catalog
inherits the catalog's blind spots.

**The load-bearing rule:** the **parenthesization axis** (the one place a
formatter changes *token content*, not just layout) — and by extension any
token-content-changing transform — **needs adversarial fixtures**, hand-
built to hit the shapes the corpus is unlikely to contain (postfix `.` on a
keyword-led operand, application under a tight-binding prefix, precedence
boundaries, etc.), **not** just catalog coverage. Corpus coverage answers
"does it break anything we already wrote"; adversarial coverage answers
"does it break anything *legal*."

**Disposition (the durable half):** for each confirmed counterexample,
**add it as a permanent regression fixture** (here: `(old x).field` and
`old (f x)` as parse-preservation cases with an AST-shape + elaboration
backstop), so the fix is verified to **exclude the counterexample** *and*
**the gate's blind spot is closed permanently** — the next regression
can't slip through the same hole. Don't just fix the bug, install the
fixture that would have caught it.

**How to apply:**
- When a WP's acceptance rests on a **property run over a corpus**
  (parse-preservation, idempotence, round-trip, "no catalog regressions"),
  treat green as **necessary, not sufficient** — ask *which legal
  constructs are absent from the corpus* and require **adversarial
  fixtures** for the highest-risk axis (the one that changes
  meaning/token-content). Put that requirement in the WP frame's
  acceptance criteria, not just the reviewer's head.
- Sibling to a reachability sweep that proves each oracle *input* is
  landed: that proves the inputs are exercised; this one proves the *gate
  corpus* actually exercises each risky construct. Both close "the test
  looked green but never ran the case that matters." See
  [[formatter-soundness-gates-are-blind-to-layout-conformance]] (the sibling
  finding from the same capstone: the gate net that DID pass was blind to
  a different axis entirely — layout conformance, not meaning).
