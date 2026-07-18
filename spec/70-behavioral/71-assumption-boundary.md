# The assumption-boundary export IR

> Status: **impl-ready** (B1). Normative for the **projection, schema, and
> seam discipline** of Ken's behavioral export; the literal wire **spellings**
> of the five fields are `(oracle)`-tagged and finalized with the sibling
> (`Ward`), but the concept, value-set, per-entry status, cross-field
> invariants, and content-hash stability discipline are **locked here**.
> **`OQ-export-ir` DECIDED** (operator, 2026-06-27): the export is an
> **assume-guarantee contract**, generated (never hand-authored) from verified
> content; **Ken-native for the propositional contract, ITF for traces**;
> generators carry **support structure only — never a sampling measure**. ADR
> 0006 (the seam); this is its first concrete deliverable.
>
> **B1 scope.** B1 fixes the **projection** (§2.1), the **two-layer
> serialization + invariants** (§3), **content-addressing + provenance** (§3.3),
> the **sealed no-measure `G`** (§4.1), and the **one-way gate** (§5.1) — in
> `crates/ken-elaborator`. The `Temporal` **datatype** and the `compile`
> faithfulness lemma are **B2/B3** (B1 fixes the `T` *channel* and the `Σ`
> *alphabet* they reuse, §5.2). The emitter **projects already-verified content
> and proves nothing new — no kernel enlargement** (the denotation `Σ` reads
> from is admitted as of K1.5, `f037451`); the kernel does **not** re-check the
> serialization, so **conformance is the net** for projection fidelity (§6).

## 1. What the export is

When Ken finishes checking a program it emits a stable artifact — the
**behavioral export** — the **rely/guarantee contract** of the verification
boundary: a faithful statement of *what Ken guaranteed* (`Q`, proved), *what Ken
relied on* (`P`, assumed), and *what Ken stated but delegated* (`T`). It is not
only the residual Ken could not close — the guaranteed half `Q` is exported too,
precisely so a downstream engine may **assume** rather than re-prove it. Every
field carries its **epistemic status** (`../20-verification/21 §5.2`), and
the whole projects verified content, so the contract states **exactly**
what Ken established — no more (§2.1).

The export is a **broadcast contract**, not a point-to-point channel: it is read
by a *family* of downstream consumers — a **static verifier** (model-check `T`
against `Q`/`P`), a **test generator** (`G` + the sampling policy), and a
**runtime conformance monitor** (`73`) — each applying its own policy to many of
the same outputs. `Ward` (ADR 0006) is the umbrella for these engines; the
runtime monitor is plausibly a *distinct* engine (a sidecar), which sharpens the
"two engines" framing to **Ken + a family of behavioral engines sharing one
export and one logic**.

Two properties make it trustworthy:

- **It is a contract, not a dump** — an **assume-guarantee** record (§2). The
  shape is the permanent framing of a verification boundary (rely/guarantee),
  not an ad-hoc payload.
- **It is *generated*, never hand-authored.** Every field is a projection of
  Ken's verified content — proved `Q`, the residual `P` from
  `trusted_base_delta` (`../20-verification/25 §3`), the effect alphabet from
  the interaction-tree denotation (`OQ-8`, `../30-surface/36 §2`), the
  `Temporal` values written in source (`72`), and the structured resource-
  lifetime template derived from a reachable resource acquisition (§2.2–§2.3).
  It therefore **cannot overclaim**: it states exactly the four-way epistemic
  status (`../20-verification/21 §5`), with no room to assert more than Ken
  proved. This is the structural antidote to model↔code drift — the model is a
  *function of the code* (`73`).

## 2. The schema — an assume-guarantee contract

The export has five parts. The first four are **Ken-native** (faithful to Ken's
own terms — they are what `Ward` *reasons about*, and "one logic, two engines"
requires their meaning be identical on both sides); concrete execution witnesses
are a separate **ITF** layer (§3).

| Part | Carries | Status | Downstream use |
|---|---|---|---|
| **`guarantees` (Q)** | proved postconditions & per-space invariants | `proved` | invariants the model may *assume*, not re-prove → smaller state space |
| **`assumptions` (P)** | the assumption boundary: `trusted_base_delta`, explicit `assume`s, boundary labels | `tested` | the nondeterministic *environment*; the generator's input domain |
| **`alphabet` (Σ)** | the interaction-tree perform-node signatures (`OQ-8`) | — | the behavioral state machine's **event alphabet**; the monitor's alphabet |
| **`obligations` (T)** | `Temporal` data (`72`) and structured correlated resource lifetimes (§2.2–§2.3) | `delegated` | behavioral properties to model-check and monitor |
| **`generators` (G)** | refinement/dependent-type **support structure** (§4) | derived | the *territory map* for spec-driven test generation |

`Σ` is **reuse, not reinvention**: the event vocabulary `Ward` monitors over
*is* the interaction tree's perform-node signatures. `Ward` watches exactly the
events Ken's denotation can emit; there is no separate alphabet to define or
keep in sync.

### 2.1 The projection — verified status to export field

The export is **generated**: each field is the image of a total projection from
already-verified content, never a hand-authored payload. Two things must be
pinned for the emitter to be implementable and for the contract to be
trustworthy: **where each verification status lands**, and **what discriminates
a guarantee from an assumption**. Both are settled here so the conformance
author inherits no silence to fill.

**Status → field (the classification boundary, pinned).** A claim's epistemic
status (`21 §5.2`: `proved` / `tested` / `delegated` / `unknown`) determines its
field. The verdict trichotomy (`21 §5.1`: `proved` / `disproved` / `unknown`)
relates to it by the projection of `21 §5.3`. The export map is total over
*exportable* claims and explicit about the one verdict that does **not** export
(a refuted claim):

| Source claim | Epistemic status | Export field | Entry status |
|---|---|---|---|
| proved postcondition / space-invariant | `proved` | **`Q`** | `proved` |
| explicit `assume`/`test` clause, boundary label | `tested` | **`P`** | `tested` |
| open typed hole (postulate of the goal) | `unknown` | **`P`** | `unknown` |
| `Temporal` property stated in source | `delegated` | **`T`** | `delegated` |
| generated correlated resource-lifetime template (§2.2–§2.3) | `delegated` | **`T`** | `delegated` |
| refuted claim (countermodel) | *(none, `21 §5.3`)* | **never exported** | — |

- **`Q` and `P` partition the propositional claims; `unknown` rides `P`, never
  `Q`.** An open hole is a *postulate* of its goal (`24 §2`) — an honest
  assumption the downstream must treat as environment, exactly like an explicit
  `assume`. It therefore lands in `P` (tagged `unknown`), beside the `tested`
  entries. A shippable artifact has an **empty `trusted_base_delta`** (`25 §3`,
  the honesty guard) — i.e. **no** `unknown` entries — or an explicit recorded
  acceptance of the listed ones; either way they are never silently promoted.
- **A `disproved` claim never exports.** A refuted claim is a verification error
  to *fix*, not a guarantee to ship (`21 §5.3`, `24 §3`); it has no epistemic
  status and no export field. The emitter that finds a `disproved` verdict emits
  **nothing** for that claim (the build does not produce a shippable export).
- **`Σ` and `G` carry no epistemic status.** `Σ` is a *vocabulary* (the
  perform-node signatures) and `G` is a *partition* — neither is a claim, so
  neither is tagged `proved`/`tested`/etc.

**The discriminator is kernel-side, not a self-reported label (`21 §5.4`).**
What puts a claim in `Q` rather than `P` is **structural**, decided from the
kernel's own state: a claim is a **guarantee iff** its certificate `check`s
**and** its goal is **not** a postulate in `GlobalEnv::trusted_base()`
(`18 §4`/`§5`); otherwise it is an **assumption**. The emitter never trusts the
(untrusted) verification layer's status string — it reads `trusted_base()`
membership + certificate presence. This is the load-bearing **no-over-claim**
invariant (AC2): the *same* proposition emits under `Q` when proved and under
`P`/`T` when its proof is a hole or a delegation — the field **flips** with
the kernel state, a structural signal, not a green-vs-green string compare.

**Per-field source of truth and projection.** Each field names the landed
artifact it reads and the function that projects it (no field invents content):

- **`Q` (guarantees)** ← claims with status `proved`. Read the discharged
  `ensures`/space-invariant obligations (verdict `proved`, `25 §3`); emit each
  proposition. The kernel already checked the certificate — the emitter
  **reads** the verdict, it does **not** re-prove (no kernel work).
- **`P` (assumptions)** ← assumption boundary: `trusted_base_delta` (`25 §3`,
  the postulates/holes this target adds) ∪ explicit `assume`/`test` clauses ∪
  boundary labels (FFI / untrusted-input / IFC labels, `../60-security/61`).
  Each entry tagged `tested` or `unknown` per the table. Boundary-`Q`/`P`
  producers such as Sec1ct's CT-in-parameter promise (`../60-security/61 §5a.4`)
  feed this channel — coordinate the boundary obligation's shape via spec, do
  **not** pre-bind field names across WPs (§3.1).
- **`Σ` (alphabet)** ← interaction-tree perform-node signatures, **verbatim**
  (`OQ-8`, `../30-surface/36 §2`: the `Effect` container `Op`/`Resp`, the `Vis`
  nodes; admitted as of K1.5, `f037451`). Emit exactly the signatures the
  program's denotation can perform — **not** a re-derived alphabet (AC4 asserts
  structural equality to the denotation's signatures). No second alphabet.
- **`T` (obligations)** ← claims with status `delegated`: the `Temporal` data
  values stated in source (`72`, `OQ-temporal`) plus the generated direct
  resource-lifetime template of §2.2–§2.3 when reachable `Σ` contains a
  resource acquisition.
  Emit each as an obligation tagged `delegated`. B1 structures **the channel**
  (the values + their status + the `Σ` they range over); the `Temporal`
  **datatype** and the `compile` faithfulness lemma are **B2/B3** (§5.2) — emit
  what exists. The direct structured resource body is a correlation-only
  schema and does not change the existing `Temporal` body.
- **`G` (generators)** ← refinement predicates `{x:A | φ}` and `match` arms in
  the checked program. Project the equivalence-class **partition**, the
  **boundaries**, and the **case decomposition** (§4) — **support only**, no
  measure (the sealed type, §4.1).

### 2.2 Correlated resource-lifetime obligations

The ordinary `Temporal` body cannot express the identity correlation required
by a resource lifetime. In particular, two independent
`Pred::Event("FsOpen")` and `Pred::Event("ResourceRelease")` atoms do not say
that the released resource is the one that was acquired. For this correlation
case, the `T` channel admits one direct `ResourceLifetimeObligation` body. It is
one obligation template over all resource lifetimes in the target, not one
unrelated obligation per operation and not one static entry per dynamically
minted resource.

The target-level obligation serializes the correlation policy, never a future
runtime witness. At runtime a successful acquisition binds the event's
`ResourceTraceIdentityV1` as `r`; each listed use and settlement must carry the
same `r` at the operation-defined role in `EffectEvent.resource_bindings`.
`ResourceTraceIdentityV1` is the host-ABI identity vocabulary retained by this
contract. An fd, resource-table slot or generation, pointer, inode, or executor
identity is neither a valid key nor a permitted fallback.

Existing `TEntry { obligation_id, formula: Temporal }` values,
`TemporalObligation`, their serialization, and their hash contribution are
unchanged. The structured body is used only when identity correlation across
acquire/use/settle is required; every other temporal obligation continues to use
the existing path.

### 2.3 Role-labelled multi-resource obligations

A positioned file transfer observes two live resources in one operation: the
file handle and the buffer. Electing either identity as the event's sole
resource would make the other lifetime uncorrelatable. Encoding either token in
opaque request bytes would make the contract depend on a private transport
representation. The sole `EffectEvent` therefore carries this field:

```text
resource_bindings:
  [(ResourceBindingRole, ResourceTraceIdentityV1)]

ResourceBindingRole = File | Buffer | Target
```

The sequence is ordered, not a map. Its order and roles are canonical for each
operation:

| Operation | Canonical `resource_bindings` |
|---|---|
| successful `FsOpen` | `[(Target, file)]` |
| successful `BufferAllocate` | `[(Target, buffer)]` |
| `FsHandleMetadata` | `[(Target, file)]` |
| `FsReadAt` | `[(File, file), (Buffer, buffer)]` |
| `FsWriteAt` | `[(File, file), (Buffer, buffer)]` |
| `BufferFreeze` | `[(Target, buffer)]` |
| `ResourceRelease` | `[(Target, released)]` |

Here `file`, `buffer`, and `released` stand for runtime-bound
`ResourceTraceIdentityV1` values, not target-level serialized witnesses. The
following record is the exact full-inventory specialization for a target whose
reachable `Σ` contains both acquisitions and every listed use:

```text
ResourceLifetimeObligation {
  body_kind: ResourceLifetimeObligation,
  obligation_id: String,
  status: delegated,
  correlation: ResourceLifetimeCorrelation {
    identity_type: ResourceTraceIdentityV1,
    event_field: EffectEvent.resource_bindings,
    role_type: ResourceBindingRole,
    canonical_order: OperationDefined,
  },
  plans: [
    ResourceLifetimePlan {
      resource_kind: FsHandle,
      bind_at: Successful(FsOpen, Target),
      require_same_at: [
        (FsHandleMetadata, Target),
        (FsReadAt, File),
        (FsWriteAt, File),
        (ResourceRelease, Target),
      ],
    },
    ResourceLifetimePlan {
      resource_kind: Buffer,
      bind_at: Successful(BufferAllocate, Target),
      require_same_at: [
        (FsReadAt, Buffer),
        (FsWriteAt, Buffer),
        (BufferFreeze, Target),
        (ResourceRelease, Target),
      ],
    },
  ],
  monitor_template: WardResourceLifetimeMonitor {
    correlate_every_role_binding: true,
    successful_acquire_settles_exactly_once: true,
    forbid_successful_use_after_settlement: true,
    require_no_live_bracket_owned_identity_on:
      [NormalReturn, ReturnedError, ControlledTrap],
    retain_settlement_outcome: true,
  },
}
```

There is no schema-version field, versioned body kind, version-dispatch arm, or
compatibility wrapper. `ResourceLifetimeObligation`,
`ResourceLifetimeCorrelation`, `ResourceLifetimePlan`, and
`ResourceLifetimeBindingPoint` are the only resource-lifetime export shapes.

The global per-kind `require_same_at` inventories are the two ordered lists in
that record:

```text
FsHandle:
  [(FsHandleMetadata, Target), (FsReadAt, File),
   (FsWriteAt, File), (ResourceRelease, Target)]
Buffer:
  [(FsReadAt, Buffer), (FsWriteAt, Buffer),
   (BufferFreeze, Target), (ResourceRelease, Target)]
```

The emitted `plans` are target-specialized to the exact reachable `Σ`:

1. include the `FsHandle` plan if and only if `FsOpen ∈ Σ`;
2. include the `Buffer` plan if and only if `BufferAllocate ∈ Σ`;
3. order included plans `FsHandle`, then `Buffer`; and
4. for each included plan, set `require_same_at` to the canonical ordered
   subsequence of that kind's global inventory whose operation is in `Σ`.

No plan may name an acquisition or use absent from `Σ`; within an entry, no
acquisition present in `Σ` may lack its plan. A full-inventory two-resource
target therefore emits the complete two-plan record above. A buffer-only target
emits only the Buffer plan and only its reachable uses. A read-only positioned
target emits both plans, retains `FsReadAt` at its `File` and `Buffer` sites,
and omits unreachable write, metadata, or freeze sites while preserving the
remaining global order.

The monitor template consumes the sole `EffectObservation`: its events are
`EffectEvent` values and its terminal observation is the closed
`TerminalExitClass = NormalReturn | ReturnedError | ControlledTrap`. On
successful acquisition, the external Ward project binds the identity at the
plan's `bind_at` role. Every listed use and settlement must carry that same
identity at the listed role. Thus, when present, `FsReadAt` and `FsWriteAt` are
checked once against each plan: their `File` binding equals the open file
identity, and their `Buffer` binding equals the allocated buffer identity.

The template requires exactly one settlement for each identity, forbids
successful use after settlement, retains each settlement outcome, and requires
that no identity owned by either bracket remains live at normal return,
returned error, or controlled trap. `TerminalExitClass` is computed before exit
normalization and is an observation, never a Ward verdict. External kill,
abort, fatal signal, and machine failure remain outside the obligation.

Ken emits the template with status `delegated`; it neither runs the monitor nor
ingests its verdict. The monitor and any resulting attestation remain external
Ward behavior, and no such result becomes a Ken proof (§5.1).

The emitter produces exactly one direct `ResourceLifetimeObligation` when the
target's reachable alphabet contains `FsOpen` or `BufferAllocate`, and none when
it contains neither acquisition. The complete static entry, including the
correlation descriptor, target-specialized ordered plans, status, and monitor
template, is canonicalized in `T` and covered by the export hash. Runtime-bound
file and buffer identities are never serialized in the target export.

Static-policy validation and runtime-observation validation are separate
phases. A wrong body kind, status, correlation field, plan set, plan order,
`require_same_at` subsequence, monitor field, or an operation absent from `Σ`
makes the **static descriptor** malformed. The emitter rejects that descriptor
before publishing `T` or an export hash. By contrast, a missing, duplicate,
extra, misordered, wrongly labelled, wrongly correlated, or split
`resource_bindings` sequence is a malformed **runtime observation**. It is
rejected when the observation is validated and when external Ward checks the
already exported policy. That rejection neither changes nor re-emits the
canonical static `T` bytes or their export hash.

The schema collapse intentionally rebaselines each resource-producing export
once. The checked file-only `px7f_export_resource` fixture emits the direct
single-plan body and hash `ken-export-v0:1bf3cb3f5b648ea7`; the checked
buffer-only fixture emits the direct Buffer plan and hash
`ken-export-v0:47f2f35b7a825ca3`. Independently, the checked no-acquire fixture
retains frozen hash `ken-export-v0:6360c2cb74f78f7e`, because it contains no
resource-lifetime body. That unchanged no-acquire export is a negative control,
not a compatibility promise.

The locked invariants are:

- **RL1 — one canonical correlation binder.** The descriptor names
  `EffectEvent.resource_bindings`, `ResourceTraceIdentityV1`,
  `ResourceBindingRole`, and operation-defined order. Removing or altering a
  descriptor field, serializing a runtime `r`, or supplying independent event
  atoms is malformed.
- **RL2 — exact target-specialized plans.** The plan set is selected by
  acquisitions in reachable `Σ`; each `require_same_at` is the ordered reachable
  subsequence of its global inventory. An absent acquisition, missing plan, or
  extra unreachable operation is a pre-export descriptor error.
- **RL3 — delegated only.** Every resource-lifetime entry has status
  `delegated`, never `proved`, `tested`, or `unknown` (I4).
- **RL4 — content-bound.** The complete entry is canonicalized with `T` and is
  covered by the export hash; no field may be omitted from that input.
- **RL5 — correlation-only supersession.** Existing `Temporal` entries retain
  their exact body and behavior. The structured body is required only for the
  resource-correlation case and cannot be represented by two independent event
  atoms.
- **RL6 — role-labelled runtime correlation.** Every lifecycle observation has
  exactly its operation-defined ordered binding sequence. A request-byte
  token, an unlabelled identity list, a single elected identity, or any malformed
  runtime sequence fails observation/external-Ward validation without changing
  static `T` or its hash.
- **RL7 — sole direct schema.** A resource-producing target emits one direct
  `ResourceLifetimeObligation`; no schema version, versioned body, parallel
  entry, alias, wrapper, or conversion view exists.
- **RL8 — no-acquire negative control.** A target with neither acquisition emits
  no resource-lifetime obligation, so the collapse does not change its bytes or
  hash.

## 3. Two layers: Ken-native contract, ITF traces

- **Propositional/contract layer → Ken-native.** `Q`, `P`, `Σ`, `T`, and `G`'s
  predicates are the objects `Ward` reasons about; they must be semantically
  faithful to Ken's terms. A lossy translation here would break the single-logic
  guarantee.
- **Trace layer → ITF** (Apalache/Quint's *Informal Trace Format*). Concrete
  execution and counterexample witnesses are the cheap interop currency, and
  `Ward`'s downstream tools already speak ITF. Adopting it buys immediate
  Quint/Apalache/MOP interop with no bespoke format to maintain. The trace layer
  carries **witnesses, not claims** — it has no epistemic status and feeds no
  `Q`; a green trace is evidence for a `delegated` `T`, never a promotion of it
  (§5.1).

### 3.1 The contract serialization — locked structure, deferred spelling

The contract layer is **Ken-native** and serialized as the five fields of §2.
What is **locked** (normative, conformance-checkable) versus **deferred** to the
`Ward` wire pass is pinned explicitly, per *defer-spelling-not-concept* — the
risk being a contract that either over-freezes a token Ward must finalize or
leaves an invariant unstated for the conformance author to fill differently.

**Locked (normative):**

- The **five-part structure** `{ Q, P, Σ, T, G }` and each field's **value-set**
  — `Q`/`P`/`T` are sets of status-tagged propositions/obligations; `Σ` is a set
  of perform-node signatures; `G` is a support structure (§4.1).
- The **per-entry status tag** (`proved`/`tested`/`unknown`/`delegated`,
  §2.1) — every `Q`/`P`/`T` entry carries exactly one, from the projection.
- The exact structured resource-body field sets, nesting, and order in
  §2.2–§2.3. Every nested descriptor, plan, operation, and monitor field is a
  canonical hash input.
- The **cross-field invariants** (below), which any conforming serialization
  must satisfy regardless of token spelling.
- The **content-hash stability discipline** (§3.3): the export is
  content-addressed, so a **rename after the spelling binds is a breaking
  change** (a new hash, a new contract version) — the concept is stable across
  the rename even though the literal key is not yet frozen.

**Deferred (`(oracle)`-tagged):** the **literal serialized keys** for the five
fields and the per-entry tag (e.g. whether the wire token is `guarantees` /
`guarantee` / `Q`). `Ward` finalizes the wire token against its parsers; Ken
fixes the concept and the stability discipline. Conformance pins the value-set
and invariants and `(oracle)`-tags the literal key (`conformance-assert-at-
locked-granularity`).

**Cross-field invariants (the consistency net — conformance asserts each):**

- **I1 — no over-claim (honesty).** Every `Q` entry traces to a `proved` verdict
  whose goal is **absent** from `trusted_base()`; **no** `Q` entry's goal is a
  postulate. Equivalently: nothing in `Q` carries status `tested`/`unknown`/
  `delegated`. (AC2; the §2.1 discriminator.)
- **I2 — assumption visibility.** Every postulate in this target's
  `trusted_base_delta` (`25 §3`) appears as a `P` entry; removing an `assume` or
  shrinking the delta **removes** the matching `P` entry (and changes the hash,
  §3.3). (AC3.)
- **I3 — alphabet closure.** Every event symbol mentioned by a `T` obligation,
  by `G`, or by the monitor contract (`73 §2`) is a member of `Σ`; and `Σ`
  contains **exactly** the denotation's perform-nodes — no orphan symbol, no
  missing node. (AC4.)
- **I4 — delegated never proved.** Every `T` entry carries status `delegated`;
  no projection path stamps a `T` (or `P`) entry `proved` (§5.1, the one-way
  gate). (AC6.)
- **I5 — no measure.** No `G` entry carries a weight / likelihood / probability;
  `G` is partition + boundaries + case-decomposition only — enforced at the type
  level (§4.1). (AC5.)

### 3.2 The trace layer — ITF

Concrete execution and counterexample **witnesses** serialize as ITF
(Apalache/Quint's *Informal Trace Format*), a separate layer from the contract.
This is interop currency, not part of the propositional contract: ITF traces are
read by `Ward`'s downstream tools (Quint/Apalache/MOP) with no bespoke format to
maintain, and — per §3 above — carry no epistemic status.

### 3.3 Content-addressing and provenance

The export is **versioned and content-addressed**. The hash is taken over a
**canonical form** of the contract (deterministic field and entry order, a
normalized form of each proposition) so that the **same checked program
yields the same hash** (AC1) — the projection is a deterministic function of the
verified content, and canonicalization removes serialization-order freedom.

The hash **travels in provenance** (`../60-security/63 §2`): it links *this
model* to *this build*, which is what makes "this `Ward` model corresponds to
this code" checkable rather than asserted — the hook trace-conformance (`73`)
builds on. The hash is what the **discharge attestation** binds to: "the Ken
export answered" is recorded as the content-hash of the `Q`/`P`/`Σ`/`T`/`G`
contract (`../60-security/63 §5a`), so `export_hash ↔ build` (provenance) and
`export_hash ↔ discharge` (attestation) are both reproducible — a delegated
obligation's classical discharge is bound 1:1 to exactly the export that stated
it.

## 4. Generators carry support, never measure

A refinement type `{x:A | φ}` is a **generator and an oracle** — but generating
*representative* tests means sampling over the combinatorics of state
equivalence classes, and that decomposes into two parts of very different
epistemic status:

- **Support — Ken owns it, faithfully.** *Which* states are valid, and the
  **structure** of that space: the equivalence-class **partition**, the
  **boundaries** between classes, and the **case decomposition** all fall out of
  refinement predicates and `match` arms (equivalence-partitioning and
  boundary-value analysis are *derivable*). Ken exports this partition as `G` —
  an honest map of the territory — claiming nothing about likelihood.
- **Measure — Ken never supplies it.** *Which* valid states are likely /
  important / risky / cheap / judged-out-of-scope. This is business logic, risk
  weighting, operational/UI exclusions, and the empirical data population in
  running systems — **human and environmental judgment**, not a derivable fact.
  A distribution fitted to production traffic is the opposite of a static proof,
  and it is **per-deployment** (the same component needs a different measure as
  an internal API vs. an external endpoint). It therefore lives **outside Ken
  source** — in the same class of deployment-adjacent artifacts as a
  `Dockerfile` or Terraform — as a separately-authored **sampling policy**
  (`OQ-sampling-policy`, hosted on `Ward`'s side, governed like the security
  policy of `../60-security/65`). Ken's exported partition is the **vocabulary**
  that policy indexes its weights over; the two compose with no overlap.

**Exclusions bifurcate** accordingly: an exclusion that is an *invariant* ("this
state never arises because operation `X` maintains `I`") is propositional and
rides the existing **`P`/`Q`** channels (tightening the support, where Ken can
*check* it); an exclusion that is a *judgment* ("valid and reachable, but
low-risk") is a **measure** decision (weight ≈ 0) and belongs to the sampling
policy. Ken handles the first kind already; it stays silent on the second.

### 4.1 The no-measure invariant — exhaustive by construction

"`G` carries support, never measure" must be a **type-level impossibility, not a
convention** a careful author upholds. The `G` payload type admits **only**
support constructors — the equivalence-class **partition**, the **boundaries**
between classes, and the **case decomposition** (the `match` arms) — and has
**no** field, variant, or escape hatch into which a number could be interpreted
as a likelihood/weight/probability. "A measure leaked into `G`" is then **not
representable**: a serialization with a per-class weight does not type, and a
new support-kind added without a measure-free type is a **compile error**, not a
silent gap (COORDINATION §7, exhaustive-by-construction — the same sealed,
no-catch-all discipline as the `LeakSink` set of `../60-security/61 §5a.2`).

This seal is load-bearing because of **what the kernel does not see**. The
emitter is an untrusted projection (no kernel enlargement); the kernel never
inspects the **export bytes**, so a measure leaked into `G` would **not** be
caught downstream of the emitter by any kernel check. The net is therefore two
things and only these: the **type-level seal** in the emitter (measure
unrepresentable) and **conformance AC5** (an attempt to attach a weight is
rejected / not representable). Enumerate what the kernel cannot backstop — the
serialized `G` — and seal it where it is built. (This is the `G` instance of the
erased-before-kernel omission-hole the security labels face: the discriminating
check must live where the untrusted projection happens, since the trust root
sees nothing.)

## 5. Seam soundness — one-way flow & translation faithfulness

Ken is intuitionistic/total/constructive; `Ward`'s engines are classical
(model-checkers decide truth in a model). The seam composes the two **soundly**
and **legibly** by three commitments (`OQ-classical-bridge`).

**One-way flow (Ken → Ward), strictly.** Ken *exports* obligations `T` and
assumptions `P`; `Ward` discharges them by classical means; **results never
re-enter Ken as proof terms.** A depth-`k` model-check is not a proof for all
`N`; a green monitor is not a proof. So a discharged obligation stays
`delegated`/`tested` in the four-way status (`../20-verification/21 §5`) and
rides in `trusted_base_delta` — it is **never promoted to `proved`**. This is a
deliberate choice for **human legibility** as much as soundness: consumers read
one direction of flow, not a bidirectional logic.

**Soundness by assume-guarantee construction.** Every Ken theorem is conditional
— "**given** `P`, **then** `Q`" — and that implication is intuitionistically
valid and kernel-checked *regardless of how `P` is later discharged*. Because
Ken never imports `Ward`'s conclusion, no classical strength leaks into the
kernel; the classical discharge of `P` is a separate, lower-trust artifact (the
discharge attestation, below). Where the obligation is *decidable/finite-state*,
classical and intuitionistic truth coincide anyway (a decision procedure gives
`P ∨ ¬P` constructively, `../20-verification/23 §2`); where it is unbounded, it
is an honest assumption.

**Translation faithfulness — the Ken-checked half.** `Ward` consumes a
model-checker input, not Ken's datatype, so a translation `τ` mediates; an
*unfaithful* `τ` (a green check on a spec that doesn't match the code) is worse
than none. `τ` splits:

- **Property translation** `compile : Temporal Σ → WardFormula` (`72 §3`; **one
  of the two sibling `compile` projections** — distinct from `73 §2.4`'s `→
  Monitor` runtime synthesis, not a second "direction" of one function). Both
  sides are syntax over the same alphabet `Σ`, so Ken proves `compile`
  **semantics-preserving once, at the compiler level** — `⟦φ⟧ = ⟦compile φ⟧`
  over `Σ`-traces, an ordinary structural induction (the "reason *about*
  formulas" power, `72 §2`). Required, but **amortized to zero per obligation**
  — every delegated property reuses the one lemma. This is the exact analog of
  the prover's Kripke-adequacy lemma (`../20-verification/23 §4`).
- **Model translation** — the transition system `Ward` explores corresponds to
  the program's denoted behaviors. Ken's contribution is *structural*: the model
  is **generated** from the code (`Σ` *is* the perform-node signatures; the
  state machine derives from the space semantics), so authoring drift is
  impossible by construction. The residual concrete-vs-abstract gap is the
  **conformance** story (`73`) plus an honest assumption — not a single Ken
  proof.

**The one trust edge, pinned.** The faithfulness proof is *relative to an
axiomatized `Ward` semantics*; that `Ward` *implements* it is the one explicit,
version-bounded assumption — **pinned as the `Ward` version in the discharge
attestation** (`../60-security/63 §5a`). The attestation is therefore not
bureaucracy: it is the anchor of the faithfulness argument.

### 5.1 The one-way gate — no promotion path (the G-Ward-seam)

The one-way-flow commitment is realized in the emitter as the **absence of a
code path**, not a runtime check that could be bypassed: the emitter is a pure
**read-and-project** over verified content (the `25 §3` verdict document + the
denotation), and there is **no function** in it from a `Ward` verdict — or any
classical result — to a `proved` status. Concretely:

- The emitter **only reads** epistemic status it did not author (`21 §5`) and
  **projects** it (§2.1); it never **writes** a status.
- There is **no ingest path**: a classically-discharged obligation re-enters the
  Ken side only as a `trusted_base_delta` entry / a discharge-attestation record
  (`63 §5a`), tagged `delegated`/`tested` — **never** a certificate the kernel
  would re-check, and **never** re-stamped `proved`.
- A `delegated`/`tested` entry is therefore **monotone**: nothing in the export
  pipeline raises it to `proved` (invariant I4).

**AC6 asserts this gate structurally** — *there is no path from a `Ward` verdict
to a `proved` status* (the G-Ward-seam gate acceptance): a `Ward` "green" fed
back through the emitter leaves it `delegated`, and the conformance corpus
pins the **absence** of any promoting transition (a guard-gated absence, named:
no `proved`-writing edge exists, not merely "the happy path doesn't take one").

### 5.2 What B1 fixes vs. what B2/B3 own

The `compile : Temporal Σ → WardFormula` faithfulness lemma (`72 §3`; the
amortized-once structural induction above, analog of the Kripke-adequacy lemma
`23 §4`) is **owned by B2/B3** — it needs the `Temporal` **datatype** (B2) and
the trace/runtime contract (B3). **B1 fixes what that lemma reuses**: the `T`
**channel** (the `delegated` obligation values + their status, structured for
the alphabet they range over) and the `Σ` **alphabet** itself (§2.1). B1 proves
no new metatheory; it pins the two interfaces `compile` and the model
translation will be stated over, so B2/B3 build the lemma without re-litigating
the channel or the alphabet (coordinate cross-WP via spec, do not hard-bind wire
spellings, §3.1).

That `compile` ownership statement applies to existing `Temporal` bodies. The
structured resource-lifetime bodies of §2.2–§2.3 carry their fixed Ward monitor
templates directly; they do not pretend that `Temporal`/`Pred::Event` acquired
an identity binder.

## 6. What this chapter delivers, and its acceptance

The export emitter lives in `crates/ken-elaborator` and is, end to end, a
**deterministic projection of verified content** — it adds nothing to the
trusted base and proves nothing new. The implementable deliverables:

1. **The five-part projection (§2.1)** — each of `Q`/`P`/`Σ`/`T`/`G` from its
   pinned source of truth, with the **status → field** map and the kernel-side
   **honesty discriminator** (`trusted_base()` membership + certificate
   presence, `21 §5.4`).
2. **The two-layer serialization (§3)** — Ken-native contract + ITF traces, with
   the **value-set and cross-field invariants I1–I5 locked** and the literal
   field **spellings `(oracle)`-tagged** (Ward finalizes the wire token; Ken
   locks the concept + the content-hash stability discipline).
3. **Content-addressing + provenance (§3.3)** — a canonical-form hash,
   deterministic in the checked program, embedded in provenance (`63 §2`) and
   bound 1:1 by the discharge attestation (`63 §5a`).
4. **The sealed no-measure `G` (§4.1)** — measure **unrepresentable** at the
   type level (exhaustive-by-construction), the net being the seal + conformance
   AC5 since the kernel does not see the export bytes.
5. **The one-way-flow gate (§5.1)** — no code path promotes a `delegated`/
   `tested` obligation to `proved`; the `compile` lemma is named as B2/B3's,
   with the `T` channel + `Σ` it reuses fixed here (§5.2).
6. **The correlated resource-lifetime body (§2.2)** — one direct
   `ResourceLifetimeObligation`, `ResourceTraceIdentityV1` correlation policy,
   delegated-only status, and hash participation alongside the existing
   `Temporal` path.
7. **The role-labelled observation and plans (§2.3)** — the sole
   `EffectObservation`/`EffectEvent.resource_bindings` shape, exact per-kind
   plans, closed `TerminalExitClass`, and external Ward monitor template.

**Acceptance criteria.** *Names align with the frame's AC1–AC6.*

- **AC1 (reproducible).** Same program → **same export hash** (structural
  assertion on the hash, §3.3), not merely "an export is produced".
- **AC2 (no over-claim).** Every `Q` entry traces to a `proved` result with no
  postulate of its goal in `trusted_base()`; an **unproved** postcondition emits
  under `P`/`assume` (tagged `unknown`/`tested`), **never** `Q`. *Flips*
  proved→`Q` vs unknown→`P` on the **same** postcondition (invariant I1).
- **AC3 (assumption visibility).** Removing an `assume` / shrinking the
  `trusted_base_delta` shows up as a changed `P` (and a changed hash, I2).
- **AC4 (`Σ` reuse).** `Σ` equals the program's L5 perform-node signatures —
  asserted as **structural equality** to the denotation's signatures, not a
  re-derived alphabet (I3).
- **AC5 (no-measure).** No `G` field can carry a weight; the attempt is rejected
  / not representable (§4.1, I5). `G` is partition + boundaries only.
- **AC6 (one-way / G-Ward-seam).** **No `Ward` result is ever recorded as
  `proved`** — assert **no** code path runs from a `Ward` verdict to a
  `proved` status (§5.1, I4).

**Conformance (`../../conformance/behavioral/export/`).** AC1–AC6 as
discriminating cases, each **routing a real checked program through the actual
emitter** and observing the projected field — **not** a synthetic export literal
with asserted fields (a test that builds an export struct and checks a field
guards nothing; the QA gate is *real verified content → real projection*). Each
case **flips** on its bug (per-case verdict/structural-flip); the **cross-case
sweep** groups by the status-projection class and asserts agreement —
{`proved`→`Q`}, {`tested`→`P`, `unknown`→`P`}, {`delegated`→`T`} — with the two
**boundary invariants** pinned: **no non-`proved` claim ever lands in `Q`** (the
honesty direction, AC2/I1) and **no `Ward` result ever lands in `proved`** (the
one-way direction, AC6/I4). *Worked flip (AC2):* a function with
`ensures result > 0` emits that postcondition under `Q` (status `proved`) when
its obligation discharges, and under `P` (status `unknown`) when the proof is
left an open hole — the field flips with `trusted_base()` membership, a
structural signal, the same program under the two kernel states.

**PX8-X incremental acceptance.** A full positive case carries the canonical
`File`/`Buffer` pair on a positioned transfer and is accepted by both complete
per-kind plans. Buffer-only and read-only-positioned targets assert the exact
plan/subsequence specialization above; adding one unreachable operation to a
static plan rejects before export under I3. Runtime controls independently fail
for a missing role, swapped roles, an out-of-order pair, duplicate/extra
bindings, and two split uncorrelated single-resource observations, but reproduce
the positive case's already-emitted static `T` bytes and hash exactly. The
resource-producing `px7f_export_resource` fixture emits one direct file plan,
retains exact denotation-derived alphabet
`{FsOpen, FsHandleMetadata, ResourceRelease}` with no `FS` family-name member,
and remains I3-coherent. The buffer-only fixture emits one direct Buffer plan.
The no-acquire control independently retains frozen hash
`ken-export-v0:6360c2cb74f78f7e`. These cases assert the complete direct body and
its `T` hash participation; no schema discriminator or compatibility path is
representable.
