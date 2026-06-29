# WP K3 — Content-addressed value model

> **Status:** Steward frame — **FAST-PATH.** F4 already elaborated `spec/40-runtime/
> 41-values.md` + `44-capacity.md` to implementation resolution and shipped a
> conformance corpus + benchmark harness (`dec_590qm2kqqak9w`, `dec_1darmky1az6gq`).
> So K3 needs **no full spec elaboration** — spec-leader does a light "is this
> build-ready?" confirm pass (not a re-elaboration), then the Runtime team builds
> directly from the on-`main` 41/44 + the design doc.
>
> **Team:** Runtime · **Deps:** F4 (done), K1 (done) · **Size:** M · **Risk:** ★★
> · **Parallel** with K1/K2/K2c (no kernel dependency on the value rep). Feeds
> **X1** (the reference interpreter — the oracle) and **X2** (hardening).

## Objective

The runtime value model: heterogeneously-typed values with **structural O(1)
equality** and **global content-addressed dedup** — the substrate the interpreter
(X1) evaluates over.

## Fixed inputs — settled by F4; do NOT reopen

These are the F4 design (`docs/design/content-addressing.md` + the deepened
`41`/`44`), six settled OQs treated as FIXED:

- **Content addressing:** canonical byte encoding (deterministic + total —
  Map/Set encode identically regardless of insert order); **FNV-1a 64-bit** hash;
  a **content-store / intern index** for global dedup; **hash is verified on
  intern** (collision → loud refusal, never silent), not trusted.
- **O(1) equality** is one comparison on the interned handle (`FNV + memcmp`
  fast path); **no Leech/`mmgroup` on the hot path** (`OQ-6`).
- **Heterogeneously-typed values from day one** — `Int` (arbitrary precision; **no
  precision-losing uniform-number model**), float, bool, handles, structs — per
  the concrete immediate-vs-interned table (`41 §5`).
- **Loud at the limit:** at capacity the store **refuses loudly** with the F4
  error type (`44 §2`), never silently corrupts; reclamation is manual + region
  (`44 §3`), GC deferred to X2.
- **Introspection:** the `StoreStats` shape (`41 §7`) for dedup/occupancy.

## Scope

**IN:** the value representation (immediates + interned); the content-store (
canonical encode → FNV-1a → verify → intern/dedup); **O(1) handle equality**; the
immediate-vs-interned boundary; `StoreStats` introspection; loud-refusal at
capacity; the value/runtime-core module + tests; make the F4 conformance corpus
(`conformance/runtime/values/` + `capacity/`) pass.

**OUT:** **evaluation** (X1 — the interpreter is a separate WP; K3 is the values
it computes over, not the reduction engine); **GC / advanced reclamation** beyond
F4's manual+region (X2 hardening); the **native backend** (X3); arena/index
resize and arena-chaining tuning (F4 flagged these as K3/X2 forward-pointers —
implement the straightforward version, defer the hardening to X2).

## Acceptance (testable — the F4 conformance corpus is the spine)

1. **Global dedup:** two values built independently with **identical content**
   share **one** store slot (not two).
2. **O(1) equality:** equality of two interned values is a single handle/`memcmp`
   comparison — not a structural walk.
3. **Canonical encoding deterministic + total:** a `Map`/`Set` encodes identically
   regardless of insertion order (F4's determinism test).
4. **No precision loss:** large integers round-trip exactly (no `f64`-style
   uniform-number truncation).
5. **Loud at limit:** at capacity, the store returns the F4 refusal error — it
   does not silently drop or corrupt.
6. **F4 corpus green:** `conformance/runtime/values/` + `capacity/` pass.

## Guardrails

- **Hash is verified, not trusted** — an FNV-1a collision must trigger the loud
  path (F4), never a silent wrong-dedup. The whole O(1)-equality claim rests on
  intern correctness.
- **No `mmgroup`/Leech on the hot path** (`OQ-6`) — `FNV + memcmp` only.
- This module is the **X1 interpreter's substrate and the differential-test
  oracle's value layer** — correctness here is load-bearing downstream.

## The K1/K2 lesson — applies here (COORDINATION §7, sharpened by K2)

Exercise the *property*, and **invoke every guard at least once**: test dedup
with **≥2 distinct constructions of equal content** (not one); cross the
**immediate↔interned boundary** explicitly; and **actually drive the loud-refusal
path** (the at-capacity guard) and the **collision-handling path** — a guard with
no invoking test is exactly the K2 `check_respect` trap (a deferred/unexercised
check that admits corruption). No silent best-effort on the dedup/verify path.

## Logistics

Branch `wp/K3-value-model` cut from `origin/main`. Runtime team (`runtime-leader`
+ `runtime-implementer` [Sonnet, medium effort] + `runtime-qa`). `scripts/ken-cargo
-p <runtime-crate>`. Ring: implementer builds → QA verifies independently →
merge Decision (**Architect** + **Spec** on `/spec`+`/conformance` paths) →
Integrator → retros. Value-representation / store design Qs → Architect;
behavioral-contract Qs → Spec.
