---
scope: enclave
audience: (see scope README)
source: private memory `higher-kinded-class-param-and-funext-definitional`
---

# Higher-kinded class params needed an outer-ring fix; funext is definitional

**CAT-1 (`wp/CAT-1-constructor-classes @ 93cc072`) core grounding, my
Architect-owned design lane (2026-07-04).** spec-leader routed the
higher-kinded-admission fork + the Functor law form to me. Both grounded on the
landed elaborator + kernel; Fact 2 is durable and I re-touch it at the CAT-1
build gate and at CAT-2 (Monad laws inherit the law form).

**Fact 1 (RESOLVED — kept as the grounding-methodology record, not a live
gap) — the `class` param was hard-coded to `Type0`; higher-kinded needed an
outer-ring extension (NOT a kernel change).** At the time, `elab_class_decl`
bound the class param with `Term::ty(Level::Zero)` at **three** unconditional
sites: the field-type elaboration context, the `chain_sort` inference context,
and the class type itself. The parser (`parse_class_decl`) took the param as
a **bare ident only**, dropping any `(f : K)` kind annotation. So
`class Functor f` bound `f : Type0`, and a field
`map : (a b : Type) -> (a->b) -> f a -> f b` elaborated `f a = App(f,a)` with
`f : Type0` → applying a non-Π → **KernelRejected**. No package-only escape
(the whole point is a class over a type constructor). **The fix was
outer-ring, kernel-untouched** (AST param carries an optional kind; parser
parses `(f : K)`; elab uses the elaborated kind term, default `Type0` for
back-compat) **and has since landed** — `parse_class_decl` now accepts
`(f : K)` and `elab_class_decl` threads the elaborated kind through
`param_kind_core`; verify current state before citing this as a live gap. The
kernel already supported the rest at the time — checked both axes:
`sort_sigma` is **level-generic** (`max(s1,s2)`, Ω iff both Ω) so a `Type1`
structure record (where `map` lands) is admitted; and `List`/`Option` are
`data X a` ⇒ `Type0 -> Type0` indformers so `instance Functor List` has a real
higher-kinded head to substitute. **Axis A (quantify over f) blocked, Axis B
(record universe) clear** — a live instance of buildability ruling must ground
every axis where the two axes give different answers (checking only B would have
wrongly cleared it).

**Fact 2 — funext is DEFINITIONAL in Ken's OTT ⇒ pointwise is the canonical law
form.** `obs.rs:90` (`eq_at_pi`):
`Eq ((x:A1)→B1) f g ⇝ (x:A1) → Eq (B1 x) (f x) (g x)` — equality at a Π/function
type whnf-reduces to the pointwise form. So the function-level Functor id-law
`Equal (f a -> f a) (map idf) idf` and the pointwise
`(x:f a) -> Equal (f a) (map idf x) x` are the **same Ω-proposition up to one
reduction step** (both Ω-clean: `Equal _ _ _ : Ω` per obs.rs; direct value
equations, so the `51 §3` truncation catch does NOT fire — no `‖·‖`, the
`Ord.total` disjunction is the only law that needed the `Bool`-equation dodge).
Ruled **pointwise canonical**: it is the normal form the prover's goal already
is (direct induction on `List`/`Option`, no funext layer to strip, zero delta);
the point-free equation is a definitionally-equal restatement available for free
⇒ **do not proliferate a second law field**. CAT-2's Monad laws inherit this —
state them pointwise-applied. Companion to proof relevant inductive cannot be
declared at omega (keep law codomains Ω-clean) and the durable-write-up
discipline enclave ruling in thread is not a durable deliverable (this ruling
owes a transcription into the CAT-1 WP doc + `spec/50-stdlib/51` §4 that I
fidelity-gate).
