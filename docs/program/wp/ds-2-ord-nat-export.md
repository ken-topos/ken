# DS-2 · Export a lawful `Ord Nat` instance + a `Nat` operations entry

**Owned by the Steward** (frame); **home: Foundation** (+ enclave for a light
abstraction-boundary confirmation). Phase-2 item of
`wp/catalog-data-structures-program.md`, kicked in the operator's autonomous
window (2026-07-10). Near-mechanical: the hard proofs already exist.

## Goal

1. **Exported lawful `instance Ord Nat`.** Lift the private `Nat`-order family
   out of `catalog/packages/Data/Collections/Map.ken` (where it is a
   build-internal duplicate, `Map.ken:2571–2621`):
   - `leqNat : Nat → Nat → Bool`
   - `reflLeqNat  : Equal Bool (leqNat x x) True`
   - `transLeqNat  : … Equal Bool (leqNat x y) True → Equal Bool (leqNat y z) True → Equal Bool (leqNat x z) True`
   - `antisymLeqNat : … Equal Bool (leqNat x y) True → Equal Bool (leqNat y x) True → Equal Nat x y`
   - `totalLeqNat  : Or (Equal Bool (leqNat x y) True) (Equal Bool (leqNat y x) True)`

   into a real exported instance of the landed `class Ord` (`Core/LawfulClasses.ken:49`):
   ```
   class Ord a {
     leq     : a → a → Bool ;
     refl    : (x : a) → IsTrue (leq x x) ;
     antisym : (x : a) → (y : a) → IsTrue (leq x y) → IsTrue (leq y x) → Equal a x y ;
     trans   : (x : a) → (y : a) → (z : a) → IsTrue (leq x y) → IsTrue (leq y z) → IsTrue (leq x z) ;
     total   : (x : a) → (y : a) → IsTrue (bool_or (leq x y) (leq y x))
   }
   ```
   The **proved** `instance Ord Bool` (`LawfulClasses.ken:226`) is the template —
   it is zero-`Axiom`, unlike the `Int` instance (postulated). Mirror its shape.

2. **A `Nat` operations entry** collecting `min`/`max`/`sub`/`compare` (today
   split in `Collections.ken`) into one place, reusing `Ord Nat` where natural.

## The one real adaptation (probe first — escalate only if it needs a ruling)

The Map.ken proofs are phrased over `Equal Bool b True`; the `Ord` class fields
are phrased over `IsTrue b`, and `total` wants `IsTrue (bool_or (leq x y) (leq y
x))` where `totalLeqNat` returns `Or (Equal…True) (Equal…True)`. So the lift is
**not** a pure copy — it needs the `Equal Bool b True ⇄ IsTrue b` bridge and an
`Or`-of-equalities → `IsTrue (bool_or …)` conversion. **The proved `Ord Bool`
instance already solved exactly this bridge** — probe it first and mirror it; it
is very likely a clean mirror, not new design. Escalate to the enclave (Architect)
**only if** the bridge or the `total` conversion turns out to need a ruling
rather than a mirror.

## Boundary / constraints

- **Zero new `Axiom` (the acceptance bar).** `Nat` is inductive and kernel-
  proved (unlike `Int`), so `Ord Nat`'s laws must be genuinely proved — the lift
  must introduce **no** new `trusted_base()` delta. The enclave confirms this at
  the gate (it is the whole point of choosing `Nat` here).
- **De-duplicate, don't fork.** After the export lands, `Map.ken` should consume
  the exported `Ord Nat` (or its `leqNat`) rather than keep its private copy —
  but only if that reconsumption is itself zero-behavior-change and clean; if it
  risks touching `Map.ken`'s capstone proofs, leave `Map.ken` as-is and file a
  follow-up (do not destabilize the capstone for a dedup).
- **Outer-ring only.** No kernel/elaborator/TCB change is expected or permitted
  here; if one appears necessary, STOP and hand back (it would mean the frame is
  wrong).
- **Format:** `.ken.md` per `07-catalog-style-guide.md`, and per the just-landed
  `PRINCIPLES #14` — required facts (the laws) live in the Ken as `law`/`fn`
  proof terms, narrative in prose, no required info in `--` comments.

## Gate

Normal ring: Foundation build → foundation-qa independent re-derivation →
Architect soundness gate (the zero-new-`Axiom` confirmation is his) →
`git_request` to Steward. CI-gated (real catalog `.ken`/`.ken.md` + acceptance
test). Own retro (acted-on).
