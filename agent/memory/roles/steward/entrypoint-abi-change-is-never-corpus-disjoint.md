---
name: entrypoint-abi-change-is-never-corpus-disjoint
description: "An entrypoint-ABI change (the shape of `main`, argv, exit-code) is NEVER corpus-disjoint — it necessarily rewrites every runnable example's entrypoint, which lives in the catalog/rosetta corpus. Before declaring a lane freeze-safe / catalog-disjoint, grep the actual file touch-set the change forces, not just the crate the logic lives in."
metadata:
  node_type: memory
  type: feedback
  scope: steward
---

**2026-07-13 — self-caught (Architect surfaced) while sequencing CLI I-1
against the kenfmt capstone-C catalog freeze.** I kicked I-1 (entrypoint ABI +
runner) as **"`crates/`-only — catalog-disjoint, safe alongside the catalog
freeze."** That premise was **wrong as built.** Changing the entrypoint ABI
(`proc main : IO Unit` → `proc main (_input : ProcessInput) (_caps :
ProgramCaps) : HostIO ExitCode = host_program (…)`, named-`main` convention)
**necessarily rewrites the `main` of every runnable example** — 19 files inside
C's frozen corpus (3 `catalog/guide/*.ken.md` + all 16
`examples/rosetta/**/*.ken`). Worse, those migrations were **load-bearing for
I-1's own gate**: the FS/Console e2e tests drive those examples, so `cargo test
--workspace --locked` won't build green without them — they **can't be split
out** of I-1. So the "disjoint" lane collided head-on with the freeze.

**Why I got it wrong:** I reasoned about **where the logic lives** (the runner
is `ken-cli`/`ken-interp` Rust) instead of **where the change lands** (every
entrypoint in the corpus). An ABI is a *contract every conforming program
restates* — so an ABI change is inherently corpus-wide, the same way a keyword
retirement or a glyph migration is. "The code is in `crates/`" says nothing
about the diff's file set.

**The rule:** before declaring a lane **freeze-safe / artifact-disjoint**, do
not infer disjointness from the crate the logic sits in — **enumerate the actual
file touch-set the change forces**, including every example/fixture/doc that must
restate the changed contract. For an **entrypoint-ABI / calling-convention /
prelude-signature** change specifically, assume it touches the whole example
corpus until a grep proves otherwise. This is the scheduling twin of
[[atomic-capstone-freeze-excludes-concurrent-work-on-frozen-artifact]] (the
deciding axis is *does the lane's output land in the frozen artifact*) and of
[[exported-name-migration-needs-whole-harness-consumer-inventory]] /
[[migration-sweep-glob-must-enumerate-every-ken-source-root]] (a contract change
needs a whole-consumer inventory, not a where-it's-defined guess).

**How I resolved it (Path B, the general shape):** when a design-approved lane
turns out to collide with an open freeze, keep the freeze whole and put the one
unavoidable rebase on the **less-delicate** WP — here, hold I-1 behind C and let
I-1 re-`ken fmt` its touched files onto reformatted main (mechanical, approval
carries verbatim), rather than force C's whole-corpus AST-preservation soundness
gate to regenerate over freshly-migrated content mid-flight. Honor the operator's
stated freeze constraint ("no catalog changes in flight during C") over a
tempting except-these-N-files carve-out.

**How to apply:** at kickoff, write the touch-set claim as a *grep result*, not
an assertion ("`git grep` confirms this change touches only X"). If the change
restates any contract that examples/prelude/fixtures embody, treat it as
corpus-touching and sequence it against any active freeze accordingly.
