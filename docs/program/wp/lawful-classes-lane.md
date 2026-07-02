# WP lawful-classes-lane — Ord Char, Num/DecEq Decimal (re-homed instances)

**Team: Language (owns `packages/lawful-classes`). Base: `origin/main`.**
Steward frame. These are the **three forward obligations** the Decimal/Char
DEMOTE (`4eea2072`) + erratum (`fcfff1c6`) explicitly deferred and re-homed here
(seed `conformance/surface/numbers/seed-decimal-char-demote.md`, the RE-HOMED
sections). Runs **parallel** to the F2/F3 tranche (independent lanes).

**SCOPE UPDATE (build-time, both rulings independent of each other):**
- **`Num Decimal` re-defers** — `class Num`/`instance Num Int` were never
  actually built (only `Eq`/`DecEq`/`Ord` landed); the `Num Int` floor this
  deliverable was framed to bottom out at doesn't exist. Steward ruling
  (b): dropped from this WP, re-homed onto a future `class Num` WP.
- **`DecEq Decimal` re-defers too** — found to be a genuine soundness hole,
  not a proof-difficulty gap: `Decimal`'s non-canonical `(coeff, exp)`
  carrier means `decimalEq` is an `Eq` (equivalence via alignment), not a
  `DecEq` (decision procedure for the kernel's structural `Equal`) —
  `decimalEq (MkDecimalPair 10 (-1)) (MkDecimalPair 1 0)` reduces `True`
  (both denote 1.0) but the pairs are structurally distinct, so `sound`
  would inhabit `Bottom`. Caught before landing (never shipped unsound).
  Both Decimal instances now sit behind a single decide-once design gate
  (canonicalize the carrier, or a setoid/quotient `Eq Decimal`), tracked in
  `90-open-decisions.md` — not this WP's to resolve.
- **`Ord Char` shipped** (`packages/lawful-classes/lawful_classes.ken`,
  conformance in `conformance/stdlib/classes/seed-lawful-classes.md`'s
  `char-ord-laws-carried-not-stubbed`). Along the way: transporting via the
  separately-defined `leqChar` view (rather than a direct `.`-projection off
  `Ord Int`'s own `leq`) hit a real kernel gap — `conv_struct`
  (`ken-kernel/src/conv.rs`) has no Eq×Eq congruence arm, so two
  syntactically-different-but-fully-reducible-to-identical stuck `Eq`
  propositions don't converge. This is K6's first REAL customer (a SOUND,
  positional fix would have closed it, unlike the `Eq Bool` case). Shipped
  without a kernel change: transporting `leq` itself via the SAME
  `.`-projection makes every later field's comparison use the literally
  identical term, sidestepping the gap. See the `.ken` source's comment and
  `es4_classes_acceptance.rs`'s module doc for the full mechanism grounding.
  A small, in-scope elaborator fix also landed alongside: `elab_instance_decl`
  (`ken-elaborator/src/elab.rs`) never threaded `class_env` into the two
  `ElabCtx::new(...)` call sites that elaborate instance field bodies, so
  `.field` projection (needed for any transport-style instance) was
  unusable inside an `instance { ... }` body — a one-line `.with_classes`
  fix at each site, mirroring every other elaboration path.

## Objective

Deliver three law-carrying typeclass instances in
**`packages/lawful-classes/lawful_classes.ken`**, homed next to their `Int`
twins (orphan instances are a hard error — `33 §5`):

- **`Ord Char`** — by **transport** from `Ord Int`. Under refinement erasure
  `Char ≡ Int` (`21 §6.3`; `Char = {c:Int | isScalar c}`, `proj` = identity —
  see `decimal_char.rs`: `charToInt c = c`, `leqChar a b = leq_int a b`). So
  `Ord Char`'s laws **are** `Ord Int`'s laws: `leq = leqChar` (already reduces
  via the landed `leq_int` arm); `refl`/`antisym`/`trans`/`total` **reference
  `Ord Int`'s existing visible `Axiom` fields** (`lawful_classes.ken` ~L86–92,
  `antisym = Axiom` at ~L89) — **adding no new `Decl::Opaque`**. This is
  **zero-NEW-delta by transport**, NOT a fresh proof, and explicitly **NOT**
  `Axiom`-free-via-`proj`-injectivity (that earlier characterization was
  corrected in the erratum).
- ~~**`Num Decimal`** and **`DecEq Decimal`** — by **real structural proof**.
  `Decimal = MkDecimalPair Int Int` is a genuine inductive carrier
  (`decimal_char.rs` ~L93–95), so the laws (reflexivity/comm/`sound`/`complete`)
  are a structural proof case-splitting `MkDecimalPair` that **bottoms out at
  `DecEq Int`/`Num Int`'s audited-delta `Axiom` leaves** (`lawful_classes.ken`
  `DecEq Int` ~L80–84). **Zero-NEW-delta** (no NEW postulate beyond `Int`'s
  existing visible ones) — but **not** `Axiom`-free.~~ **CORRECTED (build-time,
  see SCOPE UPDATE above): this was an over-claim.** `decimalEq` is an `Eq`
  (equivalence via alignment on the non-canonical `(coeff, exp)` carrier),
  not a `DecEq` (decision procedure for the kernel's structural `Equal`) —
  the "structural proof bottoming at `Int`'s `Axiom` leaves" does not exist;
  postulating `sound`/`complete` would inhabit `Bottom`. Neither `Num
  Decimal` nor `DecEq Decimal` is delivered by this WP; both re-defer behind
  the decide-once carrier design gate (`90-open-decisions.md`).

## The discriminator is HONESTY, not zero-delta

(`[[lawful-class-instances-must-carry-law-proofs]]` narrowed — read the
narrowing.) The soundness gate is **`spec/50-stdlib/51-lawful-classes.md §5`**
("Laws PROVED, not postulated"): a law field must be a real proof/transport,
**not** an empty stub, `sorry`, or hole that merely *claims* proved. Because the
carriers here bottom at an opaque floor (`Int`), an **honest visible `Axiom`
(transport)** is the SOUND realization — the discriminating conformance case
must flip a **law-less / deceptive-empty-stub instance (rejected)** against the
**honest-visible/transport instance (accepted)**; it must **NEVER** reject an
honest visible `Axiom`. Over-strict "must be Axiom-free" would false-reject the
correct transport. `§6` carries the **zero-delta (inductive carrier) vs
audited-delta (primitive carrier)** axis — cite it for why `Int`-floored
instances are honestly not zero-delta.

## Deliverables

1. **`Ord Char`** instance — `leq = leqChar`; law fields transport-reference
   `Ord Int`'s visible `Axiom`s (no new `Decl::Opaque`).
2. ~~**`DecEq Decimal`** + **`Num Decimal`** instances — structural proofs over
   `MkDecimalPair`, bottoming at `DecEq Int`/`Num Int` `Axiom` leaves.~~
   **DROPPED (build-time)** — not deliverable on `Decimal`'s non-canonical
   carrier (soundness hole, see SCOPE UPDATE above); re-defers to a future
   WP behind the decide-once carrier design gate.
3. **Conformance cases** under `conformance/stdlib/classes/` (home:
   `seed-lawful-classes.md`; §5 flip-discipline `law-fields-real-proofs-not-
   postulates`). Author the honesty discriminators the demote seed named as
   placeholders — `char-ord-laws-carried-not-stubbed`,
   `char-deceq-collapses-on-codepoint` — plus a `Decimal` structural-proof-vs-
   stub flip. Each must **flip** (honest instance accepts with the expected
   delta; deceptive empty stub rejected).

## Hard ACs (each a gate)

1. **(soundness/HONESTY)** Each instance's law fields are real proofs or
   honest-visible transports/`Axiom`s — grep-confirm **no** empty stub / `sorry`
   / hole claiming proved. The discriminating case **flips** against a
   law-less/deceptive stub and does **not** reject an honest visible `Axiom`.
2. **(soundness/delta)** **Zero-NEW-delta**: grep `trusted_base_delta` /
   `declare_postulate` — the ONLY `Axiom`s reachable are `Int`'s **pre-existing
   visible** ones (via transport for `Char`, via structural leaves for
   `Decimal`); **no new `Decl::Opaque`** is minted. Kernel diff empty
   (`crates/ken-kernel/` untouched — this is `.ken` + conformance only).
3. **(fidelity)** `Ord Char` uses the landed `leq_int`/`leqChar` reduction
   (`leq x y` computes); the transport is byte-honest (references `Ord Int`
   fields, not a re-postulate). `Decimal` proofs actually case-split the pair
   (not a wildcard that dodges the structure).
4. **(build)** Workspace-green landing (K7 discipline: QA re-runs
   `./scripts/ken-cargo test --workspace` independently).

## Flow (thin — COORDINATION §9)

`language-leader → language-implementer → language-qa → Architect (soundness) +
CV (conformance) → Integrator`. One pass each. Soundness fork → Architect;
conformance/honesty-discriminator fork → CV; scope → Steward. No new parties,
no verbatim relays.

## Closes

The 3 tracked forward obligations from Decimal/Char DEMOTE. On merge, the demote
seed's RE-HOMED placeholders become live cases; Steward drops them from the
deferred set.
