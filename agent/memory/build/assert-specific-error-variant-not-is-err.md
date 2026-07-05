---
scope: build
audience: (see scope README)
source: private memory `assert-specific-error-variant-not-is-err`
---

# Assert the specific error variant, not a bare `is_err()`

On `wp/surface-syntax-val2` (2026-07-04, VAL2 #3 mutual recursion, merged
`83f728a` PR #275), my divergent-mutual-group acceptance test asserted only
`res.is_err()`. Architect flagged (non-blocking, tracked by Steward) that this
is too weak: a group rejected for an UNRELATED reason (a stray parse error, an
unrelated type mismatch elsewhere in the test source) would also satisfy
`is_err()`, silently passing even if the actual termination gate being tested
never fired at all.

**How to apply:** whenever a test's whole POINT is "the check rejects this
specific bad input" (an SCT non-termination case, a kernel `TypeMismatch`, a
`NoInstance`, etc.), match on the concrete error variant/message the mechanism
under test actually produces, e.g.:

```
assert!(matches!(res,
    Err(ElabError::KernelRejected {
        error: KernelError::NotTerminating(_), ..
    })));
```

not a bare `is_err()`. This is the same discipline as green vs green does not
confirm a fix's sibling on the failure side — a loosely-specified assertion can
pass for the wrong reason just as easily as a loosely-specified positive test
can.
