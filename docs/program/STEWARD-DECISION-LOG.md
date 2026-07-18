# Steward decision log — autonomous window 2026-07-18

Operator (Pat) away ~03:47 UTC → ~11:30 UTC. Mandate: keep the fleet moving;
use `docs/PRINCIPLES.md` for judgment calls; route around anything needing
operator input (log it here + keep other work moving); merge on gates + CI green
per the standing gate model (Runtime/QA + Architect §14 + CV + CI — there is no
separate operator-merge gate). Pat may check in once or twice.

**Routing rule for the window:**
- **Design/component forks** → Architect (`agt_37reqftfe6g00`).
- **Genuine product/priority forks not settled by roadmap + PRINCIPLES** →
  log here, `defer_question` to Pat, and keep all other work moving. Do NOT
  guess a one-way-door product decision.
- **Everything gated + green** → merge autonomously (publisher), verify
  byte-identical, PIN, chase retros.

**Settled inputs (do NOT reopen):** R2 (no Ken affine/linear type; safety in
runtime+Ward), Ward is a separate project (Ken only exports the boundary),
collapse-V1/V2-no-compat (PRINCIPLES transient T), Linux-ABI-direct (POSIX
dropped), targeted-builds-only (no local `--workspace`), operator approvals are
fixed inputs.

## Plan of record for the window
1. **PX8-N** (bounded-Nat reply-lowering) — in fresh Runtime QA on rebased tip
   `9bd52149` (conflict-free onto PX8-X main, retains §14 `dec_vh7vdv5428p0`).
   On QA APPROVE → republish non-doc-only → merge → PIN → chase retros.
2. **PX8-F** (buffer-IO surface) — held at `60a481b5`; unblocks once PX8-N lands.
   Rebase onto combined main + re-kick Foundation (corrected terminal obligation:
   author the FIRST real linked checked `writeAll` native fixture, per Architect
   `evt_4kh6gz18tvzs6`). This completes the PX8 buffer-IO floor.
3. **Continue the Linux ABI campaign** (`docs/program/09-posix-linux-abi-campaign.md`)
   — decompose + sequence the next WPs from the charter after the PX8 floor,
   handoff-gate + kick per §2c. Author shovel-ready T1 briefs.
4. **Test-suite review WP #24** stays gated until the ABI campaign lands.

## Default stances (flag on check-in if you disagree)
- **Merge autonomy:** I will merge any WP that clears its gates + CI, including
  ABI/OS-interface landings — the gate model is the safety net. I will NOT invent
  an extra hold unless a change is a genuine one-way door with no PRINCIPLES-clear
  answer, in which case I hold at the pre-merge gate and log it.
- **Credit:** I will run the enclave for T1 brief-authoring as the roadmap needs
  but keep the compact-at-seam discipline to conserve.

## Decisions
| UTC | Decision | Basis | Reversible? |
|---|---|---|---|
| ~03:47 | Continue ABI campaign per charter after PX8 floor; merge on gates+CI green | roadmap + standing gate model | yes (Pat can re-prioritize on check-in) |
| ~03:51 | **Window priority CONFIRMED by Pat: continue the Linux ABI campaign per the charter.** | operator directive | n/a (explicit operator call) |
| ~03:53 | PX8-F brief terminal-obligation corrected (author FIRST writeAll native fixture, not rerun); re-kick-ready pending PX8-N landing. ABI next-WP = PX9 (structured errno, Foundation) per charter §6 PX7→PX8→PX9. | Architect evt_4kh6gz18tvzs6 + charter | yes |
| ~04:00 | Publish PX8-N `9bd52149` non-doc-only WITHOUT tracker bundle (base predates current-main tracker deltas → bundling would conflict); sync tracker+log via separate doc-only merge after. | avoid self-inflicted tracker merge conflict (cadence exception, logged) | yes |
| ~04:14 | PX8-N MERGED+PINNED on `origin/main = ace72db7` (PR #763). Verified all 9 reviewed paths + full tree byte-identical to §14-approved `9bd52149`; `dec_vh7vdv5428p0` discharged. Runtime §10 retro requested (`evt_2agz5010dpaz8`). | gate model + byte-identity verification | n/a (landed) |
| ~04:20 | PX8-F treated as a held-WP **semantic unfreeze** (not a quick same-WP respin): the schema collapsed underneath Foundation (PX8-X V2→sole unversioned) and the native carrier landed (PX8-N), so their held context is stale → run the FULL handoff-gate compaction of Foundation L/I/Q before re-kick. Brief flipped HELD→RESUME, base pinned to `ace72db7`. | [[held-wp-unfreeze-is-a-semantic-rebase-rederive-anchors]] + compact-before-new-work rule | n/a (process) |

