---
scope: enclave
audience: (see scope README)
source: private memory `kernel-backed-claim-grep-the-emission-not-the-name`
---

# Verify a kernel-backed claim by grepping the emission, not the name

When a spec's headline differentiator is "**X is kernel-backed** (a real
`22 §2.1` refinement obligation, prover-discharged, **kernel-re-checked**) —
stronger than a trusted check," verify the trust LEVEL in the build by grepping
the **producer** for the kernel-emission primitives — **not** by reading the
struct/method names.

**Live case (Sec2-build, `dec`-less merge_ready, blocked — 2nd blocking vote).**
`62 §3.1` promised the attenuation bound `authority c' ⊑ authority c ⊓ w` as a
kernel-re-checked refinement obligation (the whole value-add over Sec1's trusted
flow rules). The build shipped `AttenuationObligation` with
`is_satisfied() = authority_flows_to(child, bound)` — a **plain Rust boolean**.
Grep of `capabilities.rs` found **zero** `declare_postulate` / `Obligation` /
`cx.obligations` / `attempt_obligation` — `attenuate` never emits into the
V2/V3/kernel pipeline (unlike L1's overflow, which *did* `declare_postulate` +
`Obligation`). Yet the doc said "emit the kernel-re-checked refinement
obligation," the test was named `attenuate_bound_is_kernel_rechecked`, and the
QA labeled it "kernel-backed (refinement obligation)." The test (C3) just
checked the Rust boolean and **hand-constructed** the over-strong obligation —
green-vs- green w.r.t. the actual kernel-emission.

**Why it matters:** the soundness need not be a *hole* (a trusted check +
conformance net is valid, Sec1-level) — but recording it as "kernel-backed" is a
**false trust-level claim on the security model**, exactly what the §H
proven-vs-trusted split exists to prevent. A downstream that reads
"kernel-backed" trusts it more than a trusted-by-elaborator check earns.

**How to apply:** (1) The **name is not the mechanism** — a type called
`*Obligation`, a method `is_satisfied`, a test `*_kernel_rechecked` can all be
ordinary language-level logic. (2) **Grep the producer src** for the real
emission: `declare_postulate`, `Obligation {`, `obligations.push`,
`attempt_obligation`, a `: Ω`/`Term` goal. Their **absence** means the check
runs at the language level (trusted), not kernel-re-checked — regardless of
labels. (3) Confirm the discriminating test drives the **real** emission path
(`attempt_obligation` / the kernel re-check), not the language-level boolean
with a **hand-constructed** failure case. (4) Offer two paths: wire the real
emission, OR re-label honestly (trusted check) + name the kernel-emission a
tracked follow-on + erratum the spec's "kernel-backed" claim.

**Extension — a kernel check that RUNS on an UNFAITHFUL input is theater
(Lc-build, 2026-07-01, my CHANGES-REQUESTED, Architect-only sole net).** The
dual of "the call is absent": here `sct_check` (real kernel) genuinely runs, so
a grep-for-the-call passes and QA saw "real SCT" — but the elaborator reified
instance resolution into a group that **does not mirror the recursion**. Cyclic
detection was a *syntactic* `has_self_ref` check (same class+head) that
hand-built a self-loop; the well-founded arm admitted the dictionary
`pair_chain` with **zero edges** (the `where`-sub-goals were never encoded as
nodes/edges), so `sct_check` accepted vacuously. Result: it discriminates
*direct-self-ref-vs-not*, NOT *terminates-vs-not* — a mutual cycle
(`C(F a) where C(G a)` + `C(G a) where C(F a)`) slips an accept → search-time
hang, the exact thing the soundness-tagged AC forbids. **How to apply:** when a
spec makes a **reification/encoding faithful** (reify X as a group/term the
kernel bounds), verifying the kernel primitive is *called* is not enough —
verify the reified STRUCTURE actually encodes what the kernel is meant to bound
(edges = sub-goals, the descent metric = the real order); a real check on a
zero-edge / hand-shaped group is green-vs-green. Faithfulness of the reification
is the **trusted step** (the kernel bounds only the group it is handed); grep
the reification producer, confirm each sub-goal becomes a node/edge, not a
syntactic pre-check + vacuous call. Sibling of trusted by typing guarantee is
not kernel proved Q (the trust-level distinction) and conformance hand feeds the
deliverable (the hand-fed/green- vs-green tell); shares the grep-the-producer
method of soundness AC static vs runtime face.

**Extension — the rule cuts BOTH ways; grep the emission PER MECHANISM, not
uniformly (FS-driver Phase 1, 2026-07-04, spec-author).** "Grep the emission,
not the label" is **not** "assume the label lies." A `kernel-re-checked` comment
can be a **true** label on one function and a **false** one on its sibling, in
the **same file** — so a uniform skepticism ("the label is a mislabel") is as
wrong as uniform trust. **Updated fact (supersedes the Sec2-era "zero
`declare_postulate` in `capabilities.rs`" above — verify at pickup):** TODAY
`capabilities.rs` holds **both** — `discharge_attenuation` genuinely emits
`declare_postulate` + `Term::Eq`/`Refl` + `attempt_with_cert` (real
**kernel-backed** static refinement obligation, `62 §3`), while
`is_satisfied()`/`authority_flows_to` are plain Rust bools (**trusted**, the
runtime gate). Either the code gained the emission since Sec2-build or the
earlier grep was scoped to `attenuate`/`is_satisfied` and missed
`discharge_attenuation` — either way the flat "capabilities.rs has zero
emission" is now imprecise. **How to apply:** when a trust-level claim **spans
multiple mechanisms** (a runtime gate + a static obligation; two sibling
checks), grep the emission for **each** separately and state the split — don't
let a "trusted" verdict on one function smear onto its kernel-backed sibling, or
vice-versa. Live: authoring FS-driver D5, my fold correctly split the runtime
`authorizes` gate (trusted Rust) from `attenuate`'s static obligation
(kernel-re-checked); CV's §2c had lumped both as "no emission" (caught +
erratum'd at the gate), and a flat "the kernel-re-checked comment is a mislabel"
would have been the opposite error. This is two arm producer needs a case per
arm applied to trust-level prose: one arm trusted, the other kernel-backed, each
pinned by its own emission grep.
