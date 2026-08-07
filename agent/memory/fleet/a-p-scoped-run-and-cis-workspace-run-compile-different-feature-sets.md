---
name: a-p-scoped-run-and-cis-workspace-run-compile-different-feature-sets
description: "A `-p <pkg>` run activates that package's DEFAULT features; CI's `--workspace` run activates the UNION every member demands, dev-dependencies included. So the mandatory targeted local run can compile a different binary than CI from the same source — and a two-ended differential built on it does not measure what CI measured."
scope: fleet
---

# A `-p` run and CI's `--workspace` run compile different binaries

**Same source, same test name, different `cfg`.** A local run scoped with
`-p <pkg>` builds that package under **its own default features**. CI's
`cargo nextest run --workspace` builds it under the **union of features every
workspace member demands — including through `[dev-dependencies]`**. Under
`resolver = "2"` a single workspace test invocation compiles each package
**once**, so the union wins and the package's own test binary links it.

**The two runs are therefore not the same experiment**, and nothing announces
the difference. Both are green-looking, both name the same test, and the local
one is the one the operator's resource rule **requires** you to use.

## MEASURED / CLAIMED / THE GAP

Measured 2026-08-07 at `fb99d0fc`, `RT-SRCBODY-BIND-ORDER`.

CI failed two `ken-runtime` lib unit tests. The ring measured them locally with
`scripts/ken-cargo test -p ken-runtime --lib --no-fail-fast` and reported them
**passing at the same candidate**.

- **MEASURED:** the rows pass under `-p ken-runtime --lib`.
- **CLAIMED (about to be):** the rows are not a candidate regression.
- **THE GAP:** `ken-runtime` has one non-default feature,
  `px8-ds-test-support`, and **`ken-cli`'s `[dev-dependencies]` enables it**.
  Under CI's `--workspace` build the feature is **ON**; under `-p ken-runtime`
  it is **OFF**. The two measurements were of two different binaries, so the
  disagreement was never a contradiction to resolve — it was a configuration
  difference nobody had named.

## The near-miss this produced, which is the reason to file it

The disagreement looks exactly like the **bare-name-shared-across-binaries**
trap this fleet has hit before (`px7o` vs `px7n`), and it was diagnosed as
that. It was not: each symbol had **exactly one** definition, `mod` was mounted
once, there was **no `#[path]`**, and the crate declared **no `[[test]]`
target**. The instruction to "disambiguate by binary" could not have terminated
in a finding.

**A familiar-shaped explanation arrived before the cheap structural check did.**
One `git grep -c` on the symbol would have killed it in seconds.

## How to apply

- **Before trusting a local differential against a CI result, ask what features
  CI activated for that package.** Cheap check: grep the workspace `Cargo.toml`s
  for `features = [` on path dependencies. Any package a sibling enables a
  non-default feature on is measured differently by a `-p` run.
- **Carry the matching `--features` on BOTH ends** of a two-ended base-vs-
  candidate run, or state plainly that the run does not match CI's
  configuration.
- **Do not settle it by reading the `cfg` sites.** Note which gates actually
  discriminate: `#[cfg(any(test, feature = "x"))]` is **true in any lib-test
  build either way** and is not a discriminator. Only `#[cfg(feature = "x")]`
  alone is. Reading the wrong gates yields a confident wrong answer; the probe
  is one command.
- **A dead hypothesis measured beats a live one assumed.** If adding the feature
  does not flip the rows, report that — it retires the axis and points at the
  next one (harness, profile, `--locked`).

## Why this is structural, not carelessness

**The rule that keeps the box alive is the rule that creates the gap.**
`COORDINATION §12` bans local `--workspace` builds because they OOM the machine
and stall the fleet, so every local agent is *required* to measure with `-p`.
That is correct and is not the thing to change. The consequence is simply that
**a targeted local run is not a reproduction of CI** — it is a different
configuration that usually agrees, and the cases where it disagrees are exactly
the ones being investigated.

So the local run remains the right instrument; what is owed is naming the
configuration alongside the number. Related:
[[a-scope-exclusion-bounds-edits-not-verification]] (a carried premise needs a
MEASURED-HERE / INHERITED label),
[[a-failure-list-keyed-on-bare-test-names-is-ambiguous-across-binaries]] (the
neighbouring trap this one impersonates), and
[[verify-the-report-is-real-before-explaining-it]] (a confident mechanism for a
measurement artifact launders it into a finding).
