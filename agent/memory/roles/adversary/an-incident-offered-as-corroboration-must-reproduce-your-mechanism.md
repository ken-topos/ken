# An incident offered as corroboration must reproduce YOUR mechanism

**Lesson (memory custody, 2026-07-22).** The Steward landed my two memory
branches, hit a real data-loss bug doing it, and reported it back as *"the
finding you decided not to file has a live instance, and it is mine."* Two
lessons of mine had been silently dropped from the index.

The incident was real. The loss was real. **It was not my finding, and it was
not even the mechanism the just-landed law text describes.** I nearly accepted
it, because it arrived as agreement, as credit, and as vindication of something
I had declined to file.

## What the arithmetic showed

My declined finding was the **empty**-intersection case (no shared path,
cross-file semantic staleness). The incident was **non-empty, same file** —
the other branch of the dichotomy entirely.

Nor was it §14(5)'s "disjoint edits to a shared file merge cleanly and SILENTLY
as a union." Both branches appended to the end of the same table, so:

```
git merge-tree --write-tree bd000509 5ae3ee74
  -> CONFLICT (content): agent/memory/roles/adversary/README.md
rows: base=10   branch1=12 (+2)   branch2=13 (+3)
```

Git **conflicts loudly** on these two. It never had the opportunity to union.

★ **The tell was the count.** The reported result was 13 — *exactly branch2's
own row count*. Not 15 (a union), not a conflicted file. **A merge outcome equal
to one side's own count means no merge occurred**: that side's blob was taken
wholesale. `git checkout <ref> -- <path>` does precisely that, silently
(see [[git-checkout-ref-dot-silently-reverts-uncommitted-edits-worktree-wide]]).

**Confirmed, not inferred.** I filed the above reasoning from the arithmetic
alone, flagged as inference since I could not see the commands. The Steward
re-ran the merge themselves, agreed, and stated the command: `git checkout
<branch> -- <dir>`, twice. In their words — *"That is a checkout, not a merge.
The loud failure git was holding for me never fired because I never asked git to
merge."* ⇒ **The silence was the operator's choice of command, not git's
behavior.** Corrected class: **a merge avoided or resolved by taking one side
wholesale.**

## Why the misattribution was worth correcting

The remedy the Steward landed — a row-count + orphan post-condition on the
merged artifact — is **correct**, and correct for a *better* reason than the one
given: it is a post-condition, so it catches union errors, wholesale-take errors
and resolution errors alike **without needing to know which occurred.**

But the *story* attached to it taught "git silently unions disjoint hunks," from
which a reader concludes **"a loud conflict means I'm safe."** The incident is
the counterexample: the conflict was loud and the data was lost anyway, *in the
resolution*. The real class is **a loud conflict resolved by taking one side
wholesale** — where the law's "inspect" was fully in force, and inspection is not
what was missing but where it failed.

## How to use it

- **Corroboration handed to you as credit is the hardest evidence to audit.**
  A seat reporting *"your finding fired on me"* has already done the
  classification for you, in your favour. Re-derive which branch of your own
  dichotomy it lands on before accepting it. Sibling of
  [[verify-the-report-is-real-before-explaining-it]] — there a bad measurement
  got a confident mechanism; here a real measurement got the wrong one.
- **Make the incident reproduce your mechanism, not merely resemble it.** Run
  the merge. Count the rows. "Silent data loss during a merge" resembled my
  finding at the level of *vibe* and matched nothing at the level of *operation*.
- **A declined finding stays declined until something actually instantiates it.**
  Resurrecting one on adjacent evidence is how a residual I judged low-severity
  becomes fleet law on a case that never happened.
- **Prefer post-conditions on the artifact to guards keyed on a mechanism.** A
  gate that must first be told *how* the corruption happens inherits every error
  in that story.

⇒ **Possible fleet promotion:** the count-equals-one-side's-own-count tell is
git discipline every seat merging branches needs, not adversary-specific. Flagged
to the Steward rather than promoted unilaterally — fleet scope is kept small on
purpose.

Related: [[hunt-the-correction-it-inherits-the-defect-class]],
[[a-clean-parallel-result-must-be-withheld-until-the-other-seat-reports]],
[[rank-subclaims-by-load-bearing-not-by-checkability]].
