# 02 — Types, contracts, and proofs: read the promise before the body

Chapter [01](01-anatomy.md) showed you where a declaration's signature sits
relative to its body and its proofs. This chapter is about what that
signature actually promises, how a Ken program's contract is stated
separately from its evidence, and how to read that evidence once you find
it.

## 1. The signature is the contract — and it cannot lie

A definition's keyword is a **checked signal**, not a comment: the keyword
is verified against both the declared signature and the body's actual,
transitively-inferred behavior, and a disagreement in either direction is a
hard error at the definition site
(`spec/30-surface/36-effects.md`
[§1.6.2](../../../spec/30-surface/36-effects.md#162-the-bidirectional-check--the-keyword-cannot-lie)).
Concretely: an `fn` that performs an effect is rejected, and a `proc` whose
body turns out to be provably pure — carrying no real effect — is flagged
as a should-be-`fn` mismatch. This is why reading a signature first (chapter
01, §4) is not just a convenient habit: the signature is a promise the
elaborator itself enforces, not a description the author could have gotten
away with getting wrong.

That promise is about **purity and effects**. It says nothing yet about
*which value* a function returns for a given input, or *why* — that is a
separate, further contract, stated in a proof declaration next to the
function, not folded into its type.

## 2. The proof-claim vocabulary

Ken has three surface forms for stating and discharging that further
contract, all of them surface/elaboration vocabulary over already-checked
terms — none adds a new kernel declaration class or an ambient proof search
(`spec/30-surface/33-declarations.md`
[§8](../../../spec/30-surface/33-declarations.md#8-named-proof-claims--prop-lemma-and-attached-proof)):

- **`proof <name> for <subject>`** — a checked proof attached to an
  already-resolved subject, addressed afterward as `subject::name`. It
  names "a checked property *of* `subject`" — the subject must occur
  applied somewhere in the proof's claim
  ([§8.2](../../../spec/30-surface/33-declarations.md#82-attached-proofs--proof)).
- **`lemma`** — a standalone checked proof theorem in the ordinary module
  namespace, used when no single subject owns the theorem
  ([§8.3](../../../spec/30-surface/33-declarations.md#83-standalone-lemmas--lemma)).
- **`prop`** — names a proposition family / claim shape, not itself a proof (
  [§8.1](../../../spec/30-surface/33-declarations.md#81-proposition-families--prop)).

You can see both of the proof forms in the fragments this curriculum draws
from. `catalog/packages/Core/Logic/Transport.ken.md` states `cong`, `sym`,
and `trans` as `lemma`s — none of them belongs to one specific subject,
they are the general equality algebra later proofs build on. The same file
also attaches a proof directly to a subject:

```ken
proof transport for stuck_of (k : Bool) (q : Equal Bool k True) : Equal Bool (stuck_of k) True =
  ...
```

Here `stuck_of` is the resolved subject, `transport` is the proof's name,
and the claim (`Equal Bool (stuck_of k) True`) genuinely mentions
`stuck_of` applied — exactly the well-formedness condition §8.2 states.

## 3. A guided read: one signature, three proofs

Return to `get_or_else` from `catalog/packages/Data/Sums/Combinators.ken.md`,
read now with its proofs attached:

```ken
fn get_or_else (a : Type) (d : a) (x : Option a) : a =
  match x {
    None ↦ d;
    Some v ↦ v
  }

proof none for get_or_else (a : Type) (d : a) : Equal a (get_or_else a d (None a)) d = Refl

proof some for get_or_else (a : Type) (d : a) (v : a) : Equal a (get_or_else a d (Some a v)) v =
  Refl
```

The signature promises only "pure function, `a → a → Option a → a`." The
two attached proofs are the actual contract a reader wants: at `None`, the
result is exactly the default `d`; at `Some v`, the result is exactly `v`.
Each is proved by `Refl` — the equation holds by computation alone, because
`get_or_else` is a direct structural case-split with no further machinery
in the way (`docs/program/07-catalog-style-guide.md`
[§6](../../../docs/program/07-catalog-style-guide.md#6-proof-presentation)).
A third proof in the same file, `none_rhs for or_else`, needs an actual
`match` inside its own proof body rather than closing by `Refl` alone —
because that equation's left-hand side is not yet reduced until its own
scrutinee is case-split. Reading which proof closes immediately by `Refl`
and which needs its own case split is itself informative: it tells you
whether the property was true "by definition" at the call site you are
looking at, or only after further computation.

## 4. What a passing check does and does not establish

`ken check` on one of these files elaborates every declaration, including
every attached proof and lemma, against the kernel. A passing check
therefore does establish that every stated equation is proved. It does
not, by itself, tell you *which closing term* discharged each one, and
that closing term is itself informative. `get_or_else`'s two proofs above
both close with `Refl`, over the open, still-abstract variables `a` and
`d` — the two sides of each equation reduce to the identical term without
either side ever becoming a fully closed, concrete value.
`catalog/packages/Core/Logic/EmptyDec.ken.md`'s own Laws & proofs section
closes a different kind of claim — `decide`/`true_is_true`/
`true_is_not_false`, all closed, fully-applied terms — with `Proved`
instead, because there both sides reduce all the way to the same concrete
`Bool` constructor and the equality itself collapses to `Top` before
`Proved` is even checked; the entry states explicitly that `Refl` would
not apply there, "since neither side is stuck." Reading which closing term
a proof actually uses, and whether the terms it relates are still open or
already fully closed, tells you something `ken check`'s bare exit code
does not: this is the discipline chapter 03 builds on directly — a checked
file tells you a stated claim was proved, not by itself which of Ken's
several honest verification statuses that claim carries.

## Reader can now answer

- Where is a function's purity/effect promise enforced, and what happens
  when a body doesn't keep it?
- Given `proof p for s`, `lemma`, and `prop` used in the same file, what
  does each one actually name, and how do you tell them apart?
- Looking at a proof that closes with `Refl`, what does that closing term
  tell you about *when* the two sides became equal — and how is that
  different from a proof closing with `Proved`?

---

**Grounds this page:**
`spec/30-surface/36-effects.md` §1.6.2;
`spec/30-surface/33-declarations.md` §§8, 8.1–8.3;
`docs/program/07-catalog-style-guide.md` §6.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragments' own text; it does not assert a rule they
do not already state. Fragments cited are drawn from the already-selected,
registered set in [`fragments.md`](fragments.md); this chapter does not
introduce a fresh selection.
