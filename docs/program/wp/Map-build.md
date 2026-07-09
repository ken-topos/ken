# Map-build — implement the proved BST `Map k v` in `catalog/packages/collections` and retire the opaque primitive (VAL2 #8 / OQ-A)

**Steward frame → Team Foundation (build).** The spec is **already fully
elaborated and on `main`** — `spec/50-stdlib/52-map.md` (the proved bare ordered
BST) + `conformance/stdlib/map/seed-map.md` (the conformance seed). This frame is
the Steward scope/AC wrapper; **`52-map.md` is the canonical design authority —
Foundation implements it, does not re-design it.** Owner: **Foundation**. Gate:
**Architect soundness** (AC1 net-negative + the `antisym → Equal` canonical-key
localization) + **conformance-validator** (the build satisfies the merged
`seed-map.md`) + **Foundation QA** + **CI**. Findings → **Steward**.

Base: `origin/main`. Branch (Foundation cuts it fresh): **`wp/Map-build`**.

## Objective

Deliver `Map k v` as a **proved** pure ordered binary search tree in the
`collections` package, and **retire the opaque `Map`/`Set` primitive** — so the
trusted base **shrinks** (Map/Set become derived `data`, joining
`List`/`Option`/`Result`). This is `letter-frequency`'s unblock (insert + lookup
+ ordered iteration) and the OQ-A "proved-pure-map" delivery.

## Settled decisions — FIXED INPUTS, do not reopen

Pinned from `52-map.md` + the OQ-A register; a weaker model must not relitigate
these:

- **Carrier: bare ordered BST** `data Tree k v = Leaf | Node (Tree k v) k v (Tree
  k v)`, **no balance metadata** (FORK-2, Architect). Balance/O(log n) is a
  *perf* follow-on, orthogonal to correctness — **out of scope here.**
- **`Map k v` = `Tree` + the `Ordered` invariant**, keyed by **`Ord k`**; all
  proofs **parametric in the `Ord k` dictionary**.
- **`antisym → Equal` is localized to the overwrite/uniqueness law ONLY** (§2.1 /
  §5.3, ADR-0010 canonical-key) — a **hard soundness constraint**. Sound over
  `Int`/`Char`/`Bool` keys; a non-canonical carrier (`Decimal`) would inhabit
  `Bottom`, so the localization is load-bearing. Core `lookup` laws lean on
  `refl`/`total`, **not** `Equal`-promotion.
- **API has NO `delete`** — deferred (operation *and* its proof together, §7).
  Do not ship an unproved `delete`. `toList` **permutation** law is likewise a
  named follow-on; this WP proves the **ordered** `toList` law (§5.3).
- **HAMT / content-addressed fast-map is a SUGGESTION-ONLY future perf follow-on
  — NOT this WP.** The surface API (§4) is representation-independent, so the
  later rep swap is non-breaking; do not pre-build it.
- **No new kernel rule/former/primitive** (`16 §1`: the laws are `Ω`
  propositions). Adding a `declare_primitive` or a kernel former = **over-built**.

## Mandated deliverable outline (each ends in a concrete, implementable choice)

1. **Carrier + API** in `catalog/packages/collections/` (exact file path Foundation's to
   fix — alongside the landed `List`/`Nat`; `collections.ken` exists). Implement
   the **7 operations exactly as specified in `52-map.md §4`**: `empty`,
   `insert`, `lookup`, `member`, `toList` (in-order, ascending), `fromList`,
   `fold` (ascending-key-order contract). All via **`declare_inductive` /
   `declare_def`** (kernel-rechecked). `k × v` is the **Σ-pair** (`13 §3`), not
   the inductive `Prod`. Reuse the landed `list_append` for `toList`.
2. **The proof obligations from `52-map.md §5`** as **real proof terms** (AC3):
   the `Ordered` invariant, `insert`-preservation, the **3 core `lookup` laws**,
   the **`toList` ordered law (§5.3, load-bearing)**, and `lookup`↔`toList`
   agreement. Parametric in `Ord k`; `antisym → Equal` invoked **only** at the
   overwrite law.
3. **Retire the opaque primitive** — remove from
   `crates/ken-elaborator/src/prelude.rs`: the `declare_primitive(… ,
   PrimReduction::OpaqueType)` for `Map` (**~line 565**) and `Set` (**~571**),
   the `map_id`/`set_id` struct fields (**~62-63, 131-133**), and the
   `globals.insert("Map"/"Set" …)`. Point `Map`/`Set` at the derived
   `collections` definitions (or drop the built-in binding if the surface
   resolves them from the package). **This is a `ken-elaborator` (prelude) edit,
   NOT a `ken-kernel` edit** — the kernel crate stays byte-identical (AC1).
4. **Flip `es2_acceptance.rs`** — the test
   `map_set_reclassed_primitive_stay_in_trusted_base` (**~line 228**) currently
   asserts `Map`/`Set` are `Decl::Primitive` **in** `trusted_base()`. It **flips**
   to assert they are **derived** (`Decl::Inductive`/`Def`) and **out of**
   `trusted_base()` — the net-negative direction. Update the module doc lines
   (7-8, 224-225) accordingly.

## Acceptance criteria (from `52-map.md §9`, AC1–AC5)

- **AC1 — net-negative TCB (load-bearing).** `crates/ken-kernel/` diff **empty**;
  `trusted_base()` **shrinks** by exactly `Map`/`Set` (**zero additions**); all of
  `Tree`/ops/laws via `declare_inductive`/`declare_def`, **`declare_primitive` /
  `declare_postulate` absent** (grep the producer, not the report). The retirement
  is a *shrink*, not "unchanged" — verify the `trusted_base()` set before/after.
- **AC2 — operations correct end-to-end** through the **real interpreter**: the
  `letter-frequency` shape (build + iterate a frequency map), insert/lookup
  round-trips, `toList` ascending — driven by the merged `seed-map.md`, **not
  hand-fed** (construct via real `insert`, read back via real `lookup`/`toList`).
- **AC3 — proved, not stubbed.** The §5 invariant + laws are **real proof terms**;
  the discriminator **flips at the map-proof level** — swapping a §5 law for
  `Axiom` must make a downstream obligation **fail** (exercised, not textual
  absence). This is the load-bearing "proved not tested" net (§5.4) — must fail
  **for the right reason**.
- **AC4 — no regression.** `cargo test --workspace` green; lawful `Ord` and the
  rest of `catalog/packages/` behave identically pre/post.
- **AC5 — build-lane retirement is REAL (hard AC).** `prelude.rs` primitive
  **removed** + `es2_acceptance.rs` assertion **flipped**, **verified as landed**
  (not just asserted). Deliverables 3 + 4 above.

## Guardrails (do-not-reopen)

- **Kernel untouched** — Map/Set retirement is a `ken-elaborator`/`packages`
  change; `ken-kernel` byte-identical. If you find yourself editing `ken-kernel`,
  STOP and escalate.
- **No balance, no `delete`, no HAMT** — all named follow-ons, not this WP.
- **`antisym → Equal` only at the overwrite law** — do not let it leak into the
  core `lookup` laws (that would over-constrain the key type).
- **`52-map.md` is the spec** — implement it; do not re-derive the API or the
  proof structure. Any genuine spec gap → Steward (not a silent local decision).

## Sequencing

- **Lane:** Foundation. Branch `wp/Map-build` off **current** `origin/main`.
  **Independent** of `sct-completeness` (Kernel) and `[FS]` (enclave) — disjoint
  surfaces, no contention; **can start now.**
- **Gate:** Foundation builds → **Architect soundness** (AC1 net-negative +
  `antisym → Equal` localization) + **conformance-validator** (drives the merged
  `seed-map.md` for AC2/AC3) + **Foundation QA** + **CI**. `52-map.md` +
  `seed-map.md` are already on `main` — no new `/spec`/`/conformance` authoring;
  this is a pure build against the merged spec.
- **Downstream:** unblocks `letter-frequency` (VAL3 Rosetta) and the OQ-A
  proved-map delivery. HAMT perf follow-on remains suggestion-only.
