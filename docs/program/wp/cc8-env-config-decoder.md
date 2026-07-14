# CC8 — Environment/config decoder, and the `Schema` extraction

**Owner:** Team Foundation · **Size:** L · **Base:** `origin/main @ c5f73b9c`
**Branch:** `wp/cc8-env-config-decoder`

## 1. Objective

Build the **second description-driven decoder** — environment and config — and,
**because it is the second and not before**, extract the shared `Schema` that
CC7 and CC8 genuinely have in common.

**Two real consumers, then extract.** That rule is why CC7 was forbidden from
inventing a `Schema` on its own. CC8 is the second consumer, so the extraction
is now legitimate — **but only of what both decoders actually share.**

## 2. The `Schema` boundary — pinned from YOUR retros, not from theory

I asked all three of you what a second decoder would justify extracting. Your
answers converged almost word for word, so **this boundary is yours, and I am
pinning it as a fixed input.**

### 2.1 EXTRACT into the shared `Schema`

- The **declarative description vocabulary**: field identity, presence /
  requiredness, value-shape, and documentation metadata.
- The **generic traversals over that vocabulary that BOTH clients run** — the
  validation traversal and the help/doc-derivation traversal.

### 2.2 LEAVE LOCAL — do NOT unify these

*(QA, verbatim: "leave source-specific acquisition/tokenization, raw `Bytes`
versus config representation, `ArgumentOrigin`/environment/config provenance,
validation policy, and `Diagnostic → Doc` rendering at their existing
boundaries.")*

- **Acquisition / tokenization.** `ArgCursor` byte-matching stays in `ArgParse`;
  environment and config lookup stays in CC8.
- **Raw-value carriers.** Raw argv `Bytes` vs. the config representation.
- **Provenance / origin.** `ArgumentOrigin` vs. environment/config origin.
- **Validation policy.**
- **`Diagnostic → Doc` rendering.** It stays in `Diagnostic.Render`.

*(Implementer, verbatim: "do not unify merely similar carriers or erase their
source-specific diagnostics.")*

### 2.3 The acceptance witness that makes the extraction non-vacuous

**Both of you independently demanded the same test, so it is AC1:**

> **ONE change to a shared `Schema` must reach BOTH decoders** — with **no**
> second parser and **no** second location carrier.

*(Implementer: "Call it a shared `Schema` only after acceptance shows both
clients drive the same traversal.")* An extraction that only CC8 uses is not an
extraction; it is a new package with an aspirational name.

## 3. Fixed inputs — grounded at `origin/main @ c5f73b9c`

Verified via `git show origin/main:<file>`, **not** a worktree read. Treat them
as perishable anyway (§7).

### 3.1 The environment is NOT reachable yet — you must expose it

`ProcessInput` is landed as
`MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes` — argv,
**environment pairs**, cwd (`prelude.rs:1417`).

**But CC6a's `Process.Arguments` exposes only argv.** `process_arguments`
projects field 1; the environment is *matched and preserved* but **never
projected** (`Arguments.ken.md:14–16`). **There is no `process_environment`.**

⇒ **A `Process.Environment` accessor is a CC8 deliverable.** Mirror CC6a's
landed shape exactly — projector, `replace_*`, and the `round_trip` proof
(`Arguments.ken.md:19–34`). Do not invent a different idiom for the sibling
field.

### 3.2 Reuse the landed byte carrier — do NOT mint a fifth

`ArgBytes` (`Parsing/Cursor.ken.md:51–62`) is **arg-*named* but structurally
generic**:

```ken
fn ArgByteLength (bs : Bytes) (n : Nat) : Prop =
  Equal Int (bytes_length bs) (cursor_nat_to_int n)

class ArgBytes {
  arg_bytes_field : Bytes;
  arg_length_field : Nat;
  arg_length_valid_field : ArgByteLength arg_bytes_field arg_length_field
}
```

It is nothing more than **`Bytes` + a cached `Nat` length + the agreement
proof** — there is nothing argv-specific in it, and it is exactly what an
environment key or a config key needs.

**⇒ Reuse it.** Do **not** build a second cached-`Nat` carrier. If you judge
that a rename/generalization (e.g. to a neutral `MeasuredBytes`) is genuinely
warranted, **propose it to me — do not fork.** A rename is an exported-name
migration and needs a **whole-harness consumer inventory** (`ArgParse`, the CC7
acceptance harness, the Cursor tests), which is a scope call, not an
implementation detail.

### 3.3 `Schema` must not depend on its clients (the CC3 cycle, again)

`Schema` is an **abstraction**. It must take **no dependency** on `ArgParse` or
on the config decoder. It holds the vocabulary and traversals **parameterized
over the origin and value types**, and it defines its **own parameterized
result/error carriers**. **The moment it reaches for `ArgumentOrigin` or a
concrete `ParsedArgument`, the cycle is back** — that is the exact bug I shipped
in the CC3 frame and that you caught pre-edit.

**Load order: `Schema` before `ArgParse`, and before the env/config decoder.**

**⇒ CC8 REFACTORS `ArgParse` to consume `Schema`.** Touching CC7's package is
expected and correct — it is what makes CC8 the *second consumer* rather than a
parallel universe. **An extraction nobody refactors onto is not an extraction.**

## 4. Mandated deliverable

1. **`Process.Environment`** — the missing projector for `ProcessInput`'s second
   field, in CC6a's landed idiom (project / replace / `round_trip`).
2. **The shared `Schema` package** — vocabulary + generic traversals per §2.1,
   parameterized, client-independent (§3.3).
3. **The env/config decoder** — consuming `Schema`, `Cursor`/`Decoder` (CC3),
   `Validation` (CC1), `Diagnostic` (CC4), and the codecs (CC2), with its **own**
   origin/provenance type.
4. **`ArgParse` refactored onto `Schema`** — the second consumer, and the proof
   the extraction is real.

## 5. Acceptance criteria

- **AC1 — the two-consumer witness (§2.3).** One `Schema` change reaches **both**
  decoders. No second parser. No second location carrier.
- **AC2 — both clients DRIVE the same traversal** — they do not merely declare
  it. *This is CC7's AC6, and it is the AC that separates a compounding catalog
  from a fragmenting one.* Foundation's own CC7 carry: *"a package can appear to
  reuse a substrate merely because the ordered shared environment loads it; the
  non-vacuous test was behavioral."* Grep the call sites, and confirm the run
  path executes the shared traversal.
- **AC3 — byte preservation through the full pipeline.** A **genuinely invalid
  UTF-8** environment *value* survives byte-identically. Use real invalid bytes
  (CC6a/CC7 used `[ff fe 80 61]`); a UTF-8-only fixture is green-vs-green and
  cannot catch a quiet decode. **Never decode an env/config *value* to
  `String`.** (`String` is fine for the *program's own* spec/help literals — the
  Architect verified that boundary in CC7.)
- **AC4 — errors accumulate.** Two independent bad inputs produce **two**
  diagnostics with exact origins, via `Validation` — not first-error
  short-circuit.
- **AC5 — help/documentation is derived from the schema.** Add a field to the
  schema and it appears in the rendered output **with no other edit**.
- **AC6 — no second universe.** No new parser, no new error carrier, no new
  renderer, **no new cached-`Nat` carrier**. `trusted_base()` before == after.
- **AC7 — corpus oracles.** Run **both**: `crates/ken-cli/tests/ken_fmt.rs` and
  `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`, plus the live fixed-point
  oracle. **Add NO `FRAME_LINE_COUNTS` row** — that table is a discharged
  historical baseline, and a file created after it has no honest pre-frame line
  count, so a fabricated row makes its check vacuous forever.

## 6. Guardrails — do not reopen

- **⛔ Do NOT settle the `Bytes → Nat` question.** No `bytes_eq`, no
  `DecEq Bytes`, no `Ord Bytes`, no `Int → Nat`. **It is an open operator
  decision, and a build WP does not get to pre-empt it.** Byte-wise key matching
  goes through the landed route: `ArgBytes` + `bytes_at` + `uint8_to_int` +
  `eq_int` — exactly as CC7 did. **If you think you need one, STOP and
  escalate.** Three WPs in a row have escalated instead of inventing. Be the
  fourth.
- **No reflection, no macros, no derivation in v1.** Explicit schemas.
- **No `import`/`pub` smuggling.** No new primitive. Zero trust delta.
- **Do not read a config file.** CC8 decodes `Bytes` that are *handed to it*
  (environment from `ProcessInput`, config content as `Bytes`). FS acquisition
  is not in scope — `Resource`/`Bracket` is CC9.

## 7. The clause that has caught six bad pins of mine

**Treat every anchor above as perishable.** I drafted the sibling I-6 frame from
a worktree **10 commits behind `main`**, which silently invalidated every line
number in it — the findings held, the anchors did not. **Read the ref
(`git show origin/main:<file>`), never a worktree.**

**If a fixed input is false against the landed code, say so with exact tree
anchors and escalate. Do not quietly build around it.** Your ring has now caught
a dependency cycle, an unproducible field, and a semantic-field guess in my
frames — every one of them pre-edit. **I would rather be corrected than
believed.**

Module layering, instance homing, and scope route to the **Steward**. Soundness
routes to the **Architect**.
