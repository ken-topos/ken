# DOC-CATALOG-CONTENTS Preflight Record

> **Analysis only.** This record does not amend the work-package frame,
> requirements, acceptance criteria, exclusions, or scope. The catalog change
> remains held until the preceding Language-owned oracle repair lands and the
> Steward supplies a new base.
>
> **Editing constraint:** this file must not spell the retired standalone-proof
> keyword in any case form. The whole-tree source oracle scans prose as well as
> checked source and would classify each spelling as a new occurrence.

## Fixed Inputs

The preflight is bound to
`f3863b48474100f162ffbb24514399494b1ea198`. The named governing objects at that
commit are:

- frame `docs/program/wp/DOC-CATALOG-CONTENTS-index-to-contents.md`, blob
  `e69af5a362240c849e1d1b4df0993c6efd30bafe`;
- node `docs/program/issues/DOC-CATALOG-CONTENTS.md`, blob
  `74c6fb467a1b15760ce0d999f619488292958bd7`;
- campaign `docs/program/06-catalog-campaign.md`, blob
  `fa34e72999f763a1b6c4d109401ac947077de39e`.

The Librarian derived the populations directly from that tree. Counts below are
fixed-base measurements, not counts inherited from the frame.

## Heading Population

The denominator is every exact `^## Index$` heading under `catalog/`, selected
with `git grep` against the fixed commit. The population is 19 headings in 19
files:

1. `catalog/guide/decomposition-abstraction.ken.md`
2. `catalog/guide/proof-techniques.ken.md`
3. `catalog/guide/surface-reference.ken.md`
4. `catalog/packages/Capability/Formatting/Doc.ken.md`
5. `catalog/packages/Capability/Parsing/Numeric.ken.md`
6. `catalog/packages/Capability/Parsing/Parsing.ken.md`
7. `catalog/packages/Core/Classes/EffectfulClasses.ken.md`
8. `catalog/packages/Core/Classes/LawfulClasses.ken.md`
9. `catalog/packages/Core/Classes/LawfulFunctors.ken.md`
10. `catalog/packages/Core/Logic/EmptyDec.ken.md`
11. `catalog/packages/Core/Logic/Transport.ken.md`
12. `catalog/packages/Data/Collections/Derived.ken.md`
13. `catalog/packages/Data/Collections/Map.ken.md`
14. `catalog/packages/Data/Collections/NonEmpty.ken.md`
15. `catalog/packages/Data/Numeric/Nat/Order.ken.md`
16. `catalog/packages/Data/Sums/Combinators.ken.md`
17. `catalog/packages/Data/Sums/Validation.ken.md`
18. `catalog/packages/Data/Text/Codec.ken.md`
19. `catalog/packages/Tooling/Testing/Property.ken.md`

A second, exact-case `Index` scan found 22 hits. Fence state was derived by
walking each Markdown fence rather than inferred from line position:

- the 19 headings above are prose and are rename targets;
- `Derived.ken.md:1155`, `class IndexedView A {`, is inside a Ken fence;
- `Derived.ken.md:1234`, `instance IndexedView Unit {`, is inside a Ken fence;
- `Derived.ken.md:1439` is prose naming the semantic identifier `IndexedView`.

The last three hits are intentional leaves. All three must remain byte-identical
to the fixed base. A case-insensitive `index` scan is not a safe rename operand:
it also selects ordinary source variables, diagnostic positions, indexed-family
prose, and spec-index references.

## Reading-Path Population

The denominator is the union of two exact section forms under `catalog/`:

- 14 `**Named reading paths**` blocks;
- 2 `## Reading paths` sections.

Each extent was derived from its exact opener through the line before the next
H2 heading. The 16 distinct files are:

1. `catalog/packages/Capability/Parsing/Numeric.ken.md`
2. `catalog/packages/Capability/Parsing/Parsing.ken.md`
3. `catalog/packages/Core/Classes/EffectfulClasses.ken.md`
4. `catalog/packages/Core/Classes/LawfulClasses.ken.md`
5. `catalog/packages/Core/Classes/LawfulFunctors.ken.md`
6. `catalog/packages/Core/Logic/EmptyDec.ken.md`
7. `catalog/packages/Core/Logic/Transport.ken.md`
8. `catalog/packages/Data/Collections/Derived.ken.md`
9. `catalog/packages/Data/Collections/Map.ken.md`
10. `catalog/packages/Data/Collections/NonEmpty.ken.md`
11. `catalog/packages/Data/Numeric/Nat/Order.ken.md`
12. `catalog/packages/Data/Sums/Combinators.ken.md`
13. `catalog/packages/Data/Sums/Validation.ken.md`
14. `catalog/packages/Data/Text/Codec.ken.md`
15. `catalog/guide/README.md`
16. `catalog/packages/Tooling/Testing/Property.ken.md`

The union of heading and reading-path edits is 20 catalog files: the 19 heading
files plus `catalog/guide/README.md`. The Property entry belongs to both
populations.

## Format Consumers

The consumer denominator is the four documents named by the frame, followed by
a case-insensitive `index|reading path|navigation` scan and contextual review.
That method yields 17 live site groups:

- `docs/program/06-catalog-campaign.md`: five live format or layout assertions
  at fixed-base lines 89, 183, 188, 195, and 350;
- `docs/program/07-catalog-style-guide.md`: six live format or checklist sites
  at lines 40–48, 389, 405, and 423;
- `library/agents/tasks/author-package.md`: four site groups at lines 18, 28,
  29, and 39;
- `library/learn/reading-ken/01-anatomy.md`: two wrapped assertions at lines
  16–17 and 42–43.

The following are intentional leaves rather than live format assertions:

- `DOC-CATALOG.RQ-5` and its residual discussion in
  `06-catalog-campaign.md:31–40` retain the history of the removed mechanism;
- `07-catalog-style-guide.md:242–243` retains the standard-library spec-index
  reference;
- generic navigation prose remains where it does not assert the retired entry
  shape;
- `agent/playbooks/tools/write-ken.md` has zero catalog-format facts and remains
  unchanged under documentation-program decision D3;
- `agent/playbooks/tools/library-style.md` governs prose presentation, not the
  catalog schema, and remains unchanged;
- historical records remain unchanged.

## Nominal Product Closure

Before the oracle coupling was found, the complete candidate denominator was 28
paths:

- 20 catalog files from the union above;
- the four format consumers;
- `library/SOURCE-ATTESTATIONS`;
- `library/REVISION`;
- `library/STATUS.md`;
- `library/agents/manifest.toml`.

`library/manifest.toml`, the two tool playbooks, `spec/50-stdlib/README.md`, all
Ken fence payloads, and historical records are inert controls. This preflight
file is a separate analysis checkpoint and is not part of that 28-path product
denominator.

## Derived Artifacts

Exactly 12 source-attestation rows are expected to move after truth review:

1. `catalog/guide/decomposition-abstraction.ken.md`
2. `catalog/guide/proof-techniques.ken.md`
3. `catalog/guide/surface-reference.ken.md`
4. `catalog/packages/Core/Classes/EffectfulClasses.ken.md`
5. `catalog/packages/Core/Classes/LawfulClasses.ken.md`
6. `catalog/packages/Core/Classes/LawfulFunctors.ken.md`
7. `catalog/packages/Core/Logic/EmptyDec.ken.md`
8. `catalog/packages/Core/Logic/Transport.ken.md`
9. `catalog/packages/Data/Numeric/Nat/Order.ken.md`
10. `catalog/packages/Data/Sums/Combinators.ken.md`
11. `catalog/packages/Tooling/Testing/Property.ken.md`
12. `docs/program/07-catalog-style-guide.md`

The review order is load-bearing:

1. prove all Ken fence payloads byte-identical to the fixed base;
2. revalidate each cited claim and anchor through
   `library/manifest.toml` and the agent manifest;
3. reconcile the changed format statement with its direct consumers;
4. only then render and inspect the proposed ledger;
5. install exactly the reviewed 12-row change;
6. refresh `library/REVISION` and generated `library/STATUS.md`;
7. prove the status file carries the new ledger digest.

The agent token-measure closure has exactly two expected changes:

- module `tasks/author-package`;
- transitive pack `author-package`.

Every other module and pack measurement is an inert control.

## Gate and Oracle Population

The governing enumeration rule is:

> Enumerate every oracle whose population could include the paths being changed,
> including whole-tree oracles. Do not restrict the search to oracles described
> as belonging to the target corpus.

This is wider than listing catalog gates. A whole-tree oracle belongs to every
path's blast radius even when its name mentions no particular corpus.

The local product and derived-artifact checks are:

- `scripts/gen-source-attestations.sh`, which writes only the proposed ledger;
- `scripts/gen-doc-status.sh --check`;
- `scripts/ken-cargo test -p ken-cli --test library_documentation_gates`, which
  checks manifest coverage and kinds, links and source anchors, source currency,
  generated status, checked library fences, agent schema and graph integrity,
  and module and transitive-pack token measurements.

The publisher runs `gen-doc-status.sh --check` against the synthetic merge tree
before publication and again before merge. CI's full test shards include these
checks and the following catalog or whole-tree consumers:

- `crates/ken-cli/tests/ken_fmt.rs`:
  `strict_frozen_corpus_gate_is_green` and the runnable-root authority gate;
- `crates/ken-elaborator/tests/kenfmt_b1_lossless.rs`: whole-catalog lossless
  round-trip;
- `crates/ken-elaborator/tests/kenfmt_b3_layout.rs`: whole-catalog parse,
  idempotence, and width;
- `crates/ken-elaborator/tests/kenfmt_b4_splicing.rs`: whole literate-corpus
  prose and fence preservation;
- `crates/ken-elaborator/tests/kenfmt_c_capstone.rs`: canonical fixed point and
  balanced-layout corpus checks;
- `crates/ken-elaborator/tests/catalog_taxonomy.rs`: controlled package roots;
- `crates/ken-elaborator/tests/seal2_producer_closure.rs`: whole-package glob and
  source-root confinement certificate;
- `crates/ken-elaborator/tests/kw_theorem_source_oracle.rs`: whole tracked tree
  and exact occurrence census.

Thirty-three exact-path test binaries also consume one or more touched catalog
sources. Their source behavior is carried by the every-fence byte-identity
control, and CI executes them:

- `crates/ken-cli/tests/rosetta.rs`;
- `cat1_lawful_functors_package.rs`;
- `cat3_collections_package.rs`;
- `cat5_parsing_package.rs`;
- `cc1_nonempty_validation_acceptance.rs`;
- `cc2_text_codec_numeric_acceptance.rs`;
- `cc3_parsing_cursor_decoder_acceptance.rs`;
- `cc4_diagnostic_core_acceptance.rs`;
- `cc5_pretty_doc_acceptance.rs`;
- `cc6a_process_arguments_exit_acceptance.rs`;
- `cc6b_path_posix_acceptance.rs`;
- `cc7_argparse_acceptance.rs`;
- `cc8_env_config_decoder_acceptance.rs`;
- `compare_ord_lexicographic_acceptance.rs`;
- `ds1_empty_dec_acceptance.rs`;
- `ds2_ord_nat_acceptance.rs`;
- `ds3_sum_combinators_acceptance.rs`;
- `ds4_list_combinators_acceptance.rs`;
- `ds6a_int_deceq_acceptance.rs`;
- `ds7_applicative_monad_acceptance.rs`;
- `ds8_traversable_acceptance.rs`;
- `either_catalog_package_acceptance.rs`;
- `es2_acceptance.rs`;
- `es4_classes_acceptance.rs`;
- `kenfmt_signature_layout.rs`;
- `kw_theorem_source_oracle.rs`;
- `l3_strings_surface_acceptance.rs`;
- `map_build_acceptance.rs`;
- `nat_arithmetic_laws_acceptance.rs`;
- `structural_deceq_acceptance.rs`;
- `sub1_bytes_structural_view.rs`;
- `sub1b_uint8_deceq.rs`;
- `surface_transport_acceptance.rs`.

All paths in that list after `rosetta.rs` are under
`crates/ken-elaborator/tests/`.

## Whole-Tree Coupling and Hold

The Language-owned source oracle builds an exact census keyed by
`(path, line)`, then compares it for equality against a hard-coded allow-list.
Its input is every tracked UTF-8 blob selected from the committed `HEAD` tree.
The allow-list freezes 64 rows in 18 files across six top-level trees:

- `agent/memory/fleet/a-vacuous-law-has-zero-trust-delta.md`;
- `catalog/guide/proof-techniques.ken.md`;
- `catalog/guide/surface-reference.ken.md`;
- `catalog/packages/Core/Classes/EffectfulClasses.ken.md`;
- `catalog/packages/Data/Collections/Derived.ken.md`;
- `catalog/packages/Data/Collections/Map.ken.md`;
- `catalog/packages/Data/Numeric/Nat/Order.ken.md`;
- `conformance/surface/declarations/seed-named-proof-claims.md`;
- `docs/program/07-catalog-style-guide.md`;
- `docs/program/diary/2026/Jul/21.md`;
- `docs/program/wp/KM-sigma-eq-pair-refl.md`;
- `docs/program/wp/SPAN-SEAL-buffer-span-producer-closure.md`;
- `docs/program/wp/SURF-named-proof-claims.md`;
- `docs/program/wp/let3-catalog-let-pilot.md`;
- `docs/program/wp/str-bij-overclaim-erratum.md`;
- `library/learn/reading-ken/02-types-contracts-and-proofs.md`;
- `spec/30-surface/33-declarations.md`;
- `spec/30-surface/38-ffi-io.md`.

Six catalog files in this work package are pinned by that allow-list.
`proof-techniques.ken.md` and `surface-reference.ken.md` receive line-neutral
heading replacements. Reading-path deletion shifts pinned rows in
`EffectfulClasses.ken.md`, `Derived.ken.md`, `Map.ken.md`, and `Nat/Order.ken.md`.
The style-guide edit also shifts pinned rows. A normal product edit therefore
makes the source oracle red for a reason unrelated to the format property.

The Steward denied both a `crates/` re-canonicalization in this work package and
line-preserving padding. The durable remedy is a preceding Language-owned oracle
repair. The catalog work package remains held, no candidate exists, and no merge
Decision is authorized. After the oracle repair lands, the Steward supplies a
new base and the product delta is replayed and reviewed against that tree.

## Post-Repair Review Map

On the resumed exact candidate, Librarian review will:

1. bind candidate, parent, fixed replacement base, and complete path scope;
2. rederive the 19-heading, 16-section, and four-consumer populations;
3. prove the 22 exact-case hit classifications and the three `IndexedView`
   leaves;
4. prove every Ken fence payload in every touched file byte-identical;
5. verify the RQ-5, spec-index, workflow-skill, semantic-index, and historical
   leaves;
6. truth-review all moved cited sources before accepting the 12 ledger rows;
7. verify exactly two token-measure changes and all other measures invariant;
8. prove the generated status digest and current revision anchor;
9. run the targeted library, formatter, catalog, and whole-tree oracle checks;
10. inspect current-main path intersection and the synthetic merge result;
11. return an exact-SHA verdict for the `library/` domain and route catalog-lane
    authority separately under federation law.
