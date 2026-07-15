---
scope: roles/steward
audience: (see scope README)
source: live 2026-07-13 (kenfmt batch-2 AC5)
---

# A one-file formatter/tool review only certifies the constructs that file has

When a tool's output (a formatter, a codegen, a canonicalizer) is
operator-approved on **one representative file** and then swept catalog-wide and
**frozen into a gate**, the approval is only as strong as the **constructs that
one file happened to contain.** A catalog-wide sweep before publish gets a
Steward spot-check spanning **distinct constructs the representative lacked** —
not a re-look at the same shape.

**Live (2026-07-13, kenfmt batch-2).** The horizontal-first formatter was
operator-approved on `Data/Numeric/Nat/Order.ken.md` (shallow `leq_nat`-on-`Nat` proofs).
The catalog-wide sweep candidate came back scope-clean (forbidden-path empty,
token/AST preserved, `#[ignore]` removed) and `ken fmt --check` was **green
across every source** — i.e. the formatter's splayed output was **idempotent**,
so the mechanical gates all passed. But the formatter had a **coverage gap**: it
collapses signatures/returns (R1/R1a) yet leaves **nested application /
proof-term arguments splayed one token per line**. The *same* subexpression
`(pair_fst a b x)` rendered **inline in the signature** but **4 lines in the
proof body** of `LawfulClasses::pair_ord_head_sound` — pervasive (Map's
27k-line diff, Collections, Parsing). OrdNat had no deep proof terms, so the
operator never saw it; only the AC5 spot-check across `LawfulClasses`/`Map`
caught it, before the re-armed strict gate would have **frozen a splayed catalog
as canonical**.

**How to apply.** (1) When you frame a "apply approved tool X catalog-wide + arm
the gate" WP, write an **AC5 that names the distinct constructs to spot-check**
(deep proof terms, multi-constructor data, effectful/lawful classes, functors,
big containers), not just "spot-check a few files." (2) Idempotence-green /
token-preserving is **necessary, not sufficient** — a tool can be a stable fixed
point on output that is *self-consistently wrong* (splay reproduces splay).
Mechanical gates prove semantics/stability; only an eyeball proves **style**.
(3) Escalate a real gap to the operator as a **new formatter-fix WP**, never
re-tune the tool inside the sweep WP (that reopens the approved style
un-reviewed). Dual of
[corpus-property-gate-only-as-strong-as-the-corpus] and
[verify-generalization-per-category] — the operator-review analog: a
representative sample certifies only its own categories.
