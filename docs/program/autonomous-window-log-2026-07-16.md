# Autonomous-window decision log — 2026-07-16 (~02:40 → ~11:30 UTC)

Operator (Pat) offline; Steward running approved queue autonomously. Standing
instruction: lean on `docs/PRINCIPLES.md`; where no clear answer exists, **route
around the question and work on what is known**, logging the deferred question
here for review. Respect settled inputs: **PX7 HELD** (do not release), **rustix
SETTLED**, **no linear/affine types**, **multi-root FS authority DECLINED** (keep
ADR-0018 single-authority-per-family), **local builds targeted-only, never
`--workspace`**. This log is the review artifact; the durable state backbone
remains `IMPLEMENTATION-PROGRESS.md`.

## Operating posture for the window

- **Frontier:** PX5 B1–B4 repair (Runtime, in progress) → runtime-qa gate →
  §14 Decision (Architect) → CI → publish → content-verify → PX6 (Verify)
  auto-resumes on the PX5 merge.
- **Then:** route the **shared Architect capability-model ruling** (gates
  PX13/PX14/PX15) once PX5+PX6 are merged; on that ruling, author shovel-ready
  frames and release PX13/PX14/PX15. All within-federation; no operator input
  needed unless the Architect surfaces a genuine product call (→ route around +
  log here).
- **Publish authority — STANDING (Pat confirmed 02:55, "Publish approved and
  green candidates, standing authority… you never need to ask me").** Run the
  scripted publisher on any §14-resolved + CI-green candidate without asking;
  never publish anything not both design-approved and CI-green. Doc-only tracker
  syncs bundle into each merge per §2a. **Settled input — do not re-ask.**

## Decisions

| # | UTC | Decision | Grounding / rationale | Reversible? |
|---|-----|----------|-----------------------|-------------|
| 1 | 02:40 | Open this window log; treat publish authority as standing | Established flow; Pat expects the PX5→PX6 chain to progress while offline | n/a |
| 2 | 02:55 | Publish authority CONFIRMED standing by Pat ("you never need to ask me") for §14-resolved + CI-green | Pat's explicit grant; settled input, do not re-ask | Pat can revoke |
| 3 | 03:06 | Sent a bare Enter to the architect pane to submit a STRANDED §14 re-review request (transport repair, not rewrite) | Monitor bp808a7mu + wide capture confirmed `[Pasted Content]` unsubmitted >=60s; §2c transport-verify discipline | n/a |
| 4 | 03:06 | Determined the implementer's "whole-package fmt red" does NOT block the PX5 merge | Grounded `.github/workflows/ci.yml`: CI runs only `build/test --workspace --locked` + placeholder echos — no rustfmt gate. Publish gate is CI-green on those, unaffected by fmt | reversible if CI config changes |
| 5 | 03:25 | Did NOT interject on the 2nd §14 BLOCK (B2/B4 vacuous on `0b26583f`); logged a QA detector-strength gap as a §10 retro carry instead | Architect gave precise B2/B4 dispositions + implementer already on the respin; a mid-respin nudge = noise (thin-flow). QA approved a `!=`-on-temporaries closure test + textual trace at face value — capture for retro, not live | n/a |
| 6 | 03:5x | PUBLISHED PX5 `bfc12020` under standing authority (bg bp9db4a64, PR #732) | §14 APPROVE + QA APPROVE + Decision `dec_22hxvhcrah7ac` resolved; provenance verified vs origin/main (base 5babaed5, 12 linear, clean diff, zero kernel/spec/Cargo.lock/conformance). Script merges only on CI-green (`--match-head-commit bfc12020`) | merge is to main — not reversible once landed; CI-red aborts it |
| 7 | 03:5x | Published EXACTLY as approved; did NOT bundle a tracker commit onto the candidate | A tracker commit changes the SHA and unbinds the §14 approval + `--match-head-commit`. Separate --doc-only tracker-sync PR after merge (PX4B precedent) preserves the exact approved SHA | n/a |
| 8 | 03:5x | Let the publisher force-push `wp/px5-native-effect-lowering` 966f804f→bfc12020 rather than routing back to the ring for the push | Origin wp branch was stale at the frame commit (implementer's push never landed on origin); the publisher's `--force-with-lease` resolves it against the shared local ref at the exact approved SHA — no ring round-trip needed | n/a |
| 9 | 04:0x | PX5 CLOSED (all 3 §10 retros in) + doc-only tracker/window-log sync merged to main (PR #733, `c87f3d2e`) | Retros confirm-in gate met; sync is file-scoped from fresh origin/main (steward/work is far behind main — never merge it wholesale) | n/a |
| 10 | 04:1x | Resumed PX6 (Verify) via handoff-gate: compacted the verify ring + kicked the resume onto `049628f8` with the exact landed PX5 API pins | Logged new-work-vs-changed-dep decision; ring was clean/behind (0-ahead) = lossless reset; kick named `run_bound_process_effect_observation_v1` (root B = `options.cwd`), `EffectObservationV1` (import), forbade the cfg(test) proxy | n/a |
| 11 | 04:50 | PX6 §14 BLOCK (`b7e7a14d`) → **opened new WP PX5B (Runtime)** and parked PX6 on it; did NOT ask Pat | Architect found a real producer-seam defect: the interp *oracle* built its trace from `Scenario.expected_fs`, not observed dispatch (a raw-path bug could pass falsely). Fix crosses into `ken-interp` (Runtime's), and Verify authoring the oracle it judges = ADR-0018 §5 anti-pattern → Runtime owns it. Mechanism was Architect-specified (evt_6q01xcfrmzz16), so this is a routing/ownership call within my authority, not a product decision. Frame authored + pushed (`wp/px5b-… @ aed1eb32`); catalog + tracker updated | Reversible: if you'd rather fold the interp hook into PX6 with a Runtime-authored carve-out, say so and I re-route — but the producer/judge split is the principled default |
