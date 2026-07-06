# SURF-dependent-constructors-gadt

**Owner:** Spec enclave for D0/spec/conformance elaboration. Later build owner
will be Language, with Kernel only if the spec proves a missing kernel
admission or checking rule.
**Reviewer:** Architect mandatory. Conformance-validator mandatory.
**Branch:** `wp/SURF-dependent-constructors-gadt` for the enclave frame.
**Status:** Steward frame. Staged for enclave elaboration after the handoff
gate.
**Size:** L. **Risk:** high; broad surface feature with soundness-adjacent
coverage, positivity, and elaboration obligations.

## 0. Trigger

During the CAT-5 D3 D0 stop, Language found that an evidence-carrying
`Source` constructor shape would require constructor signatures of the form:

```ken
MkCheckedSource :
  (sid : SourceId) -> (bs : Bytes) -> ... -> Source
```

The current concrete parser does not accept that form. Architect ruled at
`evt_7rq52cgqsphz8` that the immediate CAT-5 unblocker is the narrower
`KM-sigma-projection-execution` mechanism, not this feature. Architect also
noted that broad GADT/dependent-constructor syntax is a real future language
feature.

This WP stages that broader feature deliberately, without making it a CAT-5 D3
workaround.

## 1. Objective

Elaborate the surface language contract for dependent constructor syntax and
GADT-like declarations so Ken's surface can express constructor telescopes with
explicit binders and result indices, while preserving the existing kernel
discipline for inductive families, strict positivity, eliminator generation,
and index-aware pattern coverage.

The output is a spec/conformance-ready feature frame, not a build patch. It
should make clear which already-specified semantics in `spec/10-kernel/14` and
`spec/30-surface/34` are already settled, which surface syntax is missing, and
which implementation WPs are needed after the enclave merge.

## 2. Fixed Inputs

- `spec/10-kernel/14-inductive.md` already specifies inductive families,
  constructor telescopes, varying indices, strict positivity, W-style
  Π-bound recursive occurrences, and dependent eliminators.
- `spec/30-surface/34-data-match.md` already commits to indexed/GADT-like
  families, index-aware exhaustiveness, and omission of index-impossible arms
  by elaborator-synthesized absurd methods.
- The current parser path is narrower:
  `crates/ken-elaborator/src/parser.rs::parse_data_decl` parses
  `data D ... = C type_atom* | ...`, and `parse_ctor_decl` parses constructor
  argument atoms only.
- CAT-5 D3 must not wait on this broad feature; its immediate route is
  `KM-sigma-projection-execution`.

## 3. Scope

The enclave should specify:

- concrete surface grammar for dependent constructor binders and explicit
  constructor result types;
- how parameters and indices are declared at the `data` head;
- how constructor telescopes bind earlier arguments for later argument types;
- how result-index expressions are scoped and checked;
- the elaboration target: kernel inductive-family declarations and generated
  eliminators, not opaque encodings;
- the admissibility boundary inherited from `14`: strict positivity, W-style
  admitted, nested/mutual still deferred unless explicitly re-opened;
- index-aware pattern coverage and diagnostics for GADT-like scrutinees;
- minimal examples, including `Vec`, a typed-expression family, and a
  proof-carrying checked-source-style constructor.

Out of scope:

- implementing the parser, elaborator, checker, or interpreter changes;
- changing the kernel's inductive admission rules unless D0 proves the existing
  kernel contract is insufficient;
- nested inductives, mutual inductives, or datatype universe-polymorphism beyond
  what `14` already admits;
- smart constructors/views as an invariant-hiding feature;
- using this WP to weaken or unblock CAT-5 D3.

## 4. Required D0 Questions

The spec enclave must answer these before authoring the final spec text:

1. What exact syntax should Ken accept for a constructor with an explicit
   dependent telescope and result type?
2. Is the `data` head syntax in `34 §2` sufficient for parameters and indices,
   or does it need a concrete grammar addition in `32`?
3. Which constructor-result forms are allowed: only the declared family at
   applied parameters and indices, or a broader definitional-equality class?
4. How are implicit arguments, named arguments, and record-style constructor
   fields staged relative to the initial feature?
5. What diagnostic should report a constructor whose result target is not an
   instance of the declared family?
6. What negative conformance case pins that type-possible constructors cannot
   be omitted, while index-impossible constructors may be omitted?

If any answer affects the kernel admission boundary, route Architect before the
spec draft is declared review-ready.

## 5. Acceptance Criteria

- **AC1 -- syntax pinned.** `spec/30-surface/32-grammar.md` and/or
  `34-data-match.md` define the accepted dependent-constructor syntax,
  including binder scoping and explicit result-type grammar.
- **AC2 -- lowering pinned.** The spec states how the surface lowers to
  kernel inductive-family declarations from `14`, including constructor
  telescopes and result indices.
- **AC3 -- positivity and stage boundary preserved.** The spec keeps strict
  positivity as the admission gate, keeps W-style behavior aligned with K1.5,
  and explicitly leaves nested/mutual forms deferred unless the enclave
  deliberately re-opens them.
- **AC4 -- coverage semantics pinned.** GADT/index-aware match coverage states
  how type-possible and index-impossible constructors are distinguished, and
  how omitted impossible methods are synthesized by absurdity.
- **AC5 -- conformance seeds discriminate.** Add or update conformance seeds
  with at least one positive indexed-family declaration, one positive
  proof-carrying constructor shape, one bad constructor result target, one
  negative positivity case, and one coverage case that distinguishes omitted
  impossible from omitted possible arms.
- **AC6 -- implementation WPs named.** The enclave output names the build WPs
  required after spec merge: parser/AST, elaboration to kernel inductives,
  coverage diagnostics, and any mechanism prerequisites discovered by D0.
- **AC7 -- CAT-5 separation explicit.** The final text states that CAT-5 D3 is
  not blocked on this broad feature and that `Source` remains governed by the
  existing CAT-5 contract.

## 6. Guardrails

- Do not consult build-team members for spec grounding; grep landed code or
  route cross-lane judgment to Architect.
- Do not rely on reference implementations. Work from Ken's spec, conformance,
  repo code, and Ken-owned event facts.
- Do not make broad dependent constructors a shortcut around class-backed record
  execution. That is `KM-sigma-projection-execution`.
- Do not over-claim buildability. The enclave should separate settled semantics
  from implementation slices and name every capability axis the build will
  touch.

## 7. Sequencing

This WP is staged for the spec enclave as the next broad surface elaboration
item. It may run in parallel with the narrow CAT-5 prerequisite mechanism
because it does not change the CAT-5 route. It should merge before any Language
build team starts parser/elaborator implementation for this feature.
