# DS-4 · `List` combinator completion

**Owned by the Steward** (frame); **home: Foundation**. The **first Data-section
item** of `wp/catalog-data-structures-program.md` — opens the Data section now
that Core is complete (Functor → Applicative → Monad → Traversable all landed).
**Near-mechanical** (the campaign classifies DS-2/DS-4 as the mechanical
siblings): it extends an existing, well-established List floor with five standard
combinators and their laws. Kicked in the operator's autonomous window
(2026-07-10) under the run's boundary rules.

## Extension point (grounded)

Append to the existing **`catalog/packages/Data/Collections/Collections.ken`**
(a plain `.ken` source, not `.ken.md`). The current List floor there —
`list_append`, `nth`, `take`, `drop`, `map`, `filter`, `mem`, `length`, with the
laws `take_drop_decomposition` / `map_length` / `length_take_min` — is the
pattern to mirror exactly (same total-`fn` style, structural recursion on the
carrier, laws as proof terms). Reuse `list_append`/`length`/`map` where the new
combinators build on them; do **not** re-prove the existing floor.

## Scope — five combinators + their laws

1. **`reverse (a : Type) (xs : List a) : List a`** — plus its **involutive**
   law `reverse (reverse xs) = xs`. This is the one non-trivial proof: it needs
   the helper lemma `reverse (list_append xs (cons y nil)) = cons y (reverse xs)`
   (reverse-of-snoc), then induction on `xs`. Also the length law
   `length (reverse xs) = length xs`. Choose the naive `append`-based `reverse`
   (not an accumulator) if it makes the involutive proof cleaner — pick the
   spelling whose laws are provable with the least machinery (PRINCIPLES:
   humans-read, small proofs).
2. **`zip (a b : Type) (xs : List a) (ys : List b) : List (Pair a b)`** —
   truncating at the shorter list (structural recursion, `nil` on either empty).
   Law: `length (zip xs ys) = min (length xs) (length ys)` (reuse the existing
   `min`). **NOTE — this is NOT the Vector `zip`:** List `zip` is
   **non-dependent** ordinary recursion with **zero** sibling-convoy/dependent-
   match involvement, so it has **none** of the DS-5c capability block that gates
   the length-indexed `Vec` `zip`. It is fully mechanical today. Do not conflate
   the two or gate this on DS-5c.
3. **`concatMap (a b : Type) (f : a → List b) (xs : List a) : List b`** — map
   then flatten via `list_append` fold. Law:
   `length (concatMap f xs) = sum-of-lengths` only if it falls out cleanly;
   otherwise ship `concatMap` with the two structural equations (nil / cons) it
   satisfies definitionally and skip a bespoke length law (don't invent a law
   that needs a `sum` combinator not in scope — subsume-don't-proliferate).
4. **`range (n : Nat) : List Nat`** — `[0, 1, …, n-1]` (or `[0..n]`; pick one and
   state it), structural recursion on `Nat`. Law: `length (range n) = n`.
5. **`foldl (a b : Type) (f : b → a → b) (z : b) (xs : List a) : b`** — the
   left fold. If a clean `foldl`/`foldr` relationship law is provable with the
   floor in scope, include it; otherwise ship `foldl` with its two structural
   equations. Don't force a law that needs machinery not present.

Laws are **`Ω`/`Prop` proof terms, pointwise, zero `Axiom`**, over the inductive
`List`/`Nat` carriers — exactly the DS-2 pattern. Where a proposed length/coherence
law would require a combinator not in the floor (e.g. a `sum`), **drop that law**
rather than proliferate helpers; ship the combinator with its definitional
equations. State plainly in the entry which laws each combinator carries.

## Boundary / constraints

- **AC1 — kernel-untouched, zero new elaborator capability, zero `trusted_base()`
  delta.** All five combinators + laws ride landed machinery (ordinary structural
  recursion + induction; no dependent-match, no new sort, no surface feature).
  Mirror DS-2/DS-7's executable before==after `trusted_base()` set-diff test. **If
  any combinator appears to need a new capability, STOP and hand back** — it
  shouldn't; these are textbook total functions.
- **Zero `Axiom`/`postulate`/`sorry`** in any proved law. The reverse-involutive
  proof is real induction, not a papered hole.
- **Outer-ring only** — `crates/ken-kernel`/`Cargo.lock` diff empty. Format the
  file to match `Collections.ken`'s existing style.
- **AC8 — discriminators flip** accept→reject on a wrong witness at the named law
  (e.g. a `reverse` that isn't involutive, or a `range` whose length law is off by
  one), asserted as the **specific** error variant, not bare `is_err()`.
- The **dot-projection / `λ`-in-type-position workaround may recur** (DS-7 Finding
  1): use a named total `fn` that δ/η-reduces to the spelling; file an Ergo Finding,
  don't smuggle a capability.

## Gate

Normal ring: Foundation build → foundation-qa independent re-derivation →
**@architect fidelity gate (build vs this frame + the campaign DS-4 scope) +
soundness gate (zero-new-`Axiom`, zero-`trusted_base`-delta)** → `git_request` to
Steward. CI-gated (real catalog `.ken` + acceptance test validating against the
landed elaborator). Own retro. Resource discipline (`CARGO_BUILD_JOBS=2`, scoped
`-p` tests). Flag every surface/elaboration/functionality judgment call in the
handback for the operator's log — especially any law dropped for
subsume-don't-proliferate reasons (with the reason).
