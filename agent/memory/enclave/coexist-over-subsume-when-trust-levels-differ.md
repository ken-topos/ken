---
scope: enclave
audience: (see scope README)
source: private memory `coexist-over-subsume-when-trust-levels-differ`
---

# Coexist over subsume when trust levels differ

**effect-composition WP, D3↔D5 / AC3 subsume-vs-coexist ruling (2026-07-04, my
Architect-owned-core D1–D3 lane).** conformance-validator surfaced a real seam:
AC3 wants generality witnessed "structurally, not by a single example," and its
executable route is "≥2 distinct effect pairings through the same machinery."
Their recommended read of AC5 ("State becomes an instance of the general
mechanism") was **option 1: make the terminal driver `run_io` subsume the pure
handler `run_state`** so State runs at top level → an executable 2nd pairing.
spec-leader confirmed the call was in my lane.

**Why I ruled COEXIST, not subsume — the decisive move.** `run_state` is a
**kernel-re-checked pure fold** (`declare_def`, a real kernel term); `run_io` is
**trusted Rust** in the outer-ring interpreter that does real-world I/O. Spec
`36 §2`'s whole layered encoding keeps effect **semantics** in the pure kernel
and puts **only real-world I/O** in the trusted driver. Literally folding
State's discharge into `run_io` would move a currently-kernel-verified fold
**into the trusted Rust surface** — a **TCB/trust-level regression**, backwards
per PRINCIPLES' small-auditable-TCB. So "subsume" here was the *more*-dangerous
option despite reading as the cleaner / more-general / subsume-don't-proliferate
one. The AC5 "instance" text is satisfied the correct way at the
**data/signature** level: State's `resp_sum s f RespF` becomes the literal
instance `resp_sum (StateOp s) f (resp_state s) RespF` of the general D1 family,
and `run_state` an instance of §5.1's general fold shape —
instance-of-the-general-machinery, NOT run-by-the-trusted-driver.

**The reusable rule.** When an AC or co-author invites you to SUBSUME A into B
("make A an instance of general B"), before ruling, check the **trust level** of
each. If A is proof-carrying / kernel-re-checked and B is trusted / outer-ring /
in the TCB, folding A into B *removes A from the kernel's protection* — rule
**coexist** and satisfy "A is an instance" at the data/signature layer, not by
merging code. Subsume freely only when A and B sit at the **same** trust level.
This is the trust-level guard on subsume-don't-proliferate — sibling of the
whole kernel backed claim grep the emission not the name / tested not trusted
posture needs reachability precondition family (know which layer a mechanism
lives at before reasoning about it).

**Companion rule — structural grep is the RIGHT guard, not a fallback.** The
honest residual: `run_io`'s coproduct dispatch is a ~3-line effect-blind
`InL`/`InR` peel, and the only *behavioral* variation available without a 3rd
base effect is FS+Console + position-swap — which a set-{Console,FS}-hardcoded-
either-order adversary passes green. That behavioral discriminator is
**fundamentally unavailable** without proliferating a throwaway 3rd effect. So
generality is guarded **structurally** (grep: no effect-literal in the peel;
`InL`/`InR`-generic), and for code that simple the grep is **dispositive** — the
correct guard for that layer, NOT a weaker option-2 fallback. Don't let a
co-author frame "executable behavioral discriminator" as strictly-stronger when
the discriminator can't exist without proliferation; name the residual plainly
and let the structural guard stand.
