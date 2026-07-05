---
scope: teams/kernel
audience: (see scope README)
source: private memory `conv-reduction-arm-gate-needs-termination-stress`
---

# A conv/reduction-arm gate needs a termination stress test

**The miss (Ken Gap-conv, 2026-07-03).** I gated + merged a 5-line
`(Term::Eq, Term::Eq)` congruence arm in `conv_struct` (`90f39fe`) — verified
soundness (Ω-proofs never reach it, fail-closed direction only), isolation-flip
red->green, **164/164 suite**, a discriminating negative rejected. It was
soundness-correct. It still shipped a **non-termination**: law 4's assembled
`toListOrdered` OOM'd (~12 GB, SIGKILL) during the kernel's `declare_def`
whole-body recheck. Root-caused only after a 6-round bisection: the arm turned a
terminating `(Eq,Eq) => false` branch into a **recursing** one, and that
recursion drove `eq_at_inductive`/funext (`eq_at_pi`/`cast_at_pi`) into a
**productive non-normalizing loop** on `Eq` at a recursive inductive (`Tree`)
with function-typed sub-structure (`leq`/`pairLeq` comparators) — the same
logical comparison regenerated one binder-cluster deeper each lap (**+7 uniform
weakening/lap**), never occurs-checked, never memoized.

**Why the gate missed it: soundness + a green suite do not test TERMINATION on
the newly-reachable path.** The suite's `Eq`-arm cases were small and
non-regenerating; the real customer (a deep recursive-inductive `Eq`) was never
exercised. **Gate rule for my role:** when gating a new
conv/reduction/congruence arm (anything that adds a recursion into the kernel's
reducer), add an explicit **termination/normalization stress on the
recursive-inductive + function-typed case**, not just a soundness argument + a
passing suite. A congruence arm that is sound can still be **non-normalizing**
on inputs the suite doesn't reach.

**Trigger vs root — the load-bearing reframe.** The arm is the **trigger /
messenger** (it makes the path reachable); the **root** is a latent
non-normalization in the observational reducer, pre-existing and masked by the
old `=> false` shortcut. The fix targets the **root** (harden obs-reduction
termination), NOT the 5-line messenger. Sibling of gate widening exposes latent
bugs in newly reachable code — a completeness widening surfacing a latent
downstream bug — with the NEW angle that the surfaced bug is a
**non-termination**, invisible to soundness+suite.

**Divergence != unsoundness (but still fatal).** A checker that infinitely loops
admits *nothing* false — the trust root holds, no wrong proof is accepted. So it
is a **completeness/usability** regression, not a soundness hole. But "the
language is non-functional with that resource usage" (Pat) applies the bar to
the kernel itself: a trusted checker that loops on a valid-looking proof must
not sit on main. Hence **revert-to-green** (temporarily remove the divergent arm
to restore termination-guarantee) while the real fix is designed off-pressure,
then re-land the *fixed* arm. Distinct from permanent-revert-as-strategy (which
re-walls the customer) — we fix and re-land, not abandon.

**Diagnosis discipline that worked (reuse it).** Hold **site != mechanism** —
the bisection *named the site* (the arm) but not *why* it diverges; do not paper
it with a plausible fix (I pre-carried an Ω-PI-routing fix that the trace
REFUTED — the equands were `Tree` DATA, not Ω-proofs). Instruments,
cheapest-first: (1) **isolation-flip** — comment out exactly the suspect (5
lines) on current main, hold all else, rerun the identical harness; OOM -> 0.05s
clean reject = decisive trigger ID (operationalizes "blame the most-recent
change / a path it created"). (2) **printf-depth trace** — one global depth
counter + ring buffer + a hard trap at depth ~5000; the repeating frame-cycle
names the loop. Read the shape: **monotonic never-popping** depth (~1:1 with
calls) = genuine infinite loop (not huge-finite exponential, which
pops/branches). The **+7 UNIFORM weakening/lap** is the tell that the recurrence
is **alpha-equal modulo a uniform de Bruijn shift**.

**Fix-vector soundness envelope for such an obs-termination loop** (pre-clear
before the build): the fix must decide **identical convertibility**, never skip
a check. **Free-sound** options: memoize the reducers (`eq_at_inductive`/
`cast_reduce`), or leave the recursive-inductive `Eq` conjunct **neutral**
(fail-closed). **Obligation-bearing** option: an occurs-guard that *returns
true* on a recurring `(ty,a,b)` goal — sound ONLY under the argued lemma
**recurrence => alpha-equal-modulo-weakening sub-goals** (the +7-uniform-shift
is evidence for it, but it must be *argued* against the repro, a HARD gate, not
assumed). Put the asymmetry (two free options + one obligation option) in the WP
frame so the build doesn't reach for the obligation path casually. This is the
perf primitive vs fix the evaluator fork class — fix the reducer, don't grow the
trust root, don't make proof authors contort.

**Revert safety needs the workspace check, not a customer-grep.** I asserted
"revert safe: law 4 not on main" from grepping the named customer. A
*completeness* arm has **workspace-wide blast radius** — some other `.ken`/test
proof could type- check *because* the arm converts something, and regress on
revert (the K7 lesson). Verify with `cargo test --workspace`-green on the revert
PR, not a grep. (Here it came back clean: 161/161 ken-kernel = pre-arm baseline.
But confirm, don't assert.) The gate-time repro must FLIP (green vs green does
not confirm a fix): reproduces the divergence pre-fix, killed post-fix,
convertibility preserved on a discriminating valid/invalid pair.
