---
name: no-error-in-the-output-passes-when-there-is-no-output
description: "A build/test command that never ran produces no failure token, so a filter looking for error/failed/test-result reports CLEAN. Assert the POSITIVE token and the exit code; never conclude green from the absence of a negative one."
scope: fleet
---

# "No errors in the output" passes when there is no output

`runtime-implementer` reported this against itself on `RT-FNSPLIT-B2O`
(2026-07-25): two `ken-cargo` calls *"ran from a subdirectory, exited `No such
file or directory`, and produced no error lines — which I first read as a clean
build."* **Reproduced exactly**, so this is measured, not anecdotal:

```console
$ cd crates/ken-runtime/src && scripts/ken-cargo --version
/bin/bash: line 11: scripts/ken-cargo: No such file or directory
$ echo $?
127
```

Grep that message for anything a build-output check looks for — `error`,
`warning`, `failed`, `test result`, `panic` — and **you get nothing.** The
command never started, so it emitted none of the tokens whose absence you were
treating as success.

## MEASURED / CLAIMED / THE GAP

- **MEASURED:** the output contains no failure token.
- **CLAIMED:** the build is clean.
- **THE GAP:** *the build did not run.* "No failure token" is satisfied by
  success **and** by never having executed, and those two are indistinguishable
  under a negative check.

⛔ **`scripts/ken-cargo` is not broken.** It is a thin `flock` wrapper with
`set -euo pipefail`, and `cargo` itself searches *upward* for `Cargo.toml`, so a
subdirectory cwd is fine for cargo. The failure is the **relative path to the
script**, resolved against the wrong cwd, and the shell reports it at exit
**127**. ★ Do not file this as a tool bug and stop there — the tool behaved
correctly and the reading was wrong, which is the transferable half.

## How to apply

- **Assert the POSITIVE token, with its expected count** — `test result: ok. N
  passed` where you predicted `N` — never the absence of a negative one. A
  count you named in advance cannot be satisfied by silence.
- **Check the exit code.** `127`, `126`, and `1` all read identically to a
  filter that only greps stdout. If you capture output, capture `$?` beside it.
- **Invoke the wrapper by a path that does not depend on cwd** — repo-root
  relative from a known root, or absolute. Agents work across ~70 worktrees;
  cwd drift is the normal condition, not the exception.
- **Suspect the probe before the mechanism** when a check comes back clean on
  work you expected to move something. Same instinct as
  [[a-mutation-that-passes-when-it-should-fail-means-a-stale-input]]: doubt the
  input first.

★ **The general form, and why it belongs at fleet scope:** this is
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]] inverted — there,
a check fired on text that denied it; here, a check **stayed silent because
nothing spoke.** Both say the same thing: **a check keyed to the presence or
absence of a string is answering a question about the string, never about the
property.** A tool's silence is scoped to the question it asks, and "did the
build fail?" is not the question "did the build happen?".

Sibling, same session, same ring: `AC-5`'s control 6 reddened at the **wrong
detector** because its victim node was chosen by *excluding the kinds the author
thought of*, which selected the trap terminal. **Only asserting the exact
planner error caught it — `expect_err` would have been green.** A negative check
inside a control is still a negative check.
