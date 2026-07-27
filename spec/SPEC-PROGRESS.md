# Ken specification — chapter status index

This file is the human-readable reconciliation index for the Ken
specification. It records what each chapter says about its own provenance or
delivery stage and how strongly that chapter says its text binds. Sequencing
readers use those declarations and their stated scope; this index does not
invent a separate maturity judgment.

The chapters remain authoritative for their own declarations. This index is a
measured view of all 63 Markdown files under `spec/` at the reconciliation
point. It is not a build tracker, a next-action queue, or a substitute for the
work-package nodes under `docs/program/issues/`.

## Status convention

This index adopts **shape B: provenance/stage plus binding force**. The former
`TODO · DRAFT · REVISED · DONE` ladder is retired because no rule derives
drafting maturity from declarations such as `K1 elaborated`, `impl-ready (L2)`,
or scope-qualified `Normative`. In particular, an `elaborated` work-package
marker says who or what elaborated a chapter; it does not prove that the
chapter passed through a hidden `REVISED` rung.

Shape A would require that unsupported maturity mapping. Shape C would call
this table derived without supplying a derivation mechanism, and a
declaration-only derivation would omit the three status-less files. Shape B
instead keeps one explicit, total inventory while using only quantities the
chapter declarations actually state.

Apply the convention as follows:

1. Read the declaration's complete opening status sentence, not its first
   Markdown-emphasized span. Record every clause in that sentence that
   predicates a provenance or delivery stage of the chapter or one of its
   named sections. Stage predicates are `DRAFT vN`, `elaborated`, a WP that
   `extends` or `completes` a fragment, `impl-ready` or
   `implementation-ready`, `contract-pinned`, `implementation-gated`,
   `CAPSTONE COMPLETE`, and an explicitly numbered audit phase. Preserve
   adjacent qualifiers that identify the WP, section, series, or delivery
   scope. Normalize multiple clauses into reading order with semicolons; do
   not translate one predicate into another.
2. Read binding force from the same declaration. Statements that text is
   normative, fixes or pins a contract, records a settled rule, or defines an
   operative boundary bind at the scope they name. Proposal-level, starter,
   `(oracle)`-tagged, deferred, and implementation-gated portions remain
   qualified exactly as the declaration says.
3. A `DRAFT` marker does not weaken an explicitly normative contract, and an
   `impl-ready` or `elaborated` marker does not strengthen a proposal-level
   spelling.
4. `Normative`, proposal-level, and operator-facing role statements belong to
   the binding-force axis, not the provenance/stage axis. If a declaration
   supplies force but no stage predicate, record `not declared` for the first
   axis. If a file has no `> Status:` declaration, assign neither axis by
   inference and record it in the unclassifiable report.

This convention has no unreachable rung: it preserves declarations that
chapters actually use and states their force independently. It adds no checker
or generated-status mechanism.

## Per-file inventory

The middle column applies the semantic provenance/stage rule to each file's
own `> Status:` declaration. The final column applies the binding-force rule
to that same declaration, including its qualifications.

| File | Declared provenance/delivery stage | D1 binding-force assignment |
|---|---|---|
| `00-overview.md` | `DRAFT v0` | Normative for terminology and scope; subject chapters prevail on disagreement. |
| `10-kernel/11-syntax.md` | `K1 elaborated` | Normative for kernel grammar and scoping; typing is assigned to `13`–`16` and `18`. |
| `10-kernel/12-universes.md` | `K1 elaborated` | Normative for the universe hierarchy, checking, and level polymorphism; K1 scope is §§1–4 and §6. |
| `10-kernel/13-pi-sigma.md` | `K1 elaborated` | Normative for Π and Σ rules and K1-scoped conversion. |
| `10-kernel/14-inductive.md` | `K1 elaborated; K1.5 extends it; nested-positive specified and implementation-gated` | Normative for inductives, positivity, and elimination; nested-positive implementation remains gated on `KERNEL-NESTED-IND`. |
| `10-kernel/15-identity.md` | `K2 elaborated` | Normative for the observational-equality interface. |
| `10-kernel/16-observational.md` | `K2 elaborated; K2c series-2 completes the three obs-reduction seams (§3.2 inductive index rewrite, §4.1 non-constant-motive J, §5.1 full quotient respect); K5 completes the observational fragment` | Normative for the observational interface, computation rules, and reduction behavior; `(oracle)` rules retain that qualification. |
| `10-kernel/17-conversion.md` | `K2c elaborated (series 1 — conversion-hardening)` | Normative for kernel conversion and the SCT gate. |
| `10-kernel/18-judgments.md` | `K-api elaborated; implementation-ready` | Normative kernel API contract; its freeze claim is conditional on the declared landed admission gates. |
| `10-kernel/18a-primitive-registry.md` | `Phase 1 — registry audit (WS-K / BUILTINS)` | Conditional native registry seal: an operation is native iff its stated correctness AC holds. |
| `10-kernel/README.md` | `K2 elaborated` | Normative WS-K contract and WS-V re-checking target. |
| `20-verification/21-spec-syntax.md` | `V1 elaborated; implementation-ready` | Normative for verification forms, meaning, elaboration, and status model; concrete spelling is cross-referenced. |
| `20-verification/22-obligations.md` | `V2 elaborated; implementation-ready` | Normative for obligations and their extraction algorithm. |
| `20-verification/23-prover.md` | `V3 elaborated; implementation-ready` | Normative for the prover contract, verdicts, classifier, and certificate route; named frame axioms remain `(oracle/standard)`. |
| `20-verification/24-diagnostics.md` | `V4 elaborated; implementation-ready` | Normative for diagnostic mechanisms and meaning; serialization belongs to `25-protocol.md`. |
| `20-verification/25-protocol.md` | `T1 elaborated; implementation-ready` | Normative for message shape, verdict cross-walk, and stability; exact JSON names remain a finalization surface. |
| `20-verification/README.md` | `DRAFT v0` | Normative for the verification interface and soundness obligations; named prover internals remain team work. |
| `30-surface/30-taxonomy.md` | not declared | Normative for the built-in/prelude/package boundary and minimality invariant; membership grows through its derivation process. |
| `30-surface/31-lexical.md` | `DRAFT v0` | Settled for `OQ-syntax` principles; the concrete token table is a revisable starter. |
| `30-surface/32-grammar.md` | `DRAFT v0` | Normative intent for which productions exist; exact spelling remains proposal-level. |
| `30-surface/33-declarations.md` | not declared | Normative for features and the named module, visibility, and class contracts; concrete spelling remains proposal-level. |
| `30-surface/34-data-match.md` | `impl-ready (L2)` | Normative for the feature; nested-positive support remains separately staged as declared. |
| `30-surface/35-numbers.md` | `impl-ready (L1)` | Normative for the numeric model and L1 scope; user-defined numeric instancing remains gated on L-classes. |
| `30-surface/36-effects.md` | `L5 elaborated; implementation-ready for Team Language` | Normative for the model and elaboration; concrete spelling remains proposal-level. |
| `30-surface/37-strings-collections.md` | `impl-ready (L3)` | Binding for concepts, laws, lowering, and staging; exact API spelling and internal representations remain `(oracle)`-tagged. |
| `30-surface/38-ffi-io.md` | `§1.1–§1.6 impl-ready (L6); §1.7 contract-pinned (PX8-T); §2–§4 impl-ready (L7); elaborated to team-ready rigor` | Normative for trust and effect discipline with the stated per-section delivery stages. |
| `30-surface/39-elaboration.md` | `DRAFT v0; §5 (V0) elaborated to implementation rigor for the G1 minimal slice` | Normative for elaboration output and guarantees; §5 is V0 implementation-rigorous and the other named sections remain frame-level. |
| `30-surface/README.md` | `DRAFT v0` | Normative for constructs and meaning; concrete syntax remains an explicitly revisable proposal. |
| `40-runtime/41-values.md` | `Elaborated (X2 contract)` | Normative for values, equality, callable boundaries, and durable canonical encoding; in-process representation is private. |
| `40-runtime/42-evaluation.md` | `X1 elaborated; implementation-ready for Team Runtime; pure-core G1 scope` | Normative reference-evaluation contract for the declared pure-core and effect-driver scope. |
| `40-runtime/43-termination.md` | `DRAFT v0` | Normative for the totality/partiality boundary. |
| `40-runtime/44-capacity.md` | `Elaborated (X2 contract)` | Normative for observable capacity failure, logical isolation, and semantics-invisible reclamation; storage organization is private. |
| `40-runtime/45-native-backend.md` | `DRAFT v0 (spec-design)` | Binding backend design for model, trust posture, and differential discipline; target choice remains open. |
| `40-runtime/46-checked-core-package.md` | `DRAFT v0 (NC1)` | Normative for the stable checked-core compiler input and its stated exclusions. |
| `40-runtime/47-erasure-runtime-ir.md` | `DRAFT v0 (NC5)` | Normative for erasure, runtime IR, loud refusal, and interpreter comparison; backend details are excluded. |
| `40-runtime/48-executable-artifact-contract.md` | `DRAFT v0 (NC19)` | Normative for the executable-artifact contract and loud rejection rules; emission and ABI details are excluded. |
| `40-runtime/README.md` | `DRAFT v0` | Normative for the value/equality/callable/reference-semantics contract; private representation choices remain outside it. |
| `50-stdlib/51-lawful-classes.md` | `DRAFT v0 (ES4-classes)` | Binding pattern for lawful catalog entries: ordinary Ken, proved laws, and zero trusted-base delta. |
| `50-stdlib/52-map.md` | `DRAFT v0 (VAL2 #8 / OQ-A)` | Binding operator-locked Map shape and proof discipline; the named staged proof history remains scoped as declared. |
| `50-stdlib/53-transport.md` | `DRAFT v0 (surface-transport WP, Map Gap A)` | Binding library contract for five derived equality combinators over `J`; no new former or trusted-base entry. |
| `50-stdlib/54-map-verified-laws.md` | `DRAFT v3 — CAPSTONE COMPLETE: all five inductive laws landed` | Binding proof-strategy and landed-capstone record for five inductive laws; permutation remains explicitly out of scope. |
| `50-stdlib/55-lawful-functors.md` | `DRAFT v0 (CAT-1)` | Binding CAT-1 pattern for value algebra and constructor classes; no new kernel feature. |
| `50-stdlib/56-effectful-classes.md` | `DRAFT v0 (CAT-2)` | Binding CAT-2 contract with the declared SURF-1/SURF-2 gates for Traversable. |
| `50-stdlib/57-collections-and-views.md` | `DRAFT v0 (CAT-3)` | Binding CAT-3 contract for collection laws and views; build work remains staged as declared. |
| `50-stdlib/58-maps-sets-relations.md` | `DRAFT v0 (CAT-4)` | Binding CAT-4 contract for keyed collections, sets, and the relations frontier; build scope remains split as declared. |
| `50-stdlib/59-parsing-syntax-diagnostics.md` | `DRAFT v0 (CAT-5)` | Binding CAT-5 package contract with the stated compiler/reflection exclusions. |
| `50-stdlib/60-length-indexed-vectors.md` | not declared | Normative for the Vec family and landed operations; `zip` and `lookup` remain gated on `DS-5c`. |
| `50-stdlib/README.md` | `DRAFT v0` | Binding standard-package-tier boundary and derivation-path requirement. |
| `60-security/61-information-flow.md` | `Sec1 + Sec1ct elaborated; implementation-ready for WS-Sec` | Normative for the declared IFC and constant-time discipline; surface spelling remains proposal-level. |
| `60-security/62-authority.md` | `Sec2 elaborated; implementation-ready for Team Verify (WS-Sec)` | Normative for authority, attenuation, revocation contract, and audit points; spelling and named runtime mechanisms remain qualified. |
| `60-security/63-supply-chain.md` | `DRAFT v0` | Normative for consumption and artifact shape under the declared provenance decision. |
| `60-security/64-trust-model.md` | not declared | Normative for the TCB, security reading, trusting-trust invariant, and honest limits. |
| `60-security/65-policy.md` | not declared | Normative for policy shape, binding guarantee, and semantics; concrete policy syntax remains deferred. |
| `60-security/README.md` | `DRAFT v0` | Normative security frame, threat model, guarantees, and limits. |
| `70-behavioral/71-assumption-boundary.md` | `impl-ready (B1)` | Normative for projection, schema, and seam discipline; literal wire spellings remain `(oracle)`-tagged. |
| `70-behavioral/72-temporal.md` | `impl-ready (B2)` | Normative for the temporal-as-data discipline and export flow; listed encoding spellings remain `(oracle)`-tagged. |
| `70-behavioral/73-conformance.md` | `impl-ready (B3)` | Normative for Ken's runtime-seam half; downstream checking policy and literal wire spellings remain outside or qualified. |
| `70-behavioral/74-agentic.md` | `impl-ready (B4); WS-B capstone` | Normative for the agentic-assurance reduction; it adds no new agentic mechanism. |
| `70-behavioral/README.md` | `DRAFT v0` | Normative for Ken's half of the behavioral seam. |
| `90-open-decisions.md` | not declared | Operator decision register; draft recommendations do not resolve the genuine forks they record. |
| `README.md` | no declaration | Unclassifiable: no provenance/stage or binding force is declared. |
| `SPEC-PROGRESS.md` | no declaration | Unclassifiable: this index has no chapter self-declaration. |
| `_notes/analysis-digest.md` | no declaration | Unclassifiable: the background note has no status declaration. |

## Unclassifiable report

Exactly three inputs lack a `> Status:` declaration and therefore receive no
inferred axis value:

- `README.md` — the spec index declares neither provenance/stage nor binding
  force.
- `SPEC-PROGRESS.md` — this reconciliation index is not a self-declaring
  chapter.
- `_notes/analysis-digest.md` — the background design note declares no status.

The other 60 inputs are classifiable by the operative rule. This report does
not repair or add chapter declarations; those declarations are inputs to this
reconciliation.

## Reconciliation controls

- **Different declaration shapes:** `10-kernel/18-judgments.md` records both
  `K-api elaborated` and its suffix qualifier `implementation-ready`, then
  binds the kernel API contract; `30-surface/34-data-match.md` records
  `impl-ready (L2)` and binds its feature; `60-security/65-policy.md` has a
  force-only `Normative` declaration, so its first axis is `not declared`
  while its policy shape, guarantee, and semantics bind and syntax stays
  deferred. Each result follows the same two-axis rule.
- **Selection controls:** `10-kernel/16-observational.md` retains the K5 clause
  outside its first bold span; `30-surface/39-elaboration.md` retains its
  section-specific V0 clause; `50-stdlib/57-collections-and-views.md` retains
  the adjacent `(CAT-3)` qualifier. Markdown emphasis selects none of them.
- **Changed and retained rows:** `10-kernel/11-syntax.md` previously read
  `DRAFT` in this index; its chapter declaration yields `K1 elaborated` plus
  normative grammar/scoping. `00-overview.md` previously read `DRAFT`; its
  declaration retains the `DRAFT v0` marker while exposing its normative
  terminology/scope. Neither result is copied from the old row.
- **No `REVISED` inheritance:** the retired ladder and its unused `REVISED`
  rung do not appear in the operative vocabulary.
- **No gate:** this reconciliation changes one Markdown artifact. It adds no
  CI configuration, script, test, or documentation-content oracle.
