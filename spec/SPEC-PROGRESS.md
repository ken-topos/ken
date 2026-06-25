# Ken specification — drafting progress & resume protocol

**This file is the backbone of a long-running spec-drafting effort.** It
survives context compaction. If you are resuming (after a compact, or cold):
**read this file first**, then read the most-recently-touched spec file, then
continue from "Next action" below. Update the status table and "Next action"
after every section you complete, and commit.

## The task

Draft a solid language specification for **Ken** — an MIT-licensed, Rust-hosted,
interpreter-first **verified topos language** — distilling, extending, and
filling out the ideas from the research conversation (`/home/pat/src/yon/Yon
Programming Language Analysis.md`) and its corrections
(`/home/pat/src/yon/01-reality-check.md`), aligned to the locked architecture in
`../docs/program/01-strategy.md` and `../docs/program/02-roadmap.md`. The goal
is a spec the build teams (Kernel, Verify, Language) can start work on — i.e. it
must cover, at full rigor, the **kernel core** that unblocks K1/K2/K3/X1/V0, and
at decreasing resolution the verification surface, language surface, runtime,
and stdlib.

This is the spec-author bootstrap the Opus Spec enclave would do; the real
enclave later refines it against the prototype as a behavioral oracle.

## Conventions (hold these across the whole effort)

- **Clean-room.** Write from the analysis digest + type-theory/topos/cubical
  first principles + the strategy docs. Do **NOT** read the AGPL prototype
  source under `/home/pat/src/yon/` (only the analysis + reality-check files
  there are in scope). Describe behavior in our own words; never transcribe
  source. See `../CLEAN-ROOM.md`.
- **Design latitude.** We *design* Ken (distill + extend). Make reasonable
  calls; log genuine forks in `90-open-decisions.md` rather than blocking.
- **Don't reproduce the prototype's gaps** (per strategy non-goals): Ken has
  checked universes (no `Type:Type`), genuinely dependent `Sigma`, `J` that
  reduces on non-`refl`, `Int` from day one (no f64-only model), and no hard
  slot ceiling.
- **Wrap markdown at 80 columns.** Run `python3 /tmp/reflow.py <file>` after
  writing a long file (it skips frontmatter/tables/fences) to guarantee it.
- **Commit after each completed section** (`spec: draft <file>`), so progress is
  in git too.

## Outline & status

Status: TODO · DRAFT (first pass written) · REVISED (refined w/ digest) · DONE

| File | Scope | Status |
|---|---|---|
| `README.md` | Index, organization, conventions, status | DRAFT |
| `00-overview.md` | Thesis, L0/L1/L2, goals, scope, non-goals, glossary | DRAFT |
| `10-kernel/README.md` | Trusted kernel overview; de Bruijn criterion; what it checks | DRAFT |
| `10-kernel/11-syntax.md` | Core term/type syntax, de Bruijn, telescopes, contexts | DRAFT |
| `10-kernel/12-universes.md` | Universe hierarchy, predicativity, checking, cumulativity | TODO |
| `10-kernel/13-pi-sigma.md` | Dependent Π and Σ: formation/intro/elim/compute + η | TODO |
| `10-kernel/14-inductive.md` | Inductive families, eliminators, strict positivity | TODO |
| `10-kernel/15-identity.md` | `Id`/`J` path induction; relation to the cubical `Path` | TODO |
| `10-kernel/16-cubical.md` | Interval, Path, transport, hcomp/comp, Glue, univalence, HITs | TODO |
| `10-kernel/17-conversion.md` | Definitional eq, NbE, decidable conversion, β/η/δ, SCT | TODO |
| `10-kernel/18-judgments.md` | Full typing judgment, algorithm, the kernel API surface | TODO |
| `20-verification/README.md` | The differentiator: overview | TODO |
| `20-verification/21-spec-syntax.md` | `requires`/`ensures`/refinements/goals | TODO |
| `20-verification/22-obligations.md` | VC generation; body-as-motive plumbing | TODO |
| `20-verification/23-prover.md` | Classifier; Z3-for-decidable; Kripke embedding; certificates | TODO |
| `20-verification/24-diagnostics.md` | Countermodels; typed holes + `unknown`; three-region Heyting | TODO |
| `20-verification/25-protocol.md` | Machine-readable diagnostic protocol (agent contract) | TODO |
| `30-surface/README.md` | Surface language overview | TODO |
| `30-surface/31-lexical.md` | Lexical structure, tokens, literals | TODO |
| `30-surface/32-grammar.md` | Concrete grammar | TODO |
| `30-surface/33-declarations.md` | Modules, declarations, definitions, visibility | TODO |
| `30-surface/34-data-match.md` | Sum types, `match`, exhaustiveness, `Result`/`Option` | TODO |
| `30-surface/35-numbers.md` | `Int`/`Decimal`/reals — the f64 correction | TODO |
| `30-surface/36-effects.md` | Effect tracking surface | TODO |
| `30-surface/37-strings-collections.md` | Strings, core collections | TODO |
| `30-surface/38-ffi-io.md` | `Bytes`, binary I/O, FFI | TODO |
| `30-surface/39-elaboration.md` | Surface → kernel elaboration, implicits, inference | TODO |
| `40-runtime/README.md` | Runtime / reference-semantics overview | TODO |
| `40-runtime/41-values.md` | Value rep, content-addressed heap, O(1) structural eq, dedup | TODO |
| `40-runtime/42-evaluation.md` | Operational semantics of the reference interpreter | TODO |
| `40-runtime/43-termination.md` | Totality, SCT, partial/`unknown` execution | TODO |
| `40-runtime/44-capacity.md` | Content store; slot-ceiling decision; lattice's real roles | TODO |
| `50-stdlib/README.md` | Prelude + core stdlib shape | TODO |
| `90-open-decisions.md` | The forks register (for the operator) | TODO |
| `_notes/analysis-digest.md` | Distilled digest of analysis + reality-check (subagent) | DONE |

(Conformance corpus seeds live in `../conformance/`, referenced from spec
sections; seeded opportunistically as core sections are written.)

## Drafting order (priority)

1. `_notes/analysis-digest.md` (subagent, in progress) — wait for it before
   writing analysis-specific detail; kernel core can start from first
   principles.
2. `00-overview.md` + `README.md` — frame.
3. `10-kernel/*` — the trust root; highest rigor; unblocks K1/K2/K3/X1/V0.
4. `20-verification/*` — the differentiator; unblocks V0–V4, T1.
5. `30-surface/*` — unblocks L-stream + V1.
6. `40-runtime/*` — unblocks X1/X2/K3.
7. `50-stdlib/README.md`, finalize `90-open-decisions.md`.
8. Seed `../conformance/` with core cases.

## Next action

Digest is DONE (`_notes/analysis-digest.md`, 750 lines — 11 sections incl. the
f64 correction, the FNV-1a-not-Leech addressing finding, 16 enumerated forks,
~25 corrected claims). Frame + `10-kernel/README.md` + `11-syntax.md` are DRAFT.
**Continue the kernel:** `12-universes.md` → `13-pi-sigma.md` → `14-inductive.md`
→ `15-identity.md` → `16-cubical.md` → `17-conversion.md` → `18-judgments.md`,
folding in digest specifics. Then seed `90-open-decisions.md` from the digest's
16 forks. Read the digest before the cubical/topos/numbers/runtime chapters.

## Open decisions captured so far

See `90-open-decisions.md` (created with the first fork). Known forks from the
strategy: content-store capacity bound (recommend: no hard ceiling); concrete
syntax; effect-tracking surface; the Space/process-isolation model.
