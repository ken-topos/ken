---
scope: build
audience: (see scope README)
source: catalog-taxonomy-build retros (implementer evt_53erjje0r8vct, QA
  evt_4m6xd05dk33r2), 2026-07-10
---

# A rename/move WP needs a whole-workspace old-basename sweep

On `wp/catalog-taxonomy-build` (2026-07-10, the PascalCase catalog move, PR
#403), `verify/proof_erasure_boundary_checker.ken` was renamed to
`Capability/Verify/ProofErasureBoundaryChecker.ken`. The implementer swept every
`include_str!` path and updated it; QA re-checked all 12 `include_str!` sites and
walked the catalog tree. Both were thorough *within their scope* — and both
missed `crates/ken-interp/tests/nc9_proof_erasure_boundary_checker.rs`'s
`assert!(report.evidence_source.contains("proof_erasure_boundary_checker.ken"))`.
It failed only in full-workspace CI.

Two blind spots stacked: (1) the sweep pattern was "grep the old PATH literal,"
which does not catch a bare-BASENAME substring check; (2) both the implementer's
and QA's scoped test runs were bounded to the crates the diff touched
(`ken-elaborator`/`ken-cli`), and the assertion lives in an untouched crate
(`ken-interp`). A `.contains("<filename>")` runtime assertion is invisible to an
`include_str!` grep (it is not a build-time include) AND to a scoped compile-test
run in a different crate (it still compiles and can pass on padding) — so on the
box, which never runs a full-workspace build, CI is the *only* place it runs.

**How to apply:** for any rename/move WP — a file, but equally a function name, a
wire-format tag, a config key, an error-lane label — sweep the **bare old
spelling** across the **whole workspace**, not just the crates you edited:

```
git grep -n '<old-basename>'   # e.g. proof_erasure_boundary_checker,
                               # not only the full old path
```

Then treat every crate whose tests reference the moved artifact *by name* as
in-scope for a test run, not just the crates whose source you changed. The blast
radius of a rename is defined by string references to the old name, not by which
crates the diff touches. Independently reached by both the implementer and QA on
the same WP. Companion to
[[assert-specific-error-variant-not-is-err]] (a `.contains` string assertion is
load-bearing, so it rots silently on a rename) and to the build law that the
laptop never runs a full-workspace build — CI is the backstop for exactly this
class, and it did its job here.
