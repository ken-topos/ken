# WP lawful-classes-lane — Ord Char, Num/DecEq Decimal (re-homed instances)

**Team: Language (owns `packages/lawful-classes`). Base: `origin/main`.**
Steward frame. These are the **three forward obligations** the Decimal/Char
DEMOTE (`4eea2072`) + erratum (`fcfff1c6`) explicitly deferred and re-homed here
(seed `conformance/surface/numbers/seed-decimal-char-demote.md`, the RE-HOMED
sections). Runs **parallel** to the F2/F3 tranche (independent lanes).

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
- **`Num Decimal`** and **`DecEq Decimal`** — by **real structural proof**.
  `Decimal = MkDecimalPair Int Int` is a genuine inductive carrier
  (`decimal_char.rs` ~L93–95), so the laws (reflexivity/comm/`sound`/`complete`)
  are a structural proof case-splitting `MkDecimalPair` that **bottoms out at
  `DecEq Int`/`Num Int`'s audited-delta `Axiom` leaves** (`lawful_classes.ken`
  `DecEq Int` ~L80–84). **Zero-NEW-delta** (no NEW postulate beyond `Int`'s
  existing visible ones) — but **not** `Axiom`-free.

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
2. **`DecEq Decimal`** + **`Num Decimal`** instances — structural proofs over
   `MkDecimalPair`, bottoming at `DecEq Int`/`Num Int` `Axiom` leaves.
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
