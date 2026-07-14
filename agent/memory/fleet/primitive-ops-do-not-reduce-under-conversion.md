---
scope: fleet
audience: (see scope README)
source: SUB-1 grounding, 2026-07-14 — settled three ways after the Bytes→Nat ruling
---

# Primitive ops compute at RUNTIME and are OPAQUE to conversion

**A primitive operation never reduces under the kernel's conversion — not even
on literals.** `bytes_length (bytes_encode "abc")` does **not** become `3` for
`conv`; `eq_int five five` does **not** become `True`. They reduce only in the
**interpreter**, at run time.

**⇒ `Refl` cannot discharge any equation whose sides mention a primitive op.**
This is the single fact behind a whole family of surprises, and it is why four
packages ended up carrying `Axiom`s.

## The evidence (verified at `origin/main @ c5f73b9c`)

1. **`conv.rs::unfold_const` unfolds ONLY transparent constants** — it calls
   `env.transparent_body(id)`. **`Decl::Primitive` has no body** and is never
   unfolded. (`conv.rs` contains **zero** `Decl::` references at all.)
2. The kernel's own comment on `PrimReduction::Op` (`env.rs:94`): *"a primitive
   operation **awaiting** its registered reduction (K3)."* **K3 never landed.**
3. **Ken's guide ships it as a REJECT** —
   `catalog/guide/proof-techniques.ken.md:133`:
   `lemma prim_eq_refl : Equal Bool (eq_int five five) True = Refl` **fails**,
   *"never reduces to `True` under conversion, even though `five` is concrete."*
4. `prim_reduce` / `prim_reduce_elaborated` live in **`ken_interp::eval`** — the
   **interpreter**, not the kernel.

## What follows from it

- **Programs still work.** A `match` on a stuck primitive application is fine:
  it is stuck for `conv` but computes at run time, and **SCT is syntactic**, so a
  fold recursing on a structurally-decreasing parameter still passes termination
  checking. *Opaque-to-conversion ≠ unusable.*
- **Proofs do not.** Any law you want *about* a primitive op must be a
  **postulate** (or a Rust-side differential test), never a `Refl` lemma.
- **This is why the cached-`Nat` carriers exist.** CAT-5's `Source`
  (`SourceLength … = Axiom`) and CC3's `ArgBytes` (`ArgByteLength … = Axiom`)
  are not sloppiness — **`Axiom` was the only available route**, because the
  obligation `Equal Int (bytes_length bs) …` has a stuck neutral on the left.

## ⚠ The spec disagrees, and the spec is the thing to distrust here

**`spec/30-surface/37-strings-collections.md §2.4` claims the opposite:** *"A
primitive op carries a registered reduction (`41`), so e.g. `byteLength "abc" ≡
3` holds **definitionally** and proofs can compute over string literals."*

**Do not design laws on that sentence.** It was escalated to the Architect +
Spec enclave (`evt_1chdn8t7s3hnv`) as a suspected over-claim (PRINCIPLES #8).
Until it is adjudicated, **assume prim ops do not reduce, and if you find `Refl`
closing a primitive-op equation, STOP and report it** — that would mean the spec
is right and this lesson is wrong, which is a *good* outcome worth knowing.

Sibling of [[opaque-primitive-constructible-not-destructible-format-gap]] and
[[spec-claim-kernel-admittance-vs-staging]]. The fix for the underlying defect is
`docs/PRINCIPLES.md` **#15** — expose the implementation's guarantee as a
structural view (`bytes_to_list`, mirroring `string_to_list_char`) rather than
making every consumer postulate it.
