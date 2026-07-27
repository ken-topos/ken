# Capacity, storage, and reclamation

> Status: **Elaborated (X2 contract)**. Normative for observable capacity
> failure, logical `space` isolation, and semantics-invisible reclamation.
> In-process storage organization is private. This chapter does not mandate
> interning, slots, a hash function, probing, a load factor, pages, arenas, or
> identifier retirement.

## 1. Logical values are separate from physical storage

Ken specifies values and their observable behavior, not a unique in-process
store.

- A runtime MAY copy, share, intern, deduplicate, hash-cons, or directly embed
  closure-free values. It MAY choose different policies for different value
  kinds or execution domains.
- Those choices MUST NOT change equality results, evaluation results, authority,
  or failure behavior. For values in the durably canonicalizable domain, they
  also MUST NOT change canonical bytes. For proved `Map`/`Set` package trees,
  only extensional equality, ordered `to_list`, and durable round-trip are
  observed; internal bytes are not. No program can observe a per-value slot,
  allocation address, insertion order, or physical provenance.
- Equal values need not share storage, and shared storage is not evidence that
  two values are equal. An implementation that uses hashing MUST resolve
  collisions without false merges.
- Ordinary closures remain outside canonical storage identity: a closure has
  no equality, canonical bytes, hash, slot identity, or durable publication
  path (`41 §2.1`). Making storage private does not grant closures an identity.

### 1a. The `space` boundary is logical

Each `space` owns encapsulated, non-aliased mutable cells (`../30-surface/36
§4`). Two spaces share no mutable authority. Immutable closure-free values may
cross between them, subject to the durable/publication rules of `41 §3a`; an
ordinary closure or a graph containing one is refused before publication.

Whether a crossing value is copied, shared, interned, indexed, or represented
by some other private mechanism is not observable. A runtime need not map each
logical `space` to a distinct arena, index, process, thread, or collector.

## 2. Capacity and loud failure (`OQ-5` DECIDED)

A runtime or deployment MAY declare finite resource limits. The declaration is
a **resource profile**, not part of value semantics. It MUST identify the
measured resource and the scope to which the limit applies; the specification
does not prescribe that the resource be slots, distinct values, bytes, pages,
or table buckets.

When a declared limit is exhausted, the operation MUST fail loudly with a typed
`CapacityExhausted`-class result before it could drop a value, alias two
unequal values, corrupt live state, or substitute a sentinel. The failure
reports the declared profile and limit sufficiently to diagnose the refusal;
its exact record layout is an implementation/API choice.

The same program may consume different amounts of private storage on conforming
runtimes. In particular, repeated equal values may or may not be physically
deduplicated. A capacity test therefore drives the declared resource to its
limit; it does not infer the limit from an assumed interning policy.

True allocator or address-space exhaustion follows the same rule: either the
runtime reports a typed resource failure, or the host terminates the process
loudly. Silent value loss, false equality, and state corruption are forbidden.

### 2a. Introspection

A runtime MAY expose aggregate resource statistics for diagnostics. The stat
set is profile-specific and MUST NOT expose per-value identity or provenance.
Programs cannot depend on a particular slot count, deduplication rate, arena
size, page count, probing load, or allocation order for semantic behavior
(`41 §7`).

## 3. Reclamation and the memory model

Ken's lower bound remains systems-adjacent
(`../../docs/PRINCIPLES.md` principle 1). The normative runtime substrate is
managed immutable values with optional, semantics-invisible reclamation; no
particular storage or reclamation mechanism is required.

- A runtime MAY use regions, tracing GC, reference counting, manual reset,
  process lifetime allocation, or a combination.
- Reclamation MUST NOT change the observable value, equality result, or
  authority of any live value. It also MUST NOT change canonical bytes for a
  value in the durably canonicalizable domain. For proved `Map`/`Set` package
  trees, extensional equality, ordered `to_list`, and durable round-trip remain
  unchanged; internal bytes are not observable.
- Ending or resetting a logical `space` affects only that space's encapsulated
  mutable cells and private resources. Other spaces remain valid.
- An immutable closure-free value that escaped a space before reclamation
  remains available to its receiver with the same observable value. The
  runtime may have copied it, shared it, or transferred private ownership.
- A non-escaping value may become unreachable and be reclaimed without an
  observable event. No stable allocation identity exists to resurrect.

An ordinary closure or closure-containing graph is not made durable by escape
or reclamation. Live-domain invocation remains governed solely by `41 §2.1`;
publication still rejects transitively before bytes exist.

`OQ-gc` is therefore resolved at the semantic level: automatic collection,
compaction, manual regions, page release, and identifier reuse are private when
they preserve the rules above. Ken is neither required to be a GC language nor
required to forbid GC.

## 4. Lattice machinery is not a semantic dependency (`OQ-6` DECIDED)

Leech/Golay/Co₀ machinery is not part of Ken's value semantics, equality,
capacity model, or durable canonical encoding. No core-language guarantee may
depend on it.

Optional libraries may provide error-correcting codes, fixed-domain sets, or
orbit canonicalization as ordinary verified data and algorithms. A runtime may
also use any private data structure whose behavior satisfies §§1–3; the
specification does not turn a private implementation choice into a language
dependency.

## 5. Scale and limits validation (X4)

X4 validates each advertised resource and performance profile under load. The
validation records the profile, target, measured resource, bound, and loud
failure behavior. It also checks that reclamation and storage-policy changes do
not change observable values.

This chapter does not fix one universal capacity number. A deployment that
advertises no finite limit has no spec-level slot or value ceiling; host
resource exhaustion remains subject to §2.

## 6. What WS-X must deliver here

X2 delivers:

1. mechanism-independent value behavior and durable publication (`41`);
2. loud typed refusal for every declared finite resource profile (§2);
3. logical `space` isolation with no shared mutable authority (§1a);
4. semantics-invisible reclamation and escape survival (§3).

X4 supplies load validation for the profiles a backend actually advertises.
Foundation and Runtime remain free to choose storage mechanisms privately.

**Acceptance criteria:**

- **AC1 — loud capacity refusal.** Exhaust a declared resource with real values
  and observe a typed failure. The negative control is silent drop, sentinel
  substitution, false merge, or corruption.
- **AC2 — storage-policy independence.** Run the same value/equality corpus
  with sharing disabled and enabled; observable values and equality results
  are identical.
- **AC3 — reclamation invisibility.** Reclaim a region or collection domain;
  surviving values retain the same observable value, while unreachable private
  storage may be released.
- **AC4 — space isolation.** Reclaim/reset space A while space B remains live;
  B's cells and previously received immutable values are unchanged.
- **AC5 — large-value safety.** Values larger than an implementation's normal
  allocation unit remain reachable and correct or fail loudly under a declared
  limit; no page size or oversized-allocation recipe is prescribed.

**Conformance:** `../../conformance/runtime/capacity/` retargets the former
slot/page/interner cases to these observable properties. A case must state the
resource profile it exercises and may not assume physical deduplication,
probing, pages, arenas, stable slot ids, or a particular collector.
