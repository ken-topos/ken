---
scope: enclave
audience: (see scope README)
source: private memory `verify-symbol-exposure-not-just-call-site-safety`
---

# Verify symbol exposure structurally, not by checking call-site safety

Checking "every call site of X that I found is safe" answers a different
question than "is X itself reachable from outside those call sites." Doing only
the first and treating it as if it answered the second is a real gap.

**Why:** a doc comment claiming a primitive is "internal-only" / "never exposed
un-guarded" is aspirational unless the registration path actually gates it from
resolution. If the primitive is registered into the normal global symbol table
(e.g. via the same `declare_primitive`/`reg_unop`-style helper as any public
op), a surface program can call it directly regardless of how carefully-guarded
its *intended* call sites are.

**How to apply:** when a comment or handoff claims a symbol is internal/private,
verify it structurally — is it gated from name resolution (a separate non-global
table, a naming convention the elaborator enforces, a visibility check), or does
it just happen to only be called from safe places in the code you've read so
far? If it's registered the same way as any public primitive, it IS
surface-callable, whatever the comment says.

Live miss: WP conversions-intn-floor's `int_to_{width}_raw` unchecked narrowing
cast — I confirmed both its intended call sites (`intToIntN`'s guarded branch,
`saturating*`'s post-clamp branches) were individually range-safe and concluded
the cast was internal-only. CV caught what I missed: `reg_unop` inserts it into
`globals` with no gate, so `int_to_int8_raw 200` is directly callable from
surface Ken. Non-blocking here only because `IntN` is a bare primitive with
nothing to fabricate a false proof against (a wrong value in the
tested-not-trusted ring, not a hole) — but on a refinement-typed carrier this
exact miss would matter.

Sibling of conformance hand feeds the deliverable (verify the real producer, not
the test's framing) and kernel backed claim grep the emission not the name (grep
the mechanism, not the name) — this is the same discipline applied to an
EXPOSURE claim rather than a trust-level claim.
