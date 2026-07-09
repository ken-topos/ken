# CAT-4-build — maps / sets / relations laws → code

**Owner:** Runtime build team (Map was Runtime's substrate). **Branch:**
`wp/CAT-4-build` (off `origin/main`). **Status:** Steward frame. **Base:**
`origin/main @ 5a780f8` (Map capstone `catalog/packages/collections/map.ken` + CAT-4
elaboration spec ch 58 + SURF-1's `const`/`fn`/`proc` + Unicode migration).
**Sequence:** Track B of the catalog build campaign — **independent, parallel
with CAT-1-build** (Language). CAT-2/CAT-3-independent (value-level).

This is the execution wrapper for the merged CAT-4 elaboration
(`spec/50-stdlib/58-maps-sets-relations.md`). **Read that chapter first — it is
canonical, on `main`, and exhaustively shovel-ready** (D0–D4 with concrete code
skeletons, proof shapes, and the four resolved forks). This frame carries only
(a) the SURF-1 respelling correction, (b) the do-not-reopen pins, (c) the hard
merge gates, and (d) sequencing. Treat any "current code state" line in the
chapter as perishable — **verify against the landed `map.ken` at pickup.**

## 0. What changed since chapter 58 was authored (SURF-1 landed) — read first

Chapter 58 was elaborated on `main @ 24a414b`, before SURF-1. Its **§1 pt 5 is
now STALE**: it says "new CAT-4 ops are written `view` … the migration re-spells
them when SURF-1's build lands." **SURF-1 has landed (`5a780f8`).** Therefore:

- **The `view` decl keyword is retired — write EVERY new CAT-4 op in
  `const`/`fn`/`proc`, NOT `view`.** All the code skeletons in ch 58 (§2–§7) are
  spelled `view`; **re-spell each to the keyword its purity earns.** Every CAT-4
  op is pure and takes ≥1 parameter ⇒ **`fn`** (there are no 0-param values ⇒ no
  `const`, and nothing is effectful ⇒ **no `proc`**). E.g. `fn leqNat …`,
  `fn bool_and …`, `fn dropKey …`, `fn delete …`, `fn union …`, `fn keys …`,
  `fn succ …`, `fn isTransitive …`.
- **The landed siblings are already migrated:** `catalog/packages/collections/map.ken`
  is 100% `const`/`fn` (0 `view`), and `bool_or`/`IsTrue` in
  `lawful_classes.ken` are `fn`. Match them: your new ops sit alongside
  `fn insert`/`fn lookup`/`fn fold`/`fn fromList` — same keyword family.
- **Unicode surface is live (SURF-1 D3):** the migrated corpus uses `→`, `⇒`,
  `Ω`, `∈` glyphs; ASCII aliases (`->`, `=>`) still lex. Match the surrounding
  file's style; the formatter canonicalizes.
- **Purity checker is live and bidirectional:** an `fn` with a non-empty effect
  row, or a `proc` that is actually pure, is a keyword/signature mismatch and
  fails. Every op here is pure total Ken → `fn` is correct and checks clean.

Everything else in ch 58 stands verbatim — only the base ref and the keyword
spelling moved. **The design (all four forks + two sub-rulings) is settled; the
build executes it.**

## 1. Objective

Land the **Layer-2 keyed-collection laws** as running code, on top of the landed
Map capstone, kernel-untouched: `delete` + the None/other-key lookup laws;
`union`/`intersection`/`difference` (combining-fn) + their lookup
characterizations + `Ordered`-preservation; the **set algebra** on
`Set = Tree _ Unit` stated **membership-extensionally**; `keys`/`values` +
coherence; and the **relations land-half** (`compose`/`converse`/property
predicates over the `Map K (Set K)` adjacency rep). All laws are `Ω` props
**proved over the landed carriers, zero `Axiom`, zero `trusted_base()` delta.**

## 2. Fixed inputs — do NOT reopen (settled by Architect, `main@7169300f`)

1. **Fork A — `union` takes a combining function** `f : v → v → v` (orientation
   `f (from-a) (from-b)`); left/right bias are `f = \x _. x` / `\_ y. y`.
   `intersection`/`difference` are membership-test folds (no combining fn).
   **Maps get the lookup characterization + `Ordered`-preservation only — NEVER
   a commutativity law** (map `union` is not commutative unless `f` is;
   commutativity/assoc/idempotence are **Set-only**, §5). Do not over-claim.
2. **Fork B — transitive closure is bounded-reachability `IsTrue`**
   (`R⁺ x y := IsTrue (reachableWithin N x y)`, `N := size (dom R)`) — the `Perm`
   move: push "there exists a path" into a **decidable bounded `Bool`**, wrap in
   `IsTrue`. **NEVER a raw multi-ctor `data TC : … : Ω`** (inadmissible
   proof-relevant inductive, `16 §1.4`+§1.1). Closure is **design-now /
   defer-build** — see §5.
3. **Fork C — a relation is `Map K (Set K)` = `Tree K (Tree K Unit)`**
   (adjacency), rides `Ord K` only, zero new machinery. NOT `Set (Pair K K)`.
4. **Fork D — `delete` is semantic filter-delete rebuild**: the canonical built
   form is the fused `deleteFromListAcc` worker over `toList`, skipping every
   order-equivalent entry and inserting every survivor into a fresh accumulator.
   This is the approved factoring of `fromList (dropKey key (toList m))`.
   **`dropKey` is FILTER** (removes ALL order-equivalent entries, not
   drop-first) ⇒ the None-law is **unconditional** (no `Ordered`/`Distinct`
   hypothesis). Rebuild uses landed `insert`, never structural `glue`/`deleteMin`.
5. **Set laws are stated MEMBERSHIP-EXTENSIONALLY** (`∀x. setMember x lhs ≡
   setMember x rhs`), **NEVER `Equal (Set K) …`** — Tree-`Equal` set laws are
   **FALSE** (`union a b` / `union b a` are shape-different trees, same key-set).
   Discharge as `bool_or`/`bool_and` corollaries via `boolDichotomy`, not fresh
   `Tree` inductions.
6. **`leqNat` + its four order laws are the D0 carrier prerequisite** — `Nat`
   (`data Nat = Zero | Suc Nat`, prelude; successor is `Suc` not `Succ`), NOT the
   `Axiom`-holed `Ord Int`/`Ord Char`. Net-new, `Axiom`-free, structural
   induction. Needed for a ≥3-key `Axiom`-free carrier so discriminators don't
   degenerate to reject-vs-reject (the CAT-3 `List Bool` vacuity lesson).
7. **Proof grammar inherited** (`54 §2`/§3, `55`/`57`): convoy idiom + Gap-A
   route-around; IH = ordinary self-recursive subtree call (never synthesized
   `ih_l`/`ih_r`); **per-branch `tt`-vs-`Refl`** sharpened (`57 §1 pt 3`) —
   `tt` when both endpoints reduce to the same **fully-collapsing** head,
   `Refl` when either stays **neutral** (incl. a non-nullary head with a neutral
   component). **Reuse, don't re-derive** the landed capstone laws
   (`preservesOrdered`(1)/`lookupAssocAgree`(5)/`toListOrdered`(4)).

## 3. Deliverables & build order (Runtime-owned)

Per ch 58 §8, in this order (each its own commit, gated before the next builds
on it): **D0 → D1 → D2 → D3 → D4-land-half.**

- **D0 — carrier prerequisites (§2):** `fn leqNat` + `reflLeq`/`transLeq`/
  `antisymLeq`/`totalLeq` (proof shapes pinned in ch 58 §2); plus the two `Bool`
  combinators `fn bool_and`/`fn bool_not` (transparent, match-based, like the
  landed `fn bool_or`). `antisymLeq` is only for the out-of-scope `Distinct`
  boundary — build it (cheap, `Nat`-structural) but nothing downstream here
  requires it.
- **D1 — `delete` (Fork D, §3):** `fn dropKey` (filter reference) +
  transparent fused rebuild worker `fn deleteFromListAcc` + `fn delete` +
  worker `Ordered` preservation + None-law (unconditional) + other-key law (via
  landed law 5 roundtrip). `orderEquivKey leq a b := bool_and (leq a b) (leq b a)`
  is the **Bool decision**; the landed Prop-valued `orderEquiv` is for laws only.
- **D2 — `union`/`intersection`/`difference` (Fork A, §4):** `fn insertWith` +
  transparent `toList`-stream fused workers for the three ops, worker
  `Ordered` preservation, and the lookup 2×2 characterization table. Fork A's
  semantic pin is the combining behavior/orientation (`f (from-a) (from-b)`)
  and lookup table, not a literal tree-fold source spelling.
- **D3 — `keys`/`values` (§6):** `fn keys` (reuse `pairKeys`+`toList`),
  `fn pairVals` + `fn values` (net-new mirror projection), keys-ascending
  coherence (off landed `toListOrdered`) + keys/values projection coherence.
- **D4 land-half (Fork C, §7):** `fn succ`/`relMember`/`fn compose`/`fn converse`
  + the property predicates `isReflexive`/`isSymmetric`/`isTransitive`/
  `isEquivalence` (Π-into-Ω, `16 §1.1`). **Full laws + discriminators land now.**

**DESIGN-NOW / DEFER-BUILD (do NOT build here, do NOT silently drop):** the
transitive-closure **proof** (faithfulness/saturation laws) + its prerequisites
`size : Tree k v → Nat` and `reachableWithin`. The **representation is pinned**
(Fork B, ch 58 §7); the proof is an explicit fast-follow. State the deferral in
the WP — **no silent truncation** (AC5).

## 4. Set algebra (§5) — membership-extensional, corollary-discharged

`setUnion s t := union … (\_ _. MkUnit) s t` etc. (at `Unit` the combining fn is
a no-op). Prove commutativity/associativity/idempotence/identity of `∪`/`∩` and
the `∖` membership shape as **corollaries of the membership-homomorphism lemma**
(`setMember x (setUnion s t) ≡ bool_or (setMember x s) (setMember x t)`, from the
D2 lookup characterization at `v = Unit`) via the landed `boolDichotomy` — a
finite 2×2, **no `Tree` induction**. Stated extensionally (pin 5).

## 5. Hard merge gates (= ch 58 AC1–AC7; re-derive from the diff)

- **G1 — Kernel-untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  no new `Term`/`Decl`; no `declare_primitive`/`declare_postulate`; **no
  `Axiom`** anywhere in the CAT-4 code. `trusted_base()` byte-unchanged. (AC1)
- **G2 — Reuse, not re-derive.** `delete`/`union`/… build on landed
  `insert`/`lookup`/`toList`/`fold`/`fromList` + laws 1/5; `leqNat`+4 laws are
  `Axiom`-free. Grep the law fields for `Axiom`/postulate/opaque → zero. (AC2)
- **G3 — `Ordered`-preservation** proved for `delete` and the three set-ops
  through their rebuild/list-worker preservation lemmas. (AC3)
- **G4 — Characterizations proved:** D2 lookup 2×2; D1 None-law (unconditional)
  + other-key law; D3 keys-ascending + keys/values projection coherence. (AC4)
- **G5 — Relation `Ω`-soundness.** Property predicates are Π-into-Ω; transitive
  closure is bounded-reachability `IsTrue`, **never** a raw `data … : Ω`; the
  deferred faithfulness split is **stated** (no silent truncation). (AC5)
- **G6 — Set laws membership-EXTENSIONAL**, never `Equal (Set K)`; discharged as
  Bool-algebra corollaries via `boolDichotomy`. (AC6)
- **G7 — Carrier vacuity guard.** Discriminators run on the **`Nat`** carrier
  with the real `leqNat` dictionary (≥3 distinct keys), never `Ord Int`/`Char`.
  Broken-law conformance cases reject **right-reason** (assert the concrete
  error variant, not `is_err()`). (AC7)
- **G8 — Purity-keyword + workspace green.** Every new op is `fn`/`const` (no
  `view`, no mis-declared `proc`), purity checks clean; `cargo test --workspace`
  green; the Map capstone + all prior packages unbroken.

**Any forced deviation (a kernel touch, a new `Term`/`Decl`, an `Axiom`, a raw
`data … : Ω` closure, or a `Set`-`Equal` law) → surface to the leader → Steward
before proceeding. Don't smuggle it.**

## 6. Gate & acceptance

- **Architect re-certifies** on the *built* diff: **AC1** (kernel-untouched) +
  **AC5** (relation `Ω`-soundness — closure is `IsTrue` bounded-reachability, not
  a proof-relevant inductive; deferral stated). His build-time obligation.
- **Runtime-QA** independent pass (re-derive G1–G8 from the diff, not the
  report) + **Verify-QA** + **CI**.
- **Conformance:** author the D5 seed with **conformance-validator** under
  `conformance/stdlib/collections/` — discriminators on the `Nat` carrier with
  the real `leqNat` dictionary, broken-law cases rejected right-reason.
- **Acceptance:** a `.ken` program builds maps/sets/relations, calls
  `delete`/`union`/`intersection`/`difference`/`keys`/`values`/`compose`/
  `converse`, the laws are proved (not postulated), the discriminating
  conformance cases pass, workspace + corpus green.

## 7. Dependencies / sequencing

- **Depends on:** Map capstone (landed, `map.ken`) + `lawful-classes`
  (`bool_or`/`IsTrue`/`boolDichotomy`) + SURF-1 (`const`/`fn`/`proc` + Unicode,
  merged `5a780f8`). CAT-1/CAT-2/CAT-3-**independent** (value-level; no class
  mechanism needed).
- **Parallel with:** CAT-1-build (Language; touches `ken-elaborator` class/
  instance desugar — CAT-4 is pure `.ken` package code, no file conflict).
- **Blocks:** nothing in the CAT tranche (leaf). Feeds L14 model-check interop +
  Lane B (the relations seam) later.
