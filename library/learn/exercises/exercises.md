# Exercises

Solutions live in a separate file: [`solutions.md`](solutions.md). Work each
exercise against the named source before checking there.

## 01 — Anatomy

**Learning objective:** read a declaration's signature — keyword, name, type,
effect row — before reading its body, and let the signature bound your
expectations of the body.

1. Open `catalog/packages/Capability/Console/Text.ken.md`. Before reading
   `print`'s body, state its full type and effect row from the signature
   alone. What does the row tell you `print` may do, and what does it rule
   out?
2. Open `catalog/packages/Core/Logic/EmptyDec.ken.md`. Is `decide` (used in
   the `yes_is_true`/`no_is_false` proofs) declared as a `fn`, a `const`, or a
   `proc`? What does that keyword alone already tell you about whether it can
   touch anything outside its own inputs?

## 02 — Types, contracts, and proofs

**Learning objective:** distinguish a passing `ken check` on a claim from the
epistemic status that claim actually carries.

1. `catalog/packages/Core/Logic/EmptyDec.ken.md`'s `yes_is_true`/`no_is_false`
   proofs both pass `ken check`. Does passing `ken check` by itself tell you
   these are `proved` rather than, say, `tested`? What additional fact, cited
   in chapter 03, is what actually decides the label?

## 03 — Assurance and trust

**Learning objective:** apply the four epistemic statuses (`proved`,
`tested`, `delegated`, `unknown`) to specific claims in specific fragments,
rather than to a whole file at once.

1. `catalog/packages/Data/Sums/Combinators.ken.md` states "`trusted_base()`
   delta: zero," and `catalog/packages/Core/Logic/EmptyDec.ken.md` states
   "Zero new trust category." Are these the same claim? If not, what
   specifically does EmptyDec's own wording keep open that Combinators'
   does not?
2. `catalog/packages/Tooling/Testing/Property.ken.md`'s own Motivation
   section draws a line between "computations" and "propositions." Does
   this fragment exhibit the spec's formal, tagged `tested` construct, or
   something else? Name what it actually shows, precisely.

## 04 — Effects, capabilities, and authority

**Learning objective:** read a `visits [ρ]` row as a checked upper bound on
behavior, and correctly separate "no fragment shows this" from "the
language can't do this."

1. `catalog/packages/Capability/Filesystem/Errors.ken.md` names its own
   `AFull` capability and states a limitation about it in its own words.
   What, exactly, does `AFull` not yet confine, per the fragment's own
   text?
2. True or false, and cite your source: no `catalog/packages/` fragment
   today carries an explicit capability-typed signature, an `attenuate`
   call, or an authority comparison in checked surface code.

## 05 — Packages and provenance

**Learning objective:** check a fragment's own casual vocabulary against the
normative section that actually defines the term, instead of repeating the
fragment's wording as fact.

1. `catalog/packages/Tooling/Testing/Property.ken.md`'s own prose calls
   `List`, `Result`, `Unit`, `Nat`, `Bytes`, and `UInt8` all "the prelude's."
   Checked against the taxonomy's own closed lists, which of those names
   is actually **prelude**, which is actually **built-in**, and which does
   the cited taxonomy section simply not pin at all?
2. `catalog/packages/Core/Logic/EmptyDec.ken.md` inlines two lemmas from
   `Transport.ken.md` rather than `import`-ing them. Is that because
   cross-file `import` doesn't work? If not, what does the fragment's own
   word for what it's doing actually claim?

## 06 — Execution

**Learning objective:** distinguish what a fragment's `ken check`-passing
status establishes from what only running it would establish — and
correctly separate a corpus-usage gap from a genuine capability question.

1. Does any of the seven fragments this curriculum is built from declare a
   `proc main`? What follows from your answer about whether the reference
   interpreter or the native backend has ever actually run any of them
   within this corpus?
2. `catalog/packages/Capability/System/IO.ken.md` states that "exactly-once
   settlement and liveness remain runtime-enforced, delegated boundary
   properties." Are the five `lemma`s directly above that sentence proofs
   of settlement and liveness themselves? If not, what are they proofs of?
3. Can both of these be true without contradiction: the native backend's
   target/toolchain decision (`OQ-backend-target`) is recorded **open** in
   the spec, while real native-backend code exists and runs programs
   today? Explain, citing both a spec source and a code source.
