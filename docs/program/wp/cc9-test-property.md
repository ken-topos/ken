# CC9-Tooling.Testing.Property — the minimal, provisional property-test scaffold

**Owner:** Team Foundation · **Size:** S/M · **Base:** `origin/main @ 0fa72ff9`
**Branch:** `wp/cc9-test-property`
**Gate:** Foundation QA + **Architect §14** (the only real soundness axis here is
**zero `trusted_base()` delta** — verify it). **No CV vote** unless you add a
`conformance/` fixture (this WP should not). **FULL CI** (touches `catalog/`).
Publisher on a RESOLVED Decision.

> ### This is the FIRST half of a SPLIT CC9. The second half is HELD.
> Grounding surfaced that the original CC9 bundled two packages with **opposite
> durability postures**. Only the first is released here:
> - **`Tooling.Testing.Property` — RELEASED (this frame).** Minimal, provisional test
>   scaffolding over existing capabilities; depends only on landed
>   Parsing/`Bytes`; **no PX7 dependency.** "A language you cannot test in isn't
>   a tool" — this is why CC9 is next.
> - **`Resource`/`Bracket` — HELD, not in this frame.** The operator reserved
>   the CC9-Resource/Bracket frame for a re-read **against PX7**
>   (`09-posix-linux-abi-campaign.md:573-578`) so the bracket shape isn't built
>   twice. FS today is whole-file (no OS handles); the handle-bracket is **PX7**,
>   which sits behind PX3/4/5/6 (all unbuilt). Do **not** build `Resource`,
>   `Bracket`, `with`, `finally`, or any resource-acquire/release combinator in
>   this WP. FS file-acquisition stays deferred until that half ships.

## 1. Objective

Build the **smallest honest property-test scaffold** the catalog can use to
*exercise* laws it has already **proved** — a `Gen` generator abstraction, a
property runner, an arbitrary-byte generator, and **one** end-to-end acceptance
witness — expressed as **ordinary Ken over the built-ins**, with **zero**
`trusted_base()` delta and **no** test-only kernel or grammar primitive.

This is **provisional scaffolding.** The federation's resource/property-safety
story moves to **Ward** behind the one-way `Ken → Ward` seam (ADR-0006); Ward
later assumes this role behind the *same* catalog seam. So the whole point of
this WP is to establish that seam **without a language change** — a plain
catalog package, subsumable later, entrenching **nothing** in the surface.

**Minimal means minimal.** The deliverable is the generator/runner core + the
byte generator + one non-vacuous witness. It is **not** a comprehensive property
suite over the whole catalog. Additional witnesses are welcome only where they
are cheaply constructible from an already-landed surface (§4); breadth is a
later refinement WP, not this one.

## 2. The line this package must not cross — the anti-postulate rule

**This is the load-bearing guardrail; read it before writing a line.**

`spec/50-stdlib/59-parsing-syntax-diagnostics.md:31-33` is normative: a package
that **postulates** parser totality, span validity, or round-trip laws has
**failed the v1 contract**; `trusted_base()` delta is expected **empty**. The
catalog's whole discipline is *laws carried as propositions — **proved**, not
postulated* (`catalog/packages/README.md`, "laws-PROVED discipline").

A property-test framework is the one package that could quietly **invert** that
discipline — by re-stating a law the catalog is obligated to prove as a
"property" it merely *asserts*, or by `postulate`/`Axiom`/holing a `Prop` and
calling the hole a test. **That is the failure mode this frame forbids.**

Concretely, a `Tooling.Testing.Property` "property" is an **empirical, decidable check**:

- a **generator** produces concrete sample inputs (real `Bytes`, real values);
- a **decidable predicate** (`a → Bool`, or a `Result`/`Validation` you can
  compute) evaluates the **computational shadow** of a law on each sample;
- the **runner** folds that predicate over the samples and returns *held* or the
  **first counterexample**.

It **never** takes the `Prop`-level law as a hypothesis, re-exports it as a
trusted fact, or discharges a proof obligation. It **evaluates**; it does not
**assert**. If you find yourself writing `postulate`, `Axiom`, a `?hole` in a
law position, or "assume `CursorLaws`," STOP — you have crossed the line.

## 3. Fixed inputs — grounded at `origin/main @ 0fa72ff9`; DO NOT reopen

### 3.1 Package identity & conventions (match the landed corpus, not the prose)

1. **Ships as exactly one file:** `catalog/packages/Tooling/Testing/Property.ken.md`
   (Section `Test` — a new section the path itself creates; leaf `Property`).
   **Single literate `.ken.md`** (narrative + `ken` fences + laws/proofs); only
   exact ` ```ken ` fences tangle. Verified: **35 of 36 catalog packages are
   literate `.ken.md`**; the lone bare `.ken` is an early outlier — match the
   35.
2. **Module is PATH-INFERRED.** No in-file `module` header; the module is
   `Tooling.Testing.Property` from the path.
3. **NO MANIFEST file.** Verified: **zero `MANIFEST` files on disk** across the
   whole catalog. The MANIFEST prose in `catalog/packages/README.md` is **stale**
   — match the disk (no manifest), not the README.
4. **§6 "Findings" is RETIRED — do not add it.** Follow the current section
   order of the exemplar `catalog/packages/Capability/Parsing/Cursor.ken.md` (Motivation →
   Definition → Using it → Laws & proofs → Design notes → References → Trust &
   derivation).
5. **Pure library — no `proc main`.** Validate with **`ken check
   catalog/packages/Tooling/Testing/Property.ken.md`**, not `ken run`.

### 3.2 Cross-file `import` does NOT resolve — INLINE the exercised surface

Verified on `origin/main`: cross-file `import` has **no disk loader** for the
catalog (`catalog/packages/README.md`, honesty note), and **every one of the
36 landed packages contains zero `import` statements** — the corpus is
self-contained. (The `catalog_roots` in `modules.rs` is the **N2 conformance**
loader, not a catalog cross-package resolver.) So: **INLINE** the minimal slice
of the Cursor/`Bytes` surface your witness needs, or restate it locally. Do
**not** write `import Capability.Parsing.Cursor`; it will not resolve.

### 3.3 The generator is DETERMINISTIC — there is no RNG, and you may not add one

Ken has **no randomness primitive**, and adding one is a **new primitive**
(forbidden, §6). So a `Gen a` is a **deterministic, bounded sample source** — a
finite `List a` of samples (edge cases + a bounded enumeration), *not* a
seeded PRNG. This is the honest, primitive-free, **total** shape; it is also
exactly what a provisional scaffold needs. Pin it and move on — do **not**
reach for randomness, effects, or a stateful seed.

### 3.4 The arbitrary-byte surface — range over `List UInt8`

Arbitrary `Bytes` are generated through the **total `List UInt8` view** of the
`Bytes` primitive (`Bytes` ⇔ `List UInt8`; element type `UInt8`; length via
`bytes_nat_length`; the exemplar uses this exact view —
`catalog/packages/Capability/Parsing/Cursor.ken.md:10`, `:41`). Your `gen_bytes : Gen
Bytes` materializes a bounded, deterministic family: the empty string,
singletons, multi-byte strings, and the `0`/`255` boundary values, built from
`List UInt8` and converted to `Bytes`. Re-ground the exact view-combinator
spellings at author time (§7).

### 3.5 The law to exercise — Cursor progress (the mandated non-vacuous witness)

The mandated witness is the **computational shadow of `CursorAdvanceProgress`**
(`catalog/packages/Capability/Parsing/Cursor.ken.md`, `fn CursorAdvanceProgress … : Prop`,
bundled in `CursorLaws`): *for a cursor with remaining input, one `advance`
step strictly decreases the remaining count.* As a **decidable `Bool` check** on
a concrete cursor built from a generated `Bytes`:

> if `peek` has remaining, then `remaining (advance c) < remaining c`.

This holds on the real Cursor and **fails on a mutant** (e.g. an `advance` that
does not move) — that is what makes the scaffold **non-vacuous**. You are
**evaluating** the decidable predicate over generated cursors; you are **not**
assuming, re-exporting, or re-proving `CursorAdvanceProgress` (which the Parsing
package already proves).

**Additional witnesses are optional and only if cheaply constructible** from an
already-landed surface (e.g. a `Decoder` progress/consume check, or a
`bytes_decode ∘ bytes_encode = Ok` round-trip **iff** a concrete encode/decode
pair is INLINE-constructible). Do **not** invent a surface to test; if a second
witness is not readily available, the single Cursor witness satisfies the WP.

## 4. Mandated deliverable outline

Each item ends in a concrete implementable choice, not a survey.

1. **`Gen a`** — a deterministic bounded sample source (§3.3). Simplest honest
   shape: a record/newtype wrapping `samples : List a`. Provide the minimum
   combinators the witness needs and **no more**: at least a way to build a
   `Gen` from an explicit `List a`, and `gen_map : (a → b) → Gen a → Gen b`.
   Do not build `bind`/monadic generators, shrinking, or size parameters in v1.
2. **`gen_bytes : Gen Bytes`** — the arbitrary-byte generator of §3.4, a bounded
   deterministic family over the `List UInt8` view including the `0`/`255`
   boundaries and the empty string.
3. **The runner** — `check : Gen a → (a → Bool) → PropertyResult`, folding the
   predicate over the samples and returning **held** or the **first
   counterexample** (`PropertyResult := Held | Failed a`, or reuse an existing
   `Data/Sums`/`Result`/`Validation` sum rather than minting a redundant one —
   subsume, don't proliferate).
4. **The Cursor-progress witness (§3.5)** — INLINE the minimal Cursor slice,
   build a cursor per generated `Bytes`, evaluate the decidable progress
   predicate, and assert (in the package's own Laws-&-proofs / examples section)
   that `check gen_bytes cursor_progress_pred` is **held** — a real, executed,
   non-vacuous check.
5. **Trust & derivation section** — state the derivation path from the built-ins
   and the **declared `trusted_base()` delta: ZERO**. This is a catalog entry;
   it names its floor like every sibling.

## 5. Acceptance criteria (testable)

- **AC1 — package shape.** Exactly `catalog/packages/Tooling/Testing/Property.ken.md`,
  single literate file, path-inferred module `Tooling.Testing.Property`, no MANIFEST, no
  `import`, no §6 Findings. `ken check` passes.
- **AC2 — deterministic generators.** `Gen a` carries an explicit finite sample
  set; `gen_bytes` yields a bounded family including empty / singleton /
  multi-byte / `0` / `255`. No RNG, no effect, no seed state.
- **AC3 — the runner reports counterexamples.** `check` returns `Held` when the
  predicate holds on every sample and `Failed <first sample>` otherwise —
  demonstrated by a *deliberately false* predicate in an example returning the
  expected first counterexample.
- **AC4 — the witness is non-vacuous.** The Cursor-progress check is **held**
  over `gen_bytes`, **and** the frame shows (in prose or a commented mutant)
  that it would **fail** on an `advance` that does not decrease remaining — i.e.
  the predicate has a reaching negative arm, it is not `True`-by-construction.
- **AC5 — ANTI-POSTULATE (the load-bearing AC).** The package contains **no**
  `postulate`, **no** `Axiom`, **no** `?hole` in a law position, and does **not**
  take any `Prop`-level catalog law (`CursorLaws`, `CursorAdvanceProgress`,
  `Decoder*`, `BytesRoundTripLaw`, `Config` totality) as a hypothesis or
  re-export it as a trusted fact. Every "property" is a computed decidable
  predicate over generated values. **Grep-clean** for those tokens in law
  positions.
- **AC6 — ZERO trust / no spill.** **Zero `trusted_base()` delta**; no new
  primitive; no kernel/Cargo/grammar/lexer change; no test-only language
  surface; no new dependency; nothing outside
  `catalog/packages/Tooling/Testing/Property.ken.md`. CI green (FULL workspace **in CI**,
  never a local `--workspace` run — §12).

## 6. Guardrails — do not reopen

- **⛔ NO test-only kernel or grammar primitive.** Operator ruling
  (`09-posix-linux-abi-campaign.md:560-571`): minimize language-surface
  entrenchment — the test framework is a **plain catalog package** so Ward can
  assume the role behind the same seam **without a language change**. No new
  keyword, no `#[test]`-style attribute, no built-in generator, no grammar hook.
- **⛔ ZERO `trusted_base()` delta.** You **consume** built-ins and landed
  catalog surface; you add nothing to the trusted base. No `Axiom`, no
  `postulate`, no primitive. (Anti-postulate, §2 / AC5.)
- **⛔ NO `Resource`, `Bracket`, `with`, `finally`, or any resource
  acquire/release combinator.** That is the HELD second half of CC9, gated on
  the operator's PX7 re-read. Not this WP. Do not read a file; do not touch FS.
- **⛔ NO RNG / effects / statefulness in generators.** Deterministic sample
  lists only (§3.3).
- **No `import` smuggling, no reflection, no macros, no derivation.** Explicit,
  self-contained Ken.
- **Subsume, don't proliferate.** Reuse an existing result sum for
  `PropertyResult` if one fits; do not mint a redundant carrier. The minimum
  combinator set only — no monadic `Gen`, no shrinking, no size params in v1.

## 7. The clause that keeps me honest — treat every anchor as perishable

**Treat every line number and combinator spelling above as perishable.** This
frame is grounded at `origin/main @ 0fa72ff9`; the exemplar's exact line
numbers and the byte-view combinator spellings can move under you. **Read the
ref (`git show origin/main:<file>`), never a stale worktree**, and re-confirm
`catalog/packages/Capability/Parsing/Cursor.ken.md` and the `Bytes`/`List UInt8` view at
author time. The *findings* here (single literate file, no manifest, no import,
deterministic generators, the anti-postulate line, zero trust delta) are
durable; the *anchors* are not.

**If a fixed input is false against the landed code, say so with exact tree
anchors and escalate — do not quietly build around it.** I would rather be
corrected than believed.

Package identity, section layout, and scope route to the **Steward**. Soundness
(the zero-trust-delta / anti-postulate axis) routes to the **Architect**.
