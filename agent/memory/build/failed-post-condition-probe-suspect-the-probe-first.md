---
scope: build
audience: (see scope README)
source: private memory `failed-post-condition-probe-suspect-the-probe-first`
---

# When a verification probe reports content MISSING, suspect the probe first

When a post-condition probe says a change is **missing**, the probe is the more
likely defect. Verify the probe against a case whose answer you already know
before acting on its verdict.

**Measured 2026-07-24** publishing four operator directives to `main`. A
post-condition grep reported 2 of 5 phrases MISSING. Both were false negatives:

- A phrase spanning a **line break** in wrapped markdown still failed after
  flattening with `tr '\n' ' '`, because a blockquote `>` marker landed
  mid-phrase.
- A second phrase was real but got grepped against the **wrong file** (it
  lived in a different file than expected).

The Steward playbook's cadence section already warns a phrase must not span
`**bold**` or `` `code` `` markers. **It also breaks on line wrapping and on
`>` blockquote markers** — i.e. on any markdown structure, which is most of
the corpus.

**The probe that actually settled it was a differential count, not a phrase:**

```sh
grep -ic "ledger" <candidate>   # 5
grep -ic "ledger" <origin/main> # 0   <- the discriminator
```

⇒ **Prefer a probe whose failure mode is not the formatting of the thing you are
searching.** A count-on-both-sides discriminates (present-here / absent-there);
an exact multi-word phrase grep over wrapped markdown does not.

⇒ **Never conclude "the change was lost" from a single failed phrase grep** —
that conclusion invites re-doing work that already landed, or worse,
re-applying it on top of itself.

Companion to [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]]:
here the negative check *failed* for the wrong reason, which is the same
disease. The positive control is grepping `origin/main` and requiring ABSENT —
if the phrase is absent on both sides, the probe is broken, not the content.

★ **The build-artifact form of the same disease — three instances in one
session (RT-FNSPLIT-B2A-C, 2026-07-25), each producing a confident WRONG
answer rather than an obviously broken one:**

1. **`target/debug/deps` accumulates.** It held **15** `libken_runtime-*.rlib`
   files spanning ten hours. A test's own freshness guard went red because the
   rlib it selects predated a source file whose **mtime** a restored scratch
   probe had bumped — content byte-identical, `git diff --quiet` clean. This
   nearly read as a defect in the candidate. Forcing a rebuild cleared it.
2. **`find`/`ls` order is not recency.** `find … -name '<binary>-*'` handed
   over an *older* binary that "failed 8 tests"; the binary cargo had just
   built passed 19/19. ⇒ **Run the artifact cargo just produced** — check its
   mtime against the source, or let cargo run it.
3. **`| tail -N` destroys both the evidence and the exit code.** A full-crate
   sweep piped through `tail -60` discarded every earlier suite's result *and*
   reported `tail`'s status as cargo's, so "exit 0" meant nothing. Redirect to
   a file and read the file.

⇒ **Before believing any artifact-derived verdict, ask "did I measure the
thing cargo just built?"** A stale artifact compiles and runs happily, so
every positive control passes while the answer is about hours-old code.
Fleet companion:
[[a-mutation-that-passes-when-it-should-fail-means-a-stale-input]]
(freshness is a third axis, beside "the harness works" and "the property
holds").

Related: [[markdown-80col-reflow-gotchas]] ·
[[git-diff-stat-always-exits-0-use-quiet-for-an-emptiness-test]] ·
[[a-green-test-can-be-green-by-3-percent-of-a-hidden-budget]]
