# C8 — function extensionality by computation (`Eq` at a Π reduces pointwise)

**Axis:** observational / OTT — the **funext** face. **Flavor:** B (should-PASS
if funext-definitional is surface-reachable, else a documented known-gap; the
unsound arm's REJECT is a soundness boundary). **Couples with C7** — C7 is the
*quotient* face of OTT, C8 is the *funext* face; together they cover the two
distinctive OTT surfaces Ken has that VAL2 breadth and C1–C7 never touched.

## Why this is a blind spot

Ken's signature kernel feature is **observational equality**: `Eq` (the surface
`Equal`) is not an inductive identity type but *computes by type*. At a function
type this is **function extensionality, definitionally** —
`Eq ((x:A)→B) f g` **reduces** to `(x:A) → Eq (B x) (f x) (g x)`
(`crates/ken-kernel/src/obs.rs::eq_at_pi`, `16 §2.2`/`§C2`). Two functions are
equal **exactly when** they agree pointwise, and that reduction is *definitional*
— no funext axiom, no truncation. VAL2 (breadth) never wrote an `Equal` at a
function type; C1–C7 exercise `Equal` at first-order types (`Bool`/`Int`) and
the quotient fragment, but **funext is exercised nowhere** (grep-confirmed). A
depth suite that omits Ken's sharpest OTT signature has a gap in the instrument
itself. (A *kernel-level* seed exists — `conformance/kernel/observational/
funext-definitional` — this is the complementary **surface-`.ken`** face.)

## The pair

Both arms use the **same proof shape** — a pointwise case-split
`\x. match x { True => Refl ; False => Refl }` — differing **only** in the two
functions being equated. That is the discriminator: the shape is identical, so
what flips the verdict is solely whether the functions agree pointwise.

- **Sound arm — `sound-funext-pointwise.ken` — should-PASS (if reachable).**
  `f := \x. x` and `g := \x. and_bool x True`. These are **pointwise equal**
  (`and_bool b True = b` on each `Bool`) but **not convertible as raw terms**:
  `and_bool x True` on an abstract `x` is a **stuck** match, so `\x. and_bool x
  True` does *not* β/η-reduce to `\x. x`. Proving `Equal (Bool→Bool) f g` is
  therefore possible **only** through the funext reduction: `Equal (Bool→Bool) f
  g` reduces to `(x:Bool) → Equal Bool x (and_bool x True)`, discharged pointwise
  by case-split (`Refl` at each constructor). funext is **load-bearing** here —
  without the `eq_at_pi` reduction the goal is opaque and the proof cannot even
  be stated.

- **Unsound arm — `unsound-differ-at-point.ken` — should-REJECT.** `f := \x.
  True` (constant) and `g := \x. x` (identity). These **differ at a point**:
  `f False = True`, `g False = False`. The *same* pointwise proof
  `\x. match x { True => Refl ; False => Refl }` is attempted. funext reduces the
  goal to `(x:Bool) → Equal Bool True x`; the `True` arm closes (`Equal Bool True
  True`), but the **`False` arm demands `Equal Bool True False`** — no
  `Refl` inhabits it, so the arm is **ill-typed and the definition is rejected**.

## Expected behavior (exact)

- **Sound arm: PASS** — `eq_at_pi` reduces the function-`Equal` to the pointwise
  Π, and the case-split proof type-checks. (Landed: `obs.rs::eq_at_pi`, `16
  §2.2`; `Equal`/`Refl`/Bool-`match` are surface-reachable — the Map laws and
  C1/C6/C7 use them.) If the elaborator does **not** reduce `Equal` at a function
  type to the pointwise Π when checking the lambda, that is a **prepared gap**
  (funext landed in the kernel but not surface-consumed) — record it, don't
  force it.
- **Unsound arm: should-REJECT** — the `False` case's goal `Equal Bool True
  False` has no `Refl` inhabitant; the elaborator/kernel must refuse the
  definition (a `TypeMismatch` on the `False` arm). **If it is ACCEPTED**, that
  is the finding: `Equal Bool True False` inhabited transports to `Bottom` (`J`/
  `cast`), so admitting a funext proof between pointwise-*unequal* functions is a
  consistency hole. The reject must be a **genuine proof attempt** failing — not
  an `Axiom` (an honest `Axiom` of a false type is a *visible postulate*, the
  C6 story, not a funext failure).

## Discriminates

Does `Equal` at a function type **reduce pointwise** (funext-definitional), and
does that reduction **stay sound** — admitting a pointwise proof for pointwise-
*equal* functions (`\x.x` ≡ `\x. and_bool x True`, PASS) while **rejecting** the
identical proof shape for functions that **differ at a point** (`\x.True` vs
`\x.x`, REJECT at `False`)? The shared proof shape makes the flip unambiguous:
the *only* variable is pointwise agreement, which is exactly what funext turns
into the function-equality — and exactly what must fail when it does not hold.

## Surface-expressibility note

Unlike C7's quotients (kernel-level `Term::Quot`/`QuotElim` with only *reserved*
surface), funext is **more surface-reachable**: `Equal`, `Refl`, and Bool
`match` are all used in landed `.ken` (the Map laws, C1/C6/C7), and the funext
reduction lives in the kernel conversion (`eq_at_pi`), transparent to the
surface. The one open question the run answers: does the *elaborator's* checking
path invoke `eq_at_pi` when a lambda is checked against an `Equal` at a function
type? If yes → sound-arm PASS; if the reduction is kernel-internal but not
triggered on this surface path, that is the prepared gap — funext is landed but
the surface can't yet lean on it. Push the encoding as far as the checker allows
before calling it a gap.
