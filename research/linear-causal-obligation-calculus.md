# A linear causal-obligation calculus for compiler lowering

**Research status:** advisory formalization, not a language or architecture
ruling

**Grounding:** `origin/main` at
`8c6ab6f9f42e5e4d63740fe87d943b9f7928fa4c`

**Date:** 2026-08-08

## Executive assessment

Section 4 of
[the causal-obligation report](causal-obligation-calculus.md#4-a-calculus-that-can-be-extracted)
can be expanded into a small standalone calculus. The central judgment is a
checked transition over two contexts:

```text
P ; Gamma ; Delta |- S --r / b / epsilon--> S' ; Delta'
```

Here `P` is an immutable validated plan, `Gamma` contains unrestricted
material, `Delta` contains outstanding causal obligations, `S` is the current
lowering state, `r` is the selected reduction rule, `b` is an optional backend
command, and `epsilon` is independently checkable discharge evidence.

The calculus is linear where the compiler requires an exact discharge, affine
where a planned record merely authorizes an event, and unrestricted where an
ordinary worker or inert identity may be reused. It therefore should not be
described as one universal affine ledger. Four different closure laws are
already visible in the compiler:

1. exact realization: `planned = consumed = emitted`;
2. alternative realization: `planned = direct disjoint-union composed`;
3. event authorization: `domain(R) = E` and `image(R) subset-of P`; and
4. visit closure: each visit claims exactly its planned seats, while the global
   image may be a subset of the authorized population.

These are instances of a shared algebra of identity, ownership, evidence, and
closure. They are not one interchangeable policy. The types of the obligations
and the closure law attached to each domain must remain explicit.

The calculus is extractable now as a compiler protocol. It still is not a
unification of affine type theory and observational type theory. It neither
changes Ken's kernel context nor supplies resource-sensitive rules for
observational equality or `cast`. The product posture is unchanged: `R2`
remains closed and ADR 0021 remains in force.

## 1. Scope and intended use

This report formalizes the compiler-local discovery, rather than proposing a
new source language. It has three intended uses:

- give one precise vocabulary to the repeated planner/lowering invariants;
- state the reduction and closure laws independently of their current Rust
  containers; and
- provide a specification against which an explicit lowering IR or checked
  transition engine could be assessed.

The formalization is deliberately smaller than Rust ownership. It omits loans,
lifetime inclusion, aliasing, destructors, unwinding, provenance, interior
mutability, and unsafe abstraction. It is also smaller than Ken's OTT: it does
not define terms, types, equality, conversion, or normalization for Ken.

The unit of study is a compiler-owned causal authority such as a checked call,
continuation discharge, aggregate allocation authorization, or effect-seat
claim. The existing compiler evidence is described in
[the parent report](causal-obligation-calculus.md#2-the-compiler-evidence).

## 2. Static ingredients

### 2.1 Sorts

Assume the following abstract sorts:

```text
kappa in Identity
o     in Owner
q     in ObligationKind
t     in TermId
g     in GeneratedTermId
u     in UnitId
e     in EventId
p     in PlanRecord
b     in BackendCommand
epsilon in Evidence
```

`Identity` values are opaque. Only the planner can mint them. A lowering rule
may compare, transport, and select an identity but cannot construct one from
its fields.

An identity is not itself the resource. It is a copyable name used to select an
entry in the obligation context. This distinction matches the compiler's
copyable `ContinuationCallIdentity` and move- or ledger-governed authority
([`lowering/units.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

### 2.2 Plans

A plan `P` is an immutable finite structure containing at least:

```text
P.identities : finite set Identity
P.owner      : Identity -> Owner
P.kind       : Identity -> ObligationKind
P.target     : partial Identity -> Target
P.origin     : TermId -> SourceOrigin
P.children   : TermId -> finite ordered sequence TermId
P.closeLaw   : ObligationKind -> ClosureLaw
```

`P` is admitted only after its internal identity, owner, target, occurrence, and
child relations validate. The compiler already centralizes the sole
origin-to-expression route in `StaticTransitionPlan::source_occurrence` and
checks that a stored occurrence agrees with the table position
([`static_transition.rs`](../crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs)).

The calculus treats validation as a premise. It does not permit lowering to
repair, extend, or reinterpret the plan.

### 2.3 Unrestricted context

`Gamma` contains material that admits weakening and contraction:

```text
Gamma ::= empty
        | Gamma, x : A
        | Gamma, term(t)
        | Gamma, generated(g)
        | Gamma, inert(kappa)
        | Gamma, worker(w)
        | Gamma, backend-handle(h)
```

The categories have different meanings even though they share structural use:

- `term(t)` names a planner-owned source occurrence;
- `generated(g)` names compiler-synthesized material that has no source origin;
- `inert(kappa)` is a copyable causal name, not authority to discharge it;
- `worker(w)` is an ordinary reusable callable; and
- `backend-handle(h)` is local to the function in which the backend minted it.

The separate generated-term category is necessary. The source machine can
synthesize a default trap that has no planned source occurrence, so replacing
every owned expression with a source-origin tag would fabricate provenance
([`OwnedSourceOccurrence`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).

Function-local backend handles also cannot be treated as portable identities.
Cranelift `FuncRef`, `GlobalValue`, `Value`, and `Inst` entities are meaningful
inside one generated function. The calculus may record them in evidence, but
only under the defining function's scope.

### 2.4 Linear obligation context

`Delta` is a finite map from opaque identities to obligation entries:

```text
Delta ::= empty | Delta, kappa : Q(o, payload)
```

Map uniqueness makes two distinct live entries with the same identity
unrepresentable. The context admits exchange, but it does not generally admit
contraction or weakening.

An obligation entry contains:

- its domain-specific kind `Q`;
- its immutable owner `o`;
- the target or contract selected by the plan; and
- any coordinates necessary to distinguish members of the planned population.

Those coordinates are semantic, not incidental. The current continuation
identity requires producer construct, alternative, call-site sequence, and
recursive position. Dropping one coordinate can turn two obligations into one
key and misreport a collision as double consumption
([`ContinuationClaimLedger`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

### 2.5 Grades and closure laws

The context discipline is parameterized by a closure law, not by one global
notion of use:

```text
ClosureLaw ::=
    ExactRealization
  | AlternativeRealization(discharge forms)
  | EventAuthorization
  | VisitClosure
  | AffineAtMostOnce
```

`ExactRealization` and `AlternativeRealization` are linear: zero uses and
multiple uses both fail. `EventAuthorization` permits unused planned records,
while requiring every event to have exactly one governing record.
`AffineAtMostOnce` permits discard but rejects a second use. An unrestricted
worker does not enter `Delta` at all.

This parameter is what preserves the distinction between obligation and
authorization. Treating `P` as universally mandatory would incorrectly reject
lawful unused aggregate records and effect seats. Treating every entry as
optional would silently drop continuation calls.

## 3. Dynamic ingredients

### 3.1 States

A minimal source-lowering machine needs only evaluation and returned-value
states:

```text
S ::= Eval(term, env, K)
    | Return(value, route, K)
    | Closed(output)
    | Refused(attribution)
```

`K` is a closed continuation datatype. It contains a constructor for each
pending evaluation context: let body, constructor argument, branch selection,
call callee, call argument, projection, checked invocation return, recursor
frame, and terminal return.

The current compiler already has this defunctionalized shape in
`SourceMachineState` and `SourceContinuation`
([`lowering/mod.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).
Its main source loop is therefore an implementation candidate for the reduction
relation rather than merely analogous code
([`lowering/core.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/core.rs)).

### 3.2 Backend commands

Backend effects should be explicit commands selected by rules:

```text
b ::= NoCommand
    | EmitConst(...)
    | EmitCall(target, args)
    | EmitBranch(test, yes, no)
    | EmitJump(block, args)
    | BeginBlock(block)
    | SealBlock(block)
    | EmitAggregate(layout, fields)
    | EmitHostDispatch(operation, operands)
    | EmitTrap(reason)
```

The list is illustrative, not a proposed frozen API. The important separation
is between selecting a semantic rule and performing a Cranelift mutation. It
allows rule coverage to be checked without pretending that actual emitted CLIF
can be reconstructed from symbolic evidence alone.

### 3.3 Evidence

Evidence is a typed observation produced after, or simultaneously with, the
backend command:

```text
epsilon ::= NoEvidence
          | Claimed(kappa, owner)
          | DirectCall(kappa, function, inst, decodedTarget)
          | ComposedCall(kappa, function, inst, decodedTarget, returnRoute)
          | Allocation(event, record)
          | SeatClaim(group, seat, observedPhase)
          | Partition(left, right)
          | ClosedLaw(kind, summary)
```

An evidence constructor is admitted only after checking the concrete artifact
it describes. For example, a direct-call identity enters the emitted set only
after the instruction exists and its decoded callee agrees with the
planner-resolved target. This is stronger than recording an intention before
emission
([`CheckedCallLedger`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

## 4. Judgments

### 4.1 Plan validity

```text
|- P valid
```

This judgment establishes uniqueness of identities, total owner/kind lookup on
the planned population, validity of targets, and consistency of source-child
relations. It also establishes that every obligation kind names one closure
law.

### 4.2 State well-formedness

```text
P ; Gamma ; Delta |- S state
```

The judgment requires every source term id in `S` or `K` to be present in the
plan, every generated term to be present in the compiler-generated arena, every
environment binding to be valid for the occurrence, every backend handle to
belong to the active function, and every causal name carried by `S` to be either
inert or backed by the matching entry in `Delta`.

### 4.3 Checked reduction

```text
P ; Gamma ; Delta |- S --r / b / epsilon--> S' ; Delta'
```

The rule `r` determines the next semantic state and declares the obligation
effect. The backend interpreter performs `b`. The evidence checker validates
`epsilon` against the resulting artifact. Only then is `Delta'` committed.

This ordering prevents a planned intention from satisfying a realized-event
law. It also makes refusal atomic: if command interpretation or evidence
checking fails, no obligation is considered discharged.

### 4.4 Closure

```text
P ; Gamma ; Delta |- close(output, evidence)
```

Closure applies each domain's own law. It does not reduce every domain to
`Delta = empty`; an authorization population may lawfully contain records that
no event used. What must be empty is the set of unsatisfied mandatory
obligations and the set of open local groups or transactions.

## 5. Core reduction rules

The rules below use `Delta - kappa` for removal of one exact entry and
`Delta(kappa)` for lookup. A failed premise yields an attributed refusal, not a
fallback rule.

### 5.1 Planner mint

Minting belongs to plan construction:

```text
kappa fresh
P' = P + plan(kappa, Q, owner, payload)
------------------------------------------------ Plan-Mint
P ==> P'
```

No lowering judgment concludes with a larger planned population. This is the
no-forgery boundary.

### 5.2 Enter source occurrence

```text
P.origin(t) = source
P ; Gamma ; Delta |- Eval(t, env, K) state
------------------------------------------------ Enter-Source
P ; Gamma ; Delta |- Eval(t, env, K)
  --enter(t) / NoCommand / NoEvidence-->
  Dispatch(source, env, K) ; Delta
```

The premise distinguishes a planned source term from a generated term. There is
no rule that guesses an origin from content or traversal position.

### 5.3 Ordinary structural steps

An ordinary source reduction does not touch the obligation context:

```text
child(P, t, i) = ti
------------------------------------------------ Eval-Let
P ; Gamma ; Delta |- Eval(Let(t0, t1), env, K)
  --let / NoCommand / NoEvidence-->
  Eval(t0, env, LetBody(t1, env, K)) ; Delta
```

Similar rules cover constructors, calls, projections, matches, and return
frames. Exhaustive pattern matching over the term and continuation datatypes is
part of the completeness argument; a catch-all that silently sends new forms
through an obligation-free path would invalidate it.

### 5.4 Exact claim

```text
Delta(kappa) = Q(owner, payload)
ambientOwner(S) = owner
------------------------------------------------ Claim
P ; Gamma ; Delta |- S
  --claim(kappa) / NoCommand / Claimed(kappa, owner)-->
  S ; Delta[kappa := claimed(Q, owner, payload)]
```

If `kappa` is absent, already claimed, or owned by another unit, no rule
applies. The implementation should report which premise failed.

Claim and discharge are distinct because some domains declare or resolve a
target before the event exists. Claiming alone cannot satisfy a realized-event
closure law.

### 5.5 Direct call discharge

```text
Delta(kappa) = claimed(Emit(owner, target))
ambientOwner(S) = owner
interpret EmitCall(target, args) = artifact(inst)
decodeCallee(inst) = target
------------------------------------------------ Discharge-Direct
P ; Gamma ; Delta |- S
  --direct(kappa) / EmitCall(target, args)
    / DirectCall(kappa, function, inst, target)-->
  Return(result(inst), Direct(kappa), K) ; Delta - kappa
```

The `decodeCallee` premise is intentionally post-emission. A recorded target
and an actual target are independent facts.

### 5.6 Composed continuation discharge

```text
Delta(kappa) = Emit(owner, continuationTarget)
ambientOwner(S) = owner
compositionContract(S, kappa) = (rawWorker, args, downstream)
interpret EmitCall(rawWorker, args) = artifact(inst)
verifyReturnRoute(inst, downstream)
------------------------------------------------ Discharge-Composed
P ; Gamma ; Delta |- S
  --compose(kappa) / EmitCall(rawWorker, args)
    / ComposedCall(kappa, function, inst, rawWorker, downstream)-->
  Return(result(inst), Composed(kappa), downstream) ; Delta - kappa
```

The direct and composed rules consume the same obligation through different
evidence. Since both remove the unique `kappa` entry, using both is
unrepresentable in a derivation. The compiler additionally asserts the
disjointness of the evidence sets at closeout so an implementation bug cannot
hide behind the abstract rule
([`ContinuationClaimLedger::close`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

### 5.7 Affine capability consumption

```text
Delta(kappa) = Affine(owner, payload)
ambientOwner(S) = owner
------------------------------------------------ Consume-Affine
P ; Gamma ; Delta |- S
  --consume(kappa) / b / epsilon-->
  S' ; Delta - kappa
```

An affine close does not require every such entry to be consumed. A second
consumption still has no derivation because the entry is absent.

### 5.8 Branch partition

```text
Delta1 disjoint Delta2
Delta = Delta1 union Delta2 union DeltaShared
P ; Gamma ; Delta1 union DeltaShared |- S1 ==> S1' ; DeltaShared'
P ; Gamma ; Delta2 union DeltaShared |- S2 ==> S2' ; DeltaShared'
------------------------------------------------ Static-Branch-Partition
P ; Gamma ; Delta |- EmitBothBodies(S1, S2) ==> ...
```

This is a compiler-static partition, not runtime exclusive choice. If the
compiler emits both static bodies, each body receives its own mandatory
obligations. Only genuinely unrestricted or authorization-only material may be
shared.

The premise must be adapted for more than two branches and for obligations
owned by a surrounding unit. The essential law is that one mandatory causal
identity cannot be assigned to two emitted bodies or to neither.

### 5.9 Unit close

```text
mandatory(Delta, owner) = empty
noOpenGroups(owner)
allEvidenceLocalTo(owner)
------------------------------------------------ Close-Unit
P ; Gamma ; Delta |- Return(v, route, Terminal(owner))
  --close-unit / Return(v) / ClosedLaw(owner, summary)-->
  Closed(v) ; Delta
```

Whole-pass closure subsequently checks cross-unit populations. A body-local
close is still required where publishing a body before detecting a discarded
group would cross the desired refusal boundary. `EffectSeatLedger` therefore
checks each body before definition and restates the relation at whole-pass
close
([`lowering/mod.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).

## 6. Domain closure laws

### 6.1 Exact realization

Let `Pq` be the planned identities for kind `q`, `Cq` the independently
recorded consumed identities, and `Eq` the identities decoded from emitted
artifacts:

```text
Cq = Pq
Eq = Pq
```

Set equality is required. Equal cardinality is insufficient because two sets
of the same size may contain different members. `CheckedCallLedger` implements
this law and separately checks each actual callee against its resolved target
([`lowering/units.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/units.rs)).

### 6.2 Alternative realization

Let `Dq` be direct-emission evidence and `Kq` verified composition evidence:

```text
Dq intersect Kq = empty
Dq union Kq = Pq
```

The first equation rejects double answers. The second rejects missing and
unplanned answers. This is the continuation-call law.

### 6.3 Event authorization

Let `E` be independently observed events, `P` the planned authorization
records, and `R subset-of E x P` the event-to-record relation:

```text
domain(R) = E
image(R) subset-of P
R is functional from E to P
```

`image(R) = P` is intentionally not required. A plan record can authorize an
allocation in a body the compiler never emits. `AggregateAllocationLedger`
records events separately from relation keys, checks `domain(R) = E` locally,
and checks the global image against the planned record population
([`lowering/mod.rs`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).

### 6.4 Visit closure

For each concrete visit group `g`, let `Pg` be its planned seats and `Cg` its
claims:

```text
Cg = Pg
observedPhase(c) in availability(c) for every c in Cg
image(all claims) subset-of global planned seats
opened groups = committed groups
```

The local equality prevents one incomplete visit from being masked by another
visit to the same source occurrence. The global subset again permits planned
seats whose containing body was never emitted. This is the effect-seat law
([`EffectSeatLedger`](../crates/ken-runtime/src/cranelift_backend/lowering/mod.rs)).

## 7. Safety properties

A standalone calculus should establish the following theorems.

### 7.1 No forgery

If `kappa` occurs in a reachable `Delta`, then `kappa` occurs in the validated
plan and its kind, owner, and payload agree with that plan.

The proof is by induction on reduction. Planner minting is outside lowering,
and no reduction rule introduces a new identity.

### 7.2 At-most-once discharge

For any derivation trace and mandatory identity `kappa`, at most one discharge
evidence item names `kappa`.

The proof follows from unique map membership and removal on discharge. The
implementation still owes an evidence-set disjointness check because the Rust
representation can violate what the abstract transition makes impossible.

### 7.3 Owner preservation

Every claim and discharge for `kappa` occurs under `P.owner(kappa)`. Transport
through a continuation may retain the identity but cannot change the owner.

### 7.4 No missing mandatory discharge

If a compilation reaches whole-pass `Closed`, every mandatory planned identity
has evidence satisfying its closure law.

This is a closure theorem, not a consequence of at-most-once use. It is the
property that makes the strongest compiler mechanisms linear rather than
merely affine.

### 7.5 Artifact agreement

Every evidence item about a backend event corresponds to an event in the
finished function, and every governed event in the finished function appears
in the relevant relation.

This property requires an observation over concrete CLIF. It cannot be proved
solely from a symbolic command trace if the backend interpreter may select the
wrong callee, reorder operands, or omit an instruction.

### 7.6 Progress or attributed refusal

For a valid state, either one reduction rule applies, the state is lawfully
closed, or a failed premise identifies a specific absent authority, owner
mismatch, unsupported representation, or backend inconsistency.

This is not ordinary source-language progress: refusing compilation is a valid
outcome. The useful property is absence of silent fallthrough and precise
attribution to the unmet premise.

## 8. Executable decision procedure

The current ledgers are a decision procedure for fragments of the calculus:

| Calculus premise | Existing evidence |
|---|---|
| identity is planned | lookup in the planner-derived set or map |
| identity has exact owner | immutable owner comparison at claim |
| no duplicate claim | occupied claim slot or failed set insertion |
| direct event exists | decoded `Inst` after function finalization |
| composed route is real | checked raw-worker call, operands, and downstream return |
| direct/composed are disjoint | set intersection is empty |
| mandatory population is complete | set union/equality with planned population |
| every allocation is governed | independently recorded `domain(R) = E` |
| every visit is complete | per-group claimed-seat equality |
| no open transaction escapes | opened/committed group and function equality |

This table suggests a mutation discipline. Each formal premise should have a
negative witness that changes only that premise and makes the checker refuse.
A negative absence check also needs a positive reachability control; otherwise
it passes when the relevant seat is never reached.

## 9. What the calculus abstracts

The following are proper abstractions:

- finite identity-indexed obligation contexts;
- immutable owner attribution;
- claim, discharge, partition, and closure transitions;
- typed evidence produced by independently observed events;
- alternative discharge forms over one mandatory population; and
- local transactions whose closure precedes artifact publication.

These abstractions can support common libraries and common diagnostic
structure without erasing domain meaning.

## 10. What must remain concrete

The following should not be collapsed into one generic token or ledger:

- the identity type and all coordinates that make it injective;
- the planner producer that is authorized to mint it;
- the owner type and the distinction among source, emission, route, and
  function ownership;
- the precise discharge forms permitted for the domain;
- whether the plan is mandatory or merely authorizing;
- the backend event that counts as realization;
- the local publication boundary at which closure must occur; and
- source terms versus compiler-generated terms.

Nor should `BTreeMap`, `BTreeSet`, Cranelift entities, or current module
boundaries appear in the mathematical core. They are implementations of finite
relations and scoped evidence.

## 11. Relationship to ATT and OTT

This calculus can sit beside Ken's current OTT layer in a stratified system:

```text
Gamma |-OTT t : A
Gamma ; Delta |-O lower(t) => artifact ; Delta'
```

Initially, OTT type formation, equality, conversion, `Eq`, and `cast` would use
only `Gamma`. Ordinary OTT terms could index obligation payloads, while linear
obligations could not be inspected during type equality. This resembles the
zero-usage boundary in Quantitative Type Theory and the cartesian indexing of
linear dependent type theories discussed in
[the parent report](causal-obligation-calculus.md#5-the-closest-prior-art).

That stratification does not solve observational equality for linear values.
In particular, it gives no rule for comparing two linear functions on the
"same" owned argument and no proof that `cast` preserves exactly one owner.
Those remain the explicit research gate
([parent report §6](causal-obligation-calculus.md#6-the-unsolved-attott-boundary)).

The compiler calculus is useful even if that gate never opens. It can specify
and check compiler protocols without becoming a source-language feature.

## 12. Recommended research sequence

1. Formalize one domain first: continuation-call alternative realization.
2. Give the small-step transition system a mechanized finite-map model.
3. Prove no forgery, at-most-once discharge, owner preservation, and mandatory
   closure.
4. Show that the current continuation ledger decides the same closure
   predicate on extracted traces.
5. Add event authorization and visit closure as separately typed policies.
6. Define a trace format that records rules and evidence without exposing
   portable Cranelift handles.
7. Only then test the two-context embedding beside frozen OTT.

The research should resist two shortcuts. First, a universal `Obligation` enum
plus flags such as `required`, `repeatable`, and `composed` would move the laws
from types into combinations of booleans. Second, a symbolic trace should not
replace post-emission inspection of concrete CLIF. Both shortcuts would make
the abstraction easier to state while weakening what the present compiler
actually proves.

## Conclusion

The liftable object is a family of typed causal-obligation systems sharing one
small algebra:

```text
validated mint
  -> opaque identity and immutable owner
  -> checked state transition
  -> concrete backend event
  -> independently validated evidence
  -> domain-specific closure law
```

The family contains linear, affine, authorization, and unrestricted cases. Its
value lies in saying which case applies and making every transition explicit,
not in making them look uniform. Formalized at that boundary, it can improve
the compiler immediately and provide a precise model problem for later ATT–OTT
research without changing Ken's settled resource posture.
