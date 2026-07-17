# Buffer I/O and multi-resource conformance — seed cases (PX8-T)

Format: `../../README.md`. These cases pin the PX8 contracts consumed by the
runtime and Foundation lanes: role-labelled multi-resource observations, the
Ward V2 lifetime body, positioned positive progress, `writeAll`, resource-kind
mismatch, and deterministic buffer admission limits.

The cases are contract roots, not claims that PX8 is already built. Every case
names the producer whose arrival makes it reachable. A schema-unit value or a
hand-fed successful result cannot satisfy a case that requires the real host,
export, derived-Ken, or Ward route.

## Producer grounding and locked vocabulary

The landed PX7 substrate supplies the starting point:

- `HostOpV1::{FsOpen, FsHandleMetadata, ResourceRelease}` and
  `ResourceKindV1::FsHandle` are real in
  `crates/ken-host/src/effect_v1.rs`;
- `EffectEventV1.resource` carries one optional
  `ResourceTraceIdentityV1`;
- `ResourceLifetimeObligationV1` and its canonical `T`-hash route are real in
  `crates/ken-elaborator/src/export.rs`; and
- the checked resource-producing `px7f_export_resource` V1 fixture has the
  denotation-derived alphabet `{FsOpen, FsHandleMetadata, ResourceRelease}`
  (and not `FS`) plus the coherent V1 body required by I3; and
- the exact checked no-acquire regression producer has export hash
  `ken-export-v0:6360c2cb74f78f7e`.

The same inventory also proves the PX8 routes are not yet live on this base:
there is no `Buffer` resource kind, `BufferAllocate`, `FsReadAt`, `FsWriteAt`,
buffer freeze, V2 role binding, progress sum, `NoProgress`, or
`BufferLimitsV1`. Those are the named PX8-R/PX8-F/Ward reachability gates below,
not values this seed pretends to observe now.

The fixed PX8 vocabulary is:

```text
ResourceBindingRoleV2 = File | Buffer | Target

resource_bindings:
  [(ResourceBindingRoleV2, ResourceTraceIdentityV1)]

ReadProgress =
  ReadSome BufferSpan TransferCount
  | ReadEof

WriteProgress =
  Wrote TransferCount

ResourceKindMismatch {
  expected: ResourceKindV1,
  actual: ResourceKindV1,
}

BufferLimitsV1 {
  per_buffer_max_capacity: Int,
  invocation_max_live_capacity: Int,
}
```

`resource_bindings` is a runtime observation field. Its identities are minted
at acquisition and are not serialized in the target-level `T` entry. The
static V2 lifetime body serializes the role-specific bind/match policy and the
per-kind acquire/use/settle plans that govern those runtime bindings. The whole
static body is canonicalized with `T`; runtime identities are not export or
hash inputs.

Each operation's ordered binding sequence is exact:

```text
successful FsOpen          -> [(Target, file_r)]
successful BufferAllocate  -> [(Target, buffer_r)]
FsHandleMetadata           -> [(Target, file_r)]
FsReadAt                   -> [(File, file_r), (Buffer, buffer_r)]
FsWriteAt                  -> [(File, file_r), (Buffer, buffer_r)]
BufferFreeze               -> [(Target, buffer_r)]
ResourceRelease            -> [(Target, released_r)]
```

For a target whose reachable alphabet contains the complete file-and-buffer
inventory, the static V2 body has this field set, nesting, and order:

```text
ResourceLifetimeObligationV2 {
  schema_version: 2,
  body_kind: ResourceLifetimeObligationV2,
  obligation_id: String,
  status: delegated,
  correlation: ResourceLifetimeCorrelationV2 {
    identity_type: ResourceTraceIdentityV1,
    event_field: EffectEventV2.resource_bindings,
    role_type: ResourceBindingRoleV2,
    canonical_order: OperationDefined,
  },
  plans: [
    ResourceLifetimePlanV2 {
      resource_kind: FsHandle,
      bind_at: Successful(FsOpen, Target),
      require_same_at: [
        (FsHandleMetadata, Target),
        (FsReadAt, File),
        (FsWriteAt, File),
        (ResourceRelease, Target),
      ],
    },
    ResourceLifetimePlanV2 {
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
  monitor_template: WardResourceLifetimeMonitorV2 {
    correlate_every_role_binding: true,
    successful_acquire_settles_exactly_once: true,
    forbid_successful_use_after_settlement: true,
    require_no_live_bracket_owned_identity_on:
      [NormalReturn, ReturnedError, ControlledTrap],
    retain_settlement_outcome: true,
  },
}
```

The `FsHandle` plan precedes the `Buffer` plan, and each operation list keeps
the order above. A target emits only a plan whose acquisition is in its exact
reachable alphabet `Σ`. Each emitted `require_same_at` is the canonical ordered
subsequence of that kind's inventory whose operations are also in `Σ`; it
neither invents an unreachable operation nor omits a reachable one. For every
identity selected by an emitted plan, the V2 monitor
requires exactly one settlement, forbids successful use after settlement,
retains the settlement outcome, and requires no live bracket-owned identity at
`NormalReturn`, `ReturnedError`, or `ControlledTrap`. The entire static entry,
including its `delegated` status, correlation descriptor, ordered plans, and
monitor template, is canonicalized in `T` and contributes to the export hash.
Runtime identities such as `file_r` and `buffer_r` do not.

## RB-A. Canonical two-resource observation validates

### buffer-io/v2-file-buffer-bindings-validate

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC1; Architect PX8 ruling,
  “Buffer ownership and lifetime”; ADR 0021, “Expressibility prerequisite”
- given: a checked full-inventory two-resource target whose exact reachable
  `Σ` contains both acquisitions and every operation in the two global
  inventories; its real runtime observation includes
  `FsReadAt -> [(File, file_r), (Buffer, buffer_r)]`, and the Spec-owned V2
  lifetime body is emitted through the real export route
- expect: schema validation accepts the exact ordered binding; the `T` body
  contains both per-kind plans and the V2 monitor policy; its status is
  `delegated`; the static body, not `file_r` or `buffer_r`, contributes to the
  export hash
- expect: Ward binds `file_r` at `Successful(FsOpen, Target)` and `buffer_r`
  at `Successful(BufferAllocate, Target)`, requires those same identities in
  the `File` and `Buffer` roles of `FsReadAt`, and matches each later
  `ResourceRelease(Target, r)` to the appropriate plan
- why: one elected identity, a request-byte token, or two unlabelled identities
  cannot satisfy this exact ordered pair and policy.

## RB-B. Malformed binding is rejected

### buffer-io/v2-missing-buffer-role-rejected

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: the RB-A observation and lifetime body, changing only the `FsReadAt`
  binding to `[(File, file_r)]`
- expect: static export produces the same canonical `T` bytes and hash as RB-A;
  observation validation rejects the malformed runtime event, and Ward does
  not accept it as discharging either emitted plan
- why: an operation-name-only or one-resource validator accepts both RB-A and
  RB-B. The correct route accepts RB-A and rejects RB-B on the missing role
  alone.

## RB-C. Swapped role labels are rejected

### buffer-io/v2-swapped-role-labels-rejected

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: the RB-A observation and lifetime body, changing only the two role
  labels to `[(Buffer, file_r), (File, buffer_r)]`
- expect: static export produces the same canonical `T` bytes and hash as RB-A;
  observation validation rejects the wrongly labelled runtime event, and Ward
  does not accept it as discharging either emitted plan
- why: an order-only validator sees two identities in the expected positions
  and accepts both RB-A and RB-C. The exact role labels alone must flip the
  verdict.

## RB-D. An out-of-order pair is rejected

### buffer-io/v2-out-of-order-pair-rejected

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: the RB-A observation and lifetime body, preserving both labelled
  tuples but reversing their sequence to
  `[(Buffer, buffer_r), (File, file_r)]`
- expect: static export produces the same canonical `T` bytes and hash as RB-A;
  observation validation rejects the misordered runtime event, and Ward does
  not reorder it into a discharge
- why: a map-shaped or sorting validator accepts RB-A and RB-D. Canonical
  operation-defined order is the only varied property.

## RB-E. Two single-resource atoms cannot replace one correlated event

### buffer-io/v2-independent-single-resource-atoms-rejected

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: the RB-A target and identities, replacing its one
  `FsReadAt -> [(File, file_r), (Buffer, buffer_r)]` observation with two
  independent `FsReadAt` atoms, one carrying only `[(File, file_r)]` and one
  carrying only `[(Buffer, buffer_r)]`
- expect: static export produces the same canonical `T` bytes and hash as RB-A;
  observation validation rejects both malformed runtime events, and Ward does
  not merge independent atoms into a synthetic pair
- why: unioning bindings across events accepts the same identities while
  losing the single-operation correlation that V2 exists to express.

## RB-F. Uncorrelated lookalike is rejected

### buffer-io/v2-buffer-identity-must-correlate

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: hold operation, role order, file identity, plans, and monitor policy
  fixed; compare:
  - positive: `BufferAllocate(buffer_r1); FsReadAt(File=file_r,
    Buffer=buffer_r1); ResourceRelease(Target=buffer_r1)`;
  - negative: `BufferAllocate(buffer_r1); FsReadAt(File=file_r,
    Buffer=buffer_r2); ResourceRelease(Target=buffer_r1)`, where
    `buffer_r1 != buffer_r2`
- expect: both traces use the same already-emitted canonical `T` bytes and
  export hash; runtime identities never become static hash inputs
- expect: the positive trace satisfies the Buffer plan; the negative trace
  fails the Ward obligation because `buffer_r1` remains unmatched at the use
  site and `buffer_r2` has no corresponding acquisition
- why: role labels without same-identity matching accept both. Identity is the
  only varied field, so the verdict flip is not confounded.

## RB-G. V1 resource exports and no-acquire exports preserve exact bytes

### buffer-io/v2-route-is-additive-for-no-buffer-targets

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.3`; PX8-T D1/AC2
- given: run the landed checked `RESOURCE_PRODUCER`, target
  `px7f_export_resource`, from
  `crates/ken-elaborator/tests/px7f_resource_lifetime_export.rs` before and
  after enabling the V2 route; its reachable alphabet contains `FsOpen` but no
  buffer acquisition
- expect: both runs derive exactly the alphabet
  `{FsOpen, FsHandleMetadata, ResourceRelease}`, exclude `FS`, and emit the same
  canonical `ResourceLifetimeObligationV1` entry, complete export bytes, and
  export hash; neither emits V2, rewrites the V1 body, or violates I3; this
  with-resource control is structural and does not freeze a hash literal
- given: also run the existing checked no-acquire producer from that test
- expect: its canonical export byte strings are identical; both hashes equal
  `ken-export-v0:6360c2cb74f78f7e`; no V2 body or
  `resource_bindings` field appears
- why: semantic equivalence or a merely stable recomputed hash is too weak.
  The concrete resource-producing fixture catches a broad V1 rewrite; the
  pinned no-acquire hash independently catches perturbation of the ordinary
  export route.

## RB-H. Wrong V2 schema version rejects before export

### buffer-io/v2-wrong-schema-version-rejected-pre-export

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC6
- given: the exact RB-A static target and body, changing only
  `schema_version: 2` to `schema_version: 1`
- expect: schema validation rejects before canonical `T` bytes or an export
  hash are emitted
- why: runtime/Ward rejection is too late for a malformed static version, and
  accepting both versions would erase the V1/V2 contract boundary.

## RB-I. Wrong V2 descriptor rejects before export

### buffer-io/v2-wrong-correlation-descriptor-rejected-pre-export

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC6
- given: the exact RB-A static target and body, changing only
  `event_field: EffectEventV2.resource_bindings` to
  `event_field: EffectEventV1.resource`
- expect: schema validation rejects before canonical `T` bytes or an export
  hash are emitted
- why: the V1 carrier cannot express the V2 ordered role-labelled event; a
  field-name-only serializer would otherwise hash an unmonitorable policy.

## RB-J. Missing reachable plan rejects before export

### buffer-io/v2-missing-reachable-buffer-plan-rejected-pre-export

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC6; `71 §3.1` I3
- given: the exact RB-A target and body, whose `Σ` contains both acquisitions,
  changing only `plans` by removing the Buffer plan
- expect: static-policy validation rejects before canonical `T` bytes or an
  export hash are emitted
- why: a merely optional plan list would leave a reachable buffer identity
  outside the monitor policy while still claiming the same target alphabet.

## RB-K. Noncanonical plan order rejects before export

### buffer-io/v2-plan-order-rejected-pre-export

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC6
- given: the exact RB-A target and body, preserving both complete plans but
  reversing them to Buffer then FsHandle
- expect: static-policy validation rejects before canonical `T` bytes or an
  export hash are emitted
- why: treating plans as a map admits two serializations and therefore two
  hashes for the same checked target.

## RB-L. Buffer-only targets emit only their reachable plan

### buffer-io/v2-buffer-only-plan-is-sigma-specialized

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC1; `71 §3.1` I3
- given: a checked buffer-only target whose exact reachable alphabet `Σ` is
  `{BufferAllocate, BufferFreeze, ResourceRelease}`
- expect: the V2 entry contains exactly one Buffer plan, with
  `bind_at: Successful(BufferAllocate, Target)` and
  `require_same_at: [(BufferFreeze, Target), (ResourceRelease, Target)]`;
  it contains no FsHandle plan, `FsReadAt`, or `FsWriteAt`
- expect: the entry validates I3, is canonicalized in `T`, and is covered by
  the export hash
- why: the target is non-degenerate because it acquires, uses, and settles a
  buffer, while any fixed two-plan emitter hashes operations absent from `Σ`.

## RB-M. Read-only positioned targets omit unreachable writes

### buffer-io/v2-read-only-positioned-plan-is-sigma-specialized

- status: **RED-UNTIL-PX8-R + PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC1; `71 §3.1` I3
- given: a checked positioned-read target whose exact reachable alphabet `Σ`
  is `{FsOpen, BufferAllocate, FsReadAt, ResourceRelease}`
- expect: the V2 entry contains the FsHandle plan
  `[(FsReadAt, File), (ResourceRelease, Target)]` followed by the Buffer plan
  `[(FsReadAt, Buffer), (ResourceRelease, Target)]`
- expect: neither plan names `FsHandleMetadata`, `FsWriteAt`, or
  `BufferFreeze`; the entry validates I3, is canonicalized in `T`, and is
  covered by the export hash
- why: preserving canonical global order while filtering by exact `Σ`
  distinguishes specialization from a fixed full inventory or ad hoc sorting.

## RB-N. An extra unreachable operation violates I3

### buffer-io/v2-extra-unreachable-operation-rejected-pre-export

- status: **RED-UNTIL-PX8-F export route**
- spec: `71 §2.3/§3.1/§3.3`; PX8-T D1/AC6; `71 §3.1` I3
- given: the RB-L buffer-only target and specialized body, changing only the
  Buffer `require_same_at` by inserting `(FsWriteAt, Buffer)`, which is absent
  from that target's exact `Σ`
- expect: static-policy/I3 validation rejects before canonical `T` bytes or an
  export hash are emitted
- why: a kind-wide inventory emitter accepts the extra operation; exact
  alphabet closure rejects policy that the target cannot execute.

## RB-O. A duplicated runtime role binding is rejected

### buffer-io/v2-duplicated-runtime-role-binding-rejected

- status: **RED-UNTIL-PX8-R + PX8-F export route + Ward V2 consumer**
- spec: `71 §2.3`; PX8-T D1/AC6
- given: the RB-A observation and lifetime body, changing only the `FsReadAt`
  binding sequence to
  `[(File, file_r), (Buffer, buffer_r), (Buffer, buffer_r)]`
- expect: static export produces the same canonical `T` bytes and hash as RB-A;
  observation validation rejects the duplicated runtime role, and Ward does
  not discard or coalesce it into a discharge
- why: a set-shaped or first-binding-wins validator accepts both RB-A and RB-O.
  Exact operation-defined sequence cardinality is the only varied property.

## PR-A. Exact read/write progress partition

### buffer-io/positive-short-is-success-zero-write-is-error

- status: **RED-UNTIL-PX8-R + PX8-F surface**
- spec: `38 §1.7.2`; PX8-T D2/D3/AC3
- given: positive effective requests of length `8`
- expect:
  - read `0` -> `ReadEof`;
  - read `3` -> `ReadSome span n`, with `n = 3` and `span.length = 3`;
  - write `3` -> `Wrote n`, with `n = 3`; and
  - write syscall `0` -> `Err NoProgress`, never `Wrote 0`
- expect: neither closed sum contains `Complete`, `Partial`, or `WouldBlock`;
  `TransferCount` cannot be constructed with zero and projects a strictly
  positive `Int`
- why: the four outcomes distinguish EOF, positive short success, and the
  load-bearing zero-write failure. A generic status/count record can represent
  forbidden combinations and fails this shape.

## PR-B. Positioned bounds and tail capping

### buffer-io/positioned-transfer-bounds

- status: **RED-UNTIL-PX8-R + PX8-F surface**
- spec: `38 §1.7.1/§1.7.2`; PX8-T D3/AC3
- given: a capacity-`8` buffer, a positive length-`4` request, and explicit
  file offsets
- expect: start `6` is capped to the available tail and a positive transfer is
  at most `2`; start `8` has zero effective length and the derived wrapper
  invokes no positive-length primitive; start `9` is an invalid-bounds error;
  a negative file offset or offset-plus-length overflow is an error
- expect: neither read nor write mutates a hidden file cursor; a later call's
  result depends on its explicit offset
- why: this separates ordinary short progress, the inclusive capacity boundary,
  an out-of-range start, and arithmetic failure. Treating the closed endpoint as
  invalid or invoking a positive-length primitive for it fails the case.

## PR-C. Failures never masquerade as progress

### buffer-io/transfer-failures-remain-errors

- status: **RED-UNTIL-PX8-R + PX8-F surface**
- spec: `38 §1.7.2`; PX8-T D2/AC3
- given: independently reach `Closed`, `MalformedResource`,
  `ResourceKindMismatch`, `RightNotHeld`, invalid offset/window/bounds,
  buffer-limit/allocation failure, unsupported nonblocking posture, host I/O
  failure, and `Interrupted`
- expect: every arm remains an error carrying its own identity; none constructs
  `ReadProgress` or `WriteProgress`; `Interrupted` is not silently retried or
  reclassified as short success
- expect: `WouldBlock` is absent from the PX8 progress vocabulary; the PX12
  asynchronous seam cannot be accepted as a PX8 status
- why: this closes the error side of the partition. Positive-short success does
  not license a generic fallback that turns unrelated failures into progress.

## Locked `writeAll` oracle

For one constructor-private input span, `writeAll` derives structural `Nat`
fuel from the span length. It terminates and has exactly two observable result
classes:

- success only after every byte in the span has been written; and
- the first transfer error unchanged, after preserving the exact prefix written
  before that error.

If every primitive call succeeds, strict positivity and `n <= remaining` imply
that the whole span is written. Fuel exhaustion with bytes remaining is
excluded by that lemma; it is not a public error. A caller-supplied fuel value
or a result that claims success with a nonempty remainder does not conform.

## WA-A. Full writes reach whole-span success

### buffer-io/write-all-full-writes

- status: **RED-UNTIL-PX8-R + PX8-F derived Ken `writeAll`**
- spec: `38 §1.7.3`; PX8-T D4/AC4
- given: span bytes `ABCDEFGH`, initial file offset `10`, and a real scripted
  transfer backend whose successive positive writes are `[8]`
- expect: `writeAll` returns success, the sink contains exactly `ABCDEFGH`, and
  the sole `writeAt` call uses file offset `10` and the whole span
- expect: fuel is derived from the span length and is not accepted from a
  caller
- why: this is the full-progress baseline reached through the derived Ken loop,
  not a hand-fed success value.

## WA-B. Short writes continue with exact accounting

### buffer-io/write-all-short-writes-complete

- status: **RED-UNTIL-PX8-R + PX8-F derived Ken `writeAll`**
- spec: `38 §1.7.3`; PX8-T D4/AC4
- given: the same span and offset as WA-A, with real successive write results
  `[Wrote 3, Wrote 2, Wrote 3]`
- expect: `writeAll` returns success and the sink contains exactly `ABCDEFGH`;
  calls use file offsets `[10, 13, 15]` and non-overlapping remaining spans of
  lengths `[8, 5, 3]`
- why: treating a positive short write as complete leaves `DEFGH` unwritten;
  treating it as an error rejects. Both bugs produce the opposite verdict or a
  different structural call trace.

## WA-C. Write zero reaches `NoProgress`

### buffer-io/write-all-zero-write-is-no-progress

- status: **RED-UNTIL-PX8-R + PX8-F derived Ken `writeAll`**
- spec: `38 §1.7.2/§1.7.3`; PX8-T D2/D4/AC3/AC4
- given: the same span and offset, with real successive write results
  `[Wrote 3, syscall-zero]`
- expect: `writeAll` returns `Err NoProgress`; the sink contains exactly `ABC`;
  no third host call occurs and the remaining span is not reported written
- why: mapping zero to success falsely completes or loops without decreasing.
  This case reaches the zero-returning host branch and observes both the named
  error and exact prefix.

## WA-D. First transfer error preserves the exact prefix

### buffer-io/write-all-transfer-error-preserves-prefix

- status: **RED-UNTIL-PX8-R + PX8-F derived Ken `writeAll`**
- spec: `38 §1.7.3`; PX8-T D4/AC4
- given: the same span and offset, with real successive results
  `[Wrote 2, Err E]`
- expect: `writeAll` returns the same first error `E` unchanged; the sink
  contains exactly `AB`; exactly two host calls occur, at offsets `[10, 12]`;
  no byte after the exact successful prefix is claimed written
- why: a loop that restarts, rewrites the error, discards the prefix, or retries
  after error differs observably. This is the indirection case: the failure is
  reached only after one successful recursive step.

## KM-A. Buffer token to file-only consumer mismatches

### buffer-io/buffer-token-rejected-by-file-consumer

- status: **RED-UNTIL-PX8-R**
- spec: `38 §1.7`; PX8-T D5/AC5; ADR 0021, “Host catalog and
  fail-visible errors”
- given: a real token minted by `BufferAllocate`, supplied to
  `FsHandleMetadata`
- expect: `ResourceKindMismatch { expected: FsHandle, actual: Buffer }`
- control: a real `FsHandle` token supplied to `FsHandleMetadata` succeeds
- why: both tokens are live and well-formed, so `MalformedResource` is not an
  admissible substitute.

## KM-B. File token to buffer-only consumer mismatches

### buffer-io/file-token-rejected-by-buffer-consumer

- status: **RED-UNTIL-PX8-R**
- spec: `38 §1.7`; PX8-T D5/AC5; ADR 0021, “Host catalog and
  fail-visible errors”
- given: a real token minted by `FsOpen`, supplied to `BufferFreeze`
- expect: `ResourceKindMismatch { expected: Buffer, actual: FsHandle }`
- control: a real Buffer token supplied to `BufferFreeze` succeeds
- why: reversing expected and actual produces the other exact payload. Together
  KM-A/KM-B fail a swapped-field implementation while the same-kind controls
  prevent an always-mismatch route from passing.

## BL-A. Per-buffer limit is deterministic

### buffer-io/per-buffer-capacity-limit

- status: **RED-UNTIL-PX8-R admission + PX8-F `withBuffer`**
- spec: `38 §1.7`; PX8-T D5/AC5
- given: checked/native plan limits `{ per_buffer_max_capacity: 8,
  invocation_max_live_capacity: 12 }`
- expect: one positive-capacity buffer of `8` is admitted; capacity `9` is
  rejected as a buffer-limit/allocation error; capacity zero is rejected
- expect: changing an environment variable cannot alter either verdict
- why: this isolates the per-buffer bound while total live capacity stays
  within `12`.

## BL-B. Invocation-wide live limit is deterministic

### buffer-io/invocation-live-capacity-limit

- status: **RED-UNTIL-PX8-R admission + PX8-F `withBuffer`**
- spec: `38 §1.7`; PX8-T D5/AC5
- given: the same plan as BL-A; keep one live buffer of capacity `8`, then
  request a second buffer
- expect: capacity `4` is admitted and makes total live capacity `12`; capacity
  `5` is rejected; after the first buffer settles, capacity `5` is admitted
- expect: the plan, not process environment, supplies both limits
- why: all individual capacities are below the per-buffer maximum, so only the
  invocation-wide live-capacity accounting varies.

## Coverage map

| PX8-T acceptance criterion | Cases |
|---|---|
| AC1 V2 roles, plans, monitor, delegated `T` body | RB-A–RB-F, RB-L, RB-M |
| AC2 no-buffer and pre-PX8 hash-byte preservation | RB-G |
| AC3 progress partition and positivity/bounds | PR-A, PR-B, PR-C, WA-C |
| AC4 four reaching `writeAll` branches | WA-A, WA-B, WA-C, WA-D |
| AC5 mismatch pair, same-kind controls, buffer limits | KM-A, KM-B, BL-A, BL-B |
| AC6 malformed/uncorrelated/I3/kind rejects | RB-B–RB-F, RB-H–RB-K, RB-N, RB-O, KM-A, KM-B |

## Cross-case, verdict-flip, and reachability sweep

- **D1 is non-degenerate on every locked axis.** RB-A/RB-B vary only presence
  of the required Buffer role; RB-A/RB-C vary only the role labels; RB-A/RB-D
  vary only tuple order; RB-A/RB-E vary only whether the two bindings inhabit
  one event or independent atoms; and RB-A/RB-F vary only the Buffer identity.
  RB-A/RB-O vary only one duplicated Buffer tuple.
  A presence-only, role-blind, map-shaped, cross-event-unioning, or
  identity-blind, set-shaped validator accepts its respective negative. The
  correct route accepts only RB-A.
- **Static and runtime failure phases are distinct.** RB-H–RB-K and RB-N alter
  the exported descriptor, version, required plan set, canonical plan order, or
  alphabet closure and therefore reject before `T`/hash emission. RB-B–RB-F
  and RB-O leave the static body byte-identical to RB-A; their runtime
  observations fail at event validation or Ward discharge, without changing
  the already-emitted canonical `T` bytes or hash.
- **Plan specialization is controlled.** RB-A retains the full two-resource
  inventory; RB-L's non-degenerate buffer-only `Σ` emits one filtered plan;
  RB-M retains both resources but removes only metadata/write operations absent
  from its read-only `Σ`; RB-N changes only the insertion of one unreachable
  operation and flips acceptance under I3.
- **D4 reaches four different producer branches.** WA-A–WA-D invoke the real
  derived Ken loop over a scripted host backend. They do not construct a final
  `writeAll` result, call a helper that bypasses `writeAt`, or infer success from
  suite greenness. The short, zero, and error arms are observed after the host
  result crosses the real runtime-to-Ken projection.
- **The `writeAll` experiments are controlled.** Input bytes, initial offset,
  buffer span, plan, and loop are identical; only the host result sequence
  varies. Prefix bytes and call offsets make progress accounting observable
  independently of the final verdict.
- **The mismatch pair reverses a real production kind.** KM-A/KM-B use tokens
  minted by the two real acquisition paths, not fabricated encodings. Their
  same-kind controls must succeed. A malformed token has its separate existing
  `MalformedResource` route and cannot satisfy either case.
- **Reachability gates are explicit.** PX8-R owns `Buffer`, both acquisition
  paths, positioned host operations, progress/error production, mismatch, and
  admission enforcement. PX8-F owns `withBuffer`, the surface sums, the
  derived-Ken `writeAll`, and the export emitter. Ward owns V2 monitor
  execution. Until those producers land, these are RED contract roots, not
  current green claims.
- **V1 preservation is byte-level and resource-producing.** RB-G reuses the
  landed checked `px7f_export_resource` producer and compares its complete V1
  body, canonical export bytes, and hash before/after V2 enablement. Its
  independent checked no-acquire control retains the pinned canonical hash. A
  schema-unit comparison or newly invented fixture cannot replace either.
