# DS-3 · `Option` / `Result` combinators (+ the `Either` ruling)

**Owned by the Steward** (frame + the `Either` design recommendation); **home:
Foundation** (combinator build) **+ enclave** (the `Either` ruling → Architect
confirms). The second Data-section item of
`wp/catalog-data-structures-program.md`. **T1-design-needed** — it carries one
genuine design fork (the `Either` question) plus a mechanical combinator build.
**Shovel-ready; not yet kicked** — hold until DS-4 clears to avoid Foundation
queue contention, then kick.

## The `Either` ruling (T1 design fork — Steward recommendation: SUBSUME)

**The fork:** `spec/50-stdlib/README.md:42` names both `Either e a` and
`Result e a` in its L2-sum list, but **only `Result e a = Err e | Ok a` is
declared** (prelude, `crates/ken-elaborator/src/prelude.rs:193`); `Either` is
declared nowhere. Rule: does the catalog carry a **distinct `Either`**, or does
**`Result` subsume it**?

**Steward recommendation — SUBSUME (no distinct `Either`; `Result e a` is the
catalog's binary sum), grounded in PRINCIPLES as the operator's stand-in:**

- **#7 subsume-don't-proliferate (decisive).** `Either e a = Left e | Right a`
  is **structurally isomorphic** to the declared `Result e a = Err e | Ok a` —
  same binary sum, two type params, differing only in constructor spelling. A
  first-party `Either` would be an **isomorphic twin** adding **zero capability**
  `Result` doesn't already provide. That is exactly the proliferation #7 forbids.
- **`Result` is already load-bearing; `Either` is vestigial.** `Result` is wired
  into the effect system (`fs_resp : … = Result IOError Bytes`,
  `prelude.rs:1077`) and named as the codec/error-path carrier; `Either` has no
  declaration or user. The catalog's committed binary sum is `Result`.
- **The trust levels are identical**, so `coexist-over-subsume-when-trust-levels-
  differ` (the standard subsume exception) does **not** apply — both are ordinary
  L2 sums; there is no kernel-checked-vs-trusted distinction to preserve.
- **The only thing `Either` offers is a "neutral" (non-error-biased) reading.**
  That is naming intent, not capability — a neutral binary sum is `Result`, or a
  user's own local `data`. Not a reason to ship an isomorphic stdlib twin.

**This ruling is not purely mine — route it to @architect (design-shape) + the
spec enclave (fidelity).** It has a **spec-fidelity face**: the README:42 L2-sum
list *names* `Either`, so subsuming requires a **spec reconcile** — drop `Either`
from the list, or annotate it "subsumed by `Result`; no distinct type." That
erratum is spec-author + CV work (the DS-5 §60 pattern). The combinator build
below **does not depend on this ruling** and proceeds in parallel; only the
"should we also declare+export `Either`" question waits on the Architect. If the
Architect rules the other way (carry a distinct `Either`), it's a small additive
`data Either` + its own combinators — a scoped add, not a rework.

## Combinator scope (mechanical — proceeds regardless of the Either ruling)

Both types are prelude-declared. **`Option` already has `option_map` + `instance
Functor Option`** (`Core/LawfulFunctors.ken:232`/`:259`) — do **not** re-declare
those; reuse them. Add the combinators the campaign names, each with its laws as
`Ω`/`Prop` proof terms, zero `Axiom`, over the inductive sums:

1. **`Option`** — `getOrElse (a) (d : a) (x : Option a) : a`,
   `isSome (a) (x : Option a) : Bool`, `orElse (a) (x y : Option a) : Option a`.
   Laws: `getOrElse d None = d`, `getOrElse d (Some v) = v`; `isSome None =
   False`, `isSome (Some v) = True`; `orElse None y = y`, `orElse (Some v) y =
   Some v` (left-biased) + `orElse x None = x` if it falls out.
2. **`Result`** — `mapErr (e f a) (g : e → f) (x : Result e a) : Result f a`,
   `andThen (e a b) (k : a → Result e b) (x : Result e a) : Result e b`,
   `unwrapOr (e a) (d : a) (x : Result e a) : a`. Laws: `mapErr g (Ok v) = Ok v`,
   `mapErr g (Err u) = Err (g u)`; `andThen k (Ok v) = k v`, `andThen k (Err u) =
   Err u`; `unwrapOr d (Ok v) = v`, `unwrapOr d (Err u) = d`. **Field-order
   caution:** `Result e a = Err e | Ok a` — `Err` is the FIRST constructor
   (`prelude.rs:1080`); the effect layer already tripped on this. Get `Err`/`Ok`
   the right way round in every law.

Drop any law needing a combinator not in scope rather than proliferate helpers
(subsume-don't-proliferate); ship the combinator with its definitional equations
and say so.

## Class-instance showcase — a NOTED opportunity, not required this WP

Now that Core is complete (Monad landed), `Option` and `Result e` are natural
`Monad` instances — `andThen` **is** bind, `getOrElse`/`unwrapOr` are eliminators,
`option_map` is already the Functor. Repackaging these as `instance Monad Option`
/ `instance Monad (Result e)` would be an elegant Data↔Core bridge. **Do NOT force
it into DS-3** (campaign scope is combinators); if the combinators land clean and
cheaply lift to instances, note it and it becomes a named follow-on (DS-3b), same
valve discipline as DS-8c. Combinators first.

## Package home (judgment call — Steward recommendation)

**Recommend one new entry `catalog/packages/Data/Sums/Sums.ken`** grouping both
L2-sum combinator families (Option + Result) — one entry, not two, per subsume-
don't-proliferate on package count (mirrors `Data/Collections/Collections.ken`
holding all List combinators). Foundation confirms; if the enclave rules a
distinct `Either` in, it joins this entry.

## Boundary / constraints

- **AC1 — kernel-untouched, zero new elaborator capability, zero `trusted_base()`
  delta.** All combinators are ordinary structural recursion / case-split on
  declared sums — no dependent-match, no new sort, no surface feature. Executable
  before==after `trusted_base()` set-diff. **If anything needs a new capability,
  STOP and hand back** — it shouldn't.
- **Zero `Axiom`/`postulate`/`sorry`** in any proved law.
- **Outer-ring only** — `crates/ken-kernel`/`Cargo.lock` diff empty. (The `Either`
  spec reconcile, if the ruling subsumes, is a separate spec-enclave erratum —
  spec/+conformance, not this build.)
- **AC8 — discriminators flip** accept→reject on a wrong witness at the named law
  (e.g. a `getOrElse` that returns `d` on `Some`, an inverted `mapErr`), specific
  error variant, not bare `is_err()`.
- Watch the recurring dot-projection/`λ`-in-type Finding (DS-7 Finding 1): named
  total `fn` δ/η-reducing to the spelling; file an Ergo Finding, don't smuggle a
  capability.

## Gate

Two lanes. **(a) Combinator build** — normal ring: Foundation → foundation-qa →
@architect (fidelity vs frame + zero-Axiom/zero-`trusted_base`) → git_request to
Steward, CI-gated (real catalog `.ken` + acceptance test). **(b) The `Either`
ruling** — @architect design-shape confirmation of the subsume recommendation +,
if subsumed, a spec-author/CV reconcile erratum on README:42 (doc/spec, DS-5 §60
pattern). Lane (a) does not block on lane (b). Own retro. Resource discipline
(`CARGO_BUILD_JOBS=2`, scoped `-p`). Flag every judgment call for the operator's
log — especially the Either ruling outcome and any dropped law.
