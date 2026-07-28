---
scope: steward
audience: (see scope README) — whoever runs the publisher path
source: I-6 publish, 2026-07-14 (bad flag) and I-8 publish, 2026-07-14 (nohup
  exit-code masking) — two publisher incidents the same day, composing into a
  convincing false "merged."
---

# Publisher flags: `--description`, NOT `--body` — and failure is silent TWICE

`scripts/scripted-pr-automerge.sh` takes:

```
--target <sha-or-branch>            # the HEAD branch, NEVER main
--title <pr-title>
(--description <text> | --description-file <path>)
[--doc-only]
```

**There is no `--body`/`--body-file` on the wrapper.** `gh pr merge` itself
takes `--body-file`, which is exactly why the wrong flag feels right — the
script wraps it under a different name. Passing `--body`/`--body-file` dies
with `error: unknown argument: --body` **after usage-parse — and the wrapping
task still reports exit code 0.** So the task-completion notification says
"completed," the PR was never created, and **nothing landed.**

## ★ A second, independent way exit-0 lies: `nohup … &`

```sh
nohup scripts/scripted-pr-automerge.sh … > log 2>&1 &   # ← the & returns IMMEDIATELY
```

**A `completed (exit code 0)` notification arrives within seconds and reads
exactly like a successful publish.** It is the launching shell reporting that
it *successfully launched* — nothing about the publisher, which is either (a)
already dead from the bad flag, or (b) still alive minutes later polling CI.

**Both happened on I-8, in that order.** The first run exit-0'd on the bad
flag having done nothing but exist; the second exit-0'd **while the publisher
was still running** — `ps` showed it alive, mid `Waiting 383s before polling
PR #636 checks`.

## The discipline — verify on `origin/main`, by CONTENT

**Never read a publish outcome from an exit code, a task notification, or a
flag that parsed.** After the publisher:

```sh
git fetch origin -q
git rev-parse --short origin/main                      # did it even move?
git show origin/main:<a-file-the-WP-ADDS> | head -1     # absent ⇒ NOT landed
```

**Verify on `origin/main` BY CONTENT — never by SHA, task status, or exit
code.** A dead publisher task ≠ a dead PR, and a *successful-looking*
publisher task ≠ a merge. Both directions lie. And if you want to know
whether the publisher is still working rather than dead, **`ps` for it** and
**`tail` its log** — the log's last line (`PR #N created`, `Waiting Ns before
polling`) is the real state.

## ★ Pick a discriminator the WP actually ADDS, not touches

When you content-verify, grep for something the WP **introduces**, not
something it **touches**. I nearly mis-verified I-6 with
`git show origin/main:crates/ken-interp/src/eval.rs | grep -c 'fn mint_fs_cap'`,
which returned **2** — but those were the two **pre-existing inherent**
methods, and the WP's actual change was *adding a third declaration on the
trait*. A count of 2 reads as "it's there" and would have passed a merge that
never happened.

The good discriminators for I-6 were the ones that did not exist before at
all: `[lib]` in `crates/ken-cli/Cargo.toml`, and the existence of
`crates/ken-cli/src/lib.rs`. Prefer a **new file** or a **new declaration**;
avoid a name that already occurs in the file for other reasons.

This is the same landing-integrity discipline as
[[committed-is-not-reachable-publish-then-verify-on-main]] arriving through
two doors at once: don't trust a SHA or a task status, don't trust an exit
code, and don't trust a grep hit that predates your change. **The only
thing that proves a merge is the content on `origin/main`.**

Sibling of [[scripted-publisher-target-is-head-branch-never-main]] and
[[kernel-backed-claim-grep-the-emission-not-the-name]] — same family: **grep
for the thing that changed, not the thing that shares its name.**
