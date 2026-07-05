---
scope: teams/kernel
audience: (see scope README)
source: private memory
  `dependent-match-construction-fails-closed-via-infer-elim`
---

# Dependent-match construction is not in the TCB; it fails closed via infer_elim

**Grep-confirmed on `origin/main` while gating `dependent-match-wstyle`
(`dec_2qecbyyghwr4d`, 2026-07-04).** When gating any WP that has the elaborator
*emit* an eliminator (`Term::Elim`) — dependent match IH-slot construction,
W-style/indexed method building, etc. — the soundness floor is the **kernel's
own recheck**, and it is REAL, not a label:

- `ken_kernel::check::infer_elim` (`check.rs`, step 3) does, per constructor:
  computes `mt = method_type(ind, k, motive, params, level_args)` then
  `check`s the supplied method `m` against `mt` — it reconstructs each
  method type from the **inductive decl + motive** via
  the kernel's OWN `method_type` (`inductive.rs`), never from an
  elaborator-supplied annotation, and error-propagates. It also runs
  `check_level_arity` (level_args vs the family's level_params) and rechecks
  motive/params/indices/scrutinee.
- **Reachability is structural:** `declare_def` kernel-checks the def body → the
  embedded `Term::Elim` dispatches to `infer_elim`. So every emitted eliminator
  is rechecked; the elaborator's `check_match_dependent` is NOT in the TCB.

**Consequence for gating:** a wrong IH/method construction (bad de Bruijn shift,
wrong Π-domain, etc.) makes `check(m, mt)` fail → the whole declaration is
rejected. That is **fail-closed / over-rejection** (a valid program won't
compile), never a false proof. So the arithmetic-correctness of a match/elim
construction is a **completeness** question (does the fix land + not
over-reject), and its soundness is bounded by this backstop regardless. Front-
loaded de Bruijn constructions (enclave-authored, implementer-transcribed) are
worth scrutinizing for *landing*, but a slip cannot be a soundness hole as long
as the emission stays reachable through the decl recheck. This is the CONFIRMED-
real instance of tested not trusted posture needs reachability precondition
(reachability precondition = "emitted Elim flows through declare_def's kernel
check") and the positive dual of kernel backed claim grep the emission not the
name (here the mechanism, when grepped, genuinely IS kernel-reconstructed).

**The specific arithmetic (dependent-match-wstyle), for when the build lands:**
W-style recursive field `k:(b̄:B̄)→D Δ_p t̄[b̄]` gets ONE method-telescope
binder whose type is `Π(b̄:B̄). M t̄[b̄] (k b̄)`. The outer reverse-build wrap
stays `weaken(&method, 1)` — **one shift per IH slot regardless of `nb`** (the
`nb` branch binders live INSIDE the domain type, never in the method telescope;
the kernel's cross-slot offset `(n−pos+j)` counts preceding IHs at +1 each,
never +nb). The `+nb` is intra-domain only: goal weakened by `n+nb`, `field_var`
at `+nb`, `b_dom = shift(subst_outer(B_bk, m, params, pos+bk), n−pos, bk)`. The
elaborator's `weaken`-accumulate idiom (`j=0` frame + `weaken(_,1)` per outer
slot) reproduces `method_type`'s explicit `+j`, preserving each domain's own
branch binders. `subst_var_generalize(term, j, u)` is a general capture-avoiding
var→term substitution (handles `u` = an application `(field_var b̄)`, not just a
var). Indexed FAMILIES (non-empty `idxs`) stay out of scope → finding→Steward.
