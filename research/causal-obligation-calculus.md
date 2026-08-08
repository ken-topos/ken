# Affine and observational type theory in Ken's compiler

**Research status:** advisory, not an architecture ruling

**Grounding:** `origin/main` at
`3dab6d7a5e04e5e23509b3deb087bd11d9ea2a22`

**Date:** 2026-08-08

## Executive assessment

Ken's recent compiler work has discovered something worth lifting, but it has
not yet discovered a unification of affine type theory (ATT) and observational
type theory (OTT).

The liftable result is a small, compiler-local **linear causal-obligation
calculus**:

- the planner mints opaque causal identities;
- an exact owner may consume each obligation once;
- direct emission and verified composition are alternative, disjoint discharge
  forms;
- the two forms must exhaust the planned population; and
- compilation cannot close while any obligation remains.

This is more coherent than a collection of Rust move idioms. It recurs across
continuation-call claims, dynamic splice edges, predecessor edges, terminal
authority, and several domain-specific ledgers. It is also selective: the
compiler explicitly records that ordinary static-worker bindings are reusable
and must **not** acquire an affine discipline.

The result is not ATT × OTT for four reasons:

1. It lives in Rust data structures and validation passes, not in Ken syntax or
   kernel judgments.
2. Most causal identities are copyable labels. Mutable ledgers, rather than
   Rust's static move checker, enforce their single discharge.
3. The mechanism has no interaction with Ken's observational `Eq`, `cast`,
   dependent functions or pairs, substitution, conversion, or proof
   irrelevance.
4. The strongest compiler obligations are **linear**, not merely affine:
   unused obligations are rejected, whereas affinity permits discard.

The best-supported research direction is consequently stratified:

> Keep OTT unchanged as an unrestricted logical layer, and place a
> quantitative or two-context program judgment beside it. Let ordinary OTT
> terms index resource obligations, but initially require type formation,
> equality, and conversion to use zero linear resources.

That design has close precedents in quantitative and linear dependent type
theory. The unsolved boundary is observational equality for resource-sensitive
values, especially linear functions and `cast`. The compiler has supplied an
excellent Ken-specific model problem and an executable checker. It has not
supplied the missing equality theory.

This finding does **not** reopen product work on Ken-level affine types. The
accepted resource design remains valid: enforce ownership in Rust, expose
copyable generation-checked handles to Ken, and delegate the correlated
lifetime obligation to Ward
([ADR 0021](../docs/adr/0021-resource-lifetime-and-ward-delegation.md)). A formal
ATT–OTT investigation should remain a bounded research track until its
metatheory is established.

## 1. The hypothesis needs four layers, not one

The phrase "Rust's affine type theory underneath OTT" compresses four distinct
systems. Separating them makes the compiler evidence legible.

### 1.1 Ken's kernel is structural OTT

The kernel has one ordinary telescope of term variables. It requires
capture-avoiding substitution and weakening, and later entries may depend on
earlier terms
([kernel syntax §§2–5](../spec/10-kernel/11-syntax.md)). Its central judgments
have the ordinary structural form `Γ ⊢ t : A` and `Γ ⊢ a ≡ b : A`; conversion
changes a term's type along definitional equality, not subtyping
([kernel judgments §§1–3](../spec/10-kernel/18-judgments.md)).

There is no usage annotation, linear zone, loan set, or ownership judgment in
that kernel. Linear and affine types are explicitly classified as research,
outside the language track
([overview §8](../spec/00-overview.md)). Adding them would therefore change the
context discipline and its metatheory, not merely add one new type former.

### 1.2 Ken-visible resources are dynamically checked

Ken's public `Resource k` is an ordinary copyable value. A handle may escape
its bracket legally at the source level; later operations on it return
`Closed`. Release invalidates every copy, and bracket settlement handles normal
return, returned error, and controlled trap paths
([ADR 0021 §§Lifetime–Rust enforces](../docs/adr/0021-resource-lifetime-and-ward-delegation.md)).

The code implements the same split:

- `ResourceTokenV1` derives `Clone` and `Copy` and contains a slot and
  generation
  ([`effect_v1.rs`](../crates/ken-host/src/effect_v1.rs));
- a live `ResourceSlotStateV1` holds one `ResourceOwnerV1`;
- `ResourceHandleV1` is explicitly unique and non-cloneable
  ([`lib.rs`](../crates/ken-host/src/lib.rs)); and
- `begin_release` moves the owner out, invalidates or advances the generation,
  and only then returns a pending close operation.

This is **dynamic generational typestate behind an unrestricted name**. It is a
sound and honest way to lift a Rust-owned resource into Ken, but it is not a Ken
ATT judgment.

### 1.3 Rust ownership is richer than bare affinity

Rust move semantics are affine in the ordinary sense: a value cannot be used
after it is moved, but it may be dropped without explicit use. Rust's safety
story also includes shared and exclusive borrowing, non-lexical lifetimes,
provenance, destructor behavior, interior mutability, and controlled `unsafe`
boundaries.

Oxide models the borrow checker with a **substructural typing judgment** and
interprets lifetimes as approximations of reference provenances
([Weiss et al. 2021](https://arxiv.org/abs/1903.00982)). RustBelt's semantic
account requires a lifetime logic in higher-order concurrent separation logic
to reason about borrowing and unsafe library abstractions
([Jung 2020](https://research.ralfj.de/thesis.html)).

Consequently, even a successful ATT–OTT core would not automatically amount to
a formal account of Rust. Loans, aliasing, lifetime inclusion, unwinding,
concurrency, and unsafe abstraction remain additional obligations.

### 1.4 The compiler has a separate resource discipline

The native compiler's relevant invariants govern control-flow and continuation
authority. They do not govern file descriptors or buffers. The difficult work
has been translating checked recursor and continuation semantics into
functionized Cranelift units while retaining the exact causal owner, source,
target, and boundary representation.

The current recursor frame says this directly: invocation-local activation,
resume, and return-hole state may not enter ABI data; only ordinary typed
values cross a unit boundary; open or ambiguous cases must refuse before
allocation or emission
([RT-RECURSOR-TRANSPORT §§3–4](../docs/program/wp/RT-RECURSOR-TRANSPORT.md)).

That compiler problem resembles resource management because continuation
edges and emission rights are single-spend authorities. It is not evidence
that OTT itself caused the implementation difficulty, and it does not connect
the OS resource table to kernel equality. The compiler work establishes a
useful analogy and a formalization opportunity, not that causal explanation.

## 2. The compiler evidence

### 2.1 Planner-minted opaque identities

`ContinuationCallIdentity` records a four-part causal identity: producer
construct, alternative, call-site sequence, and recursive position. Its fields
are private and lowering has no constructor. The validated static-transition
planner is the only source of the identity
([`static_transition.rs`](../crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs)).

This gives the first two ingredients of a resource calculus:

- **no forgery:** the consumer cannot invent an authority; and
- **provenance:** the authority retains the exact event and owner that created
  the obligation.

The identity itself derives `Clone`. It is therefore an opaque, copyable name,
not the consumable capability. The consumable authority resides in the ledger
that interprets the name.

### 2.2 Exact claim and owner agreement

`ContinuationClaimLedger` is held across the entire unit-definition pass so a
token claimed at one producer cannot be claimed again at another
([`lowering/mod.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).
Its `claim_exact` operation rejects:

- a token the planner never created;
- a second claim for the same token; and
- a claim by a context other than the token's emission owner.

The operation records the actual claimant and returns the planner-resolved
target
([`lowering/units.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).
This is a checked elimination rule for an owner-indexed obligation.

### 2.3 Disjoint and exhaustive discharge

Ledger closeout compares **sets**, not counts. It requires the resolved and
declared populations to equal the planned population, then asserts

```text
planned = direct-emitted ⊎ composed-consumed
```

where `⊎` is disjoint union. An overlap means one obligation was answered in
two forms. A shortfall means one was never answered. Closeout also rejects any
unclaimed token and checks the recorded claimant against the immutable owner
([`lowering/units.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

This is the strongest liftable invariant. It says more than "consume at most
once": every planned obligation has exactly one valid proof of discharge.

### 2.4 Move-only authorities and inert handles

The compiler uses the same separation at smaller scopes:

- `DynamicSpliceEdgeId` is a copyable inert identifier;
- the associated `DynamicSpliceEdge` is non-`Clone` and is removed and consumed
  before CFG emission; and
- `AffineSpliceCapability` has `Open` and `Consumed` states and rejects a second
  consumption.

The representation mirrors Ken's resource table at a high level—copyable name,
unique live authority—but the compiler's closeout is stronger because it also
requires total discharge.

### 2.5 Affinity is deliberately selective

`StaticWorkerBinding` is an ordinary lexical callable. It may be unused or
called twice. Its work-package contract explicitly forbids adding a
consumed-worker set or exact-once ledger
([RT-WORKER-BIND §3](../docs/program/wp/RT-WORKER-BIND.md)).

The recursor campaign also rejects a universal boundary-token vocabulary. It
uses separate authorities for host-effect seats, aggregate allocations,
continuation source slots, continuation specializations, calls, joins, and
typed unit transfers. Its governing principle is an exact domain-specific
producer paired with an exact checked consumption boundary
([RT-RECURSOR-TRANSPORT §2](../docs/program/wp/RT-RECURSOR-TRANSPORT.md)).

The common structure is therefore a reusable **algebra**, not one global token
schema. A formal theory may quantify over obligation kinds without erasing the
distinct producers, owners, and eliminators that give each kind meaning.

## 3. Affine versus linear

The repository usually calls these mechanisms affine, which is accurate for
the at-most-once half. The complete continuation contract is linear:

| Discipline | Permits zero uses | Permits one use | Permits many uses |
|---|---:|---:|---:|
| Unrestricted | yes | yes | yes |
| Affine | yes | yes | no |
| Linear | no | yes | no |

Rust move semantics are affine because an owned value may be dropped. The
generation table also supplies an affine guarantee: a live owner is consumed
at most once, while bracket finalization supplies eventual settlement on the
supported execution paths.

The continuation ledger rejects both duplicate and missing discharge. Its
planned-call population is therefore linear. `AffineSpliceCapability` alone
encodes an at-most-once transition, but the surrounding validation decides
whether non-consumption is also rejected.

This distinction matters to a future calculus. A usage semiring or modality
must express both policies rather than naming every non-duplicable object
"affine." In particular, compiler obligations need the exact-use grade, while
Rust-like destructible values need the at-most-once grade.

## 4. A calculus that can be extracted

The compiler suggests the following two-context lowering judgment:

```text
Γ ; Δ ⊢ₒ e ⇝ c ; Δ′
```

Read it as: under unrestricted context `Γ`, linear obligations `Δ`, and current
owner `o`, source object `e` lowers to output `c`, leaving obligations `Δ′`.

### 4.1 Contexts

`Γ` contains ordinary OTT values, proofs, reusable workers, and copyable opaque
identities. Its variables admit weakening and contraction.

`Δ` is a finite heterogeneous collection of obligations. Illustrative entries
are:

```text
Emit κ owner target
Splice κ child parent
JoinEdge κ predecessor join
Settle κ resource-kind
```

The entry, not the name `κ`, is the consumable authority. Linear obligations
admit exchange but neither weakening nor contraction.

### 4.2 Core rules

The compiler evidence supports these rule families:

1. **Planner mint.** Only a validated producer introduces `κ` and its
   obligation. Lowering cannot construct one.
2. **Exact claim.** A consumer may select only an obligation present in `Δ`.
3. **Owner check.** The ambient owner must equal the obligation's immutable
   owner.
4. **Direct discharge.** Emitting the planner-resolved target consumes the
   corresponding `Emit` obligation.
5. **Composed discharge.** A verified source-continuation composition consumes
   the same kind of obligation through a distinct rule.
6. **Disjointness.** No derivation may use both discharge rules for one `κ`.
7. **Static partition.** When the compiler emits several source occurrences or
   branch bodies, their obligation contexts form a disjoint partition. The
   compiler emits all static bodies, even when runtime control chooses one.
8. **Close.** A compilation unit or pass closes only with `Δ′ = ∅`.
9. **Unrestricted use.** Reusable worker bindings remain in `Γ` and may be used
   zero, one, or many times.

The existing maps and sets can be understood as an executable decision
procedure for a derivation in this calculus. That interpretation is testable:
the repository's duplicate-claim, wrong-owner, missing-emission,
double-discharge, and leftover mutations should correspond one-for-one with
failed premises.

### 4.3 What should not be lifted

Three implementation details are not type rules:

- `BTreeMap` and `BTreeSet` are checker representations, not semantic objects.
- A copyable Rust identity is a name, not proof that the named authority is
  duplicable.
- A global ledger is an implementation scope. The theory should preserve the
  repository's domain-specific producers rather than invent a universal
  runtime authority.

The durable abstraction is the relation among minting, ownership, discharge,
partition, and closeout.

## 5. The closest prior art

### 5.1 Quantitative Type Theory

Atkey's Quantitative Type Theory (QTT) records usage information for every
variable in a dependent typing judgment. Context addition accounts for uses
from subterms, scaling accounts for nested use, and semiring zero distinguishes
data used only in types from runtime resources
([Atkey 2018](https://bentnib.org/quantitative-type-theory.pdf)).

The especially relevant rule is that type formation and type equality are
judged in a context whose computational usages are all zero. QTT thereby lets
ordinary terms index types without spending runtime resources during type
checking. Its published presentation also warns that combining dependency and
linearity is not straightforward: a faulty usage design can make substitution
inadmissible.

Idris 2 demonstrates QTT in a full-scale language, including erasure and
resource protocols used for type-safe concurrent programming
([Brady 2021](https://arxiv.org/abs/2104.00480)). It establishes that dependent
types plus quantitative use are practical. It does not provide Ken's
observational equality.

### 5.2 Linear dependent type theories

Lundfall presents ordinary cartesian dependent types together with linear
types in indexed fibers. Both ordinary and linear types may depend on
**cartesian** terms, and modalities connect the two fragments
([Lundfall 2018](https://arxiv.org/abs/1806.09593)). This is close to the safe
first boundary suggested by Ken's compiler: ordinary identities describe a
linear obligation, while the obligation itself does not enter unrestricted
type formation.

Fu and Xi instead stratify a language into logical and program levels. Proofs
and types erase from programs, extracted programs make progress and run
memory-clean, and programs can be reflected into the logical layer for
verification
([Fu and Xi 2025](https://arxiv.org/abs/2309.08673)). This is a second plausible
shape for Ken because the current causal machinery is compiler-owned and does
not become a runtime value.

Multimodal Dependent Type Theory provides a broader framework parameterized by
modes and modalities
([Gratzer et al. 2021](https://arxiv.org/abs/2011.15021)). It may become useful
if Ken eventually needs explicit transitions among logical, affine, runtime,
and observed worlds. It is more machinery than the first compiler extraction
requires.

### 5.3 Observational Type Theory

OTT combines terminating computation and decidable definitional equality with
an extensional, substitutive propositional equality. Functions are equal when
they are pointwise equal, and explicit coercion transports values between
observationally equal types while preserving canonicity
([Altenkirch and McBride 2006](https://ncatlab.org/nlab/files/AltenkirchMcBrided-TowardsOTT.pdf)).

Recent TTobs and CCobs work develops proof-irrelevant observational equality,
normalization, decidable conversion, quotients, and inductive types. A recent
implementation uses normalization by evaluation and a bidirectional checker
([Sirman, Lennon-Bertrand, and Krishnaswami
2025](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.TYPES.2024.5)).

The literature search for this report used exact combinations of
"observational type theory/equality" with "linear," "affine," and
"quantitative type theory," as well as the citation neighborhoods of the QTT
and TTobs papers. It found mature work on each side and no published direct
integration of QTT-style resource accounting with TTobs/CCobs-style
observational equality. This is evidence that no off-the-shelf construction
was readily identifiable as of 2026-08-08, not a claim that no unpublished or
differently named system exists.

## 6. The unsolved ATT–OTT boundary

### 6.1 Equality of linear functions

OTT explains equality of ordinary functions pointwise. For `f` and `g` of
ordinary type `(x : A) → B x`, one observes both at the same `x` and compares
their results.

The analogous rule for a linear function `A ⊸ B` is not immediate. Supplying
the same owned resource to both `f` and `g` appears to duplicate it. A sound
account may need:

- two related resource worlds, with one corresponding input in each;
- a duplicable observation modality that exposes only non-owning information;
- a linear logical relation rather than an internal pointwise `Eq`; or
- exclusion of linear functions from observational equality.

The compiler ledger never compares two consumers extensionally, so it supplies
no answer among these alternatives.

### 6.2 `cast` and ownership preservation

OTT's `cast` transports a value between equal types. If the value owns a
resource, cast must consume exactly one capability and return exactly one
capability without changing its owner, duplicating it, or silently dropping
it. Its computation rules must also remain compatible with erasure and
canonicity.

The compiler has owner-preserving discharge operations, but no type equality
whose motive contains an owned value. That makes the ledger a useful test case
for a proposed `cast` rule, not an implementation of one.

### 6.3 Dependency on resource-sensitive terms

The conservative QTT boundary judges type formation and equality at zero
resource usage. Ken could initially permit ordinary terms in `Γ` to index
linear obligations while prohibiting dependency on entries of `Δ`.

That restriction yields a genuine combined language, but a deliberately
stratified one. It does not solve fully linear dependency, where types inspect
resource-sensitive terms. Relaxing the restriction would require a semantics
that distinguishes contemplating a value in a type from consuming it at
runtime.

### 6.4 Proof irrelevance is not ownership erasure

Ken's `Ω` is definitionally proof-irrelevant. Consumption history and owner
identity can affect whether generated code is lawful. They therefore cannot be
hidden in `Ω` merely to make the accounting proof erase: proof irrelevance must
not identify two computationally distinct authorities or make a missing
discharge unobservable.

A safe design may keep resource accounting in the typing judgment rather than
as an ordinary proof term. If it internalizes certificates, their erasure
theorem must show that the checked program still preserves the external
resource semantics.

### 6.5 Structural metatheory changes

Ken currently requires weakening and ordinary substitution. A linear `Δ` may
not admit weakening, and substitution must scale or partition usages. A
combined theory owes, at minimum:

- admissible quantitative substitution;
- preservation and progress;
- decidable type checking and conversion;
- normalization and canonicity for the OTT fragment;
- sound erasure of proofs and usage annotations;
- observational soundness for linear values; and
- a model showing that `Eq` and `cast` preserve ownership.

None of these results follows from the Rust compiler's passing tests.

## 7. A bounded research program

The following sequence extracts the discovery without committing Ken's product
language to research-grade type theory.

### R1 — Specify the causal-obligation calculus

Write a small formal syntax for obligation kinds, opaque identities, owners,
minting, direct discharge, composed discharge, branch partition, and closeout.
Start with the continuation-call ledger because it already has the strongest
set equations.

Acceptance evidence should map each formal premise to a current production
check and each formal failure to a committed mutation or negative witness.

### R2 — Prove the standalone calculus

Prove:

- no forgery;
- no duplicate discharge;
- no missing discharge;
- owner agreement;
- preservation under lowering steps; and
- progress to either a valid close or a precisely attributed refusal.

Then derive the existing set-based ledger as a decision procedure. A model
checker or proof assistant may be appropriate, but the first artifact should
remain independent of Ken's kernel.

### R3 — Embed it beside a frozen OTT layer

Add a two-context or quantitative program judgment around an unchanged OTT
logical context. Initially require zero linear usage for:

- context and type formation;
- definitional equality and conversion;
- observational `Eq` formation and elimination; and
- `cast`.

This stage should demonstrate elaboration and erasure on compiler-sized
examples without yet exposing source-level OS resources.

### R4 — Make equality the explicit hard gate

Develop and compare candidate semantics for equality of linear values:

- a two-world logical relation;
- a modality exposing duplicable observations;
- a restricted `Eq` universe; or
- an explicit refusal to compare resource-sensitive types.

Proceed only if one account preserves both the linear invariants and Ken's OTT
goals for decidability, substitution, and canonicity. Failure at this gate does
not invalidate R1 or R2; it means the calculus remains a verified compiler
protocol rather than a Ken language feature.

### R5 — Consider resources, then borrowing

Only after the equality gate should the model attempt a source-level resource
API. A useful first case is indexed typestate: an unrestricted state index in
`Γ` describes a unique capability in `Δ`, and an operation consumes the old
capability and returns the next one.

Rust-like borrowing should be a later and separate investigation, using Oxide
and RustBelt rather than assuming that quantitative use counts already model
loans and lifetimes.

## 8. Consequences for the current program

1. **Do not attribute the compiler campaign's difficulty to a missing ATT–OTT
   kernel feature.** The evidence identifies a difficult ownership and causal
   provenance problem in the lowering architecture. The current Ken source
   program remains type-correct OTT before this machinery runs.
2. **Preserve the compiler mechanisms.** The exact tokens, owner checks,
   partitions, and closeout failures are valuable executable research evidence.
3. **Do not generalize every compiler object as affine.** The explicit
   non-affine worker-binding decision is part of the discovery.
4. **Do not merge domain ledgers into a universal schema.** Extract a common
   metatheory while retaining the distinct authority producers that make each
   obligation meaningful.
5. **Keep ADR 0021 in force.** Dynamic resource handles and Ward obligations are
   the settled product design unless a later, proven theory earns replacement.
6. **Describe the research result narrowly.** Ken has an executable model of
   owner-indexed, exactly-once causal discharge. It does not yet have affine
   observational equality or Rust-like borrowing.

## Conclusion

The compiler work has crossed an important threshold. It no longer merely
contains resources that Rust happens to move. It contains a repeated,
articulated logic of causal authority:

```text
planner mint
    → opaque owner-indexed obligation
    → one exact domain-specific discharge
    → disjoint and exhaustive closeout
```

That logic can be lifted into a standalone linear calculus and then tested as
a quantitative program layer beside Ken's OTT logic. The strongest evidence
for the synthesis is the compiler's precise distinction among inert names,
consumable authority, lawful reuse, and exactly-once discharge.

The missing research is also precise. OTT's `Eq` and `cast` must be given a
resource-sensitive semantics that does not duplicate, discard, or erase
ownership. Until that is proved, Ken has discovered a compelling **input** to
ATT–OTT unification, not the unification itself.
