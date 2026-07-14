---
scope: steward
audience: (see scope README) — whoever runs the publisher path
source: I-6 publish, 2026-07-14
---

# Publisher flags: `--description`, NOT `--body` — and a bad flag EXITS 0

`scripts/scripted-pr-automerge.sh` takes:

```
--target <sha-or-branch>            # the HEAD branch, NEVER main
--title <pr-title>
(--description <text> | --description-file <path>)
[--doc-only]
```

**There is no `--body`.** Passing it dies with `error: unknown argument: --body`
— **and the wrapping task still reports exit code 0.** So the task-completion
notification says "completed," the PR was never created, and **nothing landed.**

**This is why the content-verify step is not optional.** The only thing that
caught it was checking `origin/main` for the artifact itself:

```sh
git fetch origin -q
git show origin/main:<a-file-the-WP-ADDS> | head -1   # absent ⇒ NOT landed
```

**Verify on `origin/main` BY CONTENT — never by SHA, task status, or exit code.**
A dead publisher task ≠ a dead PR, and a *successful-looking* publisher task ≠ a
merge. Both directions lie.

## ★ Pick a discriminator the WP actually ADDS

When you content-verify, grep for something the WP **introduces**, not something
it **touches**. I nearly mis-verified I-6 with
`git show origin/main:crates/ken-interp/src/eval.rs | grep -c 'fn mint_fs_cap'`,
which returned **2** — but those were the two **pre-existing inherent** methods,
and the WP's actual change was *adding a third declaration on the trait*. A
count of 2 reads as "it's there" and would have passed a merge that never
happened.

**The good discriminators for I-6 were the ones that did not exist before at
all:** `[lib]` in `crates/ken-cli/Cargo.toml`, and the existence of
`crates/ken-cli/src/lib.rs`. Prefer a **new file** or a **new declaration**;
avoid a name that already occurs in the file for other reasons.

Sibling of [[scripted-publisher-target-is-head-branch-never-main]] and
[[kernel-backed-claim-grep-the-emission-not-the-name]] — same family: **grep for
the thing that changed, not the thing that shares its name.**
