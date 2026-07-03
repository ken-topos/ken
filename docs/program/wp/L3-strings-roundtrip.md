# WP L3-strings-roundtrip — make the `String ↔ List Char` pair real (pin-2)

**Phase-3 (Language surface), slice 1 of 2. Team: Runtime. Base: `origin/main`.**
Steward frame. This is **deliverable #1 of the string half of Rosetta
readiness** (Architect ruling `evt_66g17exdhd767`): the whole L3 string surface
(`concat`/`slice`/`charAt`/`eq`/`compare`) derives over `List Char`, so it is a
**no-op until the round-trip pair is real.** Cite ADR 0010 (canonicity), spec
`30-surface/37-strings-collections.md §2.3`. Posture: F1/conversions —
native interp-ring reductions, **tested-not-trusted**, netted by an independent
oracle; kernel untouched.

## Objective

`string_to_list_char` and `list_char_to_string` are **`Neutral` stubs** today
(`crates/ken-interp/src/eval.rs:954-955` — "totality asserted at the type level"
but never decoded). Make them **real**:

- **`string_to_list_char : String → List Char`** — decode the UTF-8 buffer into
  a `Ctor` cons-list of `Char` (Unicode scalar values).
- **`list_char_to_string : List Char → String`** — encode a `List Char` back to
  the packed UTF-8 buffer. Total: the `Char` refinement guarantees every element
  is a valid scalar, so encoding never fails (spec `37 §2.3`, L98: "the carrier
  cannot encode an invalid scalar").

The derived surface slice (concat/slice/charAt/eq/compare over `List Char`) is
**slice 2** (Team Language, gated on this landing) — **out of scope here.**

## The soundness crux — `Char` is refinement-typed (read this first)

Unlike the **bare** `IntN` of the conversions WP (where an out-of-range value is
a wrong value but *never a false proof* — no refinement to fabricate against),
**`Char = { c : Int | isScalar c }` is refinement-typed.** So a `String →
List Char` decode that produced a **non-scalar** `Int` and handed it back typed
as `Char` would fabricate a false `isScalar` witness — a **genuine soundness
hole**, not a mere wrong value. The whole WP turns on `s2l` producing **only
valid scalars**.

The floor that makes this sound already exists and **reduces**:
`inRangeBool c = or_bool (0≤c≤55295) (57344≤c≤1114111)`
(`decimal_char.rs:225` — exactly the Unicode scalar range, **surrogates
[0xD800..0xDFFF] excluded**), and `intToChar : Int → Option Char =
match (inRangeBool n) { True => Some Char n ; False => None Char }`
(`decimal_char.rs:253`) is the existing **refinement-checked** `Char`
construction. A Rust `String`'s `char`s are, by Rust's own invariant, always
valid Unicode scalars — so `s2l`'s decode yields real witnesses regardless. The
**mechanism** for producing the witness (decode directly, trusting Rust's
scalar-valued `char`; **vs** routing each codepoint through the checked
`intToChar`/`inRangeBool` path) is the implementer's proposal and **the
Architect's soundness call** — flag it explicitly at the gate.

## Code sites (verify on `origin/main` — grep to confirm, lines drift)

- **The stubs:** `eval.rs:954` `("string_to_list_char",[EvalVal::Str(_)]) =>
  EvalVal::Neutral`; `:955` `("list_char_to_string",[EvalVal::Ctor{..}]) =>
  EvalVal::Neutral`. Arity already 1 (`:1508`). The prims are **already
  registered** — this fills the reduction, adds no new surface.
- **`Char` / `intToChar` / `inRangeBool`:** `crates/ken-elaborator/src/
  decimal_char.rs:223-255` (the refinement-checked construction to reuse).
- **`List Char` representation:** an `EvalVal::Ctor` cons-list (Nil/Cons); build
  it the same way the interp builds any inductive `Ctor` value (grep how
  `List`/`Cons`/`Nil` ctors are constructed — the K3 `SlotId`-interned compound
  path).

## Hard ACs (each a gate)

1. **(soundness — THE gate: the scalar witness)** Every `Char` `s2l` produces
   satisfies `isScalar` (is in the surrogate-excluding scalar range). The
   implementer states the mechanism (direct decode vs routed through the checked
   `intToChar`/`inRangeBool` path) and its trust justification; the **Architect
   rules the posture**. A decode that could emit a non-scalar typed as `Char`
   fails this — that is the false-proof face, distinct from conversions' bare
   wrong-value.
2. **(soundness — round-trip identity, the non-circular defining oracle)**
   `list_char_to_string (string_to_list_char s) ≡ s` for the boundary corpus;
   `string_to_list_char (list_char_to_string cs) ≡ cs` for well-formed `cs`.
   This is the preferred oracle (`18a §3`: a defining law, checked against the
   op's output algebra, cannot alias the reduction it audits).
3. **(the independent boundary corpus — the net a round-trip alone can miss)**
   Concrete UTF-8-boundary reference values, each an **objective Unicode fact**
   CV re-derives independently: 1-byte `U+0000`/`U+007F`, 2-byte
   `U+0080`/`U+07FF`, 3-byte `U+0800`/`U+FFFF`, 4-byte `U+10000`/`U+10FFFF`
   (max), a mixed multi-scalar string, and the empty string. Plus the
   **surrogate guard**: `s2l` never emits a codepoint in `[0xD800,0xDFFF]`
   (structurally impossible from valid UTF-8 — assert it), and `l2s` cannot be
   fed one (the `Char` refinement bounds its input).
4. **(`l2s` totality)** `list_char_to_string` returns a `String` for **any**
   `List Char` — no stuck `Neutral`, no failure arm — because the refinement
   guarantees valid scalars. Pin with a case over a non-trivial `List Char`.
5. **(whole-WP soundness)** **Kernel diff empty** (`git diff --stat
   crates/ken-kernel/` nothing). `trusted_base()` unchanged — the prims were
   already registered; this fills their interp reduction (tested-not-trusted,
   netted by AC2/AC3), no new native surface, no K3 promotion. **Workspace-green**
   (K7: QA re-runs `./scripts/ken-cargo test --workspace` independently).

## Oracle discipline (`18a §3`)

Round-trip identity (AC2) is the primary non-circular net; the boundary corpus
(AC3) is the independent reference that catches an inverse-error pair the law
alone would miss. Wire both as acceptance tests in the crate test file; **CV
independently re-derives the boundary values against Unicode and validates the
witness posture** (AC1) at its conformance gate — not implementer-trusted. A
durable `conformance/surface/strings/` seed is a welcome CV deliverable but not
required to land (the boundary values are objective; CV owns the corpus).

## Out of scope / defer (verify by absence)

- **The derived surface** — `concat`/`slice`/`substring`/`charAt`/`eq`/`compare`
  over `List Char` = **slice 2, Team Language**, gated on this. Not here.
- **`char_length` DEMOTE** (`= length ∘ s2l`) — derivable once s2l is real; a
  future demote (Architect's F-series retire-natives-that-derive posture), not
  this frame. Leave `char_length` working as-is.
- **NFC normalization / normalization-equality** — the `String` carrier stores
  NFC-normalized bytes (`37 §49`); `s2l`/`l2s` are **scalar-faithful on the
  stored buffer** (no normalization step here). Normalization-insensitive
  equality is a separately-named derived **`Eq`, never a `DecEq`** (ADR 0010) —
  a later concern, not this WP.
- **Float / bytes-IO / collections** — separate surfaces.

## Flow (thin — COORDINATION §9)

`runtime-leader → runtime-implementer → runtime-qa → Architect (soundness — the
witness posture is THE gate) + CV (conformance — boundary corpus + round-trip +
witness validation) → Integrator`. One pass each. Crates-only (spec `37` already
defines this; implementing, not changing spec) unless CV lands a corpus seed
(`conformance/` → still CV's gate, no spec-author vote). A soundness fork
(esp. the witness mechanism) → Architect; conformance fork → CV; scope fork →
Steward. Thread the whole ring under the Steward's kickoff event for **this** WP
(COORDINATION §4). A **pre-build Architect design consult on the witness
mechanism is encouraged** if the implementer wants the posture ruled before
coding rather than at the gate.
