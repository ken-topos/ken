---
scope: roles/steward
audience: (see scope README)
source: 2026-07-22, caught by the adversary, not by me — a corrected
  acceptance criterion sat on `steward/work` for five commits with zero
  publishes while I announced it as landed
---

# Committed is not reachable — publish, then verify on `main`, by content

I corrected a WP's acceptance criterion (it was **unsatisfiable**, not
merely non-discriminating), posted "it's in," and it was true **only of
`steward/work`.** Five commits, zero publishes. `COORDINATION §15` sends
build rings to **`main`**, so for the whole window a ring picking up the WP
would have read the void AC and worked against a criterion no implementation
could ever satisfy.

**Why:** I verified the *artifact* and never its *reachability*. The commit
succeeded, the file on disk was right, and every check I ran was against my
own worktree. "Committed" felt like "done."

**How to apply:**

- A Steward corpus edit is done when
  `git grep '<fragment>' origin/main -- <file>` returns the new text — **not
  at `git commit`.** Corpus branch off *current* `origin/main` → publisher
  path → verify by content.
- **Before announcing any correction as landed, grep `origin/main` for it.**
  If the claim is "the ring will now see X," the check is on the ring's
  copy.
- **The publisher can print a success message on a failed push.** Its exit
  code is worthless; only content on `main` counts. Verify by content, never
  by SHA, task status, or exit code — see
  [[publisher-flags-are-description-not-body-and-failure-is-silent]], which
  hits this same trap through two independent doors (a bad flag, and a
  `nohup … &` launch whose immediate exit-0 is the shell's, not the
  publisher's).
- **`git grep` false-negatives easily** — it is case-sensitive and a phrase
  spanning a line break will not match. Three greps false-negatived in one
  session. Grep a **short, lowercase, single-line** fragment; never a
  sentence, never across `**bold**` or `` `code` ``.

"Committed" is the proxy; "on `main`" is the mechanism. The adversary's catch
was better than the correction it delivered — it checked whether the fix
reached its consumer while I checked only whether I had written it.
