---
scope: roles/steward
audience: the Steward — anyone running `scripts/scripted-pr-automerge.sh`
source: I-8 publish, 2026-07-14 — a false "merged" that survived an exit-0
---

# The publisher's exit code is the LAUNCHER's, not the script's — and its flag is `--description-file`

Two traps in one publish, and they **compose into a convincing false "merged."**

## 1. The flag is `--description-file`. `--body-file` dies AFTER usage-parse.

```sh
scripts/scripted-pr-automerge.sh --target <branch> --title <t> \
  --description-file <path>          # ✅  (or --description <text>)
  --body-file <path>                 # ❌  → "error: unknown argument" → set -e → exit
```

`gh pr merge` itself takes `--body-file`, which is exactly why the wrong flag
feels right. The script wraps it under a different name.

## 2. ★ Launched as `nohup … &`, the exit code you are handed is the SHELL's.

```sh
nohup scripts/scripted-pr-automerge.sh … > log 2>&1 &   # ← the & returns IMMEDIATELY
```

**A `completed (exit code 0)` notification arrives within seconds and reads
exactly like a successful publish.** It is the launching shell reporting that it
*successfully launched*. It says **nothing** about the publisher, which is either
(a) already dead from the bad flag, or (b) still alive minutes later polling CI.

**Both happened on I-8, in that order.** The first run exit-0'd on the bad flag
having done nothing but exist; the second exit-0'd **while the publisher was
still running** — `ps` showed it alive, mid `Waiting 383s before polling PR
#636 checks`.

## The discipline

**Never read a publish outcome from an exit code or a task notification.** After
the publisher, verify **on `origin/main`, by CONTENT**, using a discriminator the
WP actually **ADDS**:

```sh
git fetch origin
git rev-parse --short origin/main            # did it even move?
git show origin/main:<path/the/WP/adds>      # is the thing THERE?
```

And if you want to know whether the publisher is still working rather than dead,
**`ps` for it** and **`tail` its log** — the log's last line (`PR #N created`,
`Waiting Ns before polling`) is the real state.

This is [[landing-integrity-verify-by-content-not-sha]] arriving through a new
door: the previous lesson was *don't trust a SHA or a task status*, and the exit
code is simply the **next** cheap proxy that lies. **The only thing that proves a
merge is the content on `origin/main`.**
