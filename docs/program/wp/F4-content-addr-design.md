# F4 — Content-addressing + value-model design (shovel-ready brief)

> **Owner:** Team Foundation · **Size:** M · **Risk:** ★★ · **Branch:**
> `wp/F4-content-addr-design` · **Feeds:** K3 (value model), X2 (runtime
> hardening).
>
> **Pipeline status: Steward frame → _awaiting spec-leader elaboration_ → Team
> Foundation.** This is the Steward's frame (scope, deliverable outline,
> acceptance, settled-decision pinning). Per the WP release process (steward
> playbook §2c), the **spec-leader elaborates it to full team-ready rigor**
> (deepening the `/spec 40-runtime/41,44` + `conformance/runtime/` detail) on
> this branch **before** Foundation is released. Foundation must **not** start
> until the elaborated package is on `main` and the WP is kicked off. Supersedes
> the terse recap in `03-program-of-work.md` (F4).

## 0. Read this first — the design is already decided; you are not re-deciding it

The hard design forks for F4 were **already resolved by the spec enclave + the
operator (2026-06-27)** and are normative in `spec/40-runtime/41-values.md` and
`spec/40-runtime/44-capacity.md`. Your job is **not** to re-open them — it is to
turn those settled stances into a **concrete, implementable module design + a
small benchmark** that K3/X2 can build against. Treat the following as **fixed
inputs** (cite them; do not relitigate):

| Decision | Resolution (normative source) |
|---|---|
| `OQ-hash` | In-process addressing = **FNV-1a (non-crypto) + `memcmp`** on collision. A crypto/Merkle hash is a *separate* concern for serialization only (`38 §1`). Two hashes, two jobs. (`41 §3`) |
| `OQ-5` (capacity) | **Engineering-chosen** capacity with **wide handles** (48-/64-bit slot field) → **no practical ceiling**; **loud refusal** at any limit (never silent drop/alias/corrupt). No Leech 196,560 ceiling. (`44 §2`) |
| `OQ-6` (lattice) | Leech/Golay/Co₀ math is **NOT in the core and NEVER on the allocation hot path**. It exists only as optional research/stdlib packages (WS-R). (`44 §4`) |
| `OQ-7` (interning boundary) | **Scalars immediate; compound/identity-bearing values content-addressed.** Exact tiny-aggregate boundary is empirical X2 tuning. (`41 §5`) |
| `OQ-gc` (reclamation) | **Manual + region-scoped now**; automatic GC is a deferred, semantics-invisible implementation detail (no language fork). (`44 §3`) |
| `OQ-witness` (introspection) | Process-level store stats only (slots, dedup rate, arena bytes, Merkle root); **never** per-value identity/provenance. (`41 §7`) |

**The stale `mmgroup` question is therefore answered:** because the lattice
machinery is out of the core (`OQ-6`), Ken takes **no `mmgroup`/lattice
dependency on the value-model or allocation path**. Do **not** add it. Record
this resolution explicitly in the design doc (§4 below). If a lattice *research*
package is ever built (WS-R, out of F4 scope), `mmgroup` (BSD-2, attribution)
can be evaluated then — note it as a forward pointer, nothing more.

## 1. Deliverables

1. **A design doc** at `docs/design/content-addressing.md` realizing `41`+`44`
   at implementation resolution (outline mandated in §3). This is the primary
   deliverable and the contract K3/X2 build against.
2. **A small-scale benchmark harness** (a `foundation`/bench crate or a
   `criterion` bench under the relevant crate) that exercises intern / dedup /
   equality on synthetic data and prints the metrics in §5. Benchmark **code +
   recorded results** (a short results table in the design doc).
3. **ADR confirmation.** Verify the content-store decision is captured. `41`/`44`
   + `spec/90-open-decisions.md` already carry the *normative* resolution; F4
   does **not** need a new ADR unless you find a genuine gap. If you do, add
   `docs/adr/0009-content-store.md` pointing to `41`/`44` (don't duplicate them).
   State which you did and why in the PR.

**No production allocator in F4.** F4 is *design + small benchmark*. The real
content-addressed heap implementation is **K3** (value model) and **X2**
(hardening). F4 de-risks and specifies; it does not ship the runtime store.

## 2. Scope boundary (what F4 is / is not)

- **In:** the concrete addressing/dedup design (canonical encoding, hashing,
  index, slot ids); the immediate/interned boundary as a concrete starting rule;
  capacity + loud-refusal behavior spec; reclamation model; introspection
  surface shape; the benchmark validating the approach at small scale.
- **Out:** the kernel conversion fast-path wiring (K2c/K3), the production index
  data structure tuning (X2), scale/limits validation (X4), serialization/Merkle
  (`38`), any lattice package (WS-R), `space`-cell mutable state (`36 §4`).

## 3. Mandated outline for `docs/design/content-addressing.md`

Each section must end in a **concrete, implementable choice** (or a bounded
implementer-latitude note with guardrails) — not a survey. Sub-bullets are the
specific questions you must answer.

1. **Canonical byte encoding.** A deterministic byte form per value kind so
   "same value ⇒ same bytes ⇒ same hash" (`41 §3`). Specify the encoding for:
   constructor applications (`data`), records/Σ (field order rule — declaration
   order, normative), `String`, `Bytes`, `Array`/`Map`/`Set` (Map/Set: define
   the canonical ordering of entries so two equal sets encode identically),
   closures (by code-pointer/id + captured-env hash), and big integers
   (sign-magnitude, minimal limbs). State the tag scheme that disambiguates
   kinds. **Determinism and canonicality are the correctness bar** — call out
   every place order/normalization matters.
2. **Hashing.** Specify **FNV-1a 64-bit** (offset basis `0xcbf29ce484222325`,
   prime `0x100000001b3`) over the canonical bytes; `memcmp` of canonical bytes
   to resolve hash collisions exactly. Justify non-crypto here (in-process,
   adversary-free) vs. the crypto hash reserved for serialization (`38 §1`).
3. **The store index + slot ids.** Design the `(arena_root, hash) → slot_ref`
   index: data structure (recommend an open-addressing hash table keyed on the
   64-bit hash, buckets carrying slot-ids, `memcmp` on the canonical bytes to
   disambiguate collisions), the **monotonic slot-id counter**, slot-id width
   (start **64-bit** per `OQ-5`), and the append-mostly arena page layout. The
   **intern algorithm** must be explicit: encode → hash → probe → `memcmp` → hit
   returns existing slot / miss appends + assigns next id.
4. **Dedup + the lattice non-dependency.** Show global dedup falls out of the
   intern path (one slot per *distinct* value — the accounting point in
   `44 §2`). Explicitly record: **no `mmgroup`, no Leech quantizer, no Co₀
   canonicalization on the path** (`OQ-6`); forward-pointer the optional research
   roles in `44 §4` and nothing more.
5. **Immediate vs interned (`OQ-7`).** State the concrete starting rule: scalars
   (`Int` small, machine ints, `Bool`, `Char`, `Float`, `Decimal`) immediate;
   compounds interned. Give a **recommended** tiny-aggregate cutoff (e.g. intern
   all aggregates initially; flag the 2-field-tuple question as X2 empirical) —
   recommend, don't over-engineer.
6. **O(1) structural equality.** Show `a == b` on heap values = slot-id compare;
   scalars native (`41 §4`). One paragraph; it's the headline property.
7. **Capacity + loud refusal (`OQ-5`).** Specify behavior at the (engineering)
   limit: a clear typed error, never silent drop/alias/corrupt; dedup-aware
   accounting (capacity is in *distinct* values). The exact width is an X2/X4
   constant — name it, don't fix it.
8. **Reclamation (`OQ-gc`).** Manual + region-scoped only (`clear`/`reset`,
   `madvise(MADV_DONTNEED)`, `space`-bounded working sets). State explicitly that
   automatic GC is deferred and **semantics-invisible** when added.
9. **Introspection (`OQ-witness`).** The `witness` surface exposes process-level
   store stats only; never per-value identity. Specify the stat set shape.

## 4. The benchmark (deliverable 2)

A small synthetic harness — **not** the production store — sufficient to show the
approach is sound at small scale:

- Intern **N = 10⁴–10⁶** synthetic values with a controlled duplicate ratio
  (e.g. 50% repeats); a mix of kinds (records, strings, small arrays, bignums).
- **Report:** intern throughput (values/s), **measured dedup rate** vs. expected,
  memory per distinct value, and a check that equality is a slot-id compare
  (constant-time, not traversal). Include a **loud at-limit test**: drive a small
  artificial capacity to its bound and assert a clean error (not a silent
  failure).
- Put the results table in the design doc (§3.4). Targets are *sanity*, not perf
  gates (perf is X2/X4): dedup rate within tolerance of expected; intern
  scales ~linearly; equality independent of value depth.

## 5. Acceptance criteria (testable — the definition of done)

1. `docs/design/content-addressing.md` exists and covers every §3 section, each
   ending in a concrete implementable choice, citing `41`/`44` for the settled
   stances (no settled OQ reopened).
2. The canonical-encoding rules are deterministic and total over the value kinds
   in `41 §1–2`; two structurally-equal values (incl. a `Map`/`Set` with
   differently-ordered inserts) are shown to encode to identical bytes.
3. The benchmark runs (via `scripts/ken-cargo`) and records: dedup rate matches
   expected within tolerance, equality is slot-id (O(1)), and the at-limit case
   fails **loudly**.
4. The `mmgroup`/lattice **non-dependency** is recorded with its `OQ-6`
   rationale; any reused math has clean license provenance (none expected in F4).
5. ADR status stated (confirmed-sufficient or new 0009 added).
6. Conformance/lint green; markdown wrapped at 80 cols (mermaid for any diagram).

## 6. Do NOT re-open (guardrails)

The six DECIDED OQs in §0. If you believe one is genuinely wrong or
under-determined for implementation, that is a **`question` to the Spec leader**
(behavioral contract) or the **Architect** (component design) per COORDINATION
§9 — **not** a unilateral redesign. Default to the settled stance.

## 7. Logistics

- **Deps:** F1 (done). **Build/test:** `scripts/ken-cargo -p <crate>` only
  (COORDINATION §12); full-workspace/bench-release runs in CI.
- **Clean-room:** build from `/spec`; never read `local/refs/` or the prototype
  (CLEAN-ROOM.md). The design is ours, from the spec.
- **Edges (§9):** behavioral-contract Q → Spec leader; component-design Q →
  Architect. The Architect reviews the merge Decision (where the `60-security`
  /design invariants are checked) — no pre-review edge.
- **Done:** acceptance §5 met + retro in (COORDINATION §10). Hand `merge_ready`
  to the Integrator as `message_type: git_request` (Bug 13 mapping).
