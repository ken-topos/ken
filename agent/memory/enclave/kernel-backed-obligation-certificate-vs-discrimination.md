---
scope: enclave
audience: (see scope README)
source: private memory `kernel-backed-obligation-certificate-vs-discrimination`
---

# A kernel-backed obligation can notarize without re-deriving the discrimination

**Grounded gating `wp/fs-driver` Phase-1 (`dec_7d7w0r185f1c7`, 2026-07-04)**,
against landed `ken-elaborator/src/capabilities.rs::discharge_attenuation`.

The attenuation obligation `authority c' ⊑ authority c ⊓ w` is discharged like
this: `declare_postulate` an opaque carrier type + an opaque `child` value; then
**if a Rust `obl.child_authority == obl.bound` holds**, use the SAME postulate
for `bound`, else a DISTINCT one; build `phi = Eq(auth_ty, child, bound)` and
`cert = Refl(child)`; `attempt_with_cert(env, &phi, cert)`.

- **Kernel part (real + load-bearing):** canonical (`child == bound`) → same
  postulate → `Refl : Eq(v,v)` ⇒ Proved. Over-strong (`child != bound`) →
  distinct opaque postulates → `Refl(child) : Eq(child, bound)` fails the
  convertibility check (distinct opaque consts aren't convertible) ⇒ Unknown ⇒
  obligation NOT discharged ⇒ elaboration rejects. I traced this; it is NOT
  inert / green-vs-green — the kernel genuinely refuses a forged Refl.
- **Trusted-Rust part:** the DISCRIMINATION (same-vs-distinct postulate) is a
  Rust `==`, and the monotone-downward guarantee mostly rests on
  `authority_meet` (Rust) computing a real meet. The kernel does NOT re-derive
  the authority arithmetic — it only notarizes that the presented certificate
  matches the postulate-setup Rust chose.

**The precise honest phrasing:** "kernel-backed" for such an obligation = "the
DISCHARGE (certificate) is kernel-verified," never "the kernel re-checks the
comparison/arithmetic." A doc/QA claiming a capability's soundness is
"kernel-backed" full-stop over-reads it if the value-comparison that sets up the
obligation is trusted Rust. (The FS-driver doc got this right after CV's §2c
erratum fix; my vote added the certificate-vs-discrimination precision as a
symmetric non-blocking note.)

**How to gate one:** two-sided grep. (1) Emission side — `declare_postulate` /
`Term::Eq` / `Term::Refl` / `attempt_with_cert` present ⇒ there IS a kernel
certificate (per kernel backed claim grep the emission not the name). (2)
Discrimination side — trace the Rust that decides WHICH terms are handed to the
kernel; that comparison + any arithmetic (`authority_meet`, `==`, `<=`) is
trusted, outside the kernel's protection. Confirm the kernel part is
load-bearing by the over-strong path (distinct postulates ⇒ Refl fails ⇒
reject), the differential verify which mechanism is the net move. Distinct from
the RUNTIME capability gate (`authorizes`/`authority_flows_to`/ `is_satisfied` =
plain Rust bool, no emission) which is trusted-Rust / tested not trusted posture
needs reachability precondition — don't let the runtime gate borrow the static
obligation's kernel-backing.
