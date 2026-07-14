---
scope: fleet
audience: (see scope README)
source: private memory `use-tier-labels-never-model-names`
---

# Use tier labels, never model names

Operator correction (Pat, 2026-07-04). Two parts:

1. **"~1 year behind" (and any "weak/behind build model" framing) is
   INACCURATE.** The T2 build tier is a **current** model released days ago, not
   a year behind SOTA. I picked that phrasing up from stale playbook boilerplate
   and repeated it in a WP kickoff + docs. Don't characterize the build tier as
   weak/behind — the reason to front-load design onto T1 is **structural**
   (clean-room + concentration + no mid-execution design), not tier weakness
   (opus enclave authors shovel ready wps).

2. **Never name a model or a model characteristic in any artifact/message.** Use
   **tier labels only** — **T1** (Opus-class), **T2** (Sonnet-class), **T3**
   (Haiku-class). This covers playbooks, WP frames, the tracker, convo posts,
   everything. The **only** place the tier↔model mapping is written is the table
   in **`agent/MODELS.md`** — so when the model family swaps, one table changes
   and every downstream tier reference stays correct.

**Why:** the model roster changes fast (a new T1/T2/T3 family — GPT 5.6 →
**Sol** / **Terra** / **Luna**, ordered by *mass*: star > planet > moon — is
expected soon), so any doc that hardcodes a model name or a "N-behind"
characteristic goes stale/wrong the moment the seat changes. Tier is the stable
abstraction; the mapping is centralized.

**This lesson's own example was wrong until 2026-07-14** — it had Luna and Terra
transposed, propagated from `MODELS.md`. That is the point, sharpened: a model
*name* is a fact you can get wrong and never notice, because nothing type-checks
it. `agent/MODELS.md` is the **single** place the mapping lives; cite the tier
(`T2`), never the name.

**How to apply:**
- Writing a WP frame / tracker entry / convo kickoff: say "T2 build team" / "the
  T1 enclave", never "Sonnet"/"Opus"/"the ~1yr-behind model".
- Fixing existing docs: neutralize prose to tiers; leave the **MODELS.md mapping
  table** (and role-launch frontmatter `model:` slugs in `moot.toml` / skill
  headers — those are functional config) naming models, since that IS the
  mapping.
- Historical tracker log entries that record an actual model-swap *event* can
  keep the model name (they document what happened) — the rule binds go-forward
  prose, not the historical record.

Supersedes the model-naming convention in older memories; those that name models
(e.g. rollout/proxy operational notes) stay as historical fact but new writing
uses tiers. Sibling of reason in agent team hours not human days (both are
operator recalibrations of a stale mental model about the fleet).
