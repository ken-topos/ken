# Merging: the nine-step procedure and corpus git routing

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`.

`COORDINATION §14` defines the gate; this is the mechanics. **Run every step.
None is conditional on how routine the merge feels.**

## Before you publish

### M1 — Verify the Decision is `resolved`, read fresh from the object

```sh
# HTTP, your OWN credential. MCP list_decisions is NOT exhaustive.
# GET {API}/api/spaces/spc_4q7g0se87rgje/decisions?limit=20
```

`approved`, `proposed`, and `rejected` are not `resolved`. A Decision you
watched resolve earlier can be voided by an intervening publish — **re-read it
at merge time, never from memory.**

### M2 — Verify the exact SHA from the ring's `git_request` exists on origin

```sh
git fetch origin && git cat-file -e <SHA>^{commit} && git rev-parse <SHA>
```

**Never `--target HEAD`** — it dies with `src refspec refs/heads/HEAD does not
match any`. Always an explicit SHA.

### M3 — Cited-source check

One command, not a judgment:

```sh
while IFS= read -r f; do
  git show origin/main:library/SOURCE-ATTESTATIONS | awk '{print $2}' \
    | grep -qxF "$f" && echo "CITED: $f"
done < <(git diff --name-only origin/main...<SHA>)
```

Hits route to the **Librarian, after the merge**. Never into the ring's frame.
Rationale and the already-red-`main` question are in
`release-and-handoff.md`, step 7.

## Publish

### M4 — Mint a token

Agents hold no GitHub credential.

```sh
export GH_TOKEN="$(/workspaces/ken/.devcontainer/mint-gh-token.sh)"
```

### M5 — Run the publisher

```sh
scripts/scripted-pr-automerge.sh \
  --target <SHA> --title <pr-title> \
  (--description <text> | --description-file <path>) [--doc-only]
```

- **doc-only** — about two minutes; foreground is fine.
- **code** — `run_in_background: true`, always. It waits 581 to 718 seconds
  before its first poll, which exceeds a foreground tool timeout.
- **Never `git fetch` while it runs** — a lost ref-CAS reads as unverified.
- **Never pipe its output through `grep`** — block buffering swallows the poll
  lines.

**`resolved` is not the last gate. CI is.** Four candidates in one arc cleared
a `resolved` Decision and still failed CI.

The script creates the PR, waits and polls checks for non-doc changes, and runs
the publisher merge command. If GitHub blocks the merge it must stop and route
that fact; it must not pretend the publisher identity can self-approve.

## After it lands

### M6 — Verify by blob identity, every changed path

Ancestry lies after a squash; phrase greps lie on wrapped lines.

```sh
git fetch origin
for f in $(git diff --name-only <SHA>^ <SHA>); do
  r=$(git rev-parse "origin/main:$f" 2>/dev/null); l=$(git rev-parse "<SHA>:$f")
  [ "$r" = "$l" ] && echo "MATCH  $f" || echo "DIFFER $f"
done
```

The publisher squashes. `git reset --hard origin/main` afterwards —
`steward/work` is stale the instant any publish lands.

> **Why blob identity and not a verification phrase grep.** A phrase must not
> span `**bold**` or `` `code` `` markers — and, the mode that fires every
> time, must not span a **line break**. In a corpus hard-wrapped at 80 columns,
> any phrase distinctive enough to prove identity is long enough to wrap. The
> two requirements are in direct tension.
>
> Measured on PR #955: four verification greps, **two came back empty on
> content that was byte-identical on `origin/main`.**
>
> **The dangerous direction is the false negative becoming a false positive.**
> An empty grep says "not landed", which is alarming and self-correcting. But
> the instinct it trains is to shorten the phrase until it matches, and a
> phrase short enough never to wrap is usually short enough to appear in prose
> that predates your change. **Never weaken a probe to make it pass. Replace
> the instrument.**
>
> **Blob identity is necessary, not sufficient — it proves the file landed,
> never that the file is right.** Keep the index post-condition below.

### M7 — Flip the node, regenerate the tracker

```sh
sed -i 's/^status: active$/status: merged/' docs/program/issues/<ID>.md
scripts/gen-progress.sh
```

Bundle both into your next publish.

### M8 — Notify the Adversary, if the merge carries code

A step, not a courtesy.

**Doc-only merges do not concern the Adversary** (operator, 2026-07-29). Do not
notify it for them. Frames, tracker flips, node registrations, counters, and
corpus edits are not its surface. **`--doc-only` on the publisher is exactly
the discriminator** — if you passed it, skip M8.

For a code merge, look the id up at post time (`scripts/moot-actor-id.sh
adversary`) and post:

- the merged SHA and the resulting `origin/main`;
- the code paths and the size (`git diff --shortstat <SHA>^ <SHA>`), so it can
  bound its pass;
- anything you already know is unhunted or excluded.

> **Measured 2026-07-29: this step did not exist and the seat ran blind.** The
> Adversary was *described* as "event-driven on merge notifications" with
> nobody assigned to send one. It hunted four unnotified merges off its own
> currency checks and filed two findings, one soundness-adjacent on the kernel
> gate. **A requirement living in a descriptive sentence is never discharged —
> it needs a numbered step.**
>
> **The loop has a second half: its reports do not appear in the space-level
> event read.** A 200-event scan returned zero adversary posts while both
> findings were live and fetchable by event id. Every watchdog tick, read
> `GET {API}/api/spaces/{sid}/threads` and open any thread with
> `unread_count_for_actor > 0`. Notifying it and never reading it back is the
> same silence.

### M9 — Close the loop with the ring

Then run the stay-one-release-ahead check (`../steward.md`, section 4): every
node whose `depends_on` names this one is `ready` with a shovel-ready frame
before you stop.

## Routing your own corpus edits

Your operational docs — the progress tracker, `agent/` playbook and
`COORDINATION.md` edits — skip the spec-leader step and go straight to `main`.

1. **Commit on `steward/work`** when the working change belongs there.
2. **Route to a corpus branch off current `origin/main`:** `git fetch origin`;
   `git branch -f wp/steward-<slug> origin/main`; `git switch
   wp/steward-<slug>`; apply or cherry-pick the intended change. The branch
   starts as `origin/main` plus the routed change only, never a stale base.
3. **Append the tracker-sync commit before publication.** Pull the current
   progress file from `steward/work`, commit it if it differs, and treat the
   resulting branch tip as the PR SHA:

   ```sh
   git checkout steward/work -- docs/program/IMPLEMENTATION-PROGRESS.md
   git add docs/program/IMPLEMENTATION-PROGRESS.md
   git diff --cached --quiet || git commit -m "tracker: sync implementation progress"
   ```

   The `git diff --cached --quiet ||` guard is required: without it the commit
   fails when the tracker already matches. **This is the only copy of this
   procedure** — it also applies to every ring candidate you publish, so
   `origin/main` preserves the current progress file durably.
4. **Publish with the scripted publisher path** (M4 and M5) unless the operator
   routes otherwise.
5. **Sweep only after the merge is confirmed** — M6's blob-identity loop. The
   repository deletes remote head branches automatically; local cleanup is
   optional and must not delete a branch before `origin/main` is verified.

A multi-piece corpus change is one branch (`COORDINATION §14`). Width-check
markdown at 80 display columns (codepoints, not bytes) before routing.

### The index post-condition

When a corpus change carries an index — a `README.md` table, a manifest, a
catalog — assert the post-condition, not the phrase. Blob identity proves the
file landed; it says nothing about whether the change landed **whole**.

**Sweep every scope directory in one command, never one at a time:**

```sh
# rows vs files, BOTH orphan directions, EVERY scope
for d in $(find agent/memory -type d | sort); do
  ls $d/*.md >/dev/null 2>&1 || continue; [ -f $d/README.md ] || { echo "$d NO README"; continue; }
  files=$(ls $d/*.md | xargs -n1 basename | sed 's/\.md$//' | grep -v '^README$' | sort)
  rows=$(grep -oE '\]\(([A-Za-z0-9._-]+)\.md\)' $d/README.md | sed 's/](//;s/\.md)//' | sort -u)
  echo "$d files=$(echo "$files"|grep -c .) rows=$(echo "$rows"|grep -c .)"
  comm -23 <(echo "$files") <(echo "$rows") | sed "s|^|  ORPHAN FILE: $d/|"
  comm -13 <(echo "$files") <(echo "$rows") | sed "s|^|  ORPHAN ROW:  $d/|"
done
```

Three defects this replaces, all measured 2026-07-26 on one run of the old
snippet, each invisible to the run that had it:

1. **It was run on the two scopes the role loads** and reported "8 orphans" as
   the population. The real count was 10. **A per-directory command invites a
   per-directory population, and the report says nothing about the scopes it
   never visited.**
2. **An ad-hoc lowercase-only character class** made two rows with capitals
   read as false orphans. The class must admit every filename shape actually on
   disk — derive it from the corpus. Note the direction: this over-reported
   while defect 1 under-reported, in the same run, so the totals partly
   cancelled and both looked plausible.
3. **A known non-lesson file reports as a permanent orphan.** Exempt those
   explicitly, because a standing false positive is how you learn to skim past
   real ones.

**`git checkout <ref> -- <path>` is a checkout, not a merge.** It takes the
ref's blob wholesale and destroys what was there. Applying two branches that
share a file this way silently keeps only the second. (2026-07-22: two adversary
memory branches both appending to one index gave `10 base + 3 = 13` where `10 +
5 = 15` was due; two lessons landed with no index row, both commits clean, no
warning.) To combine branches use `cherry-pick` or `merge` so conflicts are
raised, then run the post-condition regardless.

**Predict the number before you look:** `base + delta1 + delta2`. That one line
of arithmetic is the detector.

**Why a post-condition and not a mechanism guard.** The loss was first blamed
on git's silent union of disjoint hunks. Measured, that was false — those
branches conflict loudly, and the loss came from a command with no merge
semantics at all. **A guard keyed to a mechanism story fails when the story is
wrong.** A post-condition on the merged artifact catches unions, wholesale
takes, and bad conflict resolutions alike, without needing to know which
occurred.

### Check what you broadcast, not only what you committed

The artifact and the announcement fail independently, and it is the
announcement that reaches rings as binding instruction. Twice in one session a
correct landed artifact was published alongside prose carrying a false
mechanism — once to two build rings as the *reason* for a rule.

After any publish you also narrate to the fleet, re-read your own message
against the artifact and ask whether the explanation reproduces what actually
happened. An explanation that merely sounds consistent with a true rule is not
thereby true. **Watch especially for a clause whose function is to tell the
reader they need not look** — "you cannot lose X by accident", "errs in the
safe direction", "immaterial".

### Keep `steward/work` fresh against `origin/main`

`steward/work` is a working copy, not a durable log: it should always be
`origin/main` plus at most the current unpublished tracker delta. It drifts
into a stale tree when tracker commits pile up on a base that never advances
while other teams merge. The symptom is a worktree carrying a superseded layout,
a giant false `origin/main..HEAD` diff, and merge hazards — editing files
against that stale base silently reverts other teams' merged work if you route
the branch.

- **On resume, after any merge notification (yours or another team's), and
  before starting new corpus work:** `git fetch origin`; preserve the tip
  cheaply with `git branch -f preserved/steward-work-$(git rev-parse --short
  HEAD) HEAD`; then `git reset --hard origin/main`. Your last publish already
  put the tracker on `main` via the tracker-sync commit, so the reset loses
  nothing durable. **Re-derive** the current tracker block against `main`'s
  version rather than blind-carrying a stale copy.
- **Never `git rebase origin/main` a long-lived `steward/work`.** A squash-merge
  leaves the original branch commits dangling ahead of `origin/main` while
  their content is already merged, so a rebase replays already-landed commits
  into conflicts. Reset-to-`origin/main` plus re-apply is the robust move.
- The corpus-branch route already cuts from current `origin/main`, so a fresh
  `steward/work` is not required to publish — but a stale one misleads you
  about what has landed and is the root of phantom "unmerged work" scares.

### The squash-merge trap

After a squash-merge the *original* branch commits dangle **ahead** of
`origin/main` while their content is already merged. Such a branch is a stale
leftover, not unmerged work. Grep `origin/main` for the squash commit before
treating branch-ahead commits as lost or held; do not re-open or recover them.
**Branch-ahead does not imply unmerged.**
