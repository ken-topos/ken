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

> ## ⛔⛔ STOP — THIS FRAME IS STALE AND MUST NOT BE BUILT. Steward, 2026-07-27
>
> **`SPEC-STORE-SPLIT` (merged `c631841d`) retired the normative claims this
> frame calls fixed inputs.** ⛔ Do not release, do not elaborate, do not queue
> `A3` in front of it. It needs a **re-cut against the relaxed contract**, which
> is Steward-owned and not yet done.
>
> ⭐ **The dangerous part is §0's own instruction** — *"Your job is **not** to
> re-open them"* — which **pre-forbids exactly the correction this frame now
> needs.** A team following it faithfully would build against retired normative
> text and read the frame's prohibition as authority not to check. That is why
> this banner replaces the table rather than sitting beside it.
>
> ### What died, measured at `origin/main = 7d87904f`
>
> | this frame's fixed input | the landed contract |
> |---|---|
> | `OQ-hash` — in-process addressing **IS** FNV-1a + `memcmp` | `41 §3`: the spec *"fixes **no** in-process hash, collision strategy, probing policy, load factor, or identifier scheme."* `§3b` names **FNV-1a, linear probing, table growth, arena allocation, slot numbering** as *"examples of **private choices, not conformance requirements**."* |
> | `OQ-7` — compound/identity-bearing values **content-addressed** | `41 §5`: canonical compounds have deterministic canonical bytes and *"runtime representation **private**."* Content-addressing is no longer required. |
> | `OQ-witness` — process-level store stats *(slots, dedup rate, arena bytes, Merkle root)* | `41 §7` is *"realization **revised by `SPEC-STORE-SPLIT`**"*: the stat set is profile-specific and *"**none is a portable required field**."* |
> | §3's mandated outline: encode **closures by code-pointer/id + captured-env hash** | ⛔ `41 §2.1` — closures have **no** canonical hash and publication **refuses before bytes exist**. Directly forbidden, not merely optional. |
> | §3's open-addressing index, monotonic slot-id counter, slot-id width | `41 §3b` — all private, non-conformance. |
> | `OQ-6` — no Leech/Golay/Co₀ on the hot path | ✅ **survives** (`41 §3`). |
> | `OQ-5`, `OQ-gc` | ⚠ **not re-read.** They cite `44 §2`/`§3`, which the split rewrote heavily. ⛔ Do not assume they survive because they are not listed as dead.
>
> ⚠ **`A3` (catalog-coverage walker) exists ONLY to be queued in front of this
> node** — it feeds no gate and has no other dependant. Its urgency is entirely
> derived from `F4`, so it is on hold with it. ⛔ Do not frame `A3` as idle-team
> filler; that would be building a road to a retired destination.
>
> ⇒ **What is actually owed:** decide whether a node about *content-addressing as
> a value-model requirement* still has a subject once content-addressing is a
> private runtime choice — and if it does, what its subject now is. That is a
> re-cut, not an edit.

## 0. ⛔ SUPERSEDED — the "already decided" inputs below are RETIRED

⛔ **Read the banner above first. The table that stood here is retired and has
been removed rather than annotated**, because it was a list of things a reader
was instructed not to question. Its contents are reproduced in the banner's
comparison table with the landed contract beside each row, which is the only
form in which they are still safe to read.

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
- **Clean-room:** build from `/spec`; never read `local/refs/` (permissive or
  otherwise) as an implementation source (CLEAN-ROOM.md). The design is
  ours, from the spec.
- **Edges (§9):** behavioral-contract Q → Spec leader; component-design Q →
  Architect. The Architect reviews the merge Decision (where the `60-security`
  /design invariants are checked) — no pre-review edge.
- **Done:** acceptance §5 met + retro in (COORDINATION §10). Hand `merge_ready`
  to the Steward as `message_type: git_request` (Bug 13 mapping).
