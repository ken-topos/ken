# `RT-SCALE-B` D4 analytical emission model

**Bound source:** `38e054d47c3f9a01e1adf844b4a632e60087a4c4`

**Frame:** `docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md`,
blob `d62bab7587121b3ff6c7427aec7bf619f0675977`

**Research dispatch:** `evt_62fqpe7pfvym4`; advisory
`evt_5njmxwxyadqkk`

## Result

For the frame's strictly nested, well-bracketed, single-shot LIFO resource
family, the research-supported target is **affine emitted material**. Frontend
work is O(n) with expected-constant-time interning; ordered interning admits an
O(n log n) implementation upper bound. If that logarithmic factor governs the
measurements, it is residual implementation super-linearity, not a semantic
necessity. No super-linearity is inherent to this family unless measurement
first demonstrates a genuinely super-linear semantic-state population.

For the historical one-point observation, the analytical side therefore
selects **bad constants on an O(n) representation mechanism**: there is no
evidence in that datum for the residual-super-linearity branch. This is a model
classification, not proof that the current implementation is affine and not
the node's closing empirical verdict. If the completed n=3..7 table finds
super-linear material, wall time, or RSS, that finding is a further
implementation gap under the decision table below. It does not turn into an
inherent lower bound.

On the bound source, the governed family now selects
`BodyEmissionAuthority::FunctionizedUnits` and reaches completed unit emission.
The earlier emission-port blocker is closed. Runtime's D1--D3 measurements can
therefore corroborate or falsify the implementation-order prediction without
changing the analytical population.

The historic n=4 figures, including about 103 seconds, about 4 GiB, and
1,482/1,525 states/edges, are **NON-COMPARABLE**. There is no apples-to-apples
baseline. The eventual n=3..7 table must report absolute values.

This model is scoped to growth of the reached emission population. It does not
claim that the representation is complete or verified while `RT-EFFECT-DIFF`
remains open, and it does not erase the recorded pre-merge emission-port
failures. D4 supplies the analytical half; Runtime's independent D1--D3 table
and the operator-shaped D5 verdict remain separate deliverables.

## Research basis

The model uses the research dispatch rather than fitting the future five-row
table. Its relevant foundations are:

- Danvy and Nielsen's
  [defunctionalization](https://doi.org/10.7146/brics.v8i23.21684) makes a
  finite higher-order function space into first-order constructors plus an
  apply operation. Static continuation identity need not contain a dynamic
  continuation suffix.
- Maurer, Downen, Ariola, and Peyton Jones's
  [join points](https://simon.peytonjones.org/assets/pdfs/compiling-without-continuations.pdf)
  preserve shared continuations instead of duplicating their bodies at each
  branch.
- Tarjan's
  [linear graph algorithms](https://doi.org/10.1137/0201010) give the standard
  O(|V| + |E|) worklist/traversal bound once each distinct state and transition
  is represented once.
- Xie et al.'s
  [evidence translation](https://www.dhil.net/research/papers/effect_handlers_evidently-extended-icfp2020.pdf)
  shows that scoped effect-handler information can travel as explicit evidence
  rather than requiring a search through or copy of the dynamic context.
- Shao and Appel's
  [safe-for-space closure conversion](https://doi.org/10.1145/345099.345125)
  establishes that closure conversion can preserve asymptotic space through
  shared environments; a flattened copy of every live environment is not
  forced by closure conversion.

These results do not prove this implementation linear. They establish that a
strict LIFO bracket chain supplies no semantic lower bound requiring
super-linear output.

## Variables and predicted order

For bracket depth `n`, define:

- `P(n)`: checked source occurrences;
- `V(n)` and `E(n)`: distinct semantic states and transitions;
- `U(n)`: emitted function units;
- `S(n)`: persistent-store nodes;
- `A(n)`: total explicit interface material, including all frame slots,
  descriptors, call-edge records, and other per-unit metadata;
- `I(n)` and `B(n)`: emitted CLIF instructions and bytes; and
- `k(n)`: maximum production `plan_expr` recursive lowering frames.

For a syntax-generated family, `P(n) = p0 + p1*n`. Strict LIFO nesting exposes
prefixes of one stack, not independent subsets of `n` resources. With constant
branching, no concurrency, no multi-shot continuation product,
future-equivalent states interned together, constant work per transition, and
interfaces carrying IDs or shared suffix descriptors rather than flattened
prefixes:

```text
V(n), E(n), U(n), S(n), A(n), I(n), B(n) = Theta(n)
```

The natural emitted-size model is:

```text
C(n) = Theta(P(n) + V(n) + E(n) + S(n) + A(n) + I(n))
```

A worklist with expected-constant-time interning is O(C(n)). The landed planner
uses ordered maps at some interning seams, so O(C(n) log C(n)) is an admissible
implementation bound. That logarithmic lookup factor must not be relabelled
quadratic. Conversely, any full suffix serialized, hashed, or compared at
every depth; a flat all-live frame copied at every depth; cloned whole ledgers;
or a whole-body analysis repeated for every state produces a real Theta(n^2)
mechanism despite affine `V` and `E`.

Compile time is modelled as:

```text
T(n) = t0
     + tP*P(n) + tV*V(n) + tE*E(n)
     + tS*S(n) + tA*A(n) + backend(U(n), I(n), B(n))
```

The model does not assume Cranelift is a formally linear-time backend. If every
material count is affine but wall time or RSS is not, the residual lies in
repeated frontend work, per-function backend behaviour, allocation, or module
finalization. It is not evidence of an inherent semantic state product.

Peak memory is modelled as:

```text
R(n) = r0 + retained_module(C(n)) + max_unit_scratch(n)
```

The emitted module may retain affine IR/object material, so constant RSS is not
required. With affine `C(n)` and bounded per-unit scratch, the predicted peak is
O(n). Super-linear RSS with affine material counts identifies flattened
interfaces, duplicated retained bodies, allocator retention, or backend/module
scratch rather than a necessary property of the bracket semantics.

## Recursive lowering stack axis

RT-SCALE-A measured in `evt_j5d4tn3xyjmt`:

```text
n       3   4   5   6   7
k(n)   14  18  22  26  30
```

Thus `k(n) = 4n + 2` on the measured family. The stack-depth axis carries
evidence and is affine. It is not constant, and no constant-depth requirement
is sound. These values do not supply a per-frame byte cost or prove that every
other compiler stack is bounded by `k`; they only discharge the named
production recursive-lowering-frame input to this model.

At n=4 the measured value is `k(4) = 18`. If each production frame has bounded
size `s`, the named stack contribution is `s(4n + 2) = Theta(n)`. The
instrument does not measure `s`, so the model does not invent a per-frame byte
figure. In particular, the historic approximately 4 GiB process RSS cannot be
attributed to a super-linear `plan_expr` stack from this evidence.

## Four structural invariants

1. **No flattened environment, pending stack, or path member in helper
   identity -- structurally satisfied.** `PlannedHelperKey` is only
   `(TransitionKind, StaticNodeId)` or `(EdgeKind, StaticEdgeId)`. Dynamic
   activation travels separately.
2. **Constant ID and node payload width -- structurally satisfied at the
   schema boundary.** Static and persistent IDs are dense `u32` values.
   `DynamicActivationFrame` has eight persistent IDs, and
   `PersistentStoreNode` has one kind, two `u32` payloads, and one child ID.
   Per-unit ABI frames may contain different numbers of slots; therefore total
   `A(n)`, not one slot's width, must still be measured.
3. **Affine total persistent nodes -- satisfied by RT-SCALE-A's planner
   census; completed-emission corroboration is now measurable and belongs to
   Runtime's table.** This is not permission to substitute Boundary A's numbers
   into Boundary B.
4. **At most affine logical chain depth -- satisfied by the landed planner
   census; completed-emission corroboration is now measurable and belongs to
   Runtime's table.** Logical chain depth may grow Theta(n), and that is sound
   because a helper/frame carries one constant-width persistent ID, not an
   inline copy of the chain. **Constant logical-chain depth is not required.**

The first two invariants rule out width growth in identity and node schemas.
The latter two rule out a quadratic total representation while allowing an
affine chain. All four are necessary discriminators; none alone proves
end-to-end linear time.

## Production Cranelift denominator

When the benchmark selects `FunctionizedUnits`, the measured native module must
include every production Cranelift body defined by that compilation:

| Population | Included | Reason |
|---|---:|---|
| `lowering/units.rs` root adapter | yes | It is the public root body for the selected unit authority. |
| `lowering/units.rs` unit bodies | yes | One body is defined for every validated emittable unit. |
| `native_int_clif.rs` local helper graph | yes | `compile_expr_into_module` emits it unconditionally into the same module. |
| `boundary_value_clif.rs` local helper graph | yes | It is emitted unconditionally into the same module and is live from unit lowering. |
| `lowering/core.rs` recursive-descent root body | no | It is the mutually exclusive retained authority, not a body emitted by a functionized compile. Mixing it would change the denominator. |
| imported `ken_host_dispatch_v1`, `malloc`, and `free` | no | They are declarations with no Cranelift body defined in this module. |
| test-only probes and capture functions | no | They are instrumentation, not production emitter output. |
| generated C starters and object-linker stubs | no | They are production artifacts, but not Cranelift emitters or CLIF population. |

The table must count the root adapter, every unit body, seven native-Int
helpers, and twenty-nine boundary helpers in its total
DFG/instruction/block/byte population. Thus the production-function count is
`emitted_helpers + 37`. It may report the fixed helper graphs as a separate
intercept, but may not silently omit them. This is this node's denominator;
B2F AC-G0's native-Int-only count is not a substitute.

## The operator's binary

The analytical side rules out an inherent super-linear lower bound for this
strict LIFO family and predicts a linearly sized completed representation.
Apply this decision table to Runtime's independent measurements:

| Observation | Classification |
|---|---|
| `V`, `E`, or `U` is super-linear | Identity/state-partition gap, unless a new independent semantic product is proved. |
| `V`, `E`, and `U` are affine, but `S`, `A`, `I`, `B`, or descriptor work is super-linear | Representation/materialization gap. |
| All material counts are affine, but wall time or RSS is super-linear | Repeated analysis, allocation, module-finalization, or backend gap. |
| Structural counts, wall time, and RSS are affine with a large intercept/slope | Bad constants on an O(n) mechanism. |

For the historical observation, the model's answer to the operator's binary is
the last row: **bad constants on an O(n) representation mechanism**. The
approximately 103 seconds and 4 GiB at n=4 are one **NON-COMPARABLE** point and
cannot establish an exponent; they are not evidence for residual
super-linearity. The current table remains free to falsify implementation
linearity. If it does, the applicable earlier row names the mechanism gap and
the ring must not relabel it as acceptable constants. That is outcome (b): it
routes to the operator through the Steward and this node does not close.

For the last row, the constants plan is to attribute the fixed local-helper
intercept separately, rank per-unit instruction and frame-slot slopes, remove
repeated descriptor comparisons or whole-module passes, and profile backend
and allocator peaks before changing semantics. That plan is conditional on the
complete table.

For any super-linear row, the ring has found a further mechanism gap. It must
not call that growth acceptable constants. Outcome (b), if research later
establishes a genuine semantic lower bound, requires the operator's explicit
ceiling and cannot be closed by the ring.

Finally, no exponent is inferable from five points. `370n`, `93n^2`, and a
product that switches on at n=5 can all pass through the same historic n=4
datum. The structural invariants discriminate; first and second finite
differences only corroborate them.
