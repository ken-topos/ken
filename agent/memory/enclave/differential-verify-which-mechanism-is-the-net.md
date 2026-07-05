---
scope: enclave
audience: (see scope README)
source: private memory `differential-verify-which-mechanism-is-the-net`
---

# Differential-verify which mechanism is the actual soundness net

**The correction (Ken `surface-transport` Gap-A build gate, 2026-07-03,
`evt_5b3jt2fnq5ccj`).** My soundness APPROVE said "a broken `kernel_infer` net
would fail the [discriminating] negatives," attributing the backstop to
`infer_j`/`infer_eq`'s own internal final `kernel_infer` call. language-qa ran
the differential I should have: **disabled that local call (made it a no-op),
reran the suite — 6/6 still pass, both negatives still kernel-rejected.** The
true sole-necessary net is the pre-existing UNIVERSAL `declare_def` whole-body
check (`check.rs:984` `check(env,&empty,&body,&ty)`), which structurally
recurses into any embedded `Term::J` via the kernel's own infer dispatch
(`check.rs:330` `Term::J => infer_j`) — applies to EVERY declaration regardless
of what the former does locally. The per-former `kernel_infer` is
defense-in-depth (error locality at the former's span), **redundant for
soundness**. I verified the correction myself (grepped `declare_def` + the J
dispatch arm) before endorsing it — don't take even a good correction on report.

**Why the distinction matters (and why it's a STRENGTHENING, not a weakening).**
The sentence "the kernel re-checks it, so a broken net fails the negatives" is
true of the *system* but conflates two different mechanisms when both are
present. Attributing the net to the local call:
- **over-credits a removable call** — a future refactor dropping the per-former
  `kernel_infer` reads as reopening a soundness hole when it doesn't (the
  universal check still catches everything);
- **understates robustness** — the real net being a structural per-declaration
  invariant is a *stronger* guarantee than a per-former call a maintainer could
  forget.
The verdict didn't flip (soundness holds either way), but the *reason* was
mis-located. Honesty-about-the-boundary applies to which-mechanism claims in a
review vote, same as to trust-LEVEL claims.

**How to apply.** When a soundness vote is about to name the mechanism that
catches a failure (not merely assert "something kernel-rechecks"), and there's
more than one plausible backstop (a local re-check the code makes explicit + a
universal check the framework always runs), **differential-verify**: neutralize
the suspected net (no-op it in a scratch copy) and rerun the discriminating
negatives. If they still fail-closed, the local call is redundant-for-soundness
and the universal mechanism is the real net — say so precisely. Cheap test, run
it before writing "X is the net." Distinct from the trust-LEVEL-mislabel family
(kernel backed claim grep the emission not the name, trusted by typing guarantee
is not kernel proved Q): here every claim about trust level was correct; the
error was WHICH of two redundant sound mechanisms is load-bearing. Sibling of
scope review vote to my lane (over-claiming in a review vote) — the differential
test is how you avoid it for net-attribution.
