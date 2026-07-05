---
scope: teams/kernel
audience: (see scope README)
source: private memory `hand-built-elim-motive-and-method-gotchas`
---

# Hand-built Term::Elim motive/method gotchas

On `wp/State-effect-build` (2026-07-03, VAL2 #10 / OQ-C·C2), lifting `ITree` to
a 3-param dependent-response family required hand-building `Term::Elim` nodes
directly via `ken_kernel::declare_inductive`/`declare_def` (bypassing surface
`data`/`match`, since a ctor arg depending on an earlier arg's VALUE isn't
expressible in the surface `data` grammar, and routing through
`compile_match_matrix`'s `ColKind::Ih` would reopen the same soundness- adjacent
machinery `L-match-ih-fix` just touched). Three non-obvious traps surfaced, each
caught by the kernel's own fail-closed `check`/`infer` (never silently wrong —
every one showed up as a `TypeMismatch`/`VarOutOfScope`, just not always at an
intuitive location):

**1. A bare `Term::Lam` motive can never be `infer`'d, even reached via
`check`.** `check.rs`'s `infer` has an unconditional arm covering every
introduction form (`Term::Lam`, `Term::Pair`, …): each returns an error that
an introduction form can't be inferred, only checked against an ascription.
`infer_elim`'s `infer_motive_level` calls `infer` on the motive DIRECTLY — and
`check`'s own fallback for any non-Lam/Pair/Ascript term (which `Term::Elim` is)
is ITSELF "infer, then compare against expected" — so even when the WHOLE `Elim`
is being `check`ed against a known type, the motive inside it still gets
`infer`'d raw. Fix: always wrap
`Term::Ascript(Box::new(motive), Box::new(motive_ty))`, exactly like
`k1p5_wstyle.rs`'s own dependent-motive test does. Symptom before the fix:
`"cannot infer an introduction form ... without an expected type"`.

**2. Getting the ascription's OWN codomain right needs distinguishing large vs.
small elimination.** A motive that computes "a TYPE" per branch has TWO
different shapes with DIFFERENT classifiers: (a) the motive's BODY is the
LITERAL universe term `Term::Type(Level::zero())` (e.g. `resp_state`/
`resp_sum`, dispatching to decide `s` vs `Unit`) — this term's OWN classifier is
`Type1` (`Type0 : Type1`, large elimination, `motive_ty` codomain = `Type1`).
(b) the motive's body is a TYPE-FORMER APPLICATION like `ITree e resp b` (e.g.
`bind`/`runState`'s state-passing motive) — this term's classifier is whatever
level THAT family was declared at (`Level::Zero` here → `Type0` directly,
ordinary/small elimination, `motive_ty` codomain = `Type0`). Conflating the two
gives a `TypeMismatch{expected:Type0, found:Type suc 0}` (using (a)'s Type1
where (b) needed Type0) or the exact mirror-image error the other way — I hit
BOTH, one after the other, before separating them by "is the branch VALUE the
raw universe term, or an application producing something of a KNOWN declared
level?"

**3. A W-style method needs an explicit binder for the ctor's own raw recursive
FIELD, not just the higher-order IH.** The eliminator's method arity is
`Π Δk. Π IHs. M(...)` (`ken-kernel/inductive.rs::method_type`) — for
`Vis : (op:E) -> (Resp op -> ITree E Resp R) -> ITree E Resp R`, Δk includes
BOTH `op` AND the raw continuation field `cont`, THEN the IH is a THIRD binder
(`ih : Resp op -> ITree E Resp R'`), even when the method body only ever uses
`ih` and never touches `cont` directly (as in `bind`, which literally re-wraps
`Vis op ih` without inspecting the original continuation at all). My first
`declare_bind` attempt wrote `λop. λih. ...` (TWO lambdas, omitting `cont`) —
this doesn't fail immediately or obviously; it silently shifts every SUBSEQUENT
de Bruijn index in the method body by one, surfacing as a `TypeMismatch` on the
IH's CODOMAIN two layers deep (`expected (Π ... @5), found (Π ... @4)`) rather
than an obviously-missing-binder error. Tell: an off-by-EXACTLY-one Var-index
mismatch deep in a W-style method's structure, when the method was hand-built
with fewer lambdas than `recursive_args`'s reported `(usize, pis, args)` tuple's
own `pis.len()+1` (Δk count) would predict.

**How to apply:** when hand-building ANY `Term::Elim` (bypassing surface match,
e.g. for a genuinely dependent ctor the surface `data`/`match` machinery can't
express yet): (i) always `Ascript` every motive; (ii) before choosing the
ascription's codomain, ask whether the motive's BODY is the raw universe term (→
one level up) or a declared-family application (→ that family's own declared
level); (iii) count Δk from the CTOR'S OWN arg list, not from "however many
binders the body actually references" — a W-style ctor's continuation field is
ALWAYS its own Δk binder even when unused. Sibling of general fix can conflate
similar shaped different cases (each of these three is exactly the shape of bug
that's invisible until the kernel's OWN fail-closed check catches it) and
trusted by typing guarantee is not kernel proved Q-adjacent in spirit (the
kernel staying the sole arbiter is what made all three safe to iterate on rather
than a soundness risk).
