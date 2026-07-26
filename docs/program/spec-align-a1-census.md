# `SPEC-ALIGN-A1` private-mechanism census

> **Bound base:** `cf8924a8e0cd2efd40805574f9ee30313bec6635`
>
> **Governing frame blob:** `db4fe0f020dcf95525e092e39ee688db58728004`
> (supersedes `5f1e1659fbb5a31597c62d8cfb3c87e8abaa5d99`)
>
> **Campaign blob:** `3d3d326a6162c9833b097eb9a1c510589aff382d`
>
> **Scope:** the candidate families nominated by the campaign and advisory,
> classified by the assertions their conformance consumers actually make.

## Reading and closure bound

Governance is read from `origin/main=4297e55c`, while the candidate source is
the assigned base above. This split is safe by object identity: `spec/`,
`conformance/`, and `crates/` have the same respective trees at both commits
(`7fce4373`, `00850b58`, and `15296a2a`). The `docs/` control differs.

The population is the union of:

1. every class-4 candidate named in the campaign's disposition ledger:
   in-process store addressing/index/arena mechanics, same-slot sharing,
   numeric representation, and formatter layout/fence policy;
2. every normative constraint in the cited source sections
   (`31 §1d`, `35 §2`, `41 §2`–`§5`, and `44 §1`–`§3`) that fixes one of
   those mechanisms; and
3. every `conformance/**/*.md` assertion reached by following those sections'
   conformance links, followed by a repository-wide sibling search for the
   mechanism spellings and their structural consequences.

The verdict comes from reading each reached row's `given`/`expect`/`why`, not
from counting its keyword hits. Preamble-only claims are labelled as such.
This closes the nominated A1 surface; it does not claim coverage of the other
63-file mission-grounding audit or any Track B/C fork.

The `crates/` column records dependence, not authority. A live implementation
site does not make its mechanism normative.

## Constraint-to-consumer census

`STOP-4` means the candidate remains class 4 but a live conformance row requires
an Architect-owned conformance-granularity ruling. `STOP-1/2` routes a semantic
or protocol axis out of A1. A separate `STOP-C7` identifies an empty-consumer
class-4 mechanism that A1 cannot clear independently of the logical-`space`
fork and live runtime work. `cannot-determine` would be a method finding; no
candidate below has that verdict.

| constraint | spec site | class / measured axes | conformance consumers that assert it | `crates/` dependence at the bound base | external consumer | verdict |
|---|---|---|---|---|---|---|
| Exact in-process FNV-1a addressing, full-byte `memcmp`, monotonic `u64` slots | `41 §3,§3b`; `44 §1,§2` | 4 for algorithm/allocation; 1 for no false merge and no slot-id resurrection | `runtime/capacity/no-lattice-on-hot-path`; `runtime/capacity/reset-retires-ids-never-resurrected` | `ken-runtime/src/{hash,store}.rs`; `ken-foundation/src/{hash,store}.rs`; artifact/package modules also reuse the exported helper over their own byte domains | None found for the in-process store key. `41 §3` explicitly assigns cryptographic/Merkle serialization a separate pipeline; sharing a helper in current code is dependence, not external authority over the store hash | **STOP-4/1** — live rows and C7/B2E entanglement; the current helper reuse does not promote the private store algorithm to class 2 |
| No Leech/Co₀/Golay machinery on the allocation path | `41 §3`; `44 §4` | 4, negative mechanism constraint | `runtime/capacity/no-lattice-on-hot-path` | absence assertion over the store dependency/path surface | None found | **STOP-4** — a live row asserts the private mechanism exclusion |
| Open addressing, linear probing, power-of-two mask, bucket locator/tombstone shape | `41 §3b`; `44 §1a` | 4, index strategy and bucket layout | **None.** The only `probing` hit is the Map-after-delete false control; the capacity preamble mentions tombstones but no case asserts this index policy | `ken-runtime/src/store.rs` `Index`; `ken-foundation/src/store.rs` | None found | **STOP-C7** — eligible by consumer/class alone, but the store family is C7-coupled and live B2E infrastructure consumes it; no isolated A1 clearance |
| Initial `2¹⁶` buckets, `0.70` threshold, double-and-rehash, single-writer resize | `44 §1a` | 4, sizing/resize strategy | `runtime/capacity/index-resize-preserves-slot-ids` fixes `2¹⁶`, `0.70`, and double/rehash in its stimulus | both store implementations fix `65536`, `0.70`, and resize | None beyond the conformance oracle | **STOP-4** — live consumer; only slot preservation is the outcome-level property |
| Fixed 4 MiB pages, bump allocation, dedicated oversized page and fresh tail | `44 §1b` | 4, arena organization | `runtime/capacity/arena-spans-pages-oversized-safe` | `ken-runtime/src/store.rs` fixes `PAGE_SIZE = 4 * 1024 * 1024`; Foundation has a different arena shape | None found | **STOP-4** — Control B is live; C7/B2E entanglement |
| Per-`space` indexes, recipient re-interning, manual reset, no automatic GC, retired slot ids | `44 §1,§3`; `41 §3b` | 1/4, logical isolation versus physical realization | `space-reset-is-isolated`; `escape-survives-sender-reset`; `reset-retires-ids-never-resurrected`; `no-automatic-gc` | `ken-runtime/src/store.rs` `Space`/`Arena`/`Index`; interpreter publication paths | The logical `space` contract | **STOP-1/4** — exactly fork C7 |
| Same-slot dedup, slot-observable structural sharing, O(1) equality | `41 §2,§4`; `42 §3.4`; `44 §1` | 1/4, equality/cost/physical-sharing axes | `runtime/seed-runtime.md` `dedup-shares-slot`; `runtime/values/{dedup-shares-slot,equality-is-slot-id}`; `runtime/evaluation/{det-sharing-dedups-by-slot,det-canonical-order-independent}`; `surface/collections/{string-nfc-canonically-equal-shares-slot,array-update-shares-unchanged-structure,structurally-equal-collections-o1-comparable}` | store interning and interpreter slot-bearing canonical values | Surface equality/cost and conformance-observed sharing | **STOP-1/4** — live population and C7. The superseded `user-deceq-keyed-map-canonical-identity` row is not counted |
| `Int` inline-`i64` fast path with heap promotion | `35 §2.1`; `41 §1,§5` | 4 for cutoff/layout; 1 for exact arbitrary-precision value | `runtime/values/int-small-to-bignum`; `runtime/values/immediate-vs-interned-boundary`; `surface/numbers/seed-numbers.md` AC1 and lowering discipline | `ken-interp/src/eval.rs` `Int`/`BigInt` split; runtime value/store boundary | None found for the cutoff itself | **STOP-4/1** — the row combines private cutoff with retained exactness |
| Bignum canonical tag `0x01` | `35 §2.1`; `41 §3a` | 4, private kind discriminator | `surface/numbers/int-arbitrary-precision-above-2^53`; `surface/numbers/f1-store-roundtrip-above-i128-byte-identical` | both canonical encoders fix `BIG_INT = 0x01`; boundary/native integer paths consume it | None found. CheckedCore/package encoders do not call the runtime value encoder, and `41 §3` separates serialization from in-process addressing | **STOP-4** — live rows require an Architect-owned conformance-granularity ruling; no class-2 consumer was derived |
| Sign-magnitude, `u64` limbs, minimal-limb/one-zero-limb canonicalization | `35 §2.1`; `41 §3a` | 4 for the exact sign/limb layout and normalization mechanism; 1/4 for one value having one content identity | `runtime/values/bignum-minimal-limb-encoding`; `surface/numbers/{f1-store-roundtrip-above-i128-byte-identical,f1-dedup-content-address-stable-across-paths}`; `conformance/README.md` F1 row | `ken-runtime/src/{canonical,native_int}.rs`; Foundation canonical encoder | None found beyond internal store/boundary read-back. No durable package-hash edge consumes these runtime bytes at the bound base | **STOP-4/1** — exact minimal limbs are asserted by live rows; unique content identity remains protected, but this WP cannot retarget those rows |
| `Decimal` inline `{i64 coeff, i32 exp}` fast path and heap tag `0x0A` | `35 §2,§2.3`; `41 §3a,§5`, contradicted by normative `18a §5.6.1` | 4 for runtime cutoff, layout, and tag; 1 for coefficient/exponent semantics and whether `Decimal` is primitive | `surface/numbers/seed-numbers.md` repeats the old lowering; `runtime/values/immediate-vs-interned-boundary` asserts the small/immediate split; `surface/numbers/demote-removes-decimal-char-primitives` instead requires derived `(coeff : Int, exp : Int)` and removal of primitive Decimal representation | runtime/Foundation still define `SmallDecimal` and `BIG_DECIMAL`; interpreter Decimal is constructor-derived, so occurrence cannot select authority | None found for the primitive tag or inline struct; CheckedCore/package encoders use a separate byte domain | **STOP-1/4** — live normative/conformance contradiction across type/producer, coefficient carrier, tag, and fast-path axes; occurrence cannot select authority, so a separately tracked reconcile is required |
| Formatter width | `31 §1d` | Not an A1 relaxation candidate; exact-value erratum axis | `surface/formatting/seed-canonical-format.md:169`–`:187` and `:610`–`:658` assert 88 while citing `31 §1d`; `surface/elaboration/seed-multi-binding-let.md` asserts 96 | `ken-elaborator/src/layout.rs` fixes `CANONICAL_WIDTH = 96` | Canonical source bytes and formatter clients | **STOP-ERRATUM** — non-empty consumer and live 96-vs-88 contradiction; `SPEC-31-WIDTH-ERRATUM` owns derivation and value choice |
| Exactly two ASCII spaces per indentation level | `31 §1d` | 4, layout policy | `surface/formatting/{breakable-syntax-never-exceeds-88-columns,indent-is-two-space-enclosing-relative}`; `surface/formatting/{let4-long-group-uses-one-flat-level,let4-compound-match-rhs-nests-under-binding,let4-group-in-match-arm-has-disjoint-semicolons}` | `ken-elaborator/src/layout.rs` fixes `INDENT_WIDTH = 2` | Canonical source bytes | **STOP-4** — multiple live exact-byte consumers |
| Four literate fence roles and exact formatter splicing/exemption policy | `31 §1d` | 1 for role meaning; 4 for formatting/splicing mechanics | `surface/formatting/{canonical-form-is-idempotent,whole-catalog-preservation-and-fixed-point,literate-prose-is-byte-identical,four-fence-roles-and-narrow-exemption}` | `ken-elaborator/src/literate.rs` and `format.rs` | `.ken.md` producers/readers and catalog validation | **STOP-1/4** — role semantics and private formatter policy are not separable inside the live rows |

Every A1 candidate above is in the stop list; formatter width is the separately
tracked erratum stop. The qualified cleared set is therefore **empty**. This is
not a method failure: the raw consumer sets are non-vacuous because the
probing-policy row is empty while every other family has a non-empty consumer
set or preamble contract. The empty probing result does not become an A1
clearance because C7 and live B2E work own the physical store boundary, and no
five-item relaxation record can honestly claim that axis is closed in
isolation.

No five-item relaxation record is required because no constraint is cleared.
Any future clearance remains provisional on C7 as required by frame §9.

## Controls

### Control B — known positive, run first

```
$ git grep -n '4 MiB' <bound-base> -- conformance
<bound-base>:conformance/runtime/capacity/seed-capacity.md:179:
  - given: intern enough values to overflow one 4 MiB page ...
```

The page-size consumer is non-empty, so the census reaches the known answer.
The scoped companion edit above this row shifts its assembled-candidate line
number without changing the row.

### Control A — known false hit

```
$ rg -n 'probing' conformance
conformance/stdlib/collections/seed-cat4-maps-sets-relations.md:327:
  probe an untouched present key (probing only the deleted key is ...)
```

Reading the enclosing row shows a stdlib `Map` lookup after deletion. It says
nothing about the content store's collision policy, so that policy's consumer
set remains empty.

## Authority-convention sibling sweep

The exact-shape search was:

```
rg -n -i \
  'landed code is normative|conformance follows the landed code|\
impl(ementation)? is normative|normative.*landed (code|implementation)|\
follows the landed|grounded.*landed' spec conformance
```

It found the two ruled sites plus broader evidence-grounding statements.
The broader hits say that a corpus was checked against landed behavior or that
one normative spec chapter delegates a topic to another. They do not say that
an implementation outranks the specification. No third authority-reversal
sibling was found on this reading.

The normative site now identifies the F4/K3 comparison as an expired
draft-reconciliation rule. It retains all three facts:

1. per-`space` bare-hash indexes, not a process-wide `(root, hash)` index;
2. page-buffer drop, not `madvise`; and
3. single-writer resize, not lock-free resize.

The conformance companion at
`conformance/runtime/capacity/seed-capacity.md` remains for the independently
owned conformance slice.

## Guardrail and inertness accounting

This slice changes no `crates/` path and moves no conformance row. The nine
carried guarantees remain untouched:

1. small auditable kernel;
2. kernel rechecking of every claimed certificate;
3. totality and predictability by default;
4. explicit partial and foreign boundaries;
5. exhaustive obligation extraction;
6. honest `proved`/`tested`/`delegated`/`unknown`;
7. explicit effects, capabilities, IFC, provenance, and trust;
8. loud failure rather than silent weakening; and
9. no promotion of Ward, test, or monitor results to `proved`.

No hunk changes the node, campaign, mission, or any normative text defining
those guarantees. The sole normative hunk changes the expired F4/K3 authority
sentence while preserving its three divergence records.
