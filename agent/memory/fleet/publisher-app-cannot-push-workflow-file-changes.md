---
name: publisher-app-cannot-push-workflow-file-changes
description: "The scripted-publisher GitHub App CANNOT push changes to .github/workflows/ (it lacks the `workflows` permission) — a branch touching any workflow file is rejected at push with 'refusing to allow a GitHub App to create or update workflow ... without workflows permission', so it never merges. Enforce CI gates as WORKSPACE TESTS (run by the existing cargo test CI), not by editing a workflow file. Any WP that wires a gate via .github/workflows/ is unpublishable via the scripted publisher."
metadata:
  node_type: memory
  type: reference
  scope: fleet
---

**2026-07-13 — kenfmt capstone C publish blocked.** C was Architect+QA
APPROVED (`91ea984d`) and honesty-gate-clean, but the scripted publisher
(`scripts/scripted-pr-automerge.sh`) failed at the **branch push** — not CI, not
review:

```
! [remote rejected] wp/kenfmt-c-capstone -> wp/kenfmt-c-capstone
  (refusing to allow a GitHub App to create or update workflow
   `.github/workflows/ci.yml` without `workflows` permission)
error: failed to push some refs
```

The publisher GitHub App's token **lacks the `workflows` permission**, so it
cannot create or update **any** file under `.github/workflows/`. The push is
rejected **before** the PR is created — so nothing lands, no PR, `origin/main`
untouched (clean failure, no partial state to unwind). This is independent of the
change's correctness; a one-line CI-step addition is enough to make the whole
branch unpushable via the publisher — which is the **only** GitHub-touching path
(agents' direct `gh` is not authed).

**The rule / preferred design:** enforce CI gates as **workspace tests**, not as
new workflow-file steps. A gate expressed as a `#[test]` that reads the target
files from disk and asserts the property (e.g. read each corpus file, assert
`format(file) == file` / `ken fmt --check` clean, naming any offender) is run by
the **existing** `cargo test --workspace --locked` CI step — so it enforces
day-one and everywhere cargo test runs, **and** the candidate touches no
`.github/workflows/`, so the publisher can push it. This is both more robust
(broader enforcement) and publishable. Frame CI-gate WPs this way from the start.

**How to apply:**
- **Framing:** when a WP's acceptance includes "wire a strict gate into CI,"
  specify it as a workspace test, NOT a `.github/workflows/*.yml` edit. Call this
  out in the frame's guardrails.
- **Honesty-gate:** before publishing, check whether the candidate touches
  `.github/workflows/` (`git diff --name-only <base> <cand> -- '.github/**'`). If
  it does, expect a push rejection — route a gate-relocation re-spin *before*
  attempting the publish, don't burn the failed push.
- **If a real workflow change is genuinely required** (rare), that is an
  **operator action** (grant the App `workflows` permission, or the operator
  lands the workflow file manually) — and it is **security-sensitive**
  (workflow-write = supply-chain surface), so escalate it as an operator decision
  rather than working around it.

Sibling to [[scripted-publisher-target-is-head-branch-never-main]] and the
publisher discipline in the steward skill (§ scripted publisher path). The
publisher is mechanical and permission-bounded; design merges to stay inside its
permission envelope.
