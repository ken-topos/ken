---
scope: fleet
audience: (see scope README)
source: private memory `check-main-via-git-object-store-not-find`
---

# Check main via the git object store, not `find`

To check "is X on `main`?", read the **git object store**
(`git show origin/main:<path>`, `git ls-tree origin/main <dir>`,
`git cat-file`), **never `find` the worktree filesystem**. Each worktree only
has its own branch's files checked out — a `find` from `runtime-leader/work`
can't see `main`'s files even though they're in the shared object store.
Corollary of the §14 local-git model (every agent shares one repo object store
across worktrees, so any branch/commit's files are readable via git plumbing).

**Why:** I cut `wp/X2-build` correctly from `origin/main` (`e18f4aa`), then
`find`-searched the worktree for `seed-capacity.md` and
`X2-runtime-hardening.md`, found nothing, and wrongly reported to the Steward
that the frame was missing. The files were there all along in the object store.
The steward caught it — 30-second diagnosis:
`git show e18f4aa:docs/program/wp/X2-runtime-hardening.md` showed +137 lines.

**How to apply:** whenever verifying whether a file exists on `main` (or any
ref), use `git show <ref>:<path>` or `git ls-tree <ref> <dir>`, never `find` or
`ls` on the filesystem. The worktree filesystem IS NOT the repo — it's one
checkout of one branch.

**Sibling trap — two-dot diff of a stale-based branch shows PHANTOM deletions of
intervening landed work (merge-review, 2026-07-01).** Reviewing
`wp/spec-errata-43cite` (a 2-file cite errata),
`git diff --stat origin/main..afab9de` showed **1297 deletions** —
`temporal.rs -302`, `b2_acceptance.rs -653` — appearing to **revert all of
B2-build**. It did not: the errata branch was cut from `0d617a3` *before*
B2-build merged to `main` (`805bfc3`), so a **two-dot** `A..B` (tip-to-tip) diff
conflates "B is behind A" with "B deletes things." The branch's OWN change-set
is the **three-dot** `origin/main...afab9de` (merge-base→tip) = exactly the 2
cite files, zero `crates/`. Confirm with a merge SIMULATION:
`git merge-tree --write-tree origin/main <branch>` → the OID's tree still has
`temporal.rs` (present = safe, 3-way merge keeps landed work). **How to apply:**
(1) for "what does this branch change?", always three-dot (`main...branch`) or
`--name-only origin/main...branch -- crates/`, never two-dot. (2) A 3-way
`git merge` of a stale branch is SAFE (uses the merge-base), but a
**tip-snapshotting** method (squash-that-snapshots, force-push,
rebase-onto-stale) CAN revert intervening work — so on any stale-based branch
**require a rebase onto current `origin/main` before merge**: makes the Decision
diff legible AND revert-proof regardless of merge method. Git-integration face
of multipiece erratum verify all on main (that = branch AHEAD, pieces dropped by
squash; this = branch BEHIND, phantom deletions).

**Watchdog false-stall corollary — `is-ancestor <branch-tip-SHA>` gives a FALSE
"not merged" after a rebase, and stale tmux panes lose to git (2026-07-02,
Steward).** I diagnosed K7 (`wp/K7-eq-at-inductive-whnf`) as a ~19h
dropped-Architect-gate stall and nudged the kernel-leader. **Wrong — it had
merged as a *rebased* SHA `4ae2baf`** (the merge path fixed a stale base), with the
whole downstream arc (`9a82745`, `b92cad6`) closed too. Three compounding
errors: (1) `git merge-base --is-ancestor b7396ae origin/main` = NO — but
`b7396ae` is the **pre-rebase branch tip**; a rebase-then-squash lands the
*content* under a new SHA, so the old tip is *correctly* a non-ancestor even
though the WP is fully merged. **Never test "did WP land?" with `is-ancestor` on
the branch tip.** (2) `git log origin/main --oneline -15 | grep` — the merged
commits were **17-25 below tip**, outside the window → false "no match." (3) The
tmux panes: idle agents **don't re-render**, so the kernel-leader/implementer
panes still showed pre-merge "awaiting…" frames; I read those as current and
read the *actually-current* reviewer/publisher "nothing queued / no open PR"
as *confirming* the stall when it meant **DONE**. **How to apply:** to verify a
WP landed, `git log origin/main --oneline -40 | grep -i "<WP-id/subject>"`
**or** grep the delivered *content*
(`git grep "<symbol>" origin/main -- <path>`) over a DEEP window — never
`is-ancestor` on the branch SHA; and when a pane disagrees with git, **git
wins** (a pane is a possibly-stale render, pane suggestion text is not agent
state). "No open PR / nothing in the reviewer's queue" is ambiguous: it means
*done* as often as *stalled* — disambiguate with the log, not the pane.

**REFLEX (2026-07-03, hit this trap TWICE more in one session — make it
automatic).** Before flagging *any* stall or dropped vote, run the merge check
FIRST: `git log origin/main --oneline -40 | grep -i "<subject/PR>"`. (1)
surface-transport-v2: I nudged CV off a `proposed` **decision object** — but a
fold had re-anchored the candidate SHA and the work was live on `main` as
`0ed7c07` (PR #255); a `proposed` decision object is NOT evidence of an open
vote. (2) State-effect-build-rt: I flagged it a ~8h stall because
`is-ancestor df7b93b` = NO — but it was **squash-merged** as `5c8dac0` (PR
#240), all 4 gates approved 8h prior; the branch tip lingering as a non-ancestor
is the *expected* post-squash state, and runtime-leader's "holding for CV vote"
participant status was just stale. In BOTH the git log showed the merge
immediately. A participant status line and a decision-object status are both
**possibly-stale renders** that lose to `origin/main` — same as a tmux pane. The
one time I *did* pre-check (State-effect, via `get_activity`) I avoided the bad
nudge; the one time I didn't (surface-transport-v2) I posted a false nudge.
Pre-check every time.

**Reconcile-grep corollary — an independent reviewer's "grep=zero, this citation
doesn't exist" can itself be a stale-base false-negative (State-effect,
2026-07-03).** conformance-validator raised a fidelity finding against
spec-author's `wp/State-effect` citing `prelude.rs`/`Prod`/`MkProd` as absent
(grep=zero). spec-author re-grounded against the authoritative ref
(`origin/main`, plus the WP branch itself) and found both symbols present and
correctly cited — CV's grep had run against `conformance-validator/work`, whose
base was ~94 commits behind `main` (the same stale-base class already flagged on
the unrelated #235 rebase that same session). **How to apply:** when
two roles disagree over "does X exist in the code," before treating either
verdict as ground truth, confirm BOTH are grepping the SAME ref (`origin/main`
or the WP branch tip) — a reviewer's own home worktree is exactly as capable of
being stale as an author's, and a "not found" from a stale base reads identically
to a real absence. This is the reconcile-dispute face of the same
object-store lesson: check *which ref*, not just *which method* (git show vs
find). **Two-axes corollary (State-effect finding #2):** a structural-token
review has two INDEPENDENT axes — (1) *does the cited thing exist* (grep the
authoritative ref) and (2) *is it the right construct for the claim*
(category/identity). They don't cover for each other: my finding #2 hung a VALID
axis-2 catch (§4.2's Σ-pair `R × S` ≠ the inductive `data Prod` — distinct
constructs the denotation must not equate) on an INVALID axis-1 claim (`Prod`
grep=zero, stale-false — `Prod` IS landed). A passing landedness grep does not
clear a category/identity claim, and a stale axis-1 false-negative must not be
allowed to either kill or launder an axis-2 finding — surface them separately
(the category catch was real and improved the spec even though the landedness
half it rode on was wrong).

**Stacked-flip corollary — a second doc-flip stacked on an UNMERGED first flip
goes stale the instant the first squash-merges, with a scary-but-usually-phantom
crate "revert" (Eq-flip close, 2026-07-02).** When two sequential errata/flips
touch the SAME file (e.g. `§6` antisym/sound/complete realized-flip #39, then
the `Eq` sym/trans reframe), you often must cut the second off the first's
UNMERGED tip (both edit overlapping lines). Once the first squash-merges
(`9334da8` → `b92cad6`), the second branch has: (a) a **genuine 3-way prose
conflict** (its base duplicates the squashed content), AND (b) an **apparent
crate-file revert** — `git diff origin/main:lawful_classes.ken <branch>:…` shows
the branch reverting proofs that landed via an *intervening* merge (`b3cbaaa`)
the stale base predates. (b) is a **FALSE alarm iff the branch's own commits
never touched the crate files** — pure stale-base inheritance, the two-dot
phantom again, one file over. **Fix + verify (I did this, spec-leader/publisher
re-verified):** `git rebase --onto <post-first-merge-main> <old-base> <branch>`
drops the redundant first-flip commits, replays ONLY the net-new commits onto
current `origin/main`. Then prove it clean: (1)
`git diff origin/main HEAD -- crates/ catalog/packages/` is **EMPTY** (crate files
byte-identical → no revert), (2) the prose diff vs main is ONLY the net-new
reframe (grep it doesn't re-touch the already-merged first-flip lines), (3)
`merge-tree --write-tree` clean. Rebase re-parents only, changes no content →
**the gate APPROVEs carry to the fresh SHAs** (re-anchor the Decision, no
re-review). Playbook line: a stacked flip MUST be rebased onto post-first-merge
main before its own merge, and the crate-byte-identity check is what
distinguishes a real regression from the phantom.
