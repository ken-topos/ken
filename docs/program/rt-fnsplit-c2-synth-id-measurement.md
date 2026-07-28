# `RT-FNSPLIT-C2-SYNTH-ID` measurement

This ledger records the closed synthesized-constructor identity population and
the carrier payload population delivered by
`RT-FNSPLIT-C2-SYNTH-ID`.

## Synthesized-constructor identity partition

The discriminator is the production effect-lowering output position:

- A role is **included** exactly when effect lowering places it in
  `Lowered::Constructor.constructor` or
  `DynamicConstructorAlternativeV1.constructor`.
- A symbol is **excluded** when it is only an input role, a case label, a
  specialization label, or an arbitrary runtime spelling and is never placed
  in either constructor output position.

The included fixed population is:

- `FileError`;
- `FileOperation::{Read, Write, ChangeMode}`;
- `Option::Some`;
- all ten `ResourceError` alternatives;
- both `ResourceKind` alternatives;
- `ResourceTraceIdentity`;
- `PrivateBufferSpan` and `PrivateTransferCount`;
- `ReadProgress::{ReadSome, ReadEof}`;
- `WriteProgress::Wrote`; and
- `Unit::MkUnit`.

The included dynamic population is every element of
`NativeProcessSymbols.io_errors` supplied to plan construction. The planner
iterates the slice and mints one opaque role token per element. No fixed count
or copied alternative list closes that population.

The excluded population is:

- process-input, list, product, and exit roles, which are entrypoint or input
  lowering roles rather than synthesized effect payload constructors;
- `Result::{Ok, Err}` as synthesized roles. `HostResult` uses them only as
  semantic case labels and acquires no Result identity; ordinary source Result
  constructor occurrences still use the existing source-constructor identity
  route;
- `Nat` and `Bool` specialization roles; and
- every arbitrary `RuntimeSymbol` spelling.

This is a partition by the production position above, not an example list. A
new role entering either constructor output position must enter the closed
inventory; a new symbol used only outside those positions remains excluded.

## Carrier payload population

The producer covers the four effect-result graph classes required by the frame:

- `Constructor` stores the closed-role identity and recursively emitted fields;
- `DynamicConstructor` selects an alternative at runtime, then stores that
  alternative's closed-role identity and recursively emitted fields;
- `ResourceToken` stores the full opaque scalar in an invocation-owned
  `BorrowedOpaque` node; and
- `ResponseBytes` stores the borrowed pointer scalar plus one immediate length
  child in an invocation-owned `BorrowedOpaque` node.

`HostResult` retains its existing representation: the runtime success scalar
selects field zero for the success payload and field one for the error payload.
Both payload graphs are emitted independently. Consumers use `host_success`
for case selection and `host_payload` for projection; no `Result` constructor
identity or compile-time payload template is introduced for `HostResult`.

Result case spellings do not select that representation-specific route. The
consumer first reads the carried word's emitted `BoundaryClass`: `HostResult`
uses `host_success` and `host_payload`, while an ordinary source Result in a
`Constructor` node retains the general tag, field-count, and field route. Both
routes rejoin only after their selected case body produces a carried word.

The behavioral controls are
`c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` for
one same consumer fed by a separately generated `HostResult` and an ordinary
source Result constructor, with nested `Constructor` plus `DynamicConstructor`
payloads, and
`c2_ac6_host_result_covers_resource_token_and_response_bytes_payloads` for
`ResourceToken` plus `ResponseBytes`.

## Causal mutation record

Each mutation was applied alone, run against its named detector, restored, and
followed by a green baseline:

| mutation | detector that reddened |
|---|---|
| use the parent `Match` origin for a synthesized constructor | `the C2 carrier edge emits`: planner refusal, because the parent has no constructor atom |
| store error in field zero and success in field one | AC-4 runtime-success nested identity/field assertion |
| route runtime success to the error source case | AC-4 runtime-success nested tag/field assertion |
| alias `Unit` to `FileError` identity | AC-2 distinct synthesized-role spelling assertion |
| omit the last dynamic IO-error role | AC-3 exact missing `IoError` role at `Some(0)` unit epoch |
| project compile-time field zero instead of calling `host_payload` | AC-4 runtime-error nested tag/field assertion |
| force `HostResult` down the ordinary constructor route | AC-4 runtime-success nested tag/field assertion |
| force an ordinary Result constructor down the `HostResult` route | AC-4 ordinary-source-Result tag/field assertion |

The Result-case mutation originally passed when both case bodies merely returned
their payload. The final AC-4 consumer instead performs a distinct nested match
inside each Result case. That makes the success-to-case association observable
while still requiring the selected payload's runtime tag and field helpers.
