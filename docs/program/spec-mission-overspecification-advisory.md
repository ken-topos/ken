# Mission/spec overspecification advisory — tracked capture

> ## ⛔ PROVENANCE — this is a CAPTURED ADVISORY, not a Steward document
>
> **Authors:** the **conformance-validator** (the review) and **research** (the
> prior-art addendum). Dispatched directly by the operator on **2026-07-26**.
> The Steward captured it here **verbatim and unmodified** — everything below
> the horizontal rule is the authors' text, byte-for-byte as delivered.
>
> **Why it is in the repo at all.** It was delivered to
> `local/spec-mission-overspecification-review.md`, and **`local/` is
> gitignored** — so the entire analysis existed as one untracked file on one
> box, with three tracks of program work derived from it. A campaign whose
> only source can be lost by a `git clean` is not captured.
>
> **Anchors, as stated by the authors:** reviewed at
> `origin/main=78bef8503eacfedf4c2dc8b161eca67c150280eb`, revalidated by
> research at `origin/main=bb3e58ea5b61ab16da60be7111a374432338934f`, with the
> `spec/` tree byte-identical at both (`7fce43734e29c440c892b6c893cc630d3058b57d`).
> ⭐ The Steward independently confirmed the `spec/` tree is **still**
> `7fce4373` at `35a0c61a` and at `origin/main=9410d7b8` — so **none of the
> findings below is stale**, and any future reader should re-run that same
> check rather than assume it.
>
> **It is advisory.** ⛔ It is **not** a soundness judgment, **not** a design
> ruling, and **not** a licence to edit `spec/`. Its dispositions — what is
> relaxed, what is deferred, what needs a ruling — live in
> **`14-spec-mission-alignment-campaign.md`**, which is the Steward's document
> and the one that binds. ⚠ Where the campaign doc and this advisory disagree
> about *what to do*, the campaign doc governs; where they disagree about *what
> the spec says*, re-read the spec.
>
> **No reference implementations under `local/refs/` were consulted** — stated
> by both authors. The prior-art addendum cites public primary sources only.

---

# Mission/spec overspecification review

Date: 2026-07-26

This is an advisory review, not a soundness judgment. It compares
`docs/MISSION.md` with the normative specification as inspected at
`origin/main=78bef8503eacfedf4c2dc8b161eca67c150280eb`.

No reference implementations under `local/refs/` were consulted. In
particular, this review does not independently establish the prior-art premise.

Research revalidated the review on 2026-07-26 at
`origin/main=bb3e58ea5b61ab16da60be7111a374432338934f`. The `spec/` tree is
byte-identical at the two revisions
(`7fce43734e29c440c892b6c893cc630d3058b57d`), so none of the findings below
went stale between the initial review and the prior-art addendum.

## Summary

The strongest recurring risk is the same pattern found with closures: an
implementation mechanism is elevated into observable language semantics even
though the mission only requires an assurance property.

The mission is primarily outcome-oriented:

- human-readable contracts, authority, proofs, and assumptions;
- independently rechecked proof certificates through a small kernel;
- explicit tested, delegated, and unknown boundaries;
- totality and predictability by default;
- explicit effects, capabilities, information flow, provenance, and trust; and
- honest separation of proof, testing, behavioral assurance, and monitoring.

It does not by itself require particular hashes, heap layouts, runtime slot
identities, proof-search backends, wire encodings, package envelope layouts, or
concurrency mechanisms.

Absence from the mission is not by itself proof of overspecification. A language
still needs concrete semantics, interoperability contracts, security
properties, and performance commitments. The useful discriminator is whether
the spec fixes a property visible to users and reviewers, or fixes one
particular mechanism for realizing that property.

## Highest-priority scrutiny

### 1. Content-addressed runtime store

Anchors:

- `spec/40-runtime/41-values.md` sections 2 through 5
- `spec/40-runtime/42-evaluation.md` section 3.4
- `spec/40-runtime/44-capacity.md`

Conformance requires equal canonical values to share the same slot. The spec
also fixes FNV-1a, `memcmp`, monotonic 64-bit slots, open addressing, a `0.70`
load factor, an initial `2^16` buckets, 4 MiB arena pages, per-`space` stores,
reset behavior, and O(1) equality. `44-capacity.md` even states that landed code
is normative where it differs from the earlier design prose.

This is the clearest analogue of the closure problem. Two implementations can
calculate the same values with the same externally visible semantics, yet one
fails conformance because it does not allocate the same representation or
deduplicate through the same mechanism.

Likely mission-minimum contract:

- extensional equality;
- deterministic durable encoding where publication requires it;
- reproducible artifact identities;
- loud resource-limit failure with no aliasing or corruption; and
- any deliberately promised performance bound, stated as a bound rather than
  as a required data structure.

The in-process hash, index, slot lifecycle, page size, arena organization,
garbage-collection strategy, and equality implementation should receive
separate justification before remaining normative.

### 2. Runtime `unknown`

Anchors:

- `spec/40-runtime/41-values.md` section 6
- `spec/40-runtime/42-evaluation.md` section 4

The mission requires unknown as an epistemic classification. It does not
require every open verification hole to become a universal runtime third value
with Kleene propagation through application, primitives, casts, equality,
eliminators, and effects.

That choice crosses every evaluator, backend, FFI, and effect path. It also
commits Ken to allowing hole-bearing programs to run, which is a development
and deployment policy rather than an obvious consequence of honest assurance
labelling.

Review question: could `unknown` remain an artifact/verifier status, with
execution controlled by an explicit policy such as refusal, an opt-in dynamic
stub, or a development-only runtime value?

### 3. Automated-prover architecture

Anchor: `spec/20-verification/23-prover.md`, especially sections 2 through 4.

Kernel rechecking makes proof search untrusted and replaceable. Nevertheless,
the spec normatively fixes the D/FO/HO classifier, Kripke translation, Z3
route, and reflective-checker architecture.

The mission requires independently checkable proof certificates. It does not
require one permanent proof-search portfolio. Alternative solvers, native
tactics, proof reconstruction, specialized decision procedures, or future
search strategies could satisfy the same assurance boundary.

Likely mission-minimum contract:

- every obligation is accounted for;
- `proved` is possible only after kernel certificate acceptance;
- `disproved` and `unknown` retain their honest evidence/status distinctions;
  and
- a search failure or unsupported fragment cannot silently become `proved`.

The route used to find a certificate can then be an implementation or toolchain
profile.

### 4. Exact termination mechanism

Anchor: `spec/10-kernel/17-conversion.md` section 4.

Decidable checking and termination of transparent definitions are
mission-aligned. Requiring one exact size-change-termination graph and matrix
algorithm is a narrower decision. It can reject valid total programs and may
prevent alternative termination certificates, structural-recursion checkers,
well-founded recursion proofs, or future verified termination procedures.

Review question: should the kernel require the property "transparent unfolding
is certified terminating," while SCT is one accepted certificate producer, or
is exact SCT acceptance deliberately part of Ken source compatibility?

### 5. Logical `space` mapped to physical runtime structure

Anchors:

- `spec/30-surface/36-effects.md` section 4
- `spec/40-runtime/44-capacity.md` sections 1 through 3
- `spec/90-open-decisions.md`, `OQ-Space`

The logical isolation guarantee is coupled to shared-nothing actors,
closure-free content-addressed messages, per-space arenas, re-interning, and
reclamation boundaries. This may foreclose ownership-based shared memory,
regions, safe internal sharing, or alternative actor implementations that
provide the same observable guarantees.

The mission supports explicit authority, race freedom, bounded reasoning, and
auditable communication. It does not select the physical realization.

Review question: can the spec state the isolation, authority, communication,
and IFC invariants while making process/thread/actor/region/store realization
non-normative?

## Interface constraints needing consumer evidence

### 6. Ward export and trace schemas

Anchors:

- `spec/70-behavioral/71-assumption-boundary.md` sections 2 and 3
- `spec/70-behavioral/73-conformance.md` section 2

The mission requires a clean separation between proof, testing, delegation,
and monitoring. It does not obviously require:

- exactly five `Q/P/Sigma/T/G` fields;
- ITF as the permanent trace format;
- one direct `ResourceLifetimeObligation` representation;
- frozen example hashes; or
- prohibition of a schema-version field, wrapper, alias, or conversion view.

The one-way no-promotion invariant, accurate status projection, trace fidelity,
and reproducible contract identity are load-bearing. Every additional frozen
schema detail should be tied to a demonstrated consumer or compatibility
requirement.

### 7. Checked package and executable envelopes

Anchors:

- `spec/40-runtime/46-checked-core-package.md`
- `spec/40-runtime/48-executable-artifact-contract.md`

Recheck-on-consume, stable semantic identity, provenance, and explicit
unavailable evidence are strongly mission-aligned. The possible
overspecification is in the number of closed schemas, repeated dependency
hashes, mandatory empty sections, and distinct
semantic/artifact/runtime/report/native/contract hashes.

Review question: for each field and duplicated binding, which concrete
substitution, ambiguity, or trust-confusion attack does it prevent? If several
fields protect the same edge, a smaller compositional evidence graph or
manifest may preserve the assurance story.

### 8. `Ord` and `Map` restricted to canonical carriers

Anchor: `spec/50-stdlib/52-map.md` section 2.1.

Requiring order equivalence to yield kernel `Equal` excludes lawful key types
with multiple representations, including the spec's own Decimal example. That
is a substantial software-engineering restriction not evident in the mission.

Possible alternatives include:

- separating ordering equivalence from kernel representation equality;
- a stronger `CanonicalOrd` class only where representation equality is needed;
- normalization before keying; or
- quotient/key-equivalence-based maps.

This deserves scrutiny because it restricts ordinary commercial data models,
not merely an internal implementation.

### 9. Universal transitive capability revocation

Anchor: `spec/60-security/62-authority.md` section 4.

Explicit capabilities, attenuation, and absence of ambient authority are
central to the mission. It is less clear that every capability must carry
transitive revocation lineage with exact admission linearization and exact
public error projections.

Review question: would an explicitly revocable capability class plus a simpler
non-revocable capability class meet the mission with less runtime machinery?
When revocation exists, its fail-closed and race semantics should remain exact.

### 10. Specific supply-chain products

Anchor: `spec/60-security/63-supply-chain.md` section 5.

Provenance and recheck-on-consume are direct mission commitments. Permanently
fixing keyless Sigstore/Cosign and in-toto/SLSA inside the language spec is a
different matter. These may fit better as supported deployment profiles under
an abstract provenance-verification contract.

## Lower-priority scrutiny

### Purity keywords as bidirectional hard errors

`spec/30-surface/36-effects.md` section 1.6 makes every
`const`/`fn`/`proc` mismatch a hard error, including a `proc` whose
implementation becomes pure. Reliable purity signals help human review, but the
reverse-direction error can create refactoring churn without improving
soundness.

### Instance coherence and package admission

`spec/30-surface/33-declarations.md` section 5.5 fixes one canonical structure
instance, forbids orphans, and defines a detailed package-admission graph.
Coherence supports predictability, but the accumulated rules should be tested
against real multi-package use cases and compared with the smaller invariant
actually needed for legible, deterministic resolution.

### Formatter details

`spec/30-surface/31-lexical.md` makes choices such as exact width, indentation,
and literate-fence handling normative. A single readable canonical form supports
the mission. Individual layout constants may be tooling policy rather than
language conformance.

### Numeric inventory and representation

`spec/30-surface/35-numbers.md` fixes an exact suite of numeric types and several
representation choices. Arbitrary precision and no silent overflow are
defensible assurance properties; internal fast paths, tags, and coefficient
representations normally are not.

### FFI and buffer protocol

`spec/30-surface/38-ffi-io.md` correctly requires explicit foreign trust,
capability gates, checked spans, and visible resource lifetimes. Its exact
positioned-buffer, progress, error, and settlement protocol may be too narrow as
the sole I/O mechanism and could belong in a standard runtime profile.

## Constraints not to relax merely because the mission is abstract

The following guarantees directly implement the mission and should not be
discarded during a simplification pass:

- the small auditable kernel;
- kernel rechecking of every claimed proof certificate;
- totality and predictability by default;
- explicit partial and foreign boundaries;
- exhaustive obligation extraction;
- honest proof/test/delegated/unknown status;
- explicit effects, capabilities, IFC, provenance, and trust;
- loud failure rather than silent weakening or corruption; and
- the prohibition on promoting Ward, test, or monitor results to `proved`.

Their mechanisms may still be simplified, but the guarantees themselves are
load-bearing.

## Suggested review test

For each normative constraint, ask:

> Could two implementations provide identical source meaning, proof results,
> trust boundaries, security guarantees, durable artifacts, and observable
> behavior, yet one fail conformance solely because it uses different internal
> machinery?

A "yes" is a strong overspecification signal.

Apply that test first to:

1. `41-values.md`, `42-evaluation.md`, and `44-capacity.md`;
2. runtime `unknown`;
3. `23-prover.md` routing and backend commitments;
4. the logical-to-physical `space` mapping; and
5. frozen Ward and executable-artifact schemas.

For every proposed relaxation, explicitly record:

- the mission outcome that remains protected;
- the observable or security invariant retained;
- the implementation choices newly permitted;
- any external consumer requiring exact compatibility; and
- a conformance pair showing the relaxed contract still rejects an actual
  mission-breaking implementation.

## Prior-art addendum

This addendum is Research's advisory, not a design ruling. It uses public
primary sources and official project documentation. It does not consult
`local/refs/`.

### A necessary classification before simplifying

The report's mechanism-versus-property test is sound, but it needs one further
distinction. A constraint can be absent from the mission and still properly be
normative for one of three reasons:

1. **Language semantics.** Evaluation order, equality, accepted recursion, and
   effect behavior can distinguish programs. Ken must choose these even though
   the mission does not.
2. **Interoperability protocol.** A package or trace schema may need exact
   bytes, versions, and identifiers so independent producers and consumers
   agree. That contract should normally be versioned and separated from source
   meaning, but it is not merely an implementation detail.
3. **Security binding.** A repeated identity field may prevent substitution,
   rollback, type confusion, or mix-and-match attacks. Apparent duplication is
   not overreach until its threat edge is shown redundant.
4. **Private mechanism.** Hash-table policy, arena page size, solver routing,
   copying versus sharing, and similar choices are overreach when no observable
   bound, protocol consumer, or threat edge depends on them.

The strongest simplification candidates are in class 4. Classes 1 through 3
need a semantic, compatibility, or threat-model argument rather than a direct
appeal to mission text.

### Prior-art verdict on the highest-priority items

#### 1. Content-addressed runtime store — strong support, with one split

The report is strongly supported for the **in-process representation**.
Hash-consing literature describes maximal sharing as a technique that can make
equality constant-time, not as the only semantics for immutable values.
Filliâtre and Conchon encapsulate it behind an abstract type and parameterize
it by an arbitrary equivalence relation:
[Type-Safe Modular Hash-Consing](https://gallium.inria.fr/ml2006/accepted/5.html).
Empirical work also finds maximal sharing can help or hurt depending on
redundancy and equality traffic:
[Performance Modeling of Maximal Sharing](https://ir.cwi.nl/pub/25650/).

Erlang supplies a useful implementation-independence control. Message data is
normally copied, but reference-counted binaries and literals can be shared on
the same node. The logical process/message model does not expose that physical
choice:
Erlang process efficiency guide:
<https://www.erlang.org/doc/system/eff_guide_processes.html>.

Recommended split:

- retain extensional equality and deterministic canonical encoding for values
  that actually cross a durable boundary;
- make global interning, same-slot conformance, FNV-1a, probing policy, load
  factor, page size, and slot retirement private runtime choices;
- express O(1) equality, if Ken deliberately promises it, as a performance
  profile or complexity contract rather than as a mandated hash table; and
- never make "landed code is normative" the authority rule. That reverses the
  spec/implementation relationship and excludes an independent conforming
  implementation by construction.

Canonical **durable bytes** and maximal **in-process sharing** are separate
contracts. The former can be required without the latter.

#### 2. Runtime `unknown` — strong support

GHC's typed-hole policy demonstrates the missing degree of freedom. Typed holes
reject compilation by default; an explicit flag may instead defer them, and a
forced hole then fails like `undefined` at runtime:
GHC typed holes:
<https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/typed_holes.html>.
The epistemic fact and the execution policy are distinct.

Ken's universal Kleene-style value is therefore not forced by honest
`unknown` reporting. It is a substantive language and deployment choice with a
large semantic radius. The report should rank this beside closure identity,
not below it.

A mission-minimum design would:

- preserve `unknown` in the verification artifact;
- require deployment policy to state whether unknown-bearing code is rejected,
  quarantined, or allowed in a development mode;
- if execution is allowed, use an explicit typed failure/stub boundary rather
  than silently inserting a value into every ordinary type; and
- reserve three-valued propagation for domains that intentionally specify it.

This does not rule out runtime `unknown`; it says the feature needs an
independent product and semantic justification.

#### 3. Automated-prover architecture — strong support

This is the clearest prior-art-backed relaxation. Lean tactics construct proof
terms that the small kernel independently checks, so tactic bugs cannot create
accepted false proofs:
[Lean tactic proofs](https://lean-lang.org/doc/reference/latest/Tactic-Proofs/).
SMTCoq checks witnesses from several external SAT/SMT solvers through a
certified checker:
[SMTCoq](https://smtcoq.github.io/).

Those systems make the trust boundary stable while proof search evolves.
Accordingly:

- exhaustive obligation accounting and kernel acceptance remain normative;
- certificate languages and verified checkers may be normative, versioned
  interfaces;
- D/FO/HO routing, Z3 versus cvc5 selection, tactic portfolios, timeouts, and
  search heuristics should be replaceable profiles; and
- a route may not disappear merely because the portfolio changes. The total
  accounting invariant is stronger than any particular classifier.

The Kripke adequacy theorem may remain permanent semantic infrastructure if it
is how classical certificates are translated into Ken's intuitionistic logic.
The claim to relax is the **exclusive search route**, not the theorem needed to
check certificates produced through that route.

#### 4. Exact termination mechanism — downgrade to a design fork

Prior art supports multiple termination routes, but this item is not a simple
private-mechanism case. Lean accepts structural recursion and well-founded
recursion, and explains that they have different definitional-equality and
kernel-computation behavior:
Lean recursive definitions:
<https://lean-lang.org/doc/reference/latest/Definitions/Recursive-Definitions/>.

The accepted set of transparent definitions and their reduction behavior are
observable source semantics. Replacing SCT with another incomplete checker can
make previously valid programs fail or previously invalid programs pass.

The report should therefore amend the proposed relaxation:

- do not say merely "transparent unfolding is certified terminating";
- define a stable, kernel-checkable termination evidence interface first;
- allow structural, SCT, well-founded, or future producers only when each emits
  evidence accepted by that interface; and
- specify whether two routes preserve the same definitional equations.

Without such an evidence interface, exact SCT is a restrictive but coherent
language-compatibility decision, not internal overspecification.

#### 5. Logical `space` versus physical structure — strong support

The Erlang copying/sharing split above is direct precedent: isolation and
message semantics can remain stable while the runtime shares selected immutable
payloads. Ken should specify authority, race freedom, state visibility,
failure, ordering, and escape behavior. Per-space arenas, re-interning, page
ownership, actor scheduling, and physical copying should remain replaceable
unless a resource bound explicitly requires them.

The important caveat is that failure isolation and message order are
observable. Relaxing "shared-nothing storage" must not accidentally relax the
logical no-shared-mutable-authority guarantee.

### Prior-art verdict on the interface and security items

#### 6. Ward and trace schemas — support a versioned profile

ITF itself has changed: its ADR records a 2023 integer-encoding revision and a
2025 naming revision, while leaving `#meta` deliberately open:
[Apalache ITF ADR](https://apalache-mc.org/docs/adr/015adr-trace.html).
That is evidence against treating one current ITF byte shape as permanent Ken
source semantics.

Keep the no-promotion lattice and exact status meanings normative. Put field
spellings, ITF encoding, and consumer-specific envelopes in a versioned
protocol profile with:

- an explicit major version or type URI;
- canonical bytes within a version;
- monotone rules for ignorable extensions; and
- fail-closed handling when a consumer does not understand a semantic field.

The five `Q/P/Sigma/T/G` concepts may still be a useful semantic decomposition.
The report should distinguish those concepts from their one current wire
layout.

#### 7. Checked-package and executable envelopes — caution, require threats

The initial report is right to demand consumer evidence, but field count alone
is weak evidence of overreach. Mature artifact formats deliberately repeat
typed digests at layer boundaries:

- an in-toto statement binds a digest-identified subject to a separately typed
  predicate:
  in-toto Statement specification:
  <https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md>;
- OCI manifests bind typed configuration and layer descriptors by digest:
  OCI image manifest:
  <https://github.com/opencontainers/image-spec/blob/main/manifest.md>;
- TUF binds versions, lengths, and hashes to prevent rollback and mix-and-match
  attacks:
  [TUF specification](https://theupdateframework.github.io/specification/v1.0.26/).

The right audit is therefore per edge:

> Which producer signs this field, which consumer checks it, and which concrete
> substitution, rollback, type-confusion, or stale-evidence attack succeeds if
> it is removed?

Merge fields only when two bindings have the same authority, signed scope,
consumer, lifetime, and attack set. A smaller compositional manifest is a good
goal, but "duplicate hash" is not yet a finding.

One extension is strongly supported: closed schemas that reject every unknown
field impede safe evolution. in-toto and SLSA use explicit type/version
identifiers and monotone extension rules. Ken should distinguish an unknown
semantic obligation, which must fail closed, from an unknown optional metadata
field, which a versioned protocol may safely preserve or ignore.

#### 8. `Ord` and `Map` canonical carriers — very strong support

Rocq's ordered-map interface is a direct constructive counterexample. Its
`OrderedType` takes an explicit `eq : t -> t -> Prop`, proves that relation an
equivalence, and makes comparison return equality in that relation; it does not
require equality to be Leibniz/representation identity:
Rocq OrderedType:
<https://rocq-prover.org/doc/master/stdlib/Stdlib.Structures.OrderedType.html>.

Ken should seriously consider:

- `KeyEq` or an ordered-key equivalence independent of kernel `Equal`;
- a compatibility proof that ordering respects that equivalence;
- a canonicalization function when a unique stored representative is desired;
  and
- a stronger `CanonicalOrd` only for APIs that truly require kernel equality.

This is not merely ergonomic. It determines whether ordinary normalized
commercial values such as Decimal can be lawful keys without lying about
representation equality.

#### 9. Universal transitive revocation — strong support for two classes

CHERI provides ordinary fine-grained spatial capabilities while treating
temporal revocation as a separate, difficult mechanism. CHERIvoke adds
quarantine and sweeping with an explicit time/space tradeoff:
[CHERIvoke](https://doi.org/10.1145/3352460.3358288).

That supports a split between:

- ordinary non-revocable attenuable capabilities; and
- explicitly revocable capabilities carrying the additional lineage,
  synchronization, and failure contract.

Ken should retain absence of ambient authority and fail-closed use after an
actual revocation. It need not impose revocation ancestry and its cost on every
capability merely because some resources need revocation.

#### 10. Sigstore, Cosign, in-toto, and SLSA — very strong support

SLSA 1.2 calls its concrete provenance and verification-summary formats
recommended rather than required:
[SLSA 1.2 specification](https://slsa.dev/spec/v1.2/).
The mission-aligned layer is authenticated provenance, subject binding,
builder/source identity, policy evaluation, and recheck-on-consume. Sigstore,
Cosign, in-toto, SLSA, TUF, or a commercial attestation service can be versioned
deployment profiles beneath that contract.

Ken may ship a preferred profile without making the named products permanent
language semantics. Algorithm and provider agility are especially important
for a system intended to remain auditable and deployable over a long lifetime.

### Prior-art verdict on the lower-priority items

#### Purity keywords — support one-way checking

Standard effect systems use subeffecting: a pure expression may be assigned a
broader impure effect because the annotation is an upper bound on possible
effects:
Type and effect systems:
<https://xavierleroy.org/control-structures/book/main016.html>.
Rejecting an implementation that becomes purer is therefore an unusual
human-review policy, not a soundness requirement.

Keep `const`/`fn` from performing undeclared effects. Permit a `proc` body to
become pure unless Ken deliberately treats the keyword as a reviewed promise
that must be exact. If exactness is retained, document the review benefit that
outweighs refactoring churn.

#### Instance coherence — do not relax from mission text alone

Rust's orphan and overlap rules show that coherence is commonly a language
compatibility property: they prevent two downstream packages from creating
conflicting implementations and preserve future library evolution:
Rust implementation coherence:
<https://doc.rust-lang.org/reference/items/implementations.html>.

Ken's exact package-admission graph may still be over-detailed, but "one
deterministic instance" and the open-world ownership rule need an alternative
coherence proof before relaxation. This item belongs beside termination as a
design fork, not beside arena page size as private machinery.

#### Formatter details — strong support for tool-policy status

Go demonstrates the mission benefit of one canonical format while keeping
non-`gofmt` source valid: the formatter is a tool and ecosystem convention, not
a language-acceptance rule:
[gofmt](https://go.dev/blog/gofmt) and
[Go formatting compatibility](https://go.dev/blog/experiment).

Ken can require its permanent/canonical source artifact to be formatter-normal
without making width, indentation, and fence policy part of program meaning.
Version or edition the formatter so old source can be re-rendered
deterministically.

#### Numeric and FFI details — preserve semantics, profile mechanisms

The report's distinction is right:

- exact numeric ranges, rounding, overflow, normalization, and equality are
  source semantics;
- tags, limb widths, coefficient layouts, and fast paths are implementation;
- exact FFI buffer and settlement layouts are legitimate ABI profiles; and
- the language-level foreign boundary should permit additional versioned
  profiles that preserve capability, lifetime, progress, and failure
  invariants.

### Additional scrutiny suggested by prior art

#### Version and algorithm agility

Several current chapters mix a permanent semantic identity with one hash,
signature product, or schema generation. OCI, in-toto, SLSA, TUF, and Sigstore
all carry explicit versions or typed envelopes. Ken should audit every durable
hash or signature for:

- algorithm identifier and domain separation;
- migration without identity ambiguity;
- downgrade prevention;
- canonical bytes within a version; and
- whether old artifacts remain independently checkable.

In-process FNV-1a needs no durable agility if it becomes private. Any hash that
crosses a process, package, provenance, or archival boundary does.

#### Protocol evolution policy

The spec frequently chooses closed schemas and loud rejection of unknown
fields. Prior art distinguishes:

- unknown **semantic** fields that affect meaning or authority — reject;
- unknown **optional metadata** under a known major version — preserve or
  ignore according to the profile; and
- unknown major versions or type URIs — reject.

This three-way rule would retain honesty while avoiding a new major-version
fork for every additive diagnostic field.

### Revised priority

The prior-art evidence supports this order:

1. runtime `unknown` policy;
2. in-process content-store representation and same-slot conformance;
3. prover search portfolio and hard-coded solver routing;
4. logical `space` versus physical storage/process realization;
5. `Ord`/`Map` equality over non-canonical carriers;
6. universal capability revocation;
7. named supply-chain products and algorithm agility;
8. Ward/ITF wire profile and protocol evolution;
9. purity reverse errors and formatter constants;
10. termination, package envelopes, and instance coherence as **design/threat
    audits**, not presumed relaxations.

The closure review supplies the governing pattern: remove intensional
representation from observable semantics while retaining every validity,
authority, lifetime, and fail-closed boundary. Prior art strongly supports
applying that pattern beyond closures, but not indiscriminately to choices that
define accepted programs or bind adversarial artifacts.
