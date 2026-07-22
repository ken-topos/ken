---
id: SRC-ATTEST
title: "squash-stable whole-source attestation + fresh merge-result authorization"
status: ready
owner: doc (Part 1) + steward spillover (Part 2)
size: M
gate: none
depends_on: []
blocks: [ORACLE-VIS-PACKAGING]
github: null
origin: >
  Librarian impossibility proof (evt_6t6wz1aw18291) during ORACLE-VIS-PACKAGING
  re-validation; Architect ruling dec_7q3kes0jcx1kn (evt_5vp06mb9v26mh).
  Framed by the Steward from that contract.
---

## Why this exists

**The Librarian proved that no valid `library/REVISION` can ride a PR that
edits a cited source.** Constructed and executed, both arms:

| choice | outcome |
|---|---|
| `REVISION` = current `origin/main` | **source-currency fails** — the cited source differs between `REVISION` and `HEAD` |
| `REVISION` = the branch tip | **squash-stability fails** — branch-local, not an ancestor of `origin/main` |

⇒ **No third commit is simultaneously an ancestor of `origin/main` and a holder
of bytes that have not merged yet.** That is a proof by the gate's two
predicates, not a tooling inconvenience, and **rebase does not help** — it makes
`main` an ancestor of the candidate, but the cited source still differs from
`main`.

★ **This generalizes to every WP that touches a manifest-cited source, forever.**
`ORACLE-VIS-CHECK` hit it, `ORACLE-VIS-PACKAGING` is held by it, and the next one
will hit it too. It is not a property of any candidate.

## ⛔ The anomaly, and the wrong explanation it nearly got

PR #885 changed `px4b_native_production.rs` — cited — left `REVISION` stale, and
shard 4/4 came back **SUCCESS**. Two explanations were proposed and **both were
wrong**:

- *"The predicate tightened since."* **Measured false** — `gen-doc-status.sh` is
  byte-identical between `dd715950` and `b2fd95ac`.
- *"CI's shallow checkout makes the PR-time gate weaker."* **Measured false** —
  the Librarian reproduced the current merge result at depth 1 and got the same
  red. The shallow-heal path is not the divergence.

**The Architect's graph reconstruction is the actual cause:** `c0890b13` (#885's
base) had **zero** `px4b` citations and its exact-tree check exits 0; **DOC-W1-5
added the two citations at `1e148908`, after #885's green check had already
formed.** Reconstructing the *later* merge result at `dd715950` makes the
**unchanged** checker exit 1.

⇒ **#885 carried a green check for an OLDER merge result after `main` advanced.
It is a TIME-OF-CHECK defect, not a weaker PR-time predicate.**

⚠ **Framing note for whoever builds this, because it cost two seats a wrong
lead:** the citation-presence question is a **PR-base** question and was twice
measured against **post-merge `main`** (`dd715950^`), which is a different point
in the graph. *Asking a pre-merge question of post-merge state returns a
confident, wrong answer.* Check `c0890b13`, not `dd715950^`.

## Part 1 — source-currency representation

Keep `library/REVISION`, but **narrow it to an already-on-`main` provenance /
bootstrap anchor.** It is **no longer** the proof that every current cited byte
existed at that commit, and `STATUS.md` must stop describing it as a "validated
revision" in a way that implies branch-only bytes are in it.

Add a canonical ledger, e.g. `library/SOURCE-ATTESTATIONS`:

```
# ken-source-attestation-v1 object-format=<git object format>
<full blob oid><TAB><normalized repository-relative path>
...
```

**The filename is not load-bearing. These semantics are:**

1. **Population** = every source path selected by the current manifest records to
   which `source-currency` applies.
2. Strip `#anchor` **only** to deduplicate the path. **Anchor existence remains a
   separate gate.**
3. Rows sorted by path, binding each path to its **whole tracked-file Git blob
   OID at `HEAD`**.
   ⛔ **Do NOT hash extracted Markdown spans** — that introduces a new
   section-boundary parser and **weakens** the current conservative whole-file
   predicate.
4. **Exact set equality.** Missing, extra, duplicate, noncanonical, nonexistent,
   non-blob, or symlink rows **fail closed**.
5. **Exact OID equality.** Changed bytes without a fresh Librarian attestation
   are red. ★ **A candidate-time attestation is squash-stable because the blob
   survives even though the branch commit does not** — this is the whole point,
   and it is what dissolves the impossibility.
6. Render both values **distinctly** in `STATUS.md`: **provenance revision** and
   **attested source-set digest/root**. The ledger is the source-currency
   authority; `REVISION` is provenance.
7. ⛔ **Replace** the old cited-byte `REVISION → HEAD` equality with ledger
   `→ HEAD` equality. **Keeping both preserves the impossibility and defeats
   the design.**

## Part 2 — fresh merge-result authorization

**The ledger does not by itself close #885.** A later manifest change can alter
the required population after CI has formed its verdict.

Immediately before **every** merge — **not only `--doc-only`** — the publisher
must:

0. **acquire a repository-wide, fail-closed merge lock** (see the boundary
   below — without it clause 2 has no support at all);
1. read current `origin/main`;
2. construct the exact squash result with the candidate;
3. run **`origin/main`'s checker**, not the candidate's, on that result;
4. **re-read `origin/main` before merging** and abort/reconstruct if it moved;
5. **after merging, verify the LANDED tree** against the checked tree OID and
   re-run the checker on it — **alarm and freeze on mismatch, never auto-revert.**

⇒ **Old green CI attached to a prior merge result is not authorization.** That
much is unconditional, and it is what closes #885's stale-CI defect.

### ⛔ CORRECTED — the identity claim this frame originally made is WITHDRAWN

The first version of this section said: *"Because the publisher is the sole merge
router, that makes the checked result the result it lands."* **The second half
does not follow, and I wrote it. @architect ruling `dec_50fdjy68gm01j`.**

`gh pr merge` offers `--match-head-commit`, which pins the **PR head**. **GitHub
exposes no base-SHA compare-and-swap**, so the squash lands on whatever `main` is
at the instant the API executes, and nothing the publisher passes can pin that.

State the guarantee in three parts, and do not collapse them:

| | claim | holds because |
|---|---|---|
| **Unconditional** | old CI is never authorization | every publish re-derives the result on a fresh `origin/main` and runs `origin/main`'s checker immediately before merge |
| **Conditional** | the checked tree **is** the landed tree | ⚠ only within ADR-0003's exclusive-publisher model, and **only because all sanctioned merges share one enforced critical section** — the lock |
| **Residual** | ⛔ **NOT closed** | an out-of-band writer violates that precondition; with no base-CAS it can still move `main` inside the final round-trip |

⇒ *"Closes #885's stale-CI authorization defect"* is **true**.
⇒ *"Eliminates every final-round-trip race"* is **FALSE — do not write it.**

★ **Why the lock is enforced rather than documented.** The predecessor guard was
correct *because* the publisher happened to be serialized, and **nothing recorded
that dependency** — the F13 finding. This frame then reproduced the identical
defect one layer up by asserting the identity outright. **A load-bearing
precondition that lives only in prose is not a precondition; it is a hope.**
That is why step 0 exists.

★ **Steward note:** steps 1–3 already existed in `scripts/scripted-pr-automerge.sh`
as the `--doc-only` currency guard (landed `a9554a07`, F10/F11/F12 folded in).
**The mechanism was built and proved; this WP generalizes its scope to every
merge.** Steps **0, 4 and 5** are new. Read that block before writing a new one —
it carries the F10/F11/F12 reasoning in-file, including why the checker must come
from `origin/main` and why `git merge --squash` must be followed by a commit.

> ### ⚠ THE SCOPE GAP COST A PUBLISH THE SAME DAY — PR #903, measured
>
> `wp/CI-SKIPPED-NATIVE-TESTS @ aa0330e1` (the operator's declared top priority)
> failed CI on `library_documentation_gates`, because it edits
> `.github/workflows/ci.yml` — **a manifest-cited source** (`manifest.toml:316`)
> — and no valid `REVISION` can ride it. Reproduced locally: `origin/main`'s
> checker on the merge result returns *"cited source(s) changed between REVISION
> and HEAD"*, while the same checker on `main` alone passes.
>
> ⛔ **The existing guard would have caught it before the PR was created. It did
> not run, because it was gated on `--doc-only` and that was a full-CI publish.**
> The right mechanism existed, proved, and scoped to the one path that did not
> need it — which is the whole argument for this WP, arriving as evidence rather
> than as reasoning.
>
> ★ **And `ci.yml` became cited only at `1e148908` (2026-07-22 17:07, DOC-W1-5).**
> Every prior `ci.yml` commit landed the day before, *before the citation
> existed*. So this is **#885's time-of-check defect again** — a citation added
> after other work formed its assumptions — now on its **third** WP.
> ⇒ **Row 5 is not hypothetical. It has fired in production, twice.**

## Acceptance — the Architect's proof matrix, verbatim

Each row is a **required proof, executed and shown**, not an assertion:

1. cited body drift, ledger unchanged → **red**
2. exact candidate-time ledger update → **green** on candidate **and** on the
   synthetic squash result
3. citation add/remove → **set mismatch** until the ledger follows
4. extra / duplicate / wrong-path / wrong-OID / symlink row → **red**
5. old-green CI followed by a citation-bearing `main` advance → **fresh
   publisher check red**
6. **depth-1 and full-history runs agree**
7. `ORACLE-VIS-PACKAGING` + the Librarian's exact whole-source attestation →
   **green without weakening either predicate**

⚠ Row 5 is the one that closes #885 and it is the one a happy-path suite will
omit — it requires *manufacturing* a stale-CI-then-advance sequence. Per the
build-QA playbook (`:299`), enumerate the probes by the state each builds and
name which probe builds row 5. **If it has no probe, the WP is not done.**

⚠ Row 6 is a **freshness/environment** axis, independent of correctness: the
depth-1 case must be executed, not reasoned about. The prior gate carried a
comment saying nobody had proved it **accepts** a real revision in the
environment where it runs — do not repeat that.

### Acceptance additions for Part 2 (@architect, `dec_50fdjy68gm01j`)

Four more required proofs, each executed and shown:

8. **Two concurrent publisher invocations cannot both enter the merge critical
   section.** ⚠ Prove it **across two different worktrees**, not two shells in
   one — a per-worktree lock path would pass the same-worktree test and still
   never contend in production. *(Done: `A: ACQUIRED` / `B: REFUSED`, and
   `C-from-librarian-worktree: ACQUIRED` / `D-from-steward-worktree: REFUSED`,
   against the lock in the shared `--git-common-dir`.)*
9. **A base advance before the final read forces reconstruction or abort** —
   manufacture the advance; do not reason about it.
   *(Done — probe 9a: an advance during one evaluation is detected, the gate
   reconstructs, and the base it finally pins is the **advanced** base, not the
   stale one. Probe 9b: a base advancing on **every** evaluation aborts after 3
   with the bounded-retry diagnosis rather than looping. Both advances are
   manufactured from **inside** `build_and_check_merge_result`, by the checker it
   invokes, so the gate itself is never stubbed and the race is real.)*
10. **On the normal path, the landed tree OID equals the synthetic checked tree
    OID.** *(Done — probe 10, confirmed twice: once by the gate's own report and
    once independently against the sandbox origin.)*
11. **A planted landed-tree mismatch, and a planted red checker on the landed
    tree, each produce an alarm and NO automatic revert** — and leave publication
    frozen. Both arms, planted, not argued.
    *(Done — probes 11a and 11b. Each asserts four things separately: the alarm
    fires, success is **not** reported, the freeze marker is written, and
    `origin/main` is **byte-identical before and after** the alarm. 11a also
    proves the freeze **bites** — the next publish refuses to start.)*

### Where the probes live, and why they are not vacuous

**`scripts/publisher-gate-probes.sh`** — **39 assertions, all green** (the count
read off the executable, not estimated), including row 8, which is now a
re-runnable probe rather than a transcript.

### ⛔ @librarian QA found three FAIL-CLOSED defects in Part 2 — all folded

Not prose nits: three states where the gate returned a green, confident, wrong
answer. Each now has a durable probe, and each probe was verified to **FAIL
against the pre-fix publisher**.

1. **A freeze created during the CI wait was ignored.** `refuse_if_frozen` ran
   once at startup — before the lock and before a minutes-long wait. Another
   publisher's alarm inside that window left this invocation free to acquire the
   released lock and merge into a state someone else had declared unsafe. Now
   re-read **inside the lock**, on **both** paths, immediately before evaluating.
2. **Post-merge worktree creation failed open and manufactured green.** The
   verification was one condition: `if worktree_add && ! checker; then alarm`. A
   failed `worktree add` makes it false, skipping the alarm and falling through
   to the success message — **claiming a checker was green that never ran**,
   after a merge. Now separated: inability to construct the worktree freezes and
   dies as `LANDED STATE UNVERIFIED`.
   ★ Same fail-open default the runtime ring hit today in the visibility walk:
   a step that cannot reach an answer returning the permissive one.
3. **Failure to persist the freeze was swallowed** by a trailing `|| true` —
   the function returned 0, no marker existed, and the next publish proceeded
   while every message said publication was frozen. Now checked and loud.

⚠ **Probe 12a was vacuous on its first draft and is worth recording as such.**
It drove `refuse_if_frozen` twice from its *own* snippet and asserted the merge
boundary was not reached — and **passed against the unfixed publisher**, because
it tested the sequence the probe wrote rather than the sequence the script runs.
**A probe that supplies the behaviour it checks for is vacuous no matter how
green it is.** Rewritten as a structural assertion over the publisher's
top-level flow, labelled structural, and verified to fail pre-fix.

⛔ **It sources the gate's REAL function definitions out of
`scripted-pr-automerge.sh`; it does not carry a copy**, and it asserts the
extraction succeeded before running anything. A harness that silently sources
nothing passes every negative check — and a *copied* gate drifts from the
shipped gate in silence, which is exactly how `scripts/pane-busy.sh` came to
have an arm-check suite that asserted its own defect as the specification.

★ **Mutation-proved — the suite is shown to FAIL, not merely to pass.** Four
mutants, each against a copy, each caught by the probe that targets it:

| mutant | caught by |
|---|---|
| `fresh_result_gate` never re-reads the base | 9a + 9b (incl. their positive controls) |
| landed-tree comparison disabled | 11a |
| `freeze_publication` made a no-op | 11a + 11b |
| the gate **auto-reverts** `main` after the alarm | 11a's no-revert assertion |

⚠ The last mutant exists because **"no revert" is a negative assertion and
passes for any reason** — including a gate that never reverts anything because
it never got that far. Without a mutant that actually reverts, that row proves
nothing.

### ⛔ The harness itself was defeated — @adversary, and it was the plumbing again

The first version of this harness **contained the failure its own header warns
about.** The header says *"a harness that silently sources nothing passes every
negative check"*; forty lines below, it did exactly that. Three findings,
measured, all fixed:

1. **A truncated slice passed the integrity check AND the runner.**
   `grep -q "^$fn() {"` asserts each function's **opening line** and never its
   body. Truncation removes *tails*, so the check was structurally blind to the
   one drift it existed to catch. The runner used `set -uo pipefail` with **no
   `-e`** and no guard on `source`, so the syntax error stopped nothing — **exit
   0, every negative assertion passing vacuously.**
2. **Containment cannot notice its own list going stale, and it already had.**
   The slice defined **10** gate functions; the check asserted **9**.
   `cleanup_gate` was unasserted, green, live on the branch.
3. `source` was never checked for success.

**Fixes — none of them an enumeration:** `bash -n` on the slice (catches *any*
truncation, wherever it lands, and needs no list); **derive** the function set
from the publisher's gate region and assert **equality**, not containment; guard
the `source` with an explicit `exit 3`.

★ **The shape:** the old check asked *"are the things I know about present?"*
Each replacement asks *"is this artifact well-formed and complete on its own
terms?"* — the same move as a row-count post-condition and a duration-agnostic
regex. **The question that does not require a correct list is the one that
survives.**

⚠ **Two further gaps surfaced only because the derived check exists**, which is
the argument for it:

- **`acquire_merge_lock` had no probe at all.** Row 8 had been discharged by a
  hand-run transcript — not re-runnable, and it does not fail when someone
  changes the lock path, *which is the change it exists to catch*. Now **probe
  8**, in-harness: uncontended control → A acquires → **B refuses across a
  second worktree** → refusal carries its diagnosis → lock releases.
- **The first coverage check was a name-grep, i.e. a proxy**, and wrong in both
  directions: `freeze_publication` is genuinely exercised (probe 11 asserts the
  marker it writes) but never *named*, so the proxy called it uncovered — while
  a bare mention in a comment would have satisfied it. Replaced with a
  **declared coverage map asserted for set equality** against the derived set:
  every gate function is either driven by a named probe or excluded **with a
  stated reason**, and a new one forces acknowledgment.

**Plumbing mutants, both caught, harness exits 1:** anchor drift yielding a
truncated slice (caught **three** ways — parse, set equality, coverage) · a new
gate function added and unaccounted for.

### ⛔ Two gate defects the probes found — both latent, both live on a CI runner

Building the probes changed the gate, which is the point of building them:

1. **`merge` and `commit` shared one `&&` chain under a single diagnosis naming
   only the merge.** A commit failing for its *own* reasons reported *"the
   candidate needs rebasing onto current origin/main"* — **false**, and it sends
   the ring to rebase a branch that is fine. Now separated, with the second arm
   saying explicitly that this is a publisher environment fault and **not** a
   candidate defect.
2. **The scratch worktree inherited ambient git identity.** This repository sets
   `user.email` **per-repo, not globally**. `git merge --squash` needs a
   committer identity **only when the merge is not a fast-forward** — i.e.
   exactly when `origin/main` has advanced past the candidate's base, *the case
   this gate exists for*. So the failure is invisible on the happy path and
   appears on the adverse one.

★ **Worth recording how #2 was fixed, because the first fix was wrong in a
familiar way:** I covered `git commit`, the one call I had watched fail. That
passed the fast-forward probe and failed the advanced-base probe. **Enumerating
the calls observed to fail is the same move that has failed all day**; identity
is now set **once for the whole scratch worktree**, so the class cannot recur as
operations are added.

⚠ **8–11 are the probes nobody writes**, for the same reason row 5 is: each
requires *manufacturing* a condition orthogonal to the change. Per the build-QA
playbook (`:299`), enumerate the probes by the state each builds and name which
probe builds each of these. **If any has no probe, Part 2 is not done.**

## Scope boundary

**In:** `library/SOURCE-ATTESTATIONS` (or equivalent), `scripts/gen-doc-status.sh`,
`scripts/scripted-pr-automerge.sh`, `library/STATUS.md` rendering, and the gates
in `crates/ken-cli/tests/library_documentation_gates.rs`.

**Out:** anchor-existence checking (stays a separate gate, unchanged); any
Markdown span extraction; any relaxation of a predicate to make a candidate pass.

## ⛔ The non-automation boundary (@librarian, authoritative — they hold it)

The Steward's first cut said "no generator." **That was drawn in the wrong
place.** The Librarian's correction, which is the binding version:

> **The ledger MAY and SHOULD have a deterministic generator** — hand-transcribing
> dozens of blob OIDs adds error without adding judgment. **What must remain
> human is the AUTHORIZATION to update and commit its output.**

1. The **gate/check path is read-only** and fails on mismatch.
2. The generator is invoked **only after the Librarian has closed the
   reverse-citation claim ledger**.
3. ⛔ **No CI, publisher, status generator, or ordinary build step may auto-run
   it in write mode.**
4. The handoff **records which changed sources/pages were revalidated.**

⇒ *"Generated on demand"* is acceptable **only** when "demand" means an explicit
Librarian action **after semantic review**. ⛔ **"Regenerate whenever `HEAD`
differs" is the vacuous auto-bump and must remain impossible.**

★ **Therefore the generator and the check are SEPARATE entry points / flags, and
acceptance evidence must show the CHECK path CANNOT MUTATE the ledger.** (Add as
an eighth required proof.) This is the distinction that keeps the attestation an
assertion rather than a restatement of `HEAD`.

## Ring — **doc**, one WP, not split

Assigned to the **doc ring** (@librarian's read, adopted):

- **`doc-author`** built the existing `gen-doc-status.sh` + Rust
  documentation-gate substrate through `DOC-W0` and `DOC-CURRENCY-ANCHOR` —
  strongest file/mechanism familiarity; this is continuation, not a new lane.
- **@librarian** is the **only seat authorized to accept the ledger semantics**,
  and QAs the proof matrix.
- **`doc-leader`** owns sequencing and the merge Decision.
- The publisher-script generalization is **bounded by this frame and reuses an
  existing block** — it does not justify moving the currency substrate to a
  build ring.

⛔ **NOT a §14a library-only merge.** The diff reaches `scripts/` and `crates/`,
so **full CI and Architect terminal review remain required.**

⛔ **Do NOT split Part 1 from Part 2.** They close different halves of **one**
guarantee, and landing either alone leaves a known hole: *representable-but-stale
authorization*, or *fresh authorization over an unrepresentable attestation*.

### Branch and assignment (@architect, `evt_677j2nrpkv3c9`)

**One `wp/SRC-ATTEST` integration branch, one merge Decision.**

| part | owner | scope |
|---|---|---|
| **Part 1** | **doc ring** (owns the branch) | `library/SOURCE-ATTESTATIONS`, `library/STATUS.md`, `scripts/gen-doc-status.sh`, `crates/ken-cli/tests/library_documentation_gates.rs` — one documentation-currency component the ring already built and QA'd through DOC-W0 / DOC-CURRENCY-ANCHOR |
| **Part 2** | ⭐ **Steward spillover, same branch** | `scripts/scripted-pr-automerge.sh` — publisher/process machinery |

★ **Why Part 2 is not the ring's:** `scripted-pr-automerge.sh` is publisher
semantics, and *"a build ring should generalize that block only from a
Steward-supplied exact commit/contract, not independently reinterpret publisher
semantics."* The Steward authored and proved the existing guard; the
generalization is the Steward's to write.

⛔ **Runtime is the BLOCKED CONSUMER, not the owner.** Assigning this to Runtime
would couple a generic documentation substrate to the first WP that exposed it,
and add a second external semantic handoff to the Librarian.

**Gate order:** Librarian QA binds the **assembled exact SHA** → Architect
reviews the **combined contract/process diff**. **No Spec vote** unless scope
unexpectedly reaches `spec/` or `conformance/`. **Do not merge either part
alone.**

### Two integration checks, in addition to the proof matrix

1. ⛔ **Part 1 without Part 2 is INCOMPLETE even if its tests are green** —
   stale merge-result authorization remains open.
2. ⛔ **Part 2 must call the SAME SHARED GUARD for doc-only and normal
   publishes.** Generalize the existing block; **do not leave two
   implementations that can drift.**

### ⚠ Architect's precision on the automation boundary

A helper **may mechanically render a *proposed* canonical ledger** to avoid
hand-copying OIDs — but **neither the normal generator nor `--check` may
silently install it.** The committed ledger change remains **the Librarian's
reviewed assertion**. ⛔ *"Recompute and accept `HEAD` during the gate"* is
**forbidden.**
