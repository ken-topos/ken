# MAP-TRANSPORT-CODEC R1 — requirement determination

**Measured at:** `d21d05e08f58875f847309eccfb6ddba43d6149b`
(`origin/main`)

**Frame:** `docs/program/wp/MAP-TRANSPORT-CODEC-R1-requirement.md`
(`b03d66c5`)

**Node:** `docs/program/issues/MAP-TRANSPORT-CODEC.md` (`2c0f1eb2`)

## Determination

**No Map/Set transport codec is required by a current consumer.**

The two corpus-measurable candidates are absent:

1. no cross-space workload requires an extensional-equality dedup hit; and
2. no caller constructs a stable map name for caching, memoization, or a
   durable index.

The third candidate, a wire format for a non-Ken peer, is not answerable from
the repository. It remains **operator input needed**.

This is a requirement result, not a codec design. It changes no code, spec, or
conformance artifact.

## Method

The search covered the current tracked `spec/`, `catalog/`, `examples/`,
`crates/`, `conformance/`, `docs/adr/`, `docs/design/`, active issue files, and
the current strategy, roadmap, and program-of-work documents. Historical diary
entries and closed WP frames were not treated as consumers. The live R1 frame
and node were excluded from the candidate search because they restate the
questions being measured. The exact broad command includes the fixed input's
`SPEC-STORE-SPLIT` frame solely as the known-present positive-control fixture;
its hits are reported separately and are not classified as consumers.

The first pass searched for wants rather than only codec terminology:

```text
git grep -nEI \
  '(cross[- ]space|dedup|deduplic|content[- ]address|memoiz|cach(e|ing)|durable[ -]index|stable[ -](name|identity)|to_?list.{0,80}hash|hash.{0,80}to_?list|wire format|external peer|counterparty|inter[- ]process|network)' \
  -- spec catalog examples crates conformance docs/adr docs/design \
     docs/program/issues docs/program/01-strategy.md \
     docs/program/02-roadmap.md docs/program/03-program-of-work.md \
     docs/program/wp/SPEC-STORE-SPLIT-split-durable-bytes-from-in-process-sharing.md \
     ':!docs/program/issues/MAP-TRANSPORT-CODEC.md'
```

That exact broad pass returned 414 lines: 391 candidate-corpus hits and 23
positive-control-fixture hits. The 391 candidate hits were each classified as
a mechanism, test, design statement, or consumer requirement. Two narrower
passes then tested the candidate-specific claims:

```text
git grep -nEI \
  '(dedup(_|-| )?(hit|rate)|hit[ -]rate|cache[ -]hit)' \
  -- spec catalog examples crates conformance docs/adr docs/design \
     docs/program/issues docs/program/01-strategy.md \
     docs/program/02-roadmap.md docs/program/03-program-of-work.md \
     ':!docs/program/issues/MAP-TRANSPORT-CODEC.md'

git grep -nE '\b(to_list|set_to_list)\b' \
  -- catalog examples crates spec conformance docs \
     ':!docs/program/wp/MAP-TRANSPORT-CODEC-R1-requirement.md' \
     ':!docs/program/issues/MAP-TRANSPORT-CODEC.md' \
     ':!docs/program/MAP-TRANSPORT-CODEC-R1-determination.md'
```

The `to_list` census was reviewed as a producer-to-consumer flow, not as a
word count. In particular, every call outside the Map package was checked for
a downstream hash, digest, cache key, memoization key, or durable-index use.
A separate bidirectional proximity search looked for those wants within 160
characters of `to_list` or `set_to_list`.

### Positive control

The documented exact broad command itself located the known OQ-Space
cross-space value-passing and dedup path. Its actual output included:

```text
docs/program/wp/SPEC-STORE-SPLIT-split-durable-bytes-from-in-process-sharing.md:207:`spec/90-open-decisions.md` `OQ-Space` reads: **`Transport:` content-addressed
docs/program/wp/SPEC-STORE-SPLIT-split-durable-bytes-from-in-process-sharing.md:208:immutable, closure-free value passing (**cross-space dedup by hash**; composes
```

That frame is evidence that the search locates the known path, not a current
consumer or a normative dedup guarantee. The current-corpus output included:

```text
spec/90-open-decisions.md:517:  partitioned per-space with **no cross-space aliasing**, reasoning is **bounded
spec/70-behavioral/73-conformance.md:131:- **Message provenance** — on a cross-space **send**/**receive** event. Spaces
crates/ken-elaborator/src/trace.rs:48:    /// Message content address (`41 §3`) — **only** on cross-space send/receive
```

Reading the matched OQ-Space block confirms shared-nothing message passing and
immutable, closure-free value transport
(`spec/90-open-decisions.md:510-532`). The trace contract independently
confirms that cross-space sends and receives carry message correlation
(`spec/70-behavioral/73-conformance.md:131-140`). The search therefore finds
the known-present transport path before any negative below is credited.

The method also found genuine stable-content naming outside Map/Set. For
example, a behavioral export carries a hash over sorted structured content
(`crates/ken-elaborator/src/export.rs:275-281`,
`crates/ken-elaborator/src/export.rs:745-759`). That is an additional check
that the stable-name vocabulary can locate a real, unnamed-by-"codec"
consumer.

## Candidate 1 — cross-space dedup that hits

### Mechanism found

The current in-process store interns a compound runtime value by producing its
private byte representation, hashing it, and probing an index
(`crates/ken-runtime/src/store.rs:243-260`). A hit is confirmed by comparing
the stored bytes, not by hash alone
(`crates/ken-runtime/src/store.rs:302-314`).

OQ-Space specifies immutable, closure-free value passing, but leaves physical
copying, sharing, and storage private
(`spec/90-open-decisions.md:521-532`). Map/Set preserve extensional equality,
ordered `to_list`, and durable round-trip; their internal representation and
dedup result are not observations
(`spec/40-runtime/41-values.md:174-180`).

### Consumer finding

The hit-rate search returned 32 lines. They were:

- store counters and intern-path tests;
- benchmark generators comparing measured and expected duplicate rates;
- historical/current design and program bookkeeping; and
- one runtime-mechanism projection record.

No hit named a cross-space message workload, Map/Set workload, correctness
condition, capacity calculation, latency target, or throughput target that
depends on an extensional-equality hit. The follow-up query requiring
`cross-space` or `message` to occur with `dedup hit`, `dedup rate`, or
`hit rate` returned no lines.

The behavioral contract points the other way: equal values sent twice receive
distinct per-transfer correlation tokens, and programs cannot inspect those
tokens as value identity
(`spec/70-behavioral/73-conformance.md:131-140`). Thus the live trace consumer
depends on transfer correlation, not value coalescing.

### Verdict

**Not a requirement today.** A missed extensional-equality dedup opportunity
is a private optimization result. The corpus contains no workload that depends
on recovering that hit rate.

## Candidate 2 — a stable name for a map

### Caller-visible boundary

The proved Map carrier is ordinary `data Tree k v`, and `to_list` is a total
in-order traversal to `List (Pair k v)`
(`catalog/packages/Data/Collections/Map.ken.md:79-92`). That ordered list is
caller-visible. The runtime's separate durable round-trip contract produces an
extensionally equal Map/Set with the same observation, but its bytes and hashes
remain private (`spec/40-runtime/41-values.md:50-54`,
`spec/40-runtime/41-values.md:174-180`).

At this SHA, the public surface exposes no generic arbitrary-data encoder. It
explicitly leaves generic `Serialize`/`encode`/`decode` derivation to a
follow-on (`spec/30-surface/38-ffi-io.md:282-288`), and the current catalog
contains no such API. This declaration search returned no lines:

```text
git grep -nE '^(class|record|data) Serialize\b|^(fn|const) (encode|decode)\b' -- catalog/packages
```

A caller with known key and value types could define a concrete package-level
positional encoder over the ordered list. That is a possible caller-owned
function, not an existing generic composition and not access to the runtime's
private canonicalization. The question for R1 remains whether a current caller
asks for a shared operation.

### Consumer finding

The structural census returned 276 `to_list`/`set_to_list` lines:

- 120 are inside the Map package itself;
- 103 are Map specifications and verified-law material;
- 42 are conformance or Rust acceptance tests;
- 5 are generic `Foldable` declarations/instances for List and Option; and
- 6 are current program documents.

The Map-package uses are traversal, lookup agreement, deletion, set
projection, keys/values projection, and relations. The package's own summary
lists those consumers and exposes no hashing, caching, memoization, or
durable-index operation
(`catalog/packages/Data/Collections/Map.ken.md:15102-15120`,
`catalog/packages/Data/Collections/Map.ken.md:15309-15318`).

The bidirectional flow query produced no hand-rolled
`Map.to_list -> encode -> hash` consumer. Its only lexical near-matches were:

- `bytes_to_list` prose about avoiding a cached length;
- the phrase "stable names" introducing Map's public API names; and
- the runtime specification explicitly disclaiming internal Map/Set hashes.

None is a content-addressed map identity. No cache key, memo table, durable
index, artifact identity, or package identity consumes Map `to_list`.

### Verdict

**Not a requirement today.** Stable content names are real elsewhere in the
corpus, but no such consumer is fed by a Map/Set. Ordered `to_list` supplies a
stable element order, not bytes. A future caller may define a concrete
package-level encoder for known key/value types, but the corpus neither exposes
a generic arbitrary-data encoder nor justifies standardizing one now.

## Candidate 3 — a wire format for a non-Ken peer

### Method and finding

The broad pass included `wire format`, `external peer`, `counterparty`,
`inter-process`, and `network`, and it found multiple unrelated wire and
transport mechanisms. Those hits establish only that Ken has transport
surfaces. They cannot establish that a non-Ken Map/Set counterparty is planned.

OQ-Space calls the design distribution-ready while explicitly deferring the
runtime realization (`spec/90-open-decisions.md:521-532`). That statement is
not a roadmap commitment to any particular external peer or interchange
format.

### Verdict

**Operator input needed.** The repository cannot decide whether a non-Ken peer
is on the product roadmap. No roadmap is inferred from the absence or presence
of generic networking and wire-format code.

## Recommendation

Close `MAP-TRANSPORT-CODEC` as **not needed** unless the operator supplies a
current non-Ken peer requirement.

Do not open a codec design/build successor for dedup hit rate or stable map
naming: neither has a consumer today. If the roadmap later names an external
peer, remeasure that peer's interoperability requirement and frame a successor
then.

The boundary remains unchanged: a codec function's result, if one is ever
introduced, would be an ordinary observable result; Map/Set internal
representation, hashes, tree topology, and deduplication remain private
(`spec/40-runtime/41-values.md:174-183`).
