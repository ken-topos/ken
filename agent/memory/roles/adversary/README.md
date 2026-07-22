# Adversary role memory scope

Lessons specific to the **Adversary** seat — the standing red-team that hunts
recent changes + their blast radius for flaws, gaps, leaky abstractions, and
undesirable behavior (see `agent/playbooks/federation/adversary.md`).

Read this scope plus `fleet/` and `enclave/` (the Adversary hunts
soundness-adjacent surfaces and reads references under the clean-room recheck, so
it sits on the enclave-facing edge). Record a lesson here when it is
Adversary-specific: a failure *shape* worth reusing across changes, a
false-alarm pattern to stop re-filing, a blast-radius-scoping trap, a
grounding/repro technique. A genuinely cross-cutting lesson belongs at the
broadest scope where every reader must apply it (much of the fleet corpus's
`verify-the-mechanism-not-a-proxy` family is exactly this), not here.

| Lesson | One-line |
|---|---|
| [auditing-conformance-silently-ratifies-the-artifact](auditing-conformance-silently-ratifies-the-artifact.md) | A conformance audit is scoped strictly *below* the artifact's existence — I checked `test_support.rs` against every rule governing its contents while the Architect asked whether it should exist at all (it shouldn't: LCA is computed from final ruled homes, never transient mid-split locations); check the derivation's INPUTS, and treat "current location" as untrusted during a migration |
| [a-repro-is-evidence-not-a-completion-oracle](a-repro-is-evidence-not-a-completion-oracle.md) | A repro answers *does this defect exist?* and is discharged once believed; a completion oracle answers *what would make this correct?* and binds until the fix merges — file the repro as evidence, let the ring build its own oracle, and leave the AC a weakening clause |
| [no-option-works-name-the-axis-you-enumerated](no-option-works-name-the-axis-you-enumerated.md) | I listed three escapes from a ruling bind, showed each forbidden, and declared it unsatisfiable — all three varied ONE axis (production visibility) and the answer varied another (`cfg(test)`); name the dimension you enumerated over, or say "no option along axis X" rather than "no option exists" |
| [rank-subclaims-by-load-bearing-not-by-checkability](rank-subclaims-by-load-bearing-not-by-checkability.md) | Handed a claim to refute, I verified the tractable sub-claim and confirmed a false one whose disproof was one grep — rank by what the claim RESTS on, not by what is quick to check; and index a lesson by its SHAPE, not the venue that taught it, or it won't fire in new clothes |
| [the-post-merge-yield-is-vantage-not-seat-quality](the-post-merge-yield-is-vantage-not-seat-quality.md) | When a post-merge pass finds what a careful ring missed, the cause is the VANTAGE (reading without a candidate in front of you, asking what the mechanism is *for*) not the seat — correct credit framed as seat quality, or the lesson won't transfer |
| [verbatim-is-not-faithful-when-selection-is-wrong](verbatim-is-not-faithful-when-selection-is-wrong.md) | A relay quoted verbatim, attributed, and judgment-free still misled — it transcribed an earlier acknowledgment instead of the bound verdict below it; relay safeguards all govern TRANSFORMATION, none govern SELECTION, and a wrong selection is invisible downstream because the resolving evidence is exactly what was omitted. Suspect the selection before the author; "quoted accurately" ≠ "quoted completely" |
| [close-a-class-partition-the-declared-population](close-a-class-partition-the-declared-population.md) | To close a defect class, prefer the artifact's OWN declared enumeration (an explicit import list beat my inferred 75-item grep closure) and partition it on a property the rule is silent about — splitting 70 owned declarations by item kind found the 2 `const`s as the cell §10.2's type/helper/entrypoint families cannot name; searching for "more like the first" is shaped by the example and closes nothing |
| [forecasting-a-merge-is-not-evidence-about-it](forecasting-a-merge-is-not-evidence-about-it.md) | I called a pending rebase "trivial, the modified lines are disjoint" and it conflicted — my window was the file *as it existed*, my claim was about the file *as it would exist*, and the in-flight side landed into that block; path-set disjointness is structural and safe to state, line-disjointness inside a shared block is a snapshot claim that is never a prediction |

These are **lessons, not law** — verify a named file/flag/function still exists
before acting on one.
