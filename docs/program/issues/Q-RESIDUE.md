---
id: Q-RESIDUE
title: "the Track Q rework residue — 10 tests, folded from Q3-Q7"
status: ready
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/qa-triage/FINDINGS.md (Q2 triage, 2026-07-21; operator folded Q3-Q7 into one item)
---

Q3–Q7 of `docs/program/11-test-suite-and-ci-remediation.md` are **folded
into this single item** (operator, 2026-07-21). Q2 triaged all 428 flagged
tests and 91.6% classified as sound durable invariants; **Q4 and Q7 produced
zero defects.** The rework population is the ten tests below, not the ~110
the scan hit counts implied.

> ⚠ **Do not re-derive scope from the scan.** `scripts/qa-risk-scan.py`
> emits a **review queue, not a defect list** — 147 broad-outcome hits and
> 27 timing hits all triaged clean. The inventory here is the *post-triage*
> residue and is the whole of the work. Full reasoning and per-test notes:
> `docs/program/qa-triage/FINDINGS.md` + `Q2-<team>-result.md`.

## Inventory

### 1. Unlabelled transition sentinel (1) — cheapest real fix in the set

- `crates/ken-runtime/src/ir.rs:654`
  `seed_examples_are_observation_limited` — `examples.len()==5` over a
  **growable** seed list. Advisory §3.3: a sentinel is legal **only if
  labelled honestly**. Either label it (name the boundary, state why
  extension stops here, name the retiring event) or re-assert relationally
  against the authoritative seed set.

### 2. Source-text proxies (≈6) — source text asserted, not a mechanism

- `crates/ken-interp/src/eval.rs`
  `resource_table_lifetime_is_owned_by_one_interpreter_invocation`
- `crates/ken-host/src/effect_v1.rs`
  `resource_owner_and_close_allowance_are_structurally_confined`
- `crates/ken-elaborator/tests/cat5_parsing_package.rs` — `cat5_d1`,
  `cat5_d2`, `cat5_d3` (**Language's area**, see routing note)
- `list_instance_routes_the_canonical_compare_into_raw_list_compare`
  (literate `.ken.md` source text rather than the elaborated term)

Per advisory §5.4: prefer compiler visibility, type-level construction
failure, AST/token inspection, or a lint. **Where a source scan is genuinely
the only available net, that is an acceptable outcome** — but scan the
*mechanism* rather than a bare name, state the limitation in-test, and add a
mutation proving the scan bites.

### 3. Derived counts (≈3)

- `generated_manifest_is_closed_and_probe_comparison_discriminates`
  (`ken-host`) — `fact_count==23` duplicates a **generated** manifest;
  assert against the manifest, not a copy of its size.
- `emitter_adds_v0_schema_hashes_and_validates_representative_fixtures`
  (Runtime) — `fixtures.len()==1` freezes the representative corpus at one
  entry; adding a second fixture goes red for no semantic reason.
- `px8i_jit_and_object_construct_identical_local_helper_clif`
  (`cranelift_backend`) — `matches("-- helper --").count()==5` of
  **unverified provenance**; establish whether 5 is a fixed property of the
  fixture or today's codegen before deciding the assertion.

## Acceptance criteria

1. Each of the ten is either **reworked** to a durable assertion, or
   **honestly labelled** as a sentinel/compat-vector with its rationale
   recorded in-test. Both are valid outcomes — this is not a "make them all
   durable" item.
2. Each reworked test carries a **mutation proof**: break the claimed
   mechanism at its seam and show the unchanged test fails with the expected
   opposite. Do not keep the mutation. (Advisory §6 step 8; QA playbook gate
   8.) **Without this, a rewritten assertion is unverified.**
3. `scripts/qa-risk-scan.py --self-test` still passes.
4. Green in **CI** — never a local `--workspace` run (`COORDINATION §12`).

## ⚠ Routing note — this WP's inventory is NOT single-team

`owner: runtime` reflects the majority (7 of 10), **not** clean ownership.
The three `cat5_parsing_package.rs` rows are **Language's** area, and
`ken-cli`/`ken-foundation` ownership is genuinely undocumented (`CODEOWNERS`
is inert and maps no crate). The Runtime leader should route the `cat5` rows
to Language rather than absorb them. **This is a cross-cutting edit by
design — path-guard treats that as normal, not a violation.**
