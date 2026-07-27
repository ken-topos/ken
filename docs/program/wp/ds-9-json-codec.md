# DS-9 · lawful JSON codec — the data-structures tier's acceptance test

**Owner:** Foundation. **Size:** L. **Program:**
`docs/program/wp/catalog-data-structures-program.md` §"Phase 3 — the driver
validates the tier". **Charter:** `docs/program/06-catalog-campaign.md`.

> ## ⛔⛔ BLOCKED 2026-07-27 — READ THIS BEFORE ANYTHING ELSE
>
> **`Json` is not expressible in the current kernel.** `D1`'s ordinary spelling
>
> ```ken
> data Json = ... | JsonArray (List Json) | JsonObject (List (Pair String Json)) | ...
> ```
>
> is rejected as a **nested inductive** — the `List (Rose A)` class that
> `spec/10-kernel/14-inductive.md` §8.5 defers. Diagnostic scaffold preserved at
> **`4dfdb21d`**; it is evidence only, ⛔ **not** a QA or merge candidate.
>
> **Architect `dec_13af1mercv2m0` (`evt_55k9f9efvd8jk`) — option B, nested-only.**
> DS-9's ordinary **six-constructor** `Json` is **preserved**. The kernel
> restriction is lifted instead, via `SPEC-NESTED-IND` → `KERNEL-NESTED-IND`.
>
> ### ⛔ Substitutes the ruling rejects BY NAME — do not reach for any of them
>
> | rejected | why, per the ruling |
> |---|---|
> | `JsonArray Nat (Nat -> Json)` (W-shaped) | *"not the same carrier"* — values beyond the declared length are **semantically ignored**, so exact round-trip/equality depends on **function extensionality and a noncanonical suffix** |
> | `Fin n` instead of `Nat` | *"imports the deferred length-indexed carrier that the DS-9 frame expressly excludes"* |
> | splitting the **object** arm into functions | the object arm *"still nests `Json` through `Pair`"* — same finite-index and equality problem |
> | dropping arrays | *"removes JSON arrays and therefore fails `D1` and the purpose of DS-9"* |
> | flattening · Church encodings · postulates · extra malformed internal spine states | *"likewise not authorized substitutes"* |
>
> ⭐ **Why the W-shape is not a near-miss but a different thing:** it would
> *"replace the composition target rather than discharge it."* DS-9's whole purpose
> is to find out whether the **landed tier** composes. Re-encoding the value model
> to dodge the kernel would answer a question nobody asked.
>
> ⚠ **§3's carrier ruling is UNCHANGED** — `List Char` still stands. This block
> concerns the **value type**, not the codec's carrier. ⛔ Do not conflate them.

> ## ▶ READ THIS FIRST — what makes this WP different
>
> DS-1 … DS-8 each *added* a component. **DS-9 adds nothing the tier does not
> already have** — a `Json` value type, `encode`, `decode`, and the proved
> round-trip law, assembled from landed Core/Data packages. Its job is to find
> out whether the tier composes.
>
> ⇒ **Friction is the deliverable, not the failure.** Every place the assembly
> is harder than it should be is a **Finding** filed per the charter's routing
> (kernel-reduction defect → Kernel via the enclave; sugar or abstraction
> candidate → Ergo). A DS-9 that lands clean *and files nothing* has not run the
> acceptance test — it has only written a codec.
>
> ⛔ **The round-trip law is new work, not a transcription.** §2c measures why.

## 1. Fixed inputs — settled, do not reopen

All measurements below were taken at **`origin/main = 32b1b772`**. Re-derive §2
if you start from a later base.

1. **The value model is `Json` as an ordinary Ken inductive type.** No kernel
   primitive, no new elaborator capability, no language-semantics change. Every
   landed catalog entry holds a **zero `trusted_base()` delta**; DS-9 holds it
   too.
2. **DS-5 (`Vector`) is NOT a prerequisite.** The program's Mermaid graph draws
   `DS5 --> DS9`, but DS-5 is spec-gated on a `spec/50-stdlib/` `Vector` chapter
   that has no author and no node. **That edge is cut here.** DS-9 uses `List`,
   which is landed and complete after DS-4. ⛔ Do not introduce a length-indexed
   carrier to satisfy a graph edge — it would make this WP wait on a spec gap it
   has no need of.
3. **Format is `.ken.md`** per `docs/program/07-catalog-style-guide.md`, with the
   standard `## Contents` section (`DOC-CATALOG.RQ-1`).
4. **Two-phase cadence applies** (`06-catalog-campaign.md`): the functional build
   — proofs real, trusted base honest — may merge first; guide-quality prose is a
   refinement follow-on. ⛔ This is not licence to merge a *postulated* law; see
   `AC-9`.
5. **The carrier ruling in §3 is an Architect ruling, transcribed.** It is a
   fixed input, not a design option the ring re-opens.

## 2. Measured substrate — what is already built (do not rebuild)

### 2a. The parser floor is complete and carrier-neutral

`catalog/packages/Capability/Parsing/Cursor.ken.md`:

```
:13   data CursorOps c el loc = MkCursorOps (c → Nat) (c → Option el) (c → c) (c → loc)
:215  fn CursorPeekHasRemaining …
:228  fn CursorAdvanceProgress …
:241  fn CursorEndValid …
:250  fn CursorLaws (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) : Prop
:198  MkCursorOps …          — the ArgCursor instance (over List Bytes)
```

⭐ **`CursorOps` is parameterised over the carrier `c`, the element `el`, and the
location `loc`.** Exactly two instances exist today — `arg_cursor_ops` (over
`List Bytes`, `:198`) and `byte_cursor_ops` (over a `Source`,
`Parsing.ken.md:152`). Supplying a third is the intended extension point, not a
workaround.

### 2b. The decoder combinators are progress-safe and recursion-capable

`catalog/packages/Capability/Parsing/Decoder.ken.md`:

```
:14   data DecoderError loc = DecoderRejected loc | DecoderZeroProgress loc
                            | DecoderFuelExhausted loc
:16   data DecoderResult c loc a = Decoded a c | DecoderFailed (DecoderError loc)
:27   decoder_pure   :30 decoder_fail   :35 decoder_map   :44 decoder_bind
:58   decoder_seq    :68 decoder_alt    :82 decoder_satisfy  :95 decoder_token
:105  decoder_many_fuel   :153 decoder_many   :163 decoder_some
:192  decoder_recursive_fuel   :207 decoder_recursive
:225  DecoderProgress  :244 DecoderConsumesAll  :263 DecoderRejectsOnlyAtEnd
:281  DecoderManyConsumesAllLaw
```

⭐ **`decoder_recursive` is the JSON enabler, and its fuel discipline is already
the right one.** `:207` seeds the fuel from `cursor_remaining` and `:192`
decrements once per *layer application*. Every JSON nesting level consumes at
least its opening delimiter byte, so **nesting depth ≤ initial remaining** and
well-formed input can never spuriously hit `DecoderFuelExhausted`. ⚠ That
sentence is an *argument*, not yet a theorem — `AC-6` is where it becomes one.

⭐ **Zero-progress is already a named, non-backtrackable failure** (`:14`,
`:105`). A JSON array/object repetition therefore cannot silently loop or
truncate. Reuse `decoder_many`; ⛔ do not hand-roll repetition.

### 2c. The exemplar exists — and it stops exactly where DS-9 must not

`catalog/packages/Capability/Parsing/Parsing.ken.md` §4.3 builds a recursive
grammar end to end:

```
:347  data BoolExpr = BTrue | BFalse | BNot BoolExpr | BAnd BoolExpr BoolExpr
:719  const parse_bool_expr : Parser (Syntax BoolExpr)
:722  fn print_bool_expr (e : BoolExpr) : Bytes
:738  fn format_bool_expr (s : Source) : Result ParseError Bytes
:252  fn parser_from_decoder (a : Type) (decoder : Decoder ByteCursor Span a) : Parser a
```

⭐⭐ **It has a printer and a parser and NO round-trip theorem.** The package's
complete theorem list is three items — `span_to_byte_range_faithful` (`:108`),
`span_origin_source_faithful` (`:117`), `valid_zero_width_span` (`:184`). Its own
§7 trust surface confirms it: `print_bool_expr` and `format_bool_expr` are
exported as functions with no law attached.

⇒ **The exemplar gives DS-9 the shape and not the proof.** Copying its structure
and then "adding the theorem" is the shape of a WP that discovers in its last
hour that the theorem is the whole job. Plan the proof first (§4 `D4`).

### 2d. Why the printer stops there — the wall, measured

1. `print_bool_expr` is built from `bytes_concat` and `bytes_encode`.
   **`bytes_concat` occurs zero times in the entire `spec/` tree** — no chapter,
   no primitive-registry row, no stated law. Landed catalog code uses it with no
   spec authority behind it.
2. `spec/10-kernel/18a-primitive-registry.md:624` declares
   `bytes_to_list : Bytes → List UInt8` as `PrimReduction::Op`, **"opaque to
   kernel conversion."**
3. `:628-629` states its bridge laws `bytes_list_roundtrip` and
   `list_bytes_roundtrip` are **"trusted declarations, not"** proofs.

⇒ A round-trip law routed through `Bytes` concatenation is either unprovable or
provable only at a `trusted_base()` cost. **§3 rules `Bytes` out as the core
carrier for exactly this reason** — the measurement above is why the ruling went
the way it did, and it is kept here so the ruling is auditable rather than
merely obeyed.

⚠ `bytes_encode`/`bytes_decode` are **not** in the same position: the
`BytesRoundTripLaw` at `spec/30-surface/38-ffi-io.md:253-317` states
`∀ s. bytes_decode (bytes_encode s) == Ok s` and records it as **provable**. The
`String`/`Bytes` shell is sound; the `Bytes` *concatenation* is the gap.

### 2e. What the tier supplies for the codec body

| need | landed source |
|---|---|
| decision procedures, `Empty`/`absurd` | `Core/Logic/EmptyDec.ken.md` (DS-1) |
| `Ord Nat`, `Nat` operations | `Data/Numeric/Nat/Order.ken.md` (DS-2) |
| `Option`/`Result` combinators | `Data/Sums/Combinators.ken.md` (DS-3) |
| error accumulation with a `Semigroup e` | `Data/Sums/Validation.ken.md` |
| `reverse`/`zip`/`concat_map`/`range`/`foldl` + laws | `Data/Collections/Derived.ken.md:661-675` (DS-4) |
| `DecEq Char`, `Eq`/`Ord String` | `Core/Classes/LawfulClasses.ken.md:463`, `Data/Text/StringKeys.ken.md:84` (DS-6) |
| `Semigroup`/`Monoid`/`Functor`/`Foldable` | `Core/Classes/LawfulFunctors.ken.md` |
| `Applicative`/`Monad`/`Traversable` | `Core/Classes/EffectfulClasses.ken.md:1241` (DS-7/8) |
| UTF-8 and ASCII views, ASCII classifier | `Data/Text/Codec.ken.md` |
| decimal parse to arbitrary-precision `Int`, located errors | `Capability/Parsing/Numeric.ken.md` |
| `String` ↔ `List Char`, injectivity | `Data/Text/StringBijection.ken.md:16` |

⚠ **`Data/Collections/Derived.ken.md:1382-1384`** records that `DecEq String` /
`Ord String` needed a lawful `DecEq Char` and that it **is now landed** in
`LawfulClasses`. Object keys can therefore carry a lawful ordering. ⛔ Verify
that at your base before relying on it — that line is a claim in prose.

## 3. ✅ THE CARRIER RULING — Architect, `dec_3n1pp559pxrrw` RESOLVED

Transcribed from `evt_4mt4bbxrqhay3`, verified `resolved` from the decision
object. **This is a fixed input. Do not re-open it.**

> **OPTION C — `List Char` is the law-bearing carrier.** The proof-bearing core
> is:
>
> ```ken
> encode : Json -> List Char
> decode : List Char -> Result JsonError Json
> ```
>
> and `decode (encode j) = Ok j` is proved **structurally over `Json` and
> transparent list operations**. `CursorOps` is already carrier-neutral, so DS-9
> supplies a **character** cursor without changing the parser abstraction. The
> existing `ByteCursor` chose byte offsets for its *diagnostic* contract; **that
> does not make `Bytes` the authority for a codec proof.**
>
> **JSON string values and object keys may remain `String`.** Their round-trip
> leaf uses the already-landed `string_to_list_char_retraction` certificate.
> ⇒ DS-9 adds **zero new trusted declarations**, but the report must state
> honestly that this case is **proved relative to that existing `String`
> certificate** — it is **not** an absolutely trust-free theorem.
>
> **Rejected core carriers.**
> - ⛔ **Not `List UInt8`** — that puts UTF-8 validity/encoding and the opaque
>   `String`/`Bytes` bridges *inside the central law*, and it weakens the
>   driver's intended exercise of the landed `Char`/`String` layer.
> - ⛔ **Not `Bytes`** — `bytes_concat` has no spec authority, the relevant byte
>   operations are opaque to kernel conversion, and the structural-view inverses
>   are trusted declarations. A green theorem there would be
>   **postulate-dependent by construction**.
>
> **Outer shells are separate APIs.** Separately named convenience shells such
> as `encode_string`/`decode_string` and `encode_bytes`/`decode_bytes` are
> allowed, but **neither inherits the core theorem by assertion**.
> - A `String`-shell law needs a **specialized** proof that every `encode j`
>   output is **already canonical** under the NFC-normalizing conversion. ⛔ **The
>   general `List Char → String → List Char` inverse is FALSE.**
> - A `Bytes`-shell law must **expose** its dependence on the existing
>   UTF-8/`Bytes` trusted boundary.
>
> ⛔ **Do not spec-gate DS-9 on `bytes_concat`** — the law-bearing core does not
> consume it. Its missing spec entry is a separate gap, not a blocker here.

### What the ruling means for the trust report

⚠ **`string_to_list_char_retraction` is an `axiom`, not a theorem.**
`catalog/packages/Data/Text/StringBijection.ken.md:13` declares it, and `:48`
records it as *"the one named postulate selected by the operator."* The ruling's
wording is exact and this frame keeps it exact: **zero *new* trusted
declarations, and not a trust-free theorem.**

⛔ **Report the string leaf's dependence as a positive statement, not as
silence.** "Zero `trusted_base()` delta" is true and, on its own, misleading — a
reader concludes the round-trip is unconditional. The entry's "Trust &
derivation" section names the certificate, says which case rests on it, and says
which cases do not.

⭐ **Why the NFC point is sharp rather than pedantic.**
`spec/30-surface/38-ffi-io.md:265-266` records that `String` construction
**NFC-normalizes**, and that normalization is
what makes the general round-trip through `String` false: a `List Char` that is
not already NFC comes back different. The `String` shell is therefore only sound
because `encode`'s *own output* is canonical — which is a property of `encode`
that must be **proved**, not assumed from the core theorem.

## 4. Deliverables

- **`D1` · the `Json` value type.** An ordinary inductive covering null,
  booleans, numbers, strings, arrays, objects. ⭐ **The number representation is
  a design decision with a Finding attached**: JSON numbers are decimal with
  optional fraction and exponent, and `Capability/Parsing/Numeric.ken.md` parses
  *integers*. State what DS-9 accepts, prove the round-trip over exactly that,
  and file the remainder as a scoped residual — ⛔ not as a silent narrowing.
- **`D2` · a `Char` `CursorOps` instance**, per §3 — `CursorOps (List Char) Char
  loc` — with **`CursorLaws` proved for it** (`Cursor.ken.md:250`). ⛔ An instance
  without its laws is the inert-artifact shape; the decoder's guarantees are
  *conditional* on `CursorLaws` and are worth nothing without it. ⭐ Adding a
  third `CursorOps` instance is the abstraction working as designed — ⛔ do not
  reach for `ByteCursor` because it is the one the exemplar used.
- **`D3` · the core `encode` / `decode`**, at exactly the §3 signatures.
  `decode` built from the landed combinators (`§2b`) via `decoder_recursive`;
  `encode` structurally recursive on `Json`, producing `List Char`.
- **`D4` · the round-trip theorem** — `decode (encode j)` yields `j`, for every
  `j` in `D1`'s stated domain. ⭐ **Plan this before writing `D3`.** The standard
  route is a *prefix* lemma — the decoder applied to `encode j` followed by any
  remainder consumes exactly `encode j` and leaves the remainder — which is what
  makes the induction go through at the array and object cases. Discover that
  after the codec is written and `D3` gets rewritten.
- **`D5` · fuel sufficiency** — the §2b argument as a theorem: for well-formed
  input, `decoder_recursive`'s seeded fuel is never exhausted.
- **`D5a` · the shells, if you ship them — each with its OWN law or none.**
  `encode_string`/`decode_string` and `encode_bytes`/`decode_bytes` are permitted
  by §3 and ⛔ **neither inherits `D4` by assertion.** A `String` shell needs a
  *specialized* proof that `encode`'s output is already NFC-canonical; a `Bytes`
  shell must name its dependence on the UTF-8/`Bytes` trusted boundary. ⭐ **A
  shell with no law is a perfectly good deliverable** — export it as a plain
  convenience function and say it carries no round-trip guarantee. ⛔ What is not
  acceptable is a shell whose docs imply the core theorem covers it.
- **`D6` · the acceptance test**, following the landed pattern in
  `crates/ken-elaborator/tests/ds3_sum_combinators_acceptance.rs`: elaborate the
  `.ken.md` through `ElabEnv::elaborate_ken_md_file`, assert the laws are **real
  globals**, measure the `trusted_base()` delta, and include **discriminating
  accept→reject controls**. ⛔ Per the operator's test policy (2026-07-26) these
  assert *behavior* through the elaborator — ⛔ not source-text or line facts.
- **`D7` · the Findings file.** Every point of friction, routed per the charter.
  ⭐ This is a required deliverable of the acceptance test, not a courtesy.

## 5. Acceptance criteria

Each names its positive control — the mutation that must flip it red.

| AC | claim | positive control |
|---|---|---|
| `AC-1` | `Json` elaborates and every constructor is a real global. | rename one constructor in the fence → the global lookup fails |
| `AC-2` | The codec's `CursorOps` instance has `CursorLaws` **proved**, not postulated. | replace the `CursorAdvanceProgress` witness with a hole → elaboration rejects |
| `AC-3` | `decode` rejects malformed input at the **typed boundary** with a named `DecoderError`, for each of: unterminated string, trailing comma, bare `NaN`, unclosed array, unclosed object, duplicate object key. | feed each malformed input to a build that classifies it as `DecoderRejected` at the wrong offset → the asserted variant/offset pair differs |
| `AC-4` | `decode` on a **zero-progress** step fails `DecoderZeroProgress` rather than looping or truncating. | substitute a step decoder that consumes nothing → the assertion must observe `DecoderZeroProgress`, not `Decoded … (Nil …)` |
| `AC-5` | The round-trip theorem `D4` is a real kernel-checked `theorem`, reported **per Json constructor** — one row each for null, bool, number, string, array, object. | neuter the array case's induction step → **only** the array row reddens; ⛔ an aggregate pass hides one defecting constructor |
| `AC-6` | `D5` fuel sufficiency is a theorem, exercised on input nested deeply enough to distinguish it. | halve the seeded fuel in a probe build → `DecoderFuelExhausted` appears, proving the test input actually reaches the bound |
| `AC-7` | Object keys use the **landed lawful** `Ord String` / `DecEq String`. | swap in a structurally-equal but unproved comparison → the law's proof term fails |
| `AC-8` | `trusted_base()` delta is reported **as a number** for the new file, **and** the entry names `string_to_list_char_retraction` as the certificate the string leaf rests on. | add a stray `Axiom` to the fence → the delta assertion reddens |
| `AC-9` | Zero `Axiom` / `postulate` / `sorry` **declared in DS-9's own file**. ⚠ Scoped deliberately — see below. | add an `axiom` to the fence → reddens; ⛔ the control must **not** redden merely because an imported certificate is reachable |
| `AC-10` | `D7` Findings filed, **or** an explicit written statement that the assembly produced none. | ⚠ this AC has **no mechanical control** — it is discharged by the report, and it is listed so that "clean" and "never looked" cannot read identically |

⚠ **Why `AC-9` is scoped to DS-9's own file, and not to reachability.** The
ruling permits the string leaf to rest on `string_to_list_char_retraction`, which
is an `axiom` in a *landed* package. An `AC-9` written as "no postulate anywhere
in the proof's dependency closure" would therefore be **unsatisfiable by
construction** — it would block a WP for doing exactly what the Architect ruled
it should do. ⇒ `AC-9` bounds what DS-9 **introduces**; `AC-8` is what makes the
inherited dependence **visible**. ⛔ Neither one alone is enough, and swapping
either for the other silently drops half the property.

⛔ **`AC-5` is the WP.** If it cannot be discharged, DS-9 has found the tier's
real limit and that outcome is a **legitimate, valuable result** — report it,
scope the theorem to what is provable, and file the residual. ⛔ Do **not**
postulate the round-trip to make the row green.

## 6. Validation — targeted only

⛔ **NEVER `--workspace`** (operator hard rule, `agent/COORDINATION.md §12`).
Locally, run only:

```sh
scripts/ken-cargo test -p ken-elaborator --test ds9_json_codec_acceptance
scripts/ken-cargo test -p ken-elaborator --test kenfmt_b1_lossless
scripts/ken-cargo test -p ken-elaborator --test catalog_taxonomy
```

The second is the whole-catalog literate-fence byte-exact round-trip
(`kenfmt_b1_lossless.rs:96`) — a new `.ken.md` is in its scope. The third is the
package-root path lint (`catalog_taxonomy.rs:28`); a new package under
`catalog/packages/Data/` must satisfy the controlled-sections rule.

**The full-workspace build, `--locked`, and conformance run in CI on GitHub.**
"No regression" means green in CI, never a local full run.

## 7. Contention

**None with Runtime.** DS-9 touches `catalog/packages/` and one new file under
`crates/ken-elaborator/tests/`. Runtime's queue — `ABI-S3`, then
`RT-VALUE-TOTALITY` P2, then `RT-FNSPLIT-C1` — is confined to `crates/ken-host`,
`crates/ken-runtime`, and `crates/ken-interp`. ⚠ `ABI-S3` does touch
`crates/ken-elaborator` at three elaborator sites (per `evt_4rc0b25k59a6s`), but
in `src/`, not `tests/`, and DS-9 adds a new test file rather than editing one.

⚠ **Check before branching**, since these move: no other in-flight WP is editing
`Data/Collections/Derived.ken.md`, `Core/Classes/LawfulClasses.ken.md`, or
`Capability/Parsing/*.ken.md` — DS-9 reads them all through `include_str!` and a
concurrent edit changes what its `base_env()` elaborates.

## 8. Reporting

Report against §5 row by row, `AC-5` **per constructor**. State the
`trusted_base()` delta as a number. Name what `D1`'s number domain excludes.
File `D7` before requesting review — the Findings are the acceptance test's
output, and a review that lands the codec without them has graded the wrong
artifact.
