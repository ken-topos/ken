---
scope: roles/steward
audience: (see scope README)
source: 2026-07-30 — four tracker nodes audited, four false or absent
  durability claims, culminating in the live base of the in-flight candidate
  sitting on exactly one local ref
---

# A durability claim in an artifact reads as verified and was never checked

Four nodes in one pass, one shape: **an artifact asserted that work was
preserved, and nothing had ever resolved the ref.**

- `PX8-ERRID-ALLOC` called a ref *"protected"* that **did not exist at
  `origin`**; the resume point (`ad7298fb`) was local-only, and the branch
  that *did* exist was two days stale.
- `NATIVE-HANDLE-CARRIER` carried the **correct** warning — *"a recorded SHA
  is not a copy; the hazard is a hard reset from a handoff gate"* — and the
  copy had never been made. **The warning being right is what stopped
  anyone checking whether it had been acted on.**
- `RT-DECL-CLOSURE-PORT`'s **live base**, which four recuts were grounded on,
  was on one local ref: the seat's own working branch. A handoff-gate
  `git reset --hard` would have destroyed both atomic nodes' base mid-flight.

**How to apply:**

- **Resolve every ref you write down, at the moment you write it:**
  `git ls-remote --exists origin <ref>`. A `preserved at …` line in a
  handoff post is a **claim**, never a resolved ref.
- **The bounding predicate — a cited SHA needs a durable ref ONLY when an
  artifact tells someone to RESUME from it.** A corpus-wide audit returned
  ~200 undurable cited SHAs; acting on that number would have put ~200 dead
  refs on `origin` and buried the handful that matter. Almost all are
  historical evidence citations from work that landed squashed. **"Cited" is
  not "load-bearing."**
- **Divergent preservation points do NOT nest.** Three
  `NATIVE-HANDLE-CARRIER` seams were each on a separate lineage — none an
  ancestor of another — so preserving the newest silently dropped the rest.
  Check with `git merge-base --is-ancestor` before treating a newer ref as
  subsuming an older one.
- **Ancestry cannot tell a superseded branch from one holding unlanded
  work, because the publisher SQUASHES.** Two branches both read "not an
  ancestor of `main`" and needed **opposite** treatment: one was garbage (its
  content had landed under a different SHA) and one held real unlanded work.
  **Only a per-file blob comparison separates them** — so never delete or
  preserve on an ancestry read alone.
- **Pushing a `preserved/*` ref to `origin` is not an off-box copy** (which
  the operator has ruled out as waste) — it is durability against a worktree
  reset, and that precedent is accepted.

Sibling of [[committed-is-not-reachable-publish-then-verify-on-main]]: the
same failure one layer out. There I verified the artifact and not its
reachability; here I verified the *sentence about* the ref and not the ref.
