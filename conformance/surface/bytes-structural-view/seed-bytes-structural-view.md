# Bytes structural view

> **Status: build-backed (SUB-1).** The fixed `Bytes ↔ List UInt8` bridge
> exposes the implementation's byte-buffer guarantee to ordinary structural
> reasoning. Its trust cost is bounded and explicit: two primitive operations
> plus two named propositions, with every further operation derived.

## surface/bytes-view/structural-fold-needs-no-cached-length

- spec: `37 §2.6`; SUB-1 AC1
- given: `bytes_nat_length (bytes_encode "a/b")`, where
  `bytes_nat_length bs = length UInt8 (bytes_to_list bs)` and `length` is the
  existing recursive `List` fold
- expect: elaboration accepts the recursive `length` through the real SCT,
  evaluation returns `Suc (Suc (Suc Zero))`, and the package plus call site add
  no `Axiom`, cached `Nat`, primitive, or other `trusted_base()` entry
- why: this is the previously impossible consumer: a bare `Bytes` becomes
  structurally traversable without separately postulating that a cached length
  agrees with the opaque buffer. The producer is the real `bytes_to_list`, not
  a hand-built `List UInt8` supplied to the fold. (structural; runs; zero-delta.)

## surface/bytes-view/trusted-delta-is-the-four-named-entries

- spec: `37 §2.6`; `18a §5.8`; SUB-1 AC2
- given: the `trusted_base()` immediately before and after installing the
  structural view
- expect: their set difference is exactly
  `bytes_to_list`, `list_to_bytes`, `bytes_list_roundtrip`, and
  `list_bytes_roundtrip`; the first two are `PrimReduction::Op` declarations
  with their matching symbols and the latter two are opaque propositions
- why: the fixed, enumerated cost is the operator's bargain. A fifth primitive
  or postulate fails the set equality, while omitting either proposition fails
  the named membership check. (trust-ledger; exact set, not count only.)

## surface/bytes-view/both-runtime-inverses-are-total

- spec: `37 §2.6`; SUB-1 AC4
- given: the byte sequence `[0, 47, 128, 255]` and an independently constructed
  `List UInt8` `[1, 2, 200, 255]`
- expect: `list_to_bytes (bytes_to_list bs)` evaluates to the identical `Bytes`
  value, and `bytes_to_list (list_to_bytes xs)` evaluates to the identical
  constructor sequence; neither direction returns `Neutral`
- why: testing both independently rooted directions catches a pair that is only
  a one-sided inverse or changes order. The boundary values include zero,
  non-ASCII bytes, and `255`. (runtime value; bidirectional discriminator.)

## surface/bytes-view/laws-are-postulates-not-refl-reductions

- spec: `37 §2.6`; `18a §5.8`; SUB-1 AC4
- given: declarations that apply each named round-trip proposition, plus the
  same `Bytes → List UInt8 → Bytes` equation with body `Refl`
- expect: both named propositions elaborate at their exact dependent types;
  the `Refl` declaration rejects; using the propositions adds no new consumer
  entry to `trusted_base()`
- why: `PrimReduction::Op` computes in the interpreter but is opaque under
  kernel conversion. A green `Refl` arm would falsely claim trusted reduction;
  a local `Axiom` arm would recreate the unbounded trust tax SUB-1 removes.
  (honesty; accept/reject pair; trust-ledger.)

## surface/bytes-view/derived-surface-does-not-native-ize

- spec: `37 §2.6`; `18a §5.8`; SUB-1 AC3
- given: the catalog `bytes_nat_length` definition and the primitive registry
- expect: `bytes_nat_length` is transparent checked Ken and calls the existing
  `length UInt8` plus `bytes_to_list`; no `PrimReduction::Op` named
  `bytes_nat_length` exists, and loading the derived package leaves
  `trusted_base()` unchanged
- why: the bridge pays one fixed cost per opaque built-in. Ordinary byte
  algorithms belong over `List UInt8`; native-izing each one would turn the
  bounded bridge into an unbounded primitive surface. (producer-grep;
  zero-delta.)

## Coverage map

- **AC1:** `structural-fold-needs-no-cached-length`.
- **AC2:** `trusted-delta-is-the-four-named-entries`.
- **AC3:** `derived-surface-does-not-native-ize`.
- **AC4:** `both-runtime-inverses-are-total` and
  `laws-are-postulates-not-refl-reductions`.
- **AC5/AC6:** the existing CAT-5, CC3, CC7, and formatter corpus gates remain
  the regression net; SUB-1 does not migrate their cached-`Nat` carriers.
