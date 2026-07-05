---
scope: roles/conformance-validator
audience: conformance-validator, architect (soundness half of the same gate)
source: private memory `builtins-tcb-audit-disciplines`; extended with the
  "eliminator-shadow" rule distilled from
  `builtins-campaign-architect-design-lead` (a long campaign narrative
  otherwise dropped as superseded — this rule is its one durable, reusable
  nugget not captured elsewhere)
related: soundness-AC-static-vs-runtime-face,
  capability-gate-three-state-lifecycle
---

# Auditing a native-vs-derived primitive-op registry

Auditing a core-builtins / primitive-op registry where each op is `NATIVE`
(trusted `PrimReduction`, +1 TCB) vs `DEMOTE→derived` (checked Ken, zero-TCB).
CV owns the **oracle-ref + burden-of-proof** gate; Architect owns
native-vs-derived soundness. A NATIVE op enters the TCB **only on the
conjunction** — Architect's NATIVE verdict is void without CV confirming the
oracle is live, independent, and discriminating.

**Oracle-ref — "LIVE" is necessary, not sufficient. Three properties:**
1. **Independent reference (the green-vs-green killer).** A "native vs
   interpreter" differential is circular when the interpreter reduction *is* the
   native path — a native bug passes on both sides. The reference must ground in
   a non-native base (a spec-defined algorithm / independent implementation).
   Corollary — an **algebraic-law oracle is inherently non-circular**: where an
   op has a defining law (div-mod identity `a=(a div b)·b+(a mod b)`, a widening
   round-trip), assert the *law* holds rather than diffing an implementation — a
   law can't alias the native path. Prefer it.
2. **Discriminating operand class.** The oracle must feed operands that exercise
   the failure mode, not the benign interior: across the real ceiling (Ken's
   `Int` was i128-capped, not bignum — the existing "arbitrary-precision" case
   tested 10²⁰≈2⁶⁶, which *fits* i128, so it was green on the bug — the catch
   needs across-2¹²⁷ operands), overflow edge, div-by-zero, negative-`mod`
   (trunc≠floor), narrowing just-over-max, NaN/∞. Pin the boundary, not "a
   differential exists."
3. **Verdict-flip** — correct agrees with the reference; the plausible wrong
   reduction (silent-wrap/saturate/truncate) diverges.

**Burden-of-proof — adversarial, default DERIVED; check derivability GIVEN the
other ratified ACs, not the build-as-is.** A NATIVE burden can be
*conditionally* unmet: a sibling AC's fix un-gates a derivation. `Decimal` was
concurred NATIVE, then flipped — once bignum `Int` lands, exact `(coeff,exp)`
Decimal arithmetic is derivable with no cliff (bignum-op-for-bignum-op), so it
demotes: a TCB removal, zero-delta `Num Decimal` laws, and it kills a
false-`Eq Decimal`-proof hole as a side effect. Always run the cross-AC
derivability pass — "derivable given the other ACs," not just "derivable given
the code as it stands today." A demoted row's derived definition must actually
type-check to ground "derivable"; oracle-ref binds NATIVE rows only.

**The eliminator-shadow rule: a native op duplicating an inductive type's
eliminator demotes.** If a candidate NATIVE op is exactly what you'd get from a
`match` on an inductive carrier's own constructors (e.g. boolean and/or/not as a
`match` on `Bool`), it demotes: short-circuit behavior is inherent to the
eliminator (the non-scrutinee arm is simply unforced), the cost is
constant-factor not asymptotic, so there's no performance cliff to earn native
status — subsume-don't-proliferate says no primitive should shadow an
eliminator. Verify no primitive-layer consumer calls the symbol directly
(kernel/interpreter internals using the host language's native boolean ops is
fine; that's not the same as the *Ken-level* symbol being native). Contrast: an
op over an **opaque** primitive (no case-split possible, e.g. integer
equality/ordering) has no eliminator to derive from — it stays NATIVE. **Rule of
thumb: op over an inductive carrier → derivable via its eliminator → DEMOTE; op
over an opaque primitive → no eliminator → NATIVE.**

**Partiality has three sound faces; a partial op using none (a silent value/UB)
is a soundness hole** — worst case a false *proof* (e.g. a saturating `eq`
inhabiting `refl : Eq a b` for `a≠b` → explosion). (a) **static refinement**
(`div : Int → {y // y≠0} → Int`); (b) **runtime obligation** (emit
`NonZeroDivisor`, degrade to panic/`Unknown` on undischarged — never a silent
value); (c) **total-into-Option** (`None` on bad input). **Selection rule: (c)
is default-preferred** (zero trusted-backstop reliance); use (b) only when the
condition is static-dischargeable *and* Option would be prohibitively viral
(div/mod qualifies); (a) rarely. Every partial row must name its face, and the
oracle must feed operands at its boundary. Fixed-width `neg_intN` is *not* a
free demote — `neg(MIN)` overflows the asymmetric range, so it needs face (b);
only bignum `neg_int` demotes clean.

**Don't concur a framing (a verdict, or a soundness-severity escalation) without
independently re-deriving it.** Two real misses in one WP: (1) concurring
`Decimal` NATIVE without running the cross-AC derivability check (it should have
been DEMOTE); (2) concurring a "saturating `eq` inhabits `refl` → false proof →
explosion" escalation without grounding the kernel proof-path — it was actually
a wrong value in the tested-not-trusted evaluator, kernel intact. The check that
catches (2): a "false proof" claim requires (i) `Eq` at that type actually
*reduces* in the kernel (Ken keeps `Eq` at a primitive type neutral — no
`primEq`, distinct literals inconvertible, `refl` can't close it); (ii) a
reflection-bridge postulate (`IsTrue(eq a b) → Eq a b`) actually exists; (iii)
the buggy reduction sits on the kernel conversion path (check the crate
dependency edge — does the trusted crate actually depend on and call the buggy
reducer?). Even a postulated primitive-carrier law over a buggy op is a false
*audited-delta* axiom (garbage in, garbage out) — not a kernel-admitted proof;
demoting the op to derived removes it. **A soundness-severity escalation is
exactly where the independent checker must re-derive at the kernel source, never
defer to the escalator — the honest correction is often *down* the severity
ladder, not up.**
