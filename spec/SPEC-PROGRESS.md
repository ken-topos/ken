# Ken specification — drafting progress & resume protocol

**This file is the backbone of a long-running spec-drafting effort.** It
survives context compaction. If you are resuming (after a compact, or cold):
**read this file first**, then read the most-recently-touched spec file, then
continue from "Next action" below. Update the status table and "Next action"
after every section you complete, and commit.

## The task

Draft a solid language specification for **Ken** — an MIT-licensed, Rust-hosted,
interpreter-first **verified topos language** — designed from type-theory /
topos / OTT first principles, aligned to the locked architecture in
`../docs/program/01-strategy.md` and `../docs/program/02-roadmap.md`. The goal
is a spec the build teams (Kernel, Verify, Language) can start work on — i.e. it
must cover, at full rigor, the **kernel core** that unblocks K1/K2/K3/X1/V0, and
at decreasing resolution the verification surface, language surface, runtime,
and stdlib.

This is the spec-author bootstrap the Opus Spec enclave would do; the real
enclave later refines it against Ken's reference interpreter.

## Conventions (hold these across the whole effort)

- **Clean-room.** Write from type-theory/topos/OTT first principles + the
  strategy docs. Describe Ken's design in our own words; never transcribe
  outside source. See `../CLEAN-ROOM.md`.
- **Design latitude.** We *design* Ken from first principles. Make reasonable
  calls; log genuine forks in `90-open-decisions.md` rather than blocking.
- **Foundational guarantees** (per strategy non-goals): Ken has checked
  universes (no `Type:Type`), genuinely dependent `Sigma`, `J` that reduces on
  non-`refl`, an exact `Int` from day one (no f64-only numeric model), and no
  hard slot ceiling.
- **Wrap markdown at 80 columns.** Run `python3 /tmp/reflow.py <file>` after
  writing a long file (it skips frontmatter/tables/fences) to guarantee it.
- **Commit after each completed section** (`spec: draft <file>`), so progress is
  in git too.

## Outline & status

Status: TODO · DRAFT (first pass written) · REVISED (refined) · DONE

| File | Scope | Status |
|---|---|---|
| `README.md` | Index, organization, conventions, status | DRAFT |
| `00-overview.md` | Thesis, L0/L1/L2, goals, scope, non-goals, glossary | DRAFT |
| `10-kernel/README.md` | Trusted kernel overview; de Bruijn criterion; what it checks | DRAFT |
| `10-kernel/11-syntax.md` | Core term/type syntax, de Bruijn, telescopes, contexts | DRAFT |
| `10-kernel/12-universes.md` | Universe hierarchy, predicativity, checking, cumulativity | DRAFT |
| `10-kernel/13-pi-sigma.md` | Dependent Π and Σ: formation/intro/elim/compute + η | DRAFT |
| `10-kernel/14-inductive.md` | Inductive families, eliminators, strict positivity | DRAFT |
| `10-kernel/15-identity.md` | Observational `Eq`; `refl`/`cast`; `J` and its computation | DRAFT |
| `10-kernel/16-observational.md` | Strict-prop Ω, `Eq`-by-type, `cast`, quotients, truncation | DRAFT |
| `10-kernel/17-conversion.md` | Definitional eq, NbE, decidable conversion, β/η/δ, SCT | DRAFT |
| `10-kernel/18-judgments.md` | Full typing judgment, algorithm, the kernel API surface | DRAFT |
| `20-verification/README.md` | The differentiator: overview | DRAFT |
| `20-verification/21-spec-syntax.md` | `requires`/`ensures`/refinements/goals | DRAFT |
| `20-verification/22-obligations.md` | VC generation; body-as-motive plumbing | DRAFT |
| `20-verification/23-prover.md` | Classifier; Z3-for-decidable; Kripke embedding; certificates | DRAFT |
| `20-verification/24-diagnostics.md` | Countermodels; typed holes + `unknown`; three-region Heyting | DRAFT |
| `20-verification/25-protocol.md` | Machine-readable diagnostic protocol (agent contract) | DRAFT |
| `30-surface/README.md` | Surface language overview | DRAFT |
| `30-surface/31-lexical.md` | Lexical structure, tokens, literals | DRAFT |
| `30-surface/32-grammar.md` | Concrete grammar | DRAFT |
| `30-surface/33-declarations.md` | Modules, declarations, definitions, visibility | DRAFT |
| `30-surface/34-data-match.md` | Sum types, `match`, exhaustiveness, `Result`/`Option` | DRAFT |
| `30-surface/35-numbers.md` | `Int`/`Decimal`/reals — Ken's exact numeric model | DRAFT |
| `30-surface/36-effects.md` | Effect tracking surface | DRAFT |
| `30-surface/37-strings-collections.md` | Strings, core collections | DRAFT |
| `30-surface/38-ffi-io.md` | `Bytes`, binary I/O, FFI | DRAFT |
| `30-surface/39-elaboration.md` | Surface → kernel elaboration, implicits, inference | DRAFT |
| `40-runtime/README.md` | Runtime / reference-semantics overview | DRAFT |
| `40-runtime/41-values.md` | Value rep, content-addressed heap, O(1) structural eq, dedup | DRAFT |
| `40-runtime/42-evaluation.md` | Operational semantics of the reference interpreter | DRAFT |
| `40-runtime/43-termination.md` | Totality, SCT, partial/`unknown` execution | DRAFT |
| `40-runtime/44-capacity.md` | Content store; slot-ceiling decision; lattice's real roles | DRAFT |
| `50-stdlib/README.md` | Prelude + core stdlib shape | DRAFT |
| `50-stdlib/52-map.md` | Proved pure `Map k v` over `Ord k` (VAL2 #8/OQ-A) | DRAFT |
| `50-stdlib/53-transport.md` | Derived `subst`/`cong`/`cast`/`sym`/`trans` over the `J` former | DRAFT |
| `50-stdlib/54-map-verified-laws.md` | Proof skeletons for the 5 deferred inductive `Map` laws (Gap-A/Gap-B) | DRAFT |
| `60-security/README.md` | Security (tier-1): frame, threat model, taxonomy | DRAFT |
| `60-security/61-information-flow.md` | IFC: label lattice, declassification, non-interference | DRAFT |
| `60-security/62-authority.md` | Capabilities, PoLA, attenuation, revocation, audit | DRAFT |
| `60-security/63-supply-chain.md` | Package/`.keni` format, re-check, provenance | DRAFT |
| `60-security/64-trust-model.md` | TCB, de-Bruijn-as-security, trusting-trust, limits | DRAFT |
| `60-security/65-policy.md` | Policy-as-code: separately-authored, mandatory, static security policy (ADR 0007) | DRAFT |
| `70-behavioral/README.md` | Behavioral seam: assumption-boundary export to the sibling (`Ward`); proved/tested/delegated/unknown; temporal-as-data | DRAFT |
| `90-open-decisions.md` | The forks register (for the operator) | DRAFT |
| `_notes/analysis-digest.md` | Background design notes (subagent) | DONE |

(Conformance corpus seeds live in `../conformance/`, referenced from spec
sections; seeded opportunistically as core sections are written.)

## Drafting order (priority)

1. `_notes/analysis-digest.md` (subagent, in progress) — background design
   notes; kernel core can start from first principles.
2. `00-overview.md` + `README.md` — frame.
3. `10-kernel/*` — the trust root; highest rigor; unblocks K1/K2/K3/X1/V0.
4. `20-verification/*` — the differentiator; unblocks V0–V4, T1.
5. `30-surface/*` — unblocks L-stream + V1.
6. `40-runtime/*` — unblocks X1/X2/K3.
7. `50-stdlib/README.md`, finalize `90-open-decisions.md`.
8. Seed `../conformance/` with core cases.

## Next action

**DRAFT v0 of the full spec is complete** — every chapter in the outline is
DRAFT, the open-decisions register (`90-open-decisions.md`) consolidates the
forks, and the conformance corpus (`../conformance/`) is seeded with the
load-bearing cases per area. The spec is ready for the build teams (Kernel,
Verify, Language, Runtime) to start against.

Remaining / future passes (not blocking team start):
- Spec enclave: validate the **(oracle)**-tagged points against Ken's reference
  interpreter (exact observational `Eq`/`cast` by-type normal forms `16`, SCT
  size order `17`, reduction-order choices) — confirm or correct. The
  observational forms also cross-check against `CICobs`/`TTobs` (permissive
  refs, read-not-copy, ADR 0005).
- Operator: review `90-open-decisions.md`; veto/confirm DRAFT recommendations;
  decided OQs → ADRs under `../docs/adr/`.
- Grow the conformance corpus from the seeds as implementation lands (every spec
  claim should earn a case).
- Raise chapters from DRAFT → REVISED as the enclave/teams validate them.


## Open decisions — RESOLVED (2026-06-27)

The forks register (`90-open-decisions.md`) is **fully dispositioned.** Every OQ
is DECIDED / DECIDED-deferred / DECIDED-excluded / DECIDED-principles, with a
resolution-log row each. Architecturally significant ones produced **ADRs
0005–0008** (observational equality; the Ward behavioral-assurance sibling;
security policy-as-code; typeclass coherence). The behavioral seam
(`70-behavioral/71–74`) is drafted.

**Still genuinely open** (not blocking, not decidable now):
- `OQ-sampling-policy`, `OQ-discharge-attestation` — **Ward-blocked**; need the
  sibling's runner before their concrete schema/language can be designed.
- Concrete syntax **token table** (`OQ-syntax` principles are decided; spellings
  iterate with the team) and `OQ-policy` concrete policy syntax — team
  iteration.
