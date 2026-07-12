# N1 fail-closed declaration collisions — conformance seed

Format: `../../README.md`. These cases pin ADR 0014's accepted MRES-5,
MRES-7, and MRES-8 round-one rule: a second top-level definition of one name
in a compilation unit is a hard error in Ken's single flat namespace. A class
name and a constructor name are not separate namespaces. The existing
arity-gated `Eq`/`J` surface sugar remains legal beside lower-arity user
definitions of those names.

**Build state — RED UNTIL N1 LANE B.** The two reject arms state the Lane B
acceptance target and are expected to fail on Lane A's base: the current
elaborator silently overwrites an occupied global. The positive controls are
live now and must remain green. Lane B turns only the reject arms red → green;
it must not obtain a vacuous all-rejecting result by breaking the positive
arity-gated-sugar control.

**Diagnostic granularity.** The accepted design fixes a specific hard-error
kind that identifies the offending name as a duplicate/collision. It does not
yet fix the Rust variant's literal spelling. The cases therefore pin that
semantic kind and payload, with the variant token `(oracle: Lane B spelling)`;
a generic parse/type error, warning, or bare `is_err` does not conform.

---

## surface/declarations/duplicate-ordinary-top-level-definition-rejected

- spec: `33 §3` (single flat namespace), ADR 0014 MRES-5/MRES-8
- given: two compilation units with identical surroundings and bodies. The
  control declares `fn keep (x : Bool) : Bool = x` followed by
  `fn echo (x : Bool) : Bool = x`; the collision arm changes only the second
  declaration head from `echo` to `keep`.
- expect: the unique-name control **accepts**. The collision arm **rejects at
  declaration resolution** with the specific duplicate-definition/collision
  error `(oracle: Lane B variant spelling)`, whose payload names **`keep`**.
  The first definition remains authoritative; there is no warning-and-continue
  or last-writer-wins result. **Red until N1 Lane B.**
- why: this is a controlled verdict flip on collision presence: all types and
  bodies are fixed, and only a fresh name versus an already-occupied name
  varies. A generic type failure cannot make the reject arm pass
  coincidentally.

## surface/declarations/class-constructor-single-namespace-collision-rejected

- spec: `33 §3` (single flat namespace and D8-③), ADR 0014 MRES-7/MRES-8
- given: the same program as
  `arity-gated-eq-j-sugar-coexists-with-lower-arity-definitions` below, except
  `data Marker = Only` is changed to `data Marker = Eq`. Thus the unit contains
  both `class Eq a { eq : a -> a -> Bool }` and a constructor named `Eq`.
- expect: the `Only` control **accepts**. The `Eq`-constructor arm **rejects at
  declaration resolution** through the same specific
  duplicate-definition/collision error as the ordinary case
  `(oracle: Lane B variant spelling)`, whose payload names **`Eq`**. It is not
  accepted by partitioning class and constructor names, and it does not
  silently replace either global. **Red until N1 Lane B.**
- why: the two units differ only in the constructor name, so the verdict flips
  accept → reject on the single-flat-namespace collision itself. The accepted
  arm also exercises the real `Eq`/`J` sugar, preventing a blanket reserved-name
  rejection from making both arms reject.

## surface/declarations/arity-gated-eq-j-sugar-coexists-with-lower-arity-definitions

- spec: `33 §3`, ADR 0014 MRES-8 (preserve the arity-gated-sugar exclusion),
  `34 §4.3` (`J motive base eq`)
- given: one compilation unit containing:

  ```ken
  class Eq a { eq : a -> a -> Bool }
  fn J (x : Bool) : Bool = x
  data Marker = Only

  lemma eq_sugar (a : Type) (x : a) : Eq a x x = Refl
  lemma j_sugar
    (ty : Type) (a : ty) (b : ty) (q : Equal ty a b)
    : Equal ty a b = J (\b' _. Equal ty a b') Refl q
  ```

- expect: **accepts**. The user declarations occupy only the lower-arity
  `Eq a` and `J x` shapes; the three-argument `Eq a x x` and
  `J motive Refl q` sites resolve to the real equality/J sugar. Their emitted
  core contains the equality type and a real `Term::J`, not calls to the user
  class/function.
- why: this is the live, discriminating positive arm required by MRES-8. A
  blanket name-based duplicate/reserved-sugar check rejects this valid program,
  while the specified arity-aware check accepts it. Paired with the preceding
  one-token constructor change, it makes the collision axis accept ↔ reject,
  not green-vs-green.

## Cross-case consistency

Both negative cases use one declaration-resolution diagnostic mechanism and
require the offending single-namespace name in its payload. Both have an
adjacent accepted control with otherwise well-typed declarations, ruling out a
coincidental rejection. The class/constructor pair holds the real `Eq`/`J`
sugar sites fixed across both arms, so preserving the arity gate and rejecting
the actual duplicate are jointly observable rather than competing tests.
