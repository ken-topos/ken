# Implementation progress — the build backbone

**Owned by the Steward** (`agent/playbooks/federation/steward.md §2a`). This
file tracks execution **against the implementation DAG**
(`05-implementation-dag.md`), the build's analog of `spec/SPEC-PROGRESS.md`.
It **survives compaction**: on a cold start or after a compact, read this
first, then continue from the frontier (below). Update it **every synthesis
pass and on every WP state change**. The plan lives in `05`; this file
tracks *progress against it*. Run until complete, blocked, or instructed
(§2b).

**This file holds CURRENT STATE ONLY, and it is GENERATED** — edit
`docs/program/issues/*.md` and re-run `scripts/gen-progress.sh`; hand edits
here are overwritten. The full chronicle — every prior "live state"
snapshot, the detailed evidence trail for every merged WP, and the
day-by-day session logs back to project start — lives in
[`diary/`](diary/INDEX.md). If you need *why* a past call was made, or the
mechanism detail behind a closed WP, start there;
[`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) carries the live
operator briefing and the Steward's resume state.

**Status legend:** `draft` (not framed / deps unmet) · `ready` (deps met,
unassigned) · `active` (a team is building) · `in-review` (PR open / QA / CI)
· `merged` (landed + retro in) · `closed` (resolved without landing, e.g. a
superseded or withdrawn item). Gates: see `05-implementation-dag.md`.

**★ GENERATED FILE — do not hand-edit.** This file is regenerated from the
frontmatter of every `docs/program/issues/*.md` work-item file by
`scripts/gen-progress.sh`. To change tracked status, edit the relevant
`docs/program/issues/<ID>.md` file and re-run the generator. CI checks that
the committed file matches the generator's output.

## Last generated

2026-07-29 14:42:07Z — from 127 issue file(s) in `docs/program/issues/`.

## Work-item status

| ID | Title | Status | Owner | Size | Gate | GitHub |
|---|---|---|---|---|---|---|
| `A3` | catalog-coverage walker | draft | TBD | TBD | none | — |
| `ABI-A1` | promote ConsoleRead and ClockWallNow to NativeTested with differential evidence | draft | runtime | M | none | — |
| `ABI-A2` | promote FsAppendFile, FsMetadata, FsRename to NativeTested | draft | runtime | M | none | — |
| `ABI-A3` | promote FsReadDirectory, FsCreateDirectory, FsRemoveFile, FsRemoveDirectory to NativeTested | draft | runtime | M | none | — |
| `ABI-M1` | manifest v2 — family-scoped, versioned, generated from family schemas | draft | runtime | L | none | — |
| `ABI-M2` | runtime facility/operation probes, distinct from build-time facts | draft | runtime | M | none | — |
| `ABI-R1` | correct stale filesystem capability prose — scoped roots, rights, symlink policy and no-follow resolution have landed | closed | foundation | S | none | — |
| `ABI-R3` | generated operation inventory derived from catalog structure — a new operation must be a build break | draft | runtime | M | none | — |
| `ABI-REVOKE` | runtime revocation membrane — the deferred runtime face of 62 §4 | draft | runtime | TBD | none | — |
| `ABI-S1` | descriptor completion — seek, truncate, sync/data-sync, flags, duplication under explicit inheritance policy | draft | runtime | M | none | — |
| `ABI-S2` | directory streaming — supersedes whole-directory read where streaming is the honest shape | draft | runtime | M | none | — |
| `ABI-S3` | monotonic clocks, sleep/deadlines, and secure kernel entropy | merged | runtime | L | none | — |
| `ABI-S4` | statx-shaped metadata with field-availability bits | draft | runtime | M | none | — |
| `ABI-S5` | terminal basics and process signal disposition at the executable edge | draft | runtime | M | none | — |
| `ABI-S6` | ordinary anonymous and file-backed mappings as opaque runtime-owned regions and bounded byte views | draft | runtime | L | none | — |
| `BUDGET-EFF` | TransferCount.remaining must be bounded by the effective request | merged | verify | M | none | — |
| `BUDGET-EXHAUST` | transfer-budget bound checks are fail-open on variant extension | merged | verify | S | none | — |
| `CAT-C2` | Localized Map/Set key-interface split: a non-canonical carrier becomes a lawful Map/Set key under a weaker key-order dictionary while staying an unlawful Ord key wherever antisym concludes kernel Equal | draft | spec-enclave | M | none | — |
| `CAT-CAPEX` | catalog exhibits no checked capability/authority exemplar — write one against the landed Cap/Auth surface | merged | ergo | M | none | — |
| `CB-HYGIENE` | cranelift_backend facade: strip WP-token narration, separate test material from implementation | merged | runtime | S | none | — |
| `CI-SKIPPED-NATIVE-TESTS` | Restore rt_parity_native — dedicated CI job, outlier not fixed | merged | verify | S | none | — |
| `CI-TRACKER-GATE` | Wire the issue-tracker schema + regeneration gate into CI | closed | operator | S | none | 804 |
| `CONF-FMT8-LEVELTOK` | FMT8's fixture is unproducible: the row demands a 'genuine level-token fixture' but the lexer has no Level/Label token kind and never will under endpoint (b) | draft | spec-enclave | S | none | — |
| `CONF-SEC4-REFL-PAIR` | Sec4's C1/C2 refl pair is stale against ADR-0013: the true arm is unreachable and the false arm is green for the wrong reason | draft | spec-enclave | S | none | — |
| `DOC-ATTEST-LIVING` | attesting living tracker files makes every routine WP status flip redden the currency gate | ready | doc | S | none | — |
| `DOC-CAP-ASBUILT` | The capability chapter tells readers the catalog has no checked authority exemplar; CAT-CAPEX adds one, falsifying that claim in two places | draft | doc | S | none | — |
| `DOC-CATALOG-CONTENTS` | Catalog entry format: rename the `## Index` heading to `## Contents` in 19 entries and remove the 16 reading-path sections | merged | doc | M | none | — |
| `DOC-CURRENCY-ANCHOR` | library/REVISION certifies nothing about the corpus — currency is unchecked | closed | doc | S | none | — |
| `DOC-GATE-CONTROL-BINDING` | validation-gate registry: make the two DOC-GATE-RECORD-AXIS checks orphan-proof by lifting them to pure detectors with committed controls | merged | verify | S | none | https://github.com/swe-toolkit/ken/pull/928 |
| `DOC-GATE-NEEDLE` | schema-gate controls assert on a needle the test itself supplied, so one constraint class is fully vacuous | merged | verify | S | none | — |
| `DOC-GATE-RECORD-AXIS` | validation-gate registry: bind token→runner COVERAGE on the record axis, and close the `kind` vocabulary | merged | verify | S | none | https://github.com/swe-toolkit/ken/pull/922 |
| `DOC-GATE-WIRE-BINDING` | validation-gate registry: bind the kind-vocabulary RULE to its GATE by registering it as a VALIDATION_GATES row | merged | verify | XS | none | https://github.com/swe-toolkit/ken/pull/933 |
| `DOC-VALIDATION-BINDING` | validation vocabulary claims a 1:1 binding to the gates; nothing binds it | merged | verify | S | none | — |
| `DOC-W0` | documentation Wave 0 — library/ charter and currency substrate | closed | doc | M | none | 830 |
| `DOC-W1` | documentation Wave 1 — the read-Ken spine, taught from checked fragments | closed | doc | L | none | — |
| `DOC-W2` | documentation Wave 2 — agent core modules, task packs, and cold-context evals | merged | doc | L | none | 936 |
| `DS-9` | lawful JSON codec — the data-structures tier's acceptance test: a Json value type, encode/decode, and the proved round-trip law, assembled entirely from the landed Core/Data sections | draft | foundation | L | none | — |
| `EFF-SPACE-ENSURES-PRESTATE` | `old` is transparent, so a space operation's `ensures` cannot express the pre/post distinction `36 §4.3` is built on | closed | language | M | none | — |
| `F1-37` | F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion | ready | runtime | TBD | none | — |
| `F3-39` | F3 [task-list #39] — reducer: degrade-not-wrap + retire legacy arms | draft | runtime | TBD | none | — |
| `F4` | content-addressing + value-model design (aka PX8-F-PROOF) | draft | foundation+spec-enclave | M | none | — |
| `KERNEL-NESTED-IND` | admit nested strictly-positive inductives in the kernel — structural positivity through declared parameter positions, generated and checked dependent eliminators with one lifted IH per contained recursive occurrence, iota, and surface consumability | active | kernel | L | none | — |
| `KW-ORACLE-CLOSURE` | close the KW-THEOREM source oracle structurally — the occurrence sweep is never applied, and the file population is a five-arm hand enumeration | merged | language | S | none | 986 |
| `KW-ORACLE-REMOVE` | Delete the whole-tree source-text oracle: it asserts facts about repository text, which is now a prohibited test subject | merged | language | S | none | 1035 |
| `KW-THEOREM` | rename the surface keyword `lemma` to `theorem` | merged | language | M | none | — |
| `LIB-GATE-DECOUPLE` | main is red on two library documentation-census gates: the currency gate the operator decoupled from merges still fires from inside CI, and a doc-only merge invalidated the ledger unreported | merged | verify | S | none | 1039 |
| `LOADER-CITE-ANCHOR` | LOADER-STALE-PREMISE cites the spec by line number (:147-158) — rots silently in the one catalog file outside the currency gate | merged | doc | XS | none | — |
| `LOADER-STALE-PREMISE` | \"no disk loader yet\" is stale in 9 places — including already-landed library/ content | merged | doc | S | none | — |
| `MAP-TRANSPORT-CODEC` | If Map/Set need a portable canonical serialization, it is ordinary package Ken — not a runtime primitive: settle whether a codec is required at all, and if so place it out of trusted_base() | closed | ergo | TBD | none | — |
| `MODELS-TIER` | agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception | ready | steward | S | none | — |
| `NATIVE-HANDLE-CARRIER` | Native build-pipeline completeness — a constructor-private resource-carrying handle fails checked-core body-view lowering (MissingClosureMetadata) when it crosses the higher-order withBuffer normalization boundary | ready | runtime | M | none | — |
| `ORACLE-VIS-CHECK` | replace the text-pin oracle in px4b_native_production.rs with a real visibility check | merged | runtime | S | none | — |
| `ORACLE-VIS-PACKAGING` | replace the text-pin visibility oracle on build_process_starter_executable_artifact | merged | runtime | XS | none | — |
| `PUB-VERIFY` | scripted-pr-automerge.sh exits 0 on a failed push | closed | steward | S | none | — |
| `PX10` | processes — declarative spawn plan, deny-by-default inheritance, pidfd identity, typed child-exit observation | draft | runtime | L | none | — |
| `PX11` | sockets — typed addresses, bounded send/receive, explicit option families, injected resolver capability | draft | runtime | L | none | — |
| `PX12` | readiness — nonblocking transitions, epoll/eventfd/timerfd/signalfd, cancellation and timeout IN THE OPERATION TYPE | draft | runtime | L | none | — |
| `PX8-ERRID-ALLOC` | ResourceErrorV1 has no allocation-failure identity and buffer allocation is infallible, so PX8's allocation-distinct-from-BufferLimit row cannot be produced at all | ready | foundation | M | none | — |
| `PX8-ERRID-SCOPE` | PX8 clause-(a) A2b — five PR-C error identities have no independent production-reaching evidence; Architect ruled all five inside the closure | ready | verify | L | none | — |
| `PX8-F-CAP-41` | PX8 clause-(a) behavior blocker — closed buffer endpoint (start==capacity) must derive zero-effective ReadEof, not host-reject | draft | foundation | M | none | 41 |
| `PX8-SPAN-PROV` | PX8 clause-(b) gap — BufferSpan carries no originating-buffer identity; freeze accepts a same-shape span from a different buffer | merged | spec-enclave | M | none | 914 |
| `PX8-WROTE-ABS` | PX8 clause-(a) evidence gap — interpreter capped-short Wrote lacks an absolute oracle; PR-C error identities unreached | merged | verify | S | none | — |
| `PX8` | partial/positioned IO — the completion program's root; closure condition | draft | runtime | L | none | — |
| `PX9` | cross-domain System.Error — semantic identity, raw errno, operation, resource, safe context, and honest retry classification | draft | foundation | L | none | — |
| `Q-CLAIM-CLOSURE` | Q-RESIDUE adversary findings — claim-loss in multi-claim test blocks, plus R1/R2/R3 | merged | runtime | S | none | — |
| `Q-CLAIM-COMPARE-ORD` | claim-loss in list_instance_routes... (compare_ord) — both routing claims dropped, replacement only instantiates Bool | merged | runtime | XS | none | — |
| `Q-RESIDUE` | the Track Q rework residue — 10 tests, folded from Q3-Q7 | closed | runtime | S | none | 818 |
| `RT-AGG-COMPOSE` | escaping two Resources into one aggregate (Prod (Resource _) (Resource _)) fails at erasure — checked endpoints do not compose | draft | runtime | TBD | none | — |
| `RT-DECL-CLOSURE-PORT` | Transparent-declaration-closure emission port — a retained TransparentDeclarationClosure residual forces the whole object onto the monolithic RecursiveDescent root, which exceeds Cranelift's per-function ceiling | ready | runtime | L | none | — |
| `RT-DESCENT-RETIRE` | Retire RecursiveDescent — delete the migration selector, the residual enum, the authority variant, and the recursive-descent emission lane | ready | runtime | M | none | — |
| `RT-EFFECT-DIFF` | One reusable rich differential boundary over EffectObservation — interpreter vs native, first-divergence reporting, so backend-local tests can observe what only the CLI suites currently can | ready | runtime | L | none | — |
| `RT-ESCAPE` | escaping a second Resource through a bracket fails native lowering | merged | runtime | M | none | PR #911 @ 238a5c5d (origin/main 4ac9141e, CI green) |
| `RT-FNSPLIT-B1R` | RT-NATIVE-FNSPLIT Boundary B1R — encode the occurrence-local semantic material B1 counted but never stored (repair of landed B1) | merged | runtime | L | none | 937 |
| `RT-FNSPLIT-B2A-C` | plan↔lowering occurrence correspondence — transport the preallocated StaticOriginId to the site where it is out of scope | merged | runtime | L | none | 940 |
| `RT-FNSPLIT-B2A-S` | defunctionalize retained body selection — static-origin tag plus one closed consumer, replacing cloned-RuntimeExpr identity | merged | runtime | M | none | 944 |
| `RT-FNSPLIT-B2A` | RT-NATIVE-FNSPLIT Boundary B2a — make the semantic plane load-bearing for emission (behaviour-preserving port) | closed | runtime | L | none | — |
| `RT-FNSPLIT-B2B` | RT-NATIVE-FNSPLIT Boundary B2b — full emission census, finite differences, and the explicit growth verdict | closed | runtime | M | none | — |
| `RT-FNSPLIT-B2E` | semantic boundary-value elimination — an opaque boundary inhabitant plus a mechanically closed operation-by-class disposition ledger over every reachable Lowered consumer, inert | closed | runtime | L | none | — |
| `RT-FNSPLIT-B2F` | functionization and authority switch — per-static-origin Cranelift target functions, atomic with switch-over, equivalence evidence, and old-path removal | merged | runtime | L | none | 1192 |
| `RT-FNSPLIT-B2O-CHECK` | the B2O checking layer advertises more than it enforces — structural closure for the item enumerator and reachability for the validator arms | ready | runtime | M | none | — |
| `RT-FNSPLIT-B2O` | static body ownership — a total, validated occurrence → PredeclaredFunction mapping in the semantic plane, inert | merged | runtime | M | none | 963 |
| `RT-FNSPLIT-B2R` | representation and call-ABI contract — a stable executable contract for every value that crosses a generated-function boundary, inert | merged | runtime | L | none | 967 |
| `RT-FNSPLIT-B2V` | executable boundary-value ABI — one closed 64-bit tagged word for ValueWord/ResultWord plus the emitted-code interface to construct, discriminate and project it | merged | runtime | L | none | — |
| `RT-FNSPLIT-C1` | operational carrier + three executable eliminators — a runtime-general carrier at the Lowered/lowering boundary with a real producer -> validator -> eliminator edge, grounded on artifact-static semantic identity | merged | runtime | L | none | https://github.com/swe-toolkit/ken/pull/1156 |
| `RT-FNSPLIT-C2-SYNTH-ID` | closed synthesized-constructor-role identity capability, with the DynamicConstructor producer that consumes it — the identity source compiler-synthesized effect payloads have no occurrence to ask for | merged | runtime | M | none | 1186 |
| `RT-FNSPLIT-C3-ACTIVATION` | the opaque activation owner — one Rust representation authority in ken-runtime that constructs, publishes and tears down per-invocation boundary storage, with the deployment-supplied capacity profile and the one-argument public adapter seam | merged | runtime | L | none | 1181 |
| `RT-FNSPLIT-RECUR-PORT` | emission-port completion — the governed nested-bracket family (recursive ComputationalMatch + trap arms) must select FunctionizedUnits, so RT-SCALE-B can measure the completed population | merged | runtime | XL | none | — |
| `RT-JOIN-DISPOSITION` | Join-disposition phase repair — the landed RECUR-PORT `consumed XOR statically-unselected` invariant conflates structural materialization with semantic reachability and false-rejects a join materialized before its enclosing match selects | active | runtime | M | none | — |
| `RT-MATCH-FRAME-FP` | match-frame fingerprints must hash a dedicated closure-free header carrier, not a Debug rendering of closure-capable cases | merged | runtime | M | none | https://github.com/swe-toolkit/ken/pull/1108 |
| `RT-NATIVE-FNSPLIT` | Native backend: bound per-function lowering growth to O(n) — helper identity is a variable-width whole-configuration key (orig. single-Function VReg::MAX, since fixed) | merged | runtime | TBD | none | — |
| `RT-PARITY` | interpreter/native parity erratum (adversary F5 + F6) | closed | runtime | M | none | — |
| `RT-PLANNER-ATTRIB-K` | Boundary A planner: fixed K is a design invariant — move the K-exceeded rejection off the capacity channel | merged | runtime | XS | none | https://github.com/swe-toolkit/ken/pull/935 |
| `RT-PLANNER-DIAGNOSTIC-K` | Boundary A planner: report planner-invariant failures as planner defects, and assert fixed_k CONSTANT rather than merely affine | merged | runtime | S | none | https://github.com/swe-toolkit/ken/pull/929 |
| `RT-PRODUCER-MATCH-PORT` | Producer-match call port — an ordinary Match whose scrutinee is directly a Call routes the whole object to RecursiveDescent | ready | runtime | M | none | — |
| `RT-RECURSOR-TRANSPORT` | Active-recursor transport — an active computational recursor's invocation-local scope/return-hole state cannot cross a functionized unit boundary, retaining two residual classes | ready | runtime | L | none | — |
| `RT-SCALE-A` | Boundary A — re-derive the planner census for n=3..7 against the COMPLETED factored representation, superseding the provisional outer-planner numbers | merged | runtime | M | none | — |
| `RT-SCALE-B` | Boundary B — the full n=3..7 emission measurement, the research-grounded analytical model, and the operator scaling verdict that gates RT-NATIVE-FNSPLIT's merge | merged | runtime | L | none | — |
| `RT-SEED-CALL-PORT` | Seed-closure call port — a Call whose callee is the retained non-lexical closure form routes the whole object to RecursiveDescent | ready | runtime | M | none | — |
| `RT-SPLIT` | decompose cranelift_backend.rs | merged | runtime | L | none | — |
| `RT-SRC-DISPATCH-COVER` | close the source-machine scrutinee-dispatch coverage tier surfaced by RT-SPLIT slice 4 | draft | runtime | TBD | none | — |
| `RT-SYMLINK-LANE` | SymlinkPolicy is honoured by the interpreter lane and unreachable in the native lane — FollowWithinScope has no native behaviour | draft | runtime | TBD | none | — |
| `RT-VALUE-TOTALITY` | Make every total traversal of Value non-recursive in the host stack, and remove the closure capabilities the landed closure boundary forbids | merged | runtime | L | none | — |
| `SEAL-2` | carrier producer closure, over a derived enumeration | merged | foundation | M | none | PR #912 @ 4ac9141e (origin/main, CI green) |
| `SEC1-IFC-R3` | [Sec1-reduce] cannot be reified yet: NO production path can return Verdict::Disproved, so the verdict D5 requires is unreachable and every Disproved in sec1_acceptance is hand-rigged | draft | verify | M | G-Sec | — |
| `SEC1-IFC` | Reify the three named Sec1 stubs — two of them are the SOLE NETS for Sec1's two trusted surfaces, and both are placeholders under a green suite | merged | verify | M | G-Sec | https://github.com/swe-toolkit/ken/pull/1094 |
| `SEC4-TCB` | Sec4's trust-model conformance seed is fully authored and nothing executes it — Sec1/Sec1ct/Sec2 each have an acceptance suite bound to their seed, Sec4 has none | merged | verify | M | G5 | — |
| `SPAN-SEAL` | seal the BufferSpan producer surface | merged | foundation | M | none | — |
| `SPEC-31-WIDTH-ERRATUM` | spec 31-lexical mandates a 96-column canonical width while the formatting conformance suite asserts 88 in 18 places and cites 31 §1d as its source — rule the exact value and reconcile | closed | spec | S | none | https://github.com/swe-toolkit/ken/pull/1054 |
| `SPEC-38-ERRATUM` | spec 38-ffi-io self-contradicts on the transfer bound — rule and reconcile | closed | spec | S | none | 827 |
| `SPEC-ALIGN-A1` | Scope the landed-code authority convention out of the normative status blocks, and census every private-mechanism constraint against its conformance consumers before relaxing any of them | merged | spec | M | none | 1028 |
| `SPEC-ALIGN-B1` | Split the frozen interoperability and provenance schemas into versioned protocol profiles, under a per-edge threat audit rather than a field count | draft | spec | L | none | — |
| `SPEC-AUTH-EX` | 62-authority §7 worked examples are written in a retired surface — retired `view` keyword, retired `Cap_FS` spelling, and `write_at` for the landed `write_file` | draft | spec-enclave | S | none | — |
| `SPEC-CLOSURE-BOUNDARY` | Revise the runtime value spec to remove the closure-identity inconsistency and state the closure/value boundary with minimum constraints on the implementation | merged | spec | M | none | — |
| `SPEC-IDENT-BLESSED` | Settle the identifier character set: 31-lexical promises a bounded blessed-Unicode-letter table that does not exist, cites a security chapter that carries no such claim, and states a confusable gate the landed lexer does not implement | merged | spec-enclave | M | none | https://github.com/swe-toolkit/ken/pull/1147 |
| `SPEC-MISSION-GROUNDING` | Ground the spec as a whole against the mission — audit every retained constraint for which mission property fails without it, and relax the ones where nothing does | active | spec | L | none | — |
| `SPEC-NESTED-IND` | un-defer nested strictly-positive inductives in 14 §8.5 — state structural positivity through declared strictly-positive type-parameter positions, the lifted induction hypotheses, and the iota rules, WITHOUT mutual families | merged | spec-enclave | M | none | — |
| `SPEC-STATUS-RECONCILE` | the spec's two status vocabularies do not correspond — define the correspondence (or replace the ladder), then apply it | merged | spec-enclave | M | none | — |
| `SPEC-STORE-SPLIT` | Split durable canonical bytes from in-process maximal sharing: demote the store mechanism to private, retarget the conformance rows that assert it, and re-cut the runtime program against the relaxed contract | merged | spec-enclave | L | none | — |
| `SRC-ATTEST` | squash-stable whole-source attestation + fresh merge-result authorization | merged | doc | M | none | — |
| `STR-BIJ-TEST-CARRIER` | The AC2 reverse-direction test claims a universal inverse and its sole operand is an NFC fixed point — it is green under the correct law AND under the false one it pins | merged | language | S | none | https://github.com/swe-toolkit/ken/pull/1102 |
| `STR-BIJ` | the String/List Char 'bijection' over-claim (adversary A1 + A2) | merged | spec-enclave | S | none | https://github.com/swe-toolkit/ken/pull/1096 |
| `STR-NFC-CONSTRUCTION` | NFC-at-construction is normative and unimplemented: all three `EvalVal::Str` ingresses store the raw string, so `char_length`/`byte_length`/`s2l`/`==` observe unnormalized values and the interp carrier disagrees with the runtime carrier | merged | language | L | none | https://github.com/swe-toolkit/ken/pull/1109 |
| `SURF-IDENT-TR39` | The lexer's confusable-resistance is satisfied VACUOUSLY by an ASCII-only identifier rule — spec 31 §2's blessed Unicode letters are unimplemented, and the test that looks like the TR39 gate cannot see the difference | merged | ergo | S–M | none | — |
| `SURF-SPACE-CELLS` | The `space` block surface — cells and `becomes` — is unbuilt, while its entire desugaring target (the `State` effect: Get/Put/run_state) is built and live | active | language | M–L | none | https://github.com/swe-toolkit/ken/pull/1152 |
| `V3-RESIDUAL` | V3's suite has FOUR assertion-free placeholder tests carrying ordinary names — `disproved_carries_countermodel` asserts nothing, passes, and reads in cargo output exactly like a real pin | merged | verify | L | G2-G3 | https://github.com/swe-toolkit/ken/pull/1103 |
| `V4-RESIDUAL` | The Kripke countermodel is an inert shell: it is never related to `φ` at all — no interpretation of the formula, no recursive forcing evaluator — and V3's prose `description` is stuffed into `FormRef`, a slot meant for a structural subformula reference | merged | verify | L | G2-G3 | 1117 |
| `VIS-BR-LITERAL` | visibility walk: raw-string prefixes br and cr are unrecognized by the literal scanner | merged | runtime | XS | none | — |

## Releasable frontier

Items whose status is `ready` and whose every `depends_on` entry is
itself `merged` or `closed` (i.e. nothing left blocking a kickoff):

- `DOC-ATTEST-LIVING` — attesting living tracker files makes every routine WP status flip redden the currency gate
- `F1-37` — F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion
- `MODELS-TIER` — agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception
- `RT-DECL-CLOSURE-PORT` — Transparent-declaration-closure emission port — a retained TransparentDeclarationClosure residual forces the whole object onto the monolithic RecursiveDescent root, which exceeds Cranelift's per-function ceiling
- `RT-EFFECT-DIFF` — One reusable rich differential boundary over EffectObservation — interpreter vs native, first-divergence reporting, so backend-local tests can observe what only the CLI suites currently can
- `RT-FNSPLIT-B2O-CHECK` — the B2O checking layer advertises more than it enforces — structural closure for the item enumerator and reachability for the validator arms

## Blockers

Items not yet `merged`/`closed` whose `depends_on` names an id that
is itself not yet `merged`/`closed`:

- `ABI-A1` blocked by `ABI-REVOKE` (status: draft)
- `ABI-A2` blocked by `ABI-REVOKE` (status: draft)
- `ABI-A3` blocked by `ABI-REVOKE` (status: draft)
- `ABI-A3` blocked by `ABI-R3` (status: draft)
- `ABI-M1` blocked by `ABI-R3` (status: draft)
- `ABI-M2` blocked by `ABI-M1` (status: draft)
- `ABI-R3` blocked by `PX8` (status: draft)
- `ABI-REVOKE` blocked by `ABI-R3` (status: draft)
- `ABI-S1` blocked by `PX9` (status: draft)
- `ABI-S2` blocked by `ABI-A3` (status: draft)
- `ABI-S4` blocked by `ABI-M1` (status: draft)
- `ABI-S5` blocked by `PX9` (status: draft)
- `ABI-S6` blocked by `ABI-S1` (status: draft)
- `DS-9` blocked by `KERNEL-NESTED-IND` (status: active)
- `F4` blocked by `A3` (status: draft)
- `NATIVE-HANDLE-CARRIER` blocked by `RT-JOIN-DISPOSITION` (status: active)
- `PX10` blocked by `PX9` (status: draft)
- `PX10` blocked by `ABI-M1` (status: draft)
- `PX10` blocked by `ABI-S5` (status: draft)
- `PX11` blocked by `PX9` (status: draft)
- `PX11` blocked by `ABI-M1` (status: draft)
- `PX12` blocked by `PX10` (status: draft)
- `PX12` blocked by `PX11` (status: draft)
- `PX8-ERRID-ALLOC` blocked by `RT-DECL-CLOSURE-PORT` (status: ready)
- `PX8-ERRID-SCOPE` blocked by `PX8-ERRID-ALLOC` (status: ready)
- `PX8-F-CAP-41` blocked by `NATIVE-HANDLE-CARRIER` (status: ready)
- `PX8` blocked by `PX8-F-CAP-41` (status: draft)
- `PX8` blocked by `PX8-ERRID-SCOPE` (status: ready)
- `PX9` blocked by `PX8` (status: draft)
- `PX9` blocked by `ABI-REVOKE` (status: draft)
- `RT-DESCENT-RETIRE` blocked by `RT-DECL-CLOSURE-PORT` (status: ready)
- `RT-DESCENT-RETIRE` blocked by `RT-SEED-CALL-PORT` (status: ready)
- `RT-DESCENT-RETIRE` blocked by `RT-PRODUCER-MATCH-PORT` (status: ready)
- `RT-DESCENT-RETIRE` blocked by `RT-RECURSOR-TRANSPORT` (status: ready)
- `RT-PRODUCER-MATCH-PORT` blocked by `RT-SEED-CALL-PORT` (status: ready)
- `RT-RECURSOR-TRANSPORT` blocked by `RT-PRODUCER-MATCH-PORT` (status: ready)
- `RT-SEED-CALL-PORT` blocked by `RT-DECL-CLOSURE-PORT` (status: ready)

## Gate progress

Work items grouped by the gate (`05-implementation-dag.md`) they
feed; `none`/`TBD` gates are omitted here (see the status table above
for every item, gated or not):

- **G-Sec**: `SEC1-IFC-R3` (draft) `SEC1-IFC` (merged)
- **G2-G3**: `V3-RESIDUAL` (merged) `V4-RESIDUAL` (merged)
- **G5**: `SEC4-TCB` (merged)

## Archive & diary

- The complete build chronicle — every prior live-state snapshot, the full
  evidence trail behind every merged WP back to project start — and the
  day-to-day session narrative both live in [`diary/`](diary/INDEX.md), one
  file per day under `diary/YYYY/Mon/DD.md`. See
  [`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) for the live
  operator briefing and Steward resume state.
- Per-item briefs, where they exist, live under
  [`wp/`](wp/) and are linked from the corresponding
  `docs/program/issues/<ID>.md` file.
