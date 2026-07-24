---
id: DOC-GATE-RECORD-AXIS
title: "validation-gate registry: bind token→runner COVERAGE on the record axis, and close the `kind` vocabulary"
status: ready
owner: verify
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: adversary findings F1 + F2 on DOC-VALIDATION-BINDING (96ab2b4b), side thread thr_2seh2bm1kr5mh evt_382156eh3xayn, 2026-07-24. Steward-filed (agents cannot create tracked work per COORDINATION §2); Steward triage = both CONFIRMED PREVENTIVE.
---

> ## ⏳ DEADLINE IS A PRECONDITION, NOT A DATE
>
> **Both gaps are latent today and both must close BEFORE the next `library/`
> record is added.** With exactly one `kind = "status"` record in
> `library/manifest.toml`, neither is a live defect. The moment a second one is
> added, F1 becomes a **vacuous validation claim that the registry test actively
> forces the author to write.** ⇒ Any WP that adds a `library/` record depends on
> this one; sequence accordingly rather than treating it as background hardening.

## Objective

Close the two preventive gaps the Adversary found in the validation-gate registry
that `DOC-VALIDATION-BINDING` landed — both in
`crates/ken-cli/tests/library_documentation_gates.rs`. Neither reopens that WP's
design, which stands.

## F1 — token→runner **coverage** is unbound on the record axis

**This is the WP's own defect class, surviving along a different axis.**
`DOC-VALIDATION-BINDING` fixed "the test checked one description against another
description"; the rename/`E0425` proof (its AC-2) binds token → runner
**existence**. Nothing binds token → runner **coverage** — and coverage is
exactly what the manifest's per-record validation lists are read as asserting.

Grounding (adversary, `96ab2b4b`):

- `:504-508` — `struct ValidationGate { token, applies: fn(&DocEntry) -> bool,
  run: fn() }`. **`run` takes no record**, so it is structurally incapable of
  depending on which record declared the token.
- `:558-562` — `generated-current` pairs `applies_to_status_records` with
  `check_generated_current`.
- `:514-516` — `applies_to_status_records(entry) = entry.kind == "status"` — an
  **unbounded** predicate over a manifest field.
- `:949-951` → `:927-943` — `check_generated_current` shells
  `bash scripts/gen-doc-status.sh --check`, whose target is hard-coded
  (`gen-doc-status.sh:38`, `OUT_FILE="$REPO_ROOT/library/STATUS.md"`).

**Repro needs no build — the divergence is in the types.** Add a second
`kind = "status"` record; then
`gate_validation_tokens_are_closed_and_match_applicable_checks` (`:599-604`)
derives `required` by filtering on `applies`, so it **demands** the new record
declare `generated-current` and **forbids** `source-currency`; the runner then
executes once against `library/STATUS.md` and never reads the new record. In the
same step `gen-doc-status.sh:351-357` (`flush_record`) excludes that record's
`sources` from `CITED_SOURCES`, dropping those citations out of the
`SOURCE-ATTESTATIONS` set-equality requirement. **Suite green, both halves of
that record's validation list vacuous, and the mechanism steered the author into
writing them.**

**Cheapest close (Steward-endorsed route): pin the population the runner can
actually serve, in the closure test** — ~4 lines that fail loudly at exactly the
moment the assumption stops holding, and name the required action:

```rust
let status_records: BTreeSet<&str> =
    entries.iter().filter(|e| e.kind == "status").map(|e| e.path.as_str()).collect();
assert_eq!(
    status_records, BTreeSet::from(["library/STATUS.md"]),
    "check_generated_current is hard-coded to gen-doc-status.sh/library/STATUS.md; a \
     second status-kind record needs its own registered runner, not this token"
);
```

⚠ The principled alternative — widen `run` to `fn(&DocEntry)` — is **correct
but a whole-registry refactor**. Do not spend it here; the assertion above buys the
same protection at ~4 lines. If a second status record is genuinely wanted later,
the refactor is that WP's cost, not this one's.

## F2 — `kind` is the one exemption-granting field with no closed vocabulary

Asymmetry, all in the same file: `:906-923` `check_availability_labels` has a
closed `VALID` set and rejects anything else; `:2434-2458`
`check_authority_classes` has a closed D1 set and rejects anything else; but
`:447-449` `check_manifest_completeness` checks `kind` **only for
non-emptiness**, and no closed set for it exists anywhere (`crates/`, `library/`,
`docs/program/12-documentation-program.md` defines no kind vocabulary).

Yet `kind` is now the **sole discriminator for the source-currency exemption**,
in two independent parsers — `:514-519` (Rust) and `gen-doc-status.sh:351-357`
(awk). Live values: `explanatory`×7, `tutorial`×4, `portal`, `reference`,
`status` — five conventions held by nothing.

**Violated invariant — COORDINATION §7b:** *"the discriminator the case keys on
must be a structural / kernel-side signal … never a self-reported string the
untrusted layer can forge."* Here it is a self-reported string **inside the
artifact under validation**, and it grants an exemption.

★ **The adversary measured the direction rather than asserting it, and that
changes the severity honestly:** relabelling a normal record to
`kind = "status"` drops its sources from `CITED_SOURCES`, which then *requires*
the matching `SOURCE-ATTESTATIONS` rows be deleted — so the attestation loss
**surfaces as a visible ledger edit**, not silently. What is unguarded is the
**field itself**: a typo (`"Status"`, `"statuses"`) silently changes which gates
apply with nothing objecting.

**Cheapest close:** add the closed-set check for `kind` **inside
`check_manifest_completeness`** — already an every-record registered gate
(`manifest-completeness`) — rather than as a new registry row. Same shape as the
two existing checks, and **zero manifest churn**, versus a new token requiring a
14-record edit like `transport-delimiter` just took.

## Acceptance criteria

- **AC1** F1's population assertion is present, and its failure message names the
  required action (register a runner, don't declare the token). ⛔ Verify it fails
  by **adding** a second `kind = "status"` record and observing the assertion
  fire — a negative check passes for any reason, so it needs a positive control.
- **AC2** F2's `kind` closed-set check lives in `check_manifest_completeness`, is
  registered by no **new** token, and requires **no** manifest churn. It rejects
  a case-variant typo (`"Status"`) — include that as the discriminating case.
- **AC3** The closed `kind` vocabulary is **derived from the landed manifest**, not
  invented: enumerate the live values and state the set. If a value looks like a
  mistake, surface it — do not silently bless it by adding it to the set.
- **AC4** `scripts/ken-cargo test -p ken-cli --test library_documentation_gates`
  green; no-regression means **green in CI**, never a local `--workspace` run
  (COORDINATION §12).
- **AC5** ⛔ Do **not** re-baseline or weaken any existing gate to make these
  pass, and do **not** reopen the `DOC-VALIDATION-BINDING` design — one
  executable registry owning token/applicability/function item stands.

## Adversary hypotheses that DIED (do not re-litigate)

Recorded because withdrawn hypotheses are evidence, and re-running them is pure
cost:

- *"`source-currency` and `generated-current` have identical runner bodies, so
  'distinct opposite predicates' is unobservable."* Structurally true, **not a
  defect** — `gen-doc-status.sh --check` genuinely establishes both properties in
  one invocation. Two tokens naming two properties of one command is correct.
- *"The registry exempts status records from source-currency while the script
  checks them anyway."* **False in both directions** — the awk excludes status
  records' sources at extraction (`:351-357`); registry and script agree exactly.
- *"`generated-current` cannot subsume `source-currency` for `STATUS.md` because a
  `library/REVISION` change would not redden `--check`."* **False** — `REVISION`
  and the `SOURCE-ATTESTATIONS` digest are both rendered into `STATUS.md`, so both
  cited sources do redden regeneration. The exemption's justification holds.
- `check_transport_delimiter` field coverage is **complete** —
  `all_string_fields`
  (`:629-641`) enumerates all eight `DocEntry` fields (`:151-161`).
  `parse_manifest`'s
  `_ => {}` (`:246`) drops unknown keys, but every constructible drop fails loudly
  downstream rather than passing; the default branch is safe here.

## Escalation

Soundness / mechanism → **Architect**. Scope, sequencing → **Steward**. Diff is
`crates/` only, so the Architect is the required reviewer and the Librarian is not
(COORDINATION §8a); if the fix reaches into `library/manifest.toml`, both review
in **parallel** over disjoint domains.
