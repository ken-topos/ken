# Checked-source terminal trap: design recommendation

**Status:** Research advisory, recording an operator direction for Architect and
spec-enclave disposition. This is not a build kickoff or a normative spec.

**Grounding:** `origin/main` at
`2e58836795bde517f10f4f1510bf9f656c94fb08` (2026-07-17), plus the Steward's
unlaunched PREP frame at
`steward/work:docs/program/wp/FOLLOWUP-checked-source-controlled-trap-producer.md`
(`e2bdc9bd`).

## Recommendation

Ken should model a checked-source controlled trap as a **terminal, uncatchable,
sealed effect event**. A source operation such as `trap reason` should elaborate
to an ordinary `ITree` `Vis` node whose response type is empty. The admitted
checked node is the honest evidence that the computation can terminate without
returning its result type. Identity-checked erasure should lower that exact node
to the existing `RuntimeExpr::Trap`, and the interpreter should settle the same
node to the same canonical `RuntimeObservation::Trapped` value.

The operator has settled the central product choice: **trap is terminal**. It is
not a catchable exception. Recoverable failure remains `Result`; making trap
catchable would confusingly duplicate that value-level protocol and introduce a
larger handler, resumption, and cleanup design problem.

This design preserves the important boundaries:

- no inhabitant of `Empty` or arbitrary pure result type is manufactured;
- no partial match, postulated `Bottom`, `axiom`, or `absurd` is admitted;
- no general recursion or divergence is introduced;
- no kernel term, rule, or trusted primitive is added;
- the checked core carries structural trap evidence before erasure; and
- checked source cannot install a handler that discharges or resumes `Trap`.

Interpreter and native execution expose one controlled terminal identity, never
a host panic or process abort.

## Why the effect model is the right home

Ken already represents effectful programs as ordinary pure interaction trees:

```text
Ret : R -> ITree E R
Vis : (e : E.Op) -> (E.Resp e -> ITree E R) -> ITree E R
```

The effect specification makes this a pure inductive encoding and keeps all
effect-specific machinery out of the kernel
(`spec/30-surface/36-effects.md:438-511`). The landed prelude uses that same
shape for `HostIO`: filesystem operations are combined with `AmbientOp`, and
`withResource` sequences its body and release through `bind`
(`crates/ken-elaborator/src/prelude.rs:1580-1593,1715-1748`).

A terminal operation fits this representation directly:

```text
data TrapReason = UserTrap String
data TrapOp = Raise TrapReason

fn trap_resp (op : TrapOp) : Type =
  match op { Raise reason |-> Empty }

proc trap (a : Type) (reason : TrapReason)
  : HostIOWithTrap a visits [Trap] =
  Vis ... (Raise reason) (\impossible. match impossible { })
```

This is schematic Ken, not a proposed syntax ruling. The Architect and spec
enclave own the precise operation family, coproduct placement, visibility, and
surface spelling.

The key soundness point is that `trap` does **not** prove or return an arbitrary
`a`. It constructs an `ITree ... a`. Its continuation accepts a response of
type `Empty`, so the empty match is justified, but a conforming interpreter
never supplies such a response. Execution observes the `Raise` event and
terminates the computation. Partiality is therefore explicit in the checked
effect tree rather than smuggled in through false elimination.

This use of an empty response follows the algebraic-effect intuition that an
operation can be non-resuming. Frank provides a closely related typed-effect
precedent, while interaction trees provide the broader event-plus-continuation
representation on which Ken's own finite inductive design is based:
[Lindley, McBride, and McLaughlin, *Do be do be do*][frank]
and
[Xia et al., *Interaction Trees*][itrees].

Koka also usefully separates exceptional termination from divergence in its
effect types. That distinction supports treating controlled trap as a finite
terminal event, not as a license for nontermination:
[Koka, effect typing][koka].

## Checked evidence must be structural

An effect-row declaration alone is not trap evidence. Ken deliberately checks
`rho_inferred` as a subset of `rho_declared`; declarations may reserve unused
headroom (`spec/30-surface/36-effects.md:148-157`). Consequently:

```text
proc f (...) : HostIOWithTrap A visits [Trap] = Ret ... value
```

may be a valid stable interface, but it did not trap. Erasure must not translate
the declaration, latent row, or exported effect alphabet into
`RuntimeExpr::Trap`. It must find the exact, reachable, checked `Vis` operation.
This is the same declared-upper-bound versus actual-perform-node distinction
that matters elsewhere in the effect/export boundary.

The evidence rule should therefore be:

1. Elaboration admits the ordinary `Vis` node only when its operation family,
   operation constructor, payload, response family, and continuation type all
   check.
2. Effect inference observes the real `Raise` node and contributes `Trap` to
   the inferred row.
3. Dropping `Trap` from the declared row produces the existing effect-escape
   rejection.
4. Erasure recognizes the stable checked identity of that node, not a name,
   spelling, row declaration, or metadata-only claim.
5. Any malformed or near-match identity fails closed as unsupported erasure or
   a compile-time error; it is never coerced into a trap and never panics.

The raw operation family, constructor, response eliminator, and handler-facing
identity must be sealed from checked source. A small public library operation
can be the only constructor-facing surface, much as the resource package keeps
its producer protocol private. The privileged top-level runtime interpreter is
the sole consumer. A dedicated keyword is unnecessary for the first version:
grammar adds no semantic evidence that the checked `Vis` node does not already
carry.

This seal is load-bearing. Ken's general handler model can stop rather than
resume a handled operation (`spec/30-surface/36-effects.md:1041-1072`). If
checked code could name and discharge `Trap`, it would be a catchable exception
regardless of an empty response type. The visibility/capability rule must make
that program ill-formed, not merely discourage it by convention.

## Lowering and execution contract

### Erasure

When erasure encounters the exact checked trap event, it should validate:

- the canonical trap operation family and `Raise` constructor identity;
- the closed response family and its reduction to `Empty`;
- the canonical, fully initialized reason payload; and
- the absence of any reachable resumption path.

It should then emit:

```text
RuntimeExpr::Trap(RuntimeTrap {
  code: RuntimeTrapCode::ExplicitTrap,
  message: canonical_message(reason),
})
```

The runtime substrate already exists. `RuntimeExpr`, `RuntimeObservation`, and
`RuntimeTrapCode::ExplicitTrap` are defined in
`crates/ken-runtime/src/ir.rs:335-480`. Cranelift lowering already converts a
`RuntimeExpr::Trap` to a trapped observation
(`crates/ken-runtime/src/cranelift_backend.rs:1226-1233,3296`), and the native
differential path compares trap code and message. The host protocol also has a
versioned `TerminalErrorV1::RuntimeTrap` carrier
(`crates/ken-host/src/effect_v1.rs`).

What is missing is the checked-source-to-runtime bridge. The current
interpreter trap test constructs `RuntimeExpr::Trap` as a fixture rather than
deriving it from admitted source
(`crates/ken-interp/tests/nc12_runtime_ir_evaluator.rs:517-529`).

### Interpreter

The interpreter should recognize the same canonical checked `Raise` identity
before ordinary host dispatch. It must:

- not invoke or synthesize a continuation response;
- finalize invocation and resource settlement exactly once;
- produce the same code and canonical message as erasure/native execution; and
- return a controlled terminal observation, not panic or abort the host.

### Native path

The native compiler should continue through the existing
`RuntimeExpr::Trap` path. Process entry should report the controlled trap using
the current terminal protocol and nonzero exit behavior, without an uncontrolled
abort. The interpreter/native differential must compare the complete canonical
identity, including the message or structured payload chosen for version 1.

### Payload identity

Version 1 should use a closed, stable reason schema. A minimal form is a
`UserTrap String` constructor mapped to `ExplicitTrap` plus a deterministic
message encoding. If more reasons are required, add closed constructors rather
than exposing arbitrary host diagnostics. The ABI-relevant requirements are:

- fully initialized payloads;
- deterministic encoding;
- identical interpreter and native observations; and
- no inclusion of host panic text, addresses, or platform-specific formatting.

## Terminal means uncatchable

The version-1 operation must not have a public handler, `catch`, or resumption
surface. Its sealed effect identity must also be ineligible for the generic
checked-source handler form. “Terminal” means the executing Ken computation
cannot turn this event back into a value and continue.

This preserves a clean semantic division:

| Mechanism | Meaning | Caller can continue? | Result type |
|---|---|---:|---|
| `Result E A` | anticipated, recoverable domain failure | yes, by matching | explicit `Err` or `Ok` value |
| controlled trap | terminal failure with stable runtime identity | no | no returned `A` |
| divergence | computation does not terminate | no observation | no returned `A` |
| host panic or abort | implementation failure outside Ken semantics | no | forbidden proxy |

Making trap catchable would require deciding handler scope, stack unwinding,
resumption multiplicity, resource-finalizer order, payload typing, and effect-row
subtraction. More importantly, a catch that returns an `A` would reproduce the
same branch-and-recover behavior already expressed honestly by `Result`. Those
semantics are neither needed to close the current gap nor consistent with the
operator's direction.

## Relationship to partial primitives

Ken already records `Total`, `CheckedPartial`, and `TrustedPartial` metadata for
primitives (`crates/ken-elaborator/src/checked_core.rs:1110-1130`). Erasure maps
that metadata to runtime primitive partiality
(`crates/ken-elaborator/src/erasure.rs:2199-2208`), and native lowering can turn
partial primitive failure into `ExplicitTrap`.

That path is useful precedent for runtime identity, but it is the wrong public
producer here:

- the partiality witness is registry metadata on a primitive rather than a
  structural source event;
- a new primitive expands a trusted or specially recognized surface;
- it does not naturally participate in ordinary effect inference; and
- `TrustedPartial` would add an assumption where none is needed.

The ordinary checked `Vis` node gives stronger evidence with a smaller trusted
boundary.

## Resource settlement

PX7-F already specifies trap-primary, cleanup-failure-secondary ordering, but
its public checked-source fixture was honestly deferred because no source trap
producer exists (`docs/program/wp/PX7-F-system-resource-bracket.md:192-200,
308-316`). The catalog package likewise should retain its settled bracket
contract; the new mechanism is a Language/Runtime capability, not a Resource
redesign.

Once the producer lands, a public fixture should place `trap` inside
`withResource` and demonstrate:

1. acquire succeeds;
2. the body reaches the canonical trap event;
3. release or settlement occurs exactly once;
4. trap remains the primary terminal observation;
5. cleanup failures remain ordered secondary evidence; and
6. interpreter and native observations are identical.

No bracket, Ward obligation, carrier, dispatcher, or resource error contract
should be reopened to add this fixture.

## Rejected alternatives

- **Return `Result`.** This is the correct choice for recoverable failure, but
  it does not produce a terminal runtime trap or exercise trap settlement.
- **Catchable exceptions.** They duplicate `Result` for Ken's current need and
  force a much larger control-effect and cleanup contract. Rejected by operator
  direction for version 1.
- **Postulated `Bottom`, `axiom`, or `absurd`.** This claims impossible evidence
  and could manufacture any result type. Checked native correctly rejects the
  current route as `unjustified_impossible_branch`
  (`crates/ken-elaborator/src/checked_core.rs:3047-3049,6114-6121`).
- **Partial pattern matching.** Exhaustiveness rejection is correct and should
  not be weakened to create a runtime path.
- **General recursion or `div`.** Nontermination is not a finite, controlled
  terminal observation and would weaken Ken's totality boundary.
- **A checked/trusted partial primitive.** This puts evidence in registry side
  metadata, grows a special primitive surface, and may add trust unnecessarily.
- **Add `Trap` as a third generic `ITree` constructor.** This gives every
  interaction tree an unindexed trap capability, changes every fold, and is
  broader than an ordinary effect operation.
- **Add a kernel `Term::Trap`.** The effect encoding already expresses the
  behavior without growing the kernel or trusted computing base.
- **Start with a new keyword.** Surface sugar does not solve the evidence or
  lowering problem; it can be considered later if ordinary library spelling is
  ergonomically inadequate.

## Conformance matrix

The design pass should require opposite-observable controls, not only a happy
path:

| Case | Checked-source verdict | Checked evidence | Runtime verdict |
|---|---|---|---|
| `trap reason` under `[Trap]` | accept | exact reachable `Raise` `Vis` | canonical `ExplicitTrap` |
| same body with `Trap` dropped | reject | effect escape witness at `Raise` | no artifact |
| `[Trap]` declared but body only returns | accept as legal headroom | no `Raise` node | ordinary return, never trap |
| near-match operation identity | reject at check or erasure | identity mismatch | no artifact |
| wrong/non-empty response family | reject | response-family mismatch | no artifact |
| `axiom Bottom` plus `absurd` | reject | unjustified impossible branch | no artifact |
| partial match | reject | exhaustiveness error | no artifact |
| `Result.Err` control | accept | returned value, no `Raise` | ordinary returned error |
| checked attempt to handle `Trap` | reject | sealed effect cannot be discharged | no artifact |
| trap inside `withResource` | accept | exact `Raise` under bracket | settle once; trap primary |
| trap plus cleanup failure | accept | exact `Raise` under bracket | trap primary; cleanup secondary |

For every accepted trap case, the interpreter and native observations must
match in trap code and canonical payload. For every rejected case, failure must
be deterministic and fail closed without a compiler or host panic.

## Proposed ownership and next decisions

This report recommends a component shape; it does not assign or launch work.

- **Architect:** rule the component design, exact checked identity, placement in
  the `HostIO` operation coproduct, sealed handler boundary, and payload ABI.
- **Spec enclave:** state the normative source/effect/erasure contract and write
  the discriminating conformance seeds.
- **Language:** after a framed kickoff, implement checked source construction,
  effect inference, and identity-checked erasure.
- **Runtime:** connect the canonical identity to interpreter settlement and the
  existing native trap path.
- **Foundation/Resource:** add only the previously deferred public fixture after
  the mechanism is available; do not reopen PX7 semantics.

The remaining design choices are narrow:

1. the public library name and exact surface spelling;
2. how `TrapOp` is nested into the landed operation coproduct;
3. the sealed stable identity and handler-ineligibility rule used by erasure
   and the interpreter;
4. the version-1 closed reason schema and canonical encoding; and
5. the exact settlement hook shared by interpreter and native execution.

The central semantic decision is no longer open: checked-source trap is a
terminal, uncatchable effect event. Recoverable programs use `Result`.

## Sources

Repository sources are cited inline against the grounding commit. Public
literature consulted:

- Sam Lindley, Conor McBride, and Craig McLaughlin,
  [*Do be do be do*][frank], 2017.
- Li-yao Xia et al.,
  [*Interaction Trees: Representing Recursive and Impure Programs in
  Coq*][itrees],
  POPL 2020.
- Daan Leijen,
  [*The Koka Programming Language*, effect typing][koka],
  accessed 2026-07-17.

[frank]: https://arxiv.org/abs/1611.09259
[itrees]: https://arxiv.org/abs/1906.00046
[koka]: https://koka-lang.github.io/koka/doc/book.html
