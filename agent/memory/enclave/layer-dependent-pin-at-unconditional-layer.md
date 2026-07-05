---
scope: enclave
audience: (see scope README)
source: private memory `layer-dependent-pin-at-unconditional-layer`
---

# Author a layer-dependent conformance pin at the unconditional layer

When a conformance property holds **unconditionally at one layer** but is
**construction-/stub-dependent at another**, author the pin at the
**unconditional** layer and add an explicit cross-case reconciliation to any
sibling case living at the other layer.

**Live (L3-strings-surface, DS-AC4 NFC-blindness):** the pin "codepoint-wise
`eq` does not fold NFC-normalization" was nearly authored at the **String-
literal** layer — `eq "é" "e◌́" ≡ False`. Two failures at once:
- **Over-pins a deferred behavior (T1 trap, trusted by typing guarantee is not
  kernel proved Q sibling):** under the current NFC **stub** the two literals
  are codepoint- distinct → `False` (real now), but once **real
  NFC-at-construction** lands it merges them to one `String` → `eq` is
  reflexively `True`. The case would **falsely fail a valid implementation** at
  that point.
- **Reads as a cross-case contradiction:** the existing
  `string-nfc-canonically-equal-shares-slot` (oracle) case has those **same
  literals** share a slot → `==` `True`. A reader sees "`eq …` False" next to
  "`== …` True" on the "same" inputs.

**Fix:** pin at the `List Char` layer —
`list_eq eqChar [U+00E9] [U+0065,U+0301] ≡ False` — where two distinct scalar
**sequences** are **always** codepoint-unequal (no NFC/stub dependency), and add
a sweep bullet reconciling the two operations: `==` decides **content-addressed
identity** (post-normalization, slot-id); `eq`/`list_eq` decides
**scalar-sequence** equality (NFC-blind); they **agree** on `String` values, and
the codepoint- distinct witness is one `String` construction never yields as two
values ⇒ **no overlapping-input contradiction**.

**Test to apply:** "is this pin **unconditional**, or does it depend on a
stub/deferred construction step?" If conditional, drop a layer until it isn't.
Then run the internal-consistency pass against any sibling case at the other
layer and state the reconciliation in the corpus (don't leave the apparent
contradiction for a reader to trip on). Sibling of the "assert the property at
the layer where it's observable" family (soundness AC static vs runtime face).
