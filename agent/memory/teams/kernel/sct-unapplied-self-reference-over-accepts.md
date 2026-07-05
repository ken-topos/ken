---
scope: teams/kernel
audience: (see scope README)
source: private memory `sct-unapplied-self-reference-over-accepts`
---

# An SCT gate keyed on applied occurrences over-accepts unapplied self-reference

**Trust-root finding (K2c SCT gate, 2026-07-01).** A syntactic SCT termination
checker admits a transparent (δ-unfoldable) recursive def iff every *idempotent
self-loop* in the size-change-matrix closure has a strict diagonal. Its
soundness has a hidden **precondition**: it only builds a call **edge** when a
group member appears in **head position of an application** (`collect_calls`
matches `App` → `peel_app` → head `Const∈group`); a **bare/unapplied `Const` is
a leaf** → no edge. Combined with the
`if edges.is_empty() { return Ok(()) } // non-recursive` shortcut in
`sct_check`, an **unapplied self-reference produces no edge and is admitted
transparent**:

- `bad : Bottom := bad` (nullary, body = bare `Const{bad}`) — minimal repro.
- `loop : Bottom := id loop` (member passed **unapplied** to a combinator that
  invokes it — first-order-SCT laundering).
- `size (Node xs) = 1 + sum (map size xs)` — higher-order recursion-through-map;
  `size` unapplied under `map`.

Each is admitted transparent → a **closed inhabitant of `Bottom`** (proof of
False; the term's mere existence at type Bottom is the inconsistency, no
evaluation needed) → and δ-unfolding it **loops** (`whnf`'s δ-unfold is an
unbounded `loop` with **no fuel/cycle guard** — the SCT gate is the *sole*
termination guarantee; there is **no conversion-side backstop**). Present in
BOTH the Rust code (`sct.rs`) AND the spec (`17 §4.1`: *"Calls are the applied
occurrences of a group member"*) — the spec's own theorem ("every infinite call
sequence has an infinitely-decreasing thread") tacitly assumes **every δ
re-entry corresponds to an analyzed edge**, i.e. all occurrences are applied.

**Why it matters / the general shape.** This is the tested not trusted posture
needs reachability precondition / untrusted layer backstop hole for omissions
pattern at the trust root: a gate is sound **only under a precondition that is
assumed, not enforced**. The tell is a **"nothing found ⇒ accept" shortcut**
(`edges.is_empty() ⇒ Ok`, `no obligations ⇒ pass`) sitting on top of a
**detector that only sees one syntactic shape** (here: applied calls). "No calls
found" silently becomes "terminating" — but it really means "no calls *of the
shape I detect*." The applied-call algorithm itself was **fully sound** (compose
conservative, closure without union-masking, structural-descent provenance, safe
rejection direction, clean pre-admit/check/gate/promote-or-rollback protocol) —
the hole is *only* the un-detected shape. **Review tell for any
termination/positivity/emission gate: enumerate the syntactic positions the
detector recognizes, then ask what happens to the recursive/dangerous occurrence
in EVERY OTHER position — and whether the "empty ⇒ accept" base case is guarded
by an enforced precondition that those other positions can't carry the danger.**

**Fix (conservative — the gate's own §4.2 philosophy):** model an unapplied
group-member occurrence as a `?`-everywhere self-loop (idempotent, no strict
diagonal) → **reject**; equivalently a cheap `occurs`-style guard rejecting a
transparent def with any not-fully-applied group-member reference. Safe
direction (§4.2: under-record → reject more, never admit non-terminating); such
recursion must then use an eliminator (`17 §4.3` Scope: "eliminator recursion is
already structural; SCT covers the rest"). Spec `17 §4.1` must **state the
applied-only precondition** and close the `edges.is_empty` shortcut; `conv.rs`'s
header comment *"δ is acyclic (non-recursive transparent defs only)"* is **stale
post-K2c** (K2c is precisely what makes δ cyclic) and must be fixed.

**RESOLVED (2026-07-01, `dec_227p0vyh6rm66` @ `2f3b9ac`).** Fix folded into the
same branch (Steward ruled: don't merge the hole). Shape: (a) kernel — a bare
group-member `Const g` in `body(f)` pushes an all-`?` edge `f → g`
(`ScMatrix::zero`), rejecting-self-loop when `f=g` and cycle-poisoning
(`?`-absorbing) when `f≠g`; (b) spec §4.1 reframed from my "applied-only
precondition" to the stronger **call-graph completeness invariant** (*every*
occurrence contributes an edge — applied its matrix, bare the all-`?`), which is
why "no edges ⇒ accept" becomes sound; (c) two-arm conformance net + kernel
reject tests, flip verified empirically (revert only `sct.rs` to pre-guard →
both reject tests FAIL "admitted" → restore → pass). **Implementation caution
(reusable): splitting a catch-all leaf arm to add detection can DOUBLE-FIRE with
an enclosing arm that already handles the applied case** — here the `App` arm
emitted the real edge then recursed into the head `Const g`, which the new
bare-`Const` arm re-counted as a spurious all-`?` self-loop → *broke every
accept test* (`plus` etc. flipped to reject). Fix: the `App` arm tracks
`head_is_applied_group_call` and skips the head recursion in exactly that case;
args always recurse. When reviewing such a guard, check the three spots:
skip-condition scoped to the exact already-handled shape; args/other-positions
still recurse; the real edge preserved.

**Process:** the delta under review (`cc17122`) was a benign rename +
accept-test — the gap was in the **already-landed substantive code**
(`7d38b55`), surfaced by using the deferred-for-clean-headroom deep review to
re-derive the SCT gate rather than rubber-stamp the tiny diff. **A posed
"certify no over-acceptance" gate on the trust root must not be waved through on
a benign delta when the re-review finds a counterexample** — withhold the
soundness certification, hold the Decision, hand a concrete repro + a cheap fix,
and recommend folding the guard into the same branch so the gate lands
certified-sound in one shot. Sibling of two arm producer needs a case per arm
(there: a dropped match arm; here: a dropped syntactic position).
