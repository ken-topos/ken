---
name: formatter-soundness-gates-are-blind-to-layout-conformance
description: "A formatter's soundness gate net (AST/parse-preservation, idempotence, max-column) checks that MEANING is unchanged — it is structurally blind to whether the LAYOUT is CONFORMANT/good. A reformat can pass every soundness gate and still be grotesque (e.g. every token broken to column 0). Any formatter WP must ALSO gate what it actually produces against the layout spec, not just that meaning survived."
metadata:
  node_type: memory
  type: feedback
  scope: fleet
---

**2026-07-13 — kenfmt capstone C, Architect gate BLOCK
(`evt_6zk2xfma2y8mc`).** C's whole-catalog reformat passed **every** gate —
AST/elaboration-preservation, idempotence, 96-column, byte-identity of `.ken.md`
prose/markers — and QA + the frame-vs-candidate oracle all went green. Yet the
output was **§1d-nonconformant and grotesque**: a pre-existing B3
signature-layout engine defect broke every declaration signature wider than 96
cols **one token per line at column 0** (zero continuation indent + shattered
even binders that fit), producing ~16,145 malformed lines across 9 catalog
packages — the entire +50k-line diff *was* the bug (`Map.ken.md` 6,710 → 31,452
lines). Committing it under a strict `ken fmt --check` CI gate would have
**permanently canonicalized the catalog to spec-violating layout.**

**Why every gate was green — the structural blindness (memorize this):**
- **Parse/AST-preservation** — newlines are whitespace, so mangled layout parses
  to an identical AST. ✓ (sees meaning, not layout)
- **Idempotence** — the mangled output is a fixed point of the buggy formatter. ✓
- **Max-column (96)** — every over-broken line is *tiny*, so the width bound is
  trivially satisfied. ✓ (a width *ceiling* cannot catch pathological *under*-use)
- **`.ken.md` byte-identity** — only guards non-body prose/markers, not body
  layout quality. ✓

The whole net verifies **the reformat preserved meaning**; **none of it verifies
the reformat produced good/conformant layout.** A formatter is exactly the tool
whose *output shape* is the deliverable — so a gate net that only checks meaning
is necessary but wildly insufficient.

**The rule:** a formatter (or any layout/codegen/canonicalizer) WP must gate
**what it actually emits against the layout spec**, not merely that the round-trip
preserved semantics. Concretely, add **positive layout-conformance assertions**:
e.g. no required-continuation line at column 0; continuation indent is exactly N
levels past the enclosing construct (not a coincidental column); a fitting
sub-group stays flat; and a **sanity bound on output size** (a canonical reformat
does not pathologically expand line count — a Nx blowup is a red flag). Column
*ceilings* and idempotence do not substitute for these.

This is the sharpest instance yet of "gate the real mechanism, not a proxy"
([[verify-the-mechanism-not-a-proxy]] family, [[green-vs-green-does-not-confirm-a-fix]],
[[corpus-property-gate-only-as-strong-as-the-corpus]]): AST-preservation is a
*proxy* for "the formatter is correct," and it is a proxy that cannot see the
formatter's primary failure mode. Sibling to the standing B3 carry "a formatter
WP must gate what it actually produces." Also note the **attribution**: the
defect was a latent B3-engine bug that only became *visible* when C ran the
engine over the whole corpus — a capstone that first applies an engine at scale
surfaces the engine's latent output defects, so gate the capstone's *output*, not
just its *preservation*.

**How to apply:** when framing/reviewing any formatter, pretty-printer,
canonicalizer, or codegen WP, require an acceptance criterion of the form "the
emitted output conforms to <layout spec section>" with executable positive
assertions on indent/breaking/size — separate from and in addition to the
meaning-preservation net. If the only gates are parse-preservation + idempotence
+ a width ceiling, the layout is ungated.
