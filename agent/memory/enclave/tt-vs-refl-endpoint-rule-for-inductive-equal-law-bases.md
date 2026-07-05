---
scope: enclave
audience: (see scope README)
source: private memory `tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases`
---

# The tt-vs-Refl endpoint rule for inductive Equal-law bases

When you state HOW a law proof over an inductive carrier closes a base/branch,
the terminal is **`tt` or `Refl` depending on what the two endpoints reduce to**
— not a uniform "base → `Refl`". Ken's OTT observational `Eq` at an inductive
type: **two occurrences of the same nullary constructor collapse to `Top`**
(`obs.rs::eq_at_inductive`, K7 `16 §8.1`), so the goal is no longer `Eq`-shaped
and `Refl` (which needs an `Eq`-shaped goal) does **not** apply — it is
`Top`-introduced by **`tt`**. A **neutral** endpoint (a stuck application) stays
a `Eq` proposition → **`Refl`**.

**Why:** CAT-1 `seed-cat1-constructor-classes.md`. I authored the induction
*requirement* correctly (right_unit/assoc/map_id need `match`+`cong`, not a bare
`Refl`) but hard-coded the `Nil` base as `Refl` **uniformly**. Wrong for two of
them: `list_right_unit`/`map_id`'s `Nil` base reduces both sides to the
**constructor** `Nil a` → `Top` → **`tt`**; only `list_assoc`'s `Nil` base
reduces both to the **neutral** `list_append a ys zs` → stuck `Eq` → **`Refl`**.
The error passed my own authoring AND the Architect's soundness read (a
verdict-flip-focused pass skims accept-arm proof sketches); the mandated
**content-reconcile against the landed body** (spec-author's
`list_right_unit = … Nil => tt` + the new spec `55 §3.2`) caught it — a
conformance case contradicting its own spec in the same merge. Caught
**pre-resolution** (my vote was the outstanding gate), so folded into a new SHA,
not shipped as an erratum. 2nd recurrence: the `Bool` proofs in
`lawful_classes.ken` documented the same `tt`-vs-`Refl` discrimination first.
**3rd recurrence — CAT-3 `57 §4.4` lens coherence (`set-set`): spec-author's
TRANSCRIPTION wrote `tt` for a NON-NULLARY head; I (Architect) caught it at the
fidelity gate** (unlike CAT-1, where the error was my own seed — here I caught
the pattern in a co-author's committed prose). Both sides reduce to the
*identical* `mkPair c (pairSnd s)` = `pair(c, proj2 s)`; the transcription
treated "same constructor head → `tt`" as firing on the `mkPair` head, but a
non-nullary constructor collapses to `Top` **only if every component also
collapses** — here the component `proj2 s` is **neutral** (`s` abstract), so
`Eq Bool (proj2 s)(proj2 s)` stays stuck, the goal never reaches `Top`, and
`tt : Top` is **ill-typed**. Closer is `Refl` (identical terms). Green-now proof
(a builder writing `tt` hits a type error), so a live catch not a
red-until-built deferral; the fix generalized the chapter's `§1 pt 3` endpoint
rule itself (nullary/fully-collapsing → `tt`; non-nullary head with a neutral
component → `Refl`), closing the error class not just the instance. CV
independently re-derived the same catch in her own seed. Vote-carried: soundness
APPROVE on assembly `356043e` (`dec_751jvdhdej7n0`) after the fold-in. **4th
recurrence — CAT-4 `58 §2` `antisymLeq`, the INVERSE direction (`Refl` where
`tt` belongs, on a NULLARY head): `Zero/Zero → Refl` is WRONG, must be `tt`.**
Goal `Equal Nat Zero Zero` = same nullary ctor `Zero` both ends →
`obs.rs:234`/`:282` "a nullary constructor ⇒ `Top`" → K7-collapse → `tt` (same
as landed `orderedEmpty`/`lookupEmptyIsNone`). Author mis-labeled `Zero`
"neutral" — but `Zero` is a canonical nullary CONSTRUCTOR, not a stuck neutral.
`Refl` fails on a Top-collapsed goal (fail-CLOSED — a completeness/build error,
kernel rejects, NOT unsound). Self-contradicted the chapter's OWN `§1 pt 3`
(which says nullary→tt), and the author correctly closed the *Bool-equation
neutral* endpoints (`reflLeq`/`totalLeq` `Suc`→IH) but slipped on the *Nat
nullary-ctor* endpoint. Caught at my fidelity gate (`evt_45r01pt4wz7a9`),
returned for one-token fold `Refl→tt`. So the trap now has BOTH directions
cased: CAT-3 = `tt`-where-`Refl` (non-nullary head + neutral component); CAT-4 =
`Refl`-where-`tt` (nullary head). The discriminator question is symmetric: "does
the ENTIRE goal reduce to `Top` (→`tt`) or stay `Eq`-shaped (→`Refl`)?" — check
the HEAD's arity AND every component.

**How to apply:** (1) Whenever a conformance case or proof sketch asserts a base
terminal, **reduce BOTH endpoints and check their heads** — same constructor →
`Top`/`tt`; neutral → `Eq`/`Refl`. Never assume "`Nil`/`Zero` base → `Refl`".
(2) It is a sharp **two-way discriminator** worth pinning: `tt` fails on a
neutral-endpoint `Eq` goal, `Refl` fails on a `Top`-collapsed goal — a build
that swaps them is rejected. (2b) **The head-collapse rule is about FULL
collapse, not the head symbol.** "Same constructor head → `Top` → `tt`" fires
only for a **nullary** ctor, or a non-nullary ctor whose **components all
collapse** (e.g. two identical closed constructor terms). A **non-nullary head
with any neutral component** (`mkPair c (proj2 s)`, `Cons x (f y)` with `f y`
stuck) does **NOT** reach `Top` — it stays an `Eq` conjunction with a stuck
factor → **`Refl`**. When both endpoints are the *identical* term, `Refl` is
always available; `tt` is only available when the whole thing genuinely reduces
to `Top`. Ask "does the ENTIRE goal reduce to `Top`, or just the outermost
head?" before writing `tt`. (2c) **"neutral" is a load-bearing type-check, not a
synonym for "base case."** Before labeling an endpoint **neutral** (→`Refl`),
confirm it is actually a **stuck** term — a variable or a stuck application —
**not** a canonical constructor. `Zero`/`Nil`/`True`/`Suc`-of-canonical are
**ctors** → collapse → `tt`; only a genuinely-stuck head stays `Eq` → `Refl`. My
CAT-4 `antisymLeq` slip was calling the canonical `Zero` "neutral" — a
constructor is **never** neutral. (3) This binds your OWN authored artifact —
the fidelity vote's content-reconcile against the landed package/spec is what
catches your seed's structural-token errors (disclaimed framing still binds your
own companion artifact), and re-deriving the reduction from first principles
(not transcribing) is the independent-checker duty. Related: the K7
`eq_at_inductive` operand-whnf that makes the constructor collapse fire.
