---
id: DS-9
title: "lawful JSON codec — the data-structures tier's acceptance test: a Json value type, encode/decode, and the proved round-trip law, assembled entirely from the landed Core/Data sections"
status: draft
owner: foundation
size: L
gate: none
depends_on: [KERNEL-NESTED-IND]
blocks: []
github: null
origin: Phase 3 of the catalog data-structures enrichment program (docs/program/wp/catalog-data-structures-program.md), under the catalog campaign charter (docs/program/06-catalog-campaign.md), which homes catalog authoring in Foundation. Steward-filed; Steward owns the frame and AC/control placement. Carrier design fork ruled by the Architect as dec_3n1pp559pxrrw and transcribed into frame §3. The node is now draft because it is BLOCKED on KERNEL-NESTED-IND — see the banner.
---

> ## ⛔⛔ BLOCKED 2026-07-27 — `Json` IS NOT EXPRESSIBLE IN THE CURRENT KERNEL
>
> **Released, started, and blocked at `D1` inside half an hour.** Foundation
> found that the ordinary spelling
>
> ```ken
> data Json = ... | JsonArray (List Json) | JsonObject (List (Pair String Json)) | ...
> ```
>
> is **rejected by the kernel** as a nested inductive — the `List (Rose A)` class
> that `spec/10-kernel/14-inductive.md` §8.5 defers (`:126-128`, `:569-570`,
> `:709`). Diagnostic scaffold preserved at **`4dfdb21d`**.
>
> ⭐ **This is DS-9 succeeding, not failing.** The frame's premise was that DS-9
> adds no component and exists to discover whether the tier composes, with
> friction as the deliverable. It found a real kernel limit at the first
> deliverable — a better result than a clean landing.
>
> **Architect ruling `dec_13af1mercv2m0` (`evt_55k9f9efvd8jk`): option B,
> nested-only.** DS-9's ordinary six-constructor `Json` is **preserved**; the
> kernel restriction is lifted instead. ⇒ `depends_on: [KERNEL-NESTED-IND]`,
> behind `SPEC-NESTED-IND`.
>
> ⛔ **Foundation stands down.** ⛔ Do not re-encode `JsonArray` to get moving —
> the ruling rejects W-shaped, `Fin n`, flattening, Church encodings, postulates,
> and extra malformed internal spine states **by name**. The scaffold is
> diagnostic evidence only, ⛔ **not** a QA or merge candidate.
>
> ⚠ **The carrier ruling below is UNCHANGED** — `List Char` still stands. This
> block is about the *value type*, not the codec's carrier.

> ## ▶ THE TIER'S ACCEPTANCE TEST
>
> Frame: [`ds-9-json-codec.md`][f], under `docs/program/wp/`. The frame is the
> executable artifact; this node carries the sequencing and the gate.
>
> **DS-1 … DS-8 are all landed.** DS-9 adds no new component — it finds out
> whether the ones already there compose.

## ✅ The carrier is RULED — `dec_3n1pp559pxrrw`, transcribed

**Option C: `List Char` is the law-bearing carrier.** Core API
`encode : Json -> List Char` / `decode : List Char -> Result JsonError Json`,
round-trip proved structurally over `Json` and transparent list operations.
`Bytes` and `List UInt8` are both **rejected as the core carrier**; convenience
shells are permitted but ⛔ **inherit no theorem by assertion**. Full ruling text
is transcribed into frame §3 — read it there, not here.

⭐ **DS-9 adds zero new trusted declarations**, and it is **not** an absolutely
trust-free theorem: the string leaf rests on the landed `axiom`
`string_to_list_char_retraction`
(`catalog/packages/Data/Text/StringBijection.ken.md:13`). `AC-8` makes that
dependence visible; `AC-9` bounds what DS-9 itself introduces.

⛔ **`bytes_concat` does NOT gate this node.** The law-bearing core does not
consume it; its missing spec entry is a separate gap.

### The measurement that produced the fork

Kept because it is why the ruling went the way it did. Measured at
`origin/main = 32b1b772`:

| measurement | where |
|---|---|
| `bytes_concat` occurs **zero times in the entire `spec/` tree** — no chapter, no registry row, no law | `spec/` (whole-tree grep) |
| `bytes_to_list : Bytes → List UInt8` is `PrimReduction::Op`, **"opaque to kernel conversion"** | `spec/10-kernel/18a-primitive-registry.md:624` |
| its bridge laws `bytes_list_roundtrip` / `list_bytes_roundtrip` are **"trusted declarations, not"** proofs | `:628-629` |

⇒ A `Json → Bytes` encoder makes the round-trip either unprovable or provable
only at a `trusted_base()` cost — against the zero-delta discipline every landed
catalog entry has held. The carrier was a component-design call, so it went to
the Architect rather than being settled in this frame.

⚠ `bytes_encode`/`bytes_decode` are **not** in the same position — the
`BytesRoundTripLaw` at `spec/30-surface/38-ffi-io.md:253` records
`∀ s. bytes_decode (bytes_encode s) == Ok s` as **provable**. The gap is `Bytes`
*concatenation*, not the `String`/`Bytes` boundary.

## ⛔ The `DS-5 → DS-9` graph edge is CUT

The program's Mermaid graph draws `DS5[DS-5 Vector] --> DS9`. **DS-5 is
spec-gated** on a `spec/50-stdlib/` `Vector` chapter that has no author and no
node, and its own program text lists it under "Deferred / prerequisites."

⇒ Honoring that edge would park the tier's acceptance test behind a spec gap it
has no need of. DS-9 uses `List`, complete since DS-4.

⚠ **That cut still holds, and it is not what blocks the node.** `depends_on` was
empty when this was written — every *catalog* prerequisite was already merged, and
that is still true. The single dependency now recorded,
`KERNEL-NESTED-IND`, is a **kernel expressiveness** prerequisite discovered during
execution, ⛔ not a revival of the `Vector` edge. Introducing a length-indexed
carrier would still be wrong, and the Architect's ruling says so explicitly:
substituting `Fin n` *"imports the deferred length-indexed carrier that the DS-9
frame expressly excludes."*

## What the tier supplies

DS-1 `Empty`/`Dec` · DS-2 `Ord Nat` · DS-3 `Option`/`Result` combinators ·
DS-4 `List` combinators + laws · DS-6 lawful `DecEq Char` → `Eq`/`Ord String` ·
DS-7 `Applicative`/`Monad` · DS-8 `Traversable` · plus the parsing floor
(`Capability/Parsing/{Cursor,Decoder,Numeric,Parsing}.ken.md`), which is
carrier-neutral by construction and already recursion-capable.

⭐ **The exemplar stops exactly where DS-9 must not.**
`Capability/Parsing/Parsing.ken.md` §4.3 builds a recursive `BoolExpr` grammar
with both a parser and a printer — and **no round-trip theorem**. Its complete
theorem list is three items, none of them about printing. So the exemplar gives
DS-9 its shape and not its proof, and the proof is the work.

## Findings are a deliverable, not a byproduct

Per the charter's routing: kernel-reduction defect → **Kernel** via the enclave;
sugar or abstraction candidate → **Ergo**; abstraction kept in-catalog →
Foundation. ⛔ A DS-9 that lands clean and files nothing has written a codec
without running the acceptance test. `AC-10` exists so that "clean" and "never
looked" cannot read identically.

## Contention

**None with Runtime.** DS-9 touches `catalog/packages/` and adds one file under
`crates/ken-elaborator/tests/`. Runtime's queue — `ABI-S3` → `RT-VALUE-TOTALITY`
P2 → `RT-FNSPLIT-C1` — is confined to `crates/ken-host`, `crates/ken-runtime`,
`crates/ken-interp`, and `crates/ken-elaborator/src`. ⚠ Frame §7 carries the
one caveat worth re-checking at branch time.

[f]: ../wp/ds-9-json-codec.md
