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
- **`Diagnostic → Doc` rendering.** It stays in `Capability.Diagnostics.Render`.

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

### 3.1 The environment is reachable — **I-7 delivers it. CC8 CONSUMES it.**

> **★ AMENDED 2026-07-14 (Steward). This section previously said "a
> `Capability.Process.Environment` accessor is a CC8 deliverable." THAT WAS AN
> OWNERSHIP ERROR AND IT IS WITHDRAWN.** The projector sits on the
> **`ProcessInput` runtime ABI**, which is **Team Runtime's** boundary, and it
> is already scoped to them by the CLI contract §7 (*"I-7 Env/Process families:
> env read/enumerate, cwd, clocks"*). Had both frames stood, **Foundation and
> Runtime would each have built `process_environment`**, on two branches, and
> whichever landed second would have collided. **Do not build it.**

`ProcessInput` is landed as
`MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes` — argv,
**environment pairs**, cwd (`prelude.rs:1417`). CC6a's `Capability.Process.Arguments`
projects field 0 only; the environment is *matched and preserved* but never
named.

**`Capability.Process.Environment` (`process_environment : ProcessInput → List (Prod Bytes
Bytes)`) is delivered by I-7** (Team Runtime,
`docs/program/wp/i7-env-process-projectors.md`). **CC8 depends on I-7 and must
not duplicate it.**

**⇒ CC8's dependency set is: I-7 (the env projector) ✅ MERGED (`9ae7acd1`) AND
SUB-1b (lawful `DecEq Bytes`) — in flight.** **The seam is clean and it is
worth stating, because it is exactly why the split works:**

- **I-7 gives you the environment as a `List (Prod Bytes Bytes)`.** Pure
  projection. No key comparison, so **no `DecEq Bytes`, no `Axiom`, no TCB.**
- **CC8 does the LOOKUP** — and a lookup compares keys, so it needs
  **`DecEq Bytes`**. *That is the whole reason CC8 is held and I-7 is not.*

> **★ AMENDED 2026-07-14: your unblock is SUB-1b, NOT SUB-1.** SUB-1 (merged,
> `6e415f23`) gives the **spine** — `bytes_to_list : Bytes → List UInt8` — which is
> all a *fold* needs. **But `UInt8` is itself an opaque type you cannot case on**,
> so `DecEq UInt8` — and therefore `DecEq Bytes` — **was still unwritable.**
> **SUB-1 moved the wall from `Bytes` to `UInt8`; it did not remove it.** *A fold
> never looks at an element; a key comparison does.* **SUB-1b** (`wp/sub1b-uint8-deceq`)
> adds the one retraction certificate that closes the chain and hands you a
> **lawful, decidable `DecEq Bytes` at zero further trust.** **Wait for it.**

### 3.2 ★★ AMENDED 2026-07-14 — YOUR KEYS ARE PLAIN `Bytes`. NO CARRIER.

> **★ THIS SECTION WAS WRITTEN FOR THE OLD SUBSTRATE AND ITS ADVICE IS NOW
> WRONG. Read the amendment; the struck-through text below is kept only so you
> can see what changed and why.**

**A cached-`Nat` carrier existed for exactly one reason: you could not reason
about `Bytes` structurally.** `bytes_length : Bytes → Int` is a `PrimReduction::Op`
— **opaque to kernel conversion** — so you could not *prove* anything about a
length; you cached a `Nat` alongside the bytes and carried an agreement proof to
get a fact you could actually use. **That wall is being torn down:**

- **SUB-1 (landed, `origin/main @ 6e415f23`)** — `bytes_to_list : Bytes → List
  UInt8` + the round-trip postulates. `bytes_nat_length = length ∘ bytes_to_list`
  is now a **real structural fold**: it computes *and* you can reason about it,
  **with no cached carrier and no `Axiom`.**
- **SUB-1b (in flight, `wp/sub1b-uint8-deceq`)** — **lawful `DecEq Bytes`**, at
  one audited trusted entry. **This is your unblock**: a lookup compares keys, and
  now the comparison is *decidable and provably correct*.

**⇒ An environment key is a plain `Bytes`, compared with `DecEq Bytes`.**

1. **⛔ DO NOT mint a cached-`Nat` carrier for env or config keys.** Not a new
   one, and **do not extend the `ArgBytes` idiom to a new key type.** The idiom is
   a **workaround for a wall that no longer stands at the key layer**, and
   **SUB-2 exists to retire the ones we already have** — do not add to the pile it
   has to clear.
2. **CC3's `Cursor` ABI takes plain `Bytes`.** `ArgBytes` and its cached-length
   carrier were **retired by SUB-2** and no longer exist anywhere in the catalog.
   **Consume `Bytes` directly. There is no carrier to pass and none to build.**
3. **If you find yourself wanting a cached length for a NEW type, STOP AND REPORT.**
   That is the signal the substrate is still short somewhere, and I want to see it
   rather than have another carrier quietly minted. *(That is precisely how the
   fifth carrier would have been born.)*

---

<details><summary>SUPERSEDED — the original §3.2 (old substrate). Do not follow.</summary>

### ~~Reuse the landed byte carrier — do NOT mint a fifth~~

`ArgBytes` (`Capability/Parsing/Cursor.ken.md:51–62`) is **arg-*named* but structurally
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

</details>

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

1. ~~**`Capability.Process.Environment`** — the missing projector.~~ **WITHDRAWN — this is
   I-7's (Team Runtime), not CC8's.** See §3.1. **CC8 consumes
   `process_environment`; it does not build it.**
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

- **✅ `DecEq Bytes` is LANDED and LAWFUL** (`BytesKeys.ken.md:109`,
  `origin/main @ 82cb8fd0`) — `sound` + `complete`, transported from the
  kernel's `DecEq Int` certificate at exactly one audited trusted-base entry
  (`uint8_int_retract`). **An environment/config key is a plain `Bytes`,
  compared with `DecEq Bytes`. USE IT.**
- **⛔ Still forbidden: no NEW primitive, no NEW postulate, no `Axiom`, no
  `Ord Bytes`, no `Int → Nat`, no kernel change. ZERO `trusted_base()` delta**
  — you **consume** the law, you do not add to it. **The family-wide
  opaque-primitive fork remains PAT'S and is untouched by this WP.**
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
