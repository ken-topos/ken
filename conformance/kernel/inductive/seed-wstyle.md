# Kernel-1.5 conformance — W-style (Π-bound) recursive inductives

Format: `../../README.md`. These are the **K1.5**-scoped seed cases for
W-style (Π-bound, branching) recursive inductives: the admission rule, the
dependent eliminator with its **Π-abstracted** induction hypothesis, and the
W-style ι-reduction. They extend `../seed-k1.md` (AC-4 eliminator ι, AC-5
strict positivity) — they **add** admittance for the strictly-positive Π-bound
class and **do not regress** any existing K1 case.

Ground truth: `spec/10-kernel/14-inductive.md` §2.1 (admission), §3.1
(eliminator + Π-abstracted IH), §7.7 (W-style ι), §8.4–§8.5 (admission gate /
deferred nested-mutual), §9.4 (subject reduction + termination + the adversarial
boundary). Frame: `docs/program/wp/K1p5-wstyle-inductives.md`. Clean-room:
grounded in the landed §-bodies + first principles; `yon` not consulted.

Every discriminating case **flips** (correct accepts / the targeted bug rejects,
or vice versa) or asserts a **verdict-independent structural output** (the ι
reduct, the neutral form of a stuck inner eliminator). No free implementation
choices remain to tag `(oracle)` — admittance and ι are spec-settled.

The two canonical admitted shapes used throughout:

```
data W (A : Type ℓ) (B : A → Type ℓ) : Type ℓ where
  sup : (a : A) → (B a → W A B) → W A B

data ITree (E : Sig ℓ_op ℓ_resp) (R : Type ℓ_R) : Type (max ℓ_R ℓ_op ℓ_resp) where
  Ret : R → ITree E R
  Vis : (e : E.Op) → (E.Resp e → ITree E R) → ITree E R
```

---

## AC1 — W-style admitted; negative / ill-formed still rejected (verdict-flip)

Spec: `14 §2.1`, `§8.4`, `§8.5`, `§9.4`.

### kernel/inductive/wstyle-w-type-admitted
- spec: `14 §2.1`, `§8.4`
- given: declaration `data W (A : Type ℓ) (B : A → Type ℓ) : Type ℓ where sup :
  (a : A) → (B a → W A B) → W A B`
- expect: **accepted** — `W` type-checks and `elim_W` is generated
- why: `sup`'s second argument `(B a → W A B)` is a W-style recursive
  occurrence — `W` is the arrow's **target** and the domain `B a` is `D`-free,
  so it is strictly positive (`§8.2` already accepts it). Pre-K1.5 the kernel
  rejected it through the **separate** blanket gate
  `check_no_pi_bound_recursive` (`§8.4`); K1.5 **retires that gate**, so the
  shape is now admitted with its Π-abstracted IH (`§3.1`). This is the headline
  case — it flips accept under K1.5 from reject under K1.

### kernel/inductive/wstyle-itree-admitted
- spec: `14 §2.1`, `§3.1`
- given: declaration of `ITree E R` with `Ret : R → ITree E R` and `Vis : (e :
  E.Op) → (E.Resp e → ITree E R) → ITree E R`
- expect: **accepted** — `ITree` type-checks and `elim_ITree` is generated
- why: `Vis`'s second argument `(E.Resp e → ITree E R)` is W-style (`ITree`
  target, domain `E.Resp e` is `D`-free). A second, distinct W-style family
  (`B` is a genuine dependent family `E.Resp e`, not a fixed type) — the L5
  client (`36 §2.1`). Confirms admittance is by the structural test, not a
  hard-coded `W` special case. `§2.1`.

### kernel/inductive/wstyle-negative-domain-rejected (soundness)
- spec: `14 §2.1`, `§8.3`, `§9.4`
- given: declaration `data Bad : Type 0 where mk : (Bad → Bool) → Bad`
- expect: **rejected** at admission (non-strictly-positive occurrence)
- why: the adversarial boundary (`§9.4`). `Bad` occurs to the **left** of the
  inner arrow in the argument type `(Bad → Bool)` — a negative occurrence;
  `Pos_Bad^-(Bad)` fails (`§8.3`). This is exactly the occurrence whose
  eliminator would let one build a non-terminating, inconsistent fixpoint.
  **Verdict-flip:** under the precise K1.5 bug — removing the blanket Π-bound
  gate *and* over-broadly relaxing admission so any Π-bound argument whose body
  head is `D` is admitted — this would flip reject→accept. K1.5 retires only the
  blanket gate; `§8.2` positivity stays the sole structural test and **still
  rejects** this (`§8.4`). Shares its program with `../seed-k1.md`
  `negative-bad-rejected` but guards a **different** bug: that the gate removal
  did not also remove the positivity rejection of negatives.

### kernel/inductive/wstyle-branching-domain-not-d-free-rejected (soundness)
- spec: `14 §2.1`, `§8.2`, `§8.3`
- given: declaration `data Bad5 : Type 0 where mk : (Bad5 → Bad5) → Bad5`
- expect: **rejected** at admission (non-strictly-positive occurrence)
- why: `mk`'s argument `(Bad5 → Bad5)` has its **target** `Bad5` in a positive
  position, but its **branching domain** is `Bad5` itself — not `D`-free, the
  second admission condition of `§2.1`. `§8.2` rejects it because the domain is
  checked at flipped (`-`) polarity and `check-pos-arg(Bad5, -, Bad5)` is false.
  **Verdict-flip:** a buggy admission that peeled the leading Π binder, saw the
  body head `Bad5`, and admitted **without re-checking the domain** would
  wrongly accept this; correct admission rejects. Sharpens the boundary on the
  exact axis K1.5 newly touches (Π-bound domains), distinct from any K1 case.

---

## AC2 — eliminator computes; the IH is USED, not β-discarded (verdict-flip)

Spec: `14 §3.1`, `§7.7`.

### kernel/inductive/wstyle-elim-w-iota
- spec: `14 §3.1`, `§7.7`
- given: `elim_W M s (sup a k)` well-typed (motive `M : W A B → Type ℓ'`, method
  `s : (a : A) (k : B a → W A B) (ih : (b : B a) → M (k b)) → M (sup a k)`)
- expect: **reduces-to** `s a k (λ b. elim_W M s (k b))` (W-ι)
- why: the W-style ι (`§7.7`, `§3.1`). The recursive result is threaded
  **through the branching function**: the IH supplied to the method is the
  eliminator applied **under the branch binder**, `λ b. elim_W M s (k b)` — a
  *function* `(b : B a) → M (k b)`, not a value. Structural assertion on the
  reduct (verdict-independent): a correct ι emits exactly this λ-abstracted IH;
  a mis-threaded ι (e.g. `elim_W M s k`, applying the eliminator to the
  branching function itself rather than under the binder; or `s a k` with the
  IH dropped)
  emits a structurally different reduct.

### kernel/inductive/wstyle-elim-itree-iota
- spec: `14 §3.1`, `§7.7`
- given: `elim_ITree M mr mv (Ret r)` and `elim_ITree M mr mv (Vis e k)`
  well-typed (methods `mr : (r : R) → M (Ret r)`, `mv : (e : E.Op) (k : E.Resp e
  → ITree E R) (ih : (x : E.Resp e) → M (k x)) → M (Vis e k)`)
- expect: `elim_ITree M mr mv (Ret r) ⇝ mr r` (base, no IH) and `elim_ITree M mr
  mv (Vis e k) ⇝ mv e k (λ x. elim_ITree M mr mv (k x))` (W-style, function IH)
- why: the L5 client's ι. `Ret` is a base constructor (no recursive argument, no
  IH); `Vis` is W-style — its IH is the λ-abstracted recursive call over the
  response domain (`§3.1` `elim_ITree`, `§7.7`). Structural assertion on both
  reducts.

### kernel/inductive/wstyle-elim-uses-ih-flips (soundness)
- spec: `14 §3.1`, `§9.4`
- given: `data Tree : Type 0 where leaf : Tree ; node : (Bool → Tree) → Tree`
  (`node`'s argument `(Bool → Tree)` is W-style, `Bool` `D`-free); two methods
  for the node case under motive `M := λ _. Nat`:
  - **correct** `mn := λ (k : Bool → Tree) (ih : (b : Bool) → Nat). add (ih
    true) (ih false)` (uses the IH at both branches)
  - **buggy** `mn' := λ (k : Bool → Tree) (ih : (b : Bool) → Nat). 1`
    (β-discards the IH)
  both with `ml := 1`; evaluate the leaf-count of `node (λ _. leaf)`
- expect: **different results** — correct `elim_Tree M ml mn (node (λ _. leaf))`
  computes `add (elim_Tree … leaf) (elim_Tree … leaf) = add 1 1 = 2`; buggy
  `elim_Tree M ml mn' (node (λ _. leaf))` computes `1`
- why: the discriminating AC2 case `§3.1` explicitly calls for — "a method that
  *uses* `ih b` … an IH-discarding method must reach a *different* result." The
  W-ι (`§7.7`) supplies `ih = λ b. elim_Tree M ml mn (k b)`, so a method that
  applies `ih true`/`ih false` actually recurses into both children, while one
  that ignores `ih` loses all subtree information. **Verdict-flip: 2 vs 1.** The
  motive is chosen so the value genuinely depends on the IH (a
  constant-into-the-node motive that ignored children would be green-vs-green —
  the vacuity trap this case is designed to avoid).

### kernel/inductive/wstyle-elim-dependent-motive-uses-ih
- spec: `14 §3.1`
- given: a **dependent** motive `M : Tree → Type 0` and an `elim_Tree` whose
  node method consumes the Π-abstracted IH at type `(b : Bool) → M (k b)` to
  build `M (node k)` (e.g. proving an all-nodes predicate by combining the
  per-branch proofs `ih true`, `ih false`)
- expect: **accepts and computes** — the dependent eliminator type-checks; the
  node method's IH parameter has type exactly `(b : Bool) → M (k b)`
- why: the *induction* principle (dependent motive), not just a recursor.
  Pin the IH type `(b : B) → M (k b)` from `§3.1`. **Verdict-flip:** a
  non-dependent recursor (motive not depending on the scrutinee, IH typed `(b :
  B) → C` for a fixed `C`) **cannot** prove a goal whose type mentions `node k`;
  the dependent eliminator accepts where the recursor rejects. Exercises a
  dependent motive that *uses* the IH (the frame's explicit requirement).

### kernel/inductive/wstyle-elim-two-levels
- spec: `14 §2.1`, `§3.1`
- given: `elim_W` instantiated at two distinct motive levels — `M₀ := λ _. Nat`
  at `Type 0`, and a large-elimination motive `M₁ := λ _. Type 0` at `Type 1`
  (computing a *type* by recursion on a `W`-value)
- expect: **both accept** — `elim_W` is polymorphic in the motive level `ℓ'`;
  the W-style argument type `(B a → W A B)` sits at `max(level B, ℓ_W)` with
  `level B ≤ ℓ_W` (no new universe rule), and large elimination into `Type 0` is
  permitted
- why: exercises the eliminator at **≥2 distinct levels** (catches a level-
  monomorphic eliminator that hard-codes `ℓ' = 0`), and confirms the `§2.1`
  level computation (`max(level B, ℓ_D)`, an instance of `14 §1` predicativity,
  `12 §2`) holds — admittance does **not** add a universe rule.

---

## AC3 — W-style ι participates in conversion and decides (no regression)

Spec: `14 §7.7`, `§9.4`.

### kernel/inductive/wstyle-iota-in-conversion
- spec: `14 §7.7`
- given: a conversion check between `elim_ITree M mr mv (Vis e k)` and its
  reduct `mv e k (λ x. elim_ITree M mr mv (k x))` in a well-typed context
- expect: **convertible** — the W-style ι fires during conversion/normalization
  on the constructor-headed scrutinee `Vis e k`
- why: `§7.7` — the W-style ι is the algorithmic form the conversion checker
  calls; a constructor-headed scrutinee always fires ι (`§7.2`). A bug that left
  the W-style eliminator stuck on a constructor head (treating it like a
  neutral) would make these **inconvertible** — the case flips accept→reject
  under that
  bug.

### kernel/inductive/wstyle-inner-elim-stuck-under-binder (soundness)
- spec: `14 §7.7`, `§7.6`, `§9.4`
- given: the reduct of one W-ι step, `… (λ b. elim_W M s (k b)) …`, where `b` is
  λ-bound and `k b` therefore has no constructor head
- expect: the inner `elim_W M s (k b)` is **neutral / stuck** — it does **not**
  re-fire during normalization (no further ι until the IH is *applied* to a
  concrete branch whose image is a constructor)
- why: this is the exact mechanism that keeps conversion **decidable** with a
  function-typed IH (`§7.7`, `§9.4`): each ι consumes **one** constructor layer
  and the residual `elim_W` sits under a binder applied to the neutral `k b`
  (`§7.6` stuck eliminator), producing **no constructor-free redex**.
  Verdict-independent **structural** assertion (the dual of a value-flip, per
  the non-observable-property rule): assert the inner eliminator's *form* is
  neutral.
  A bug that eagerly forced `k b` / re-fired the inner eliminator would either
  diverge or mis-normalize — the structural check catches it where a value
  comparison on a terminating example could not.

### kernel/inductive/wstyle-iota-decides-halts (soundness)
- spec: `14 §9.4`, `§7.7`
- given: normalize `elim_W`/`elim_ITree` applied to a finite (closed,
  constructor-built) W-tree under the conversion checker
- expect: **terminates** with a normal form — no infinite loop, no regression to
  the K2c SCT/decidability guarantee
- why: structural decrease on a **finite, inductive** (not coinductive) W-tree
  (`§9.4`): the recursion lands on **children** `k b` reached through the
  branching function, staged under a binder and stuck until applied, bottoming
  out at a base constructor (`Ret`/`leaf`) or a W-branching with empty domain.
  Eliminator recursion is total **without** SCT, and W-style ι introduces no
  general recursive δ-definition — the K2c decidability argument is untouched
  (`§9.4`).

---

## AC4 — no regression; the `14 §2` erratum is reconciled

Spec: `14 §2`, `§6`, `§8.4`, `§8.5`.

### kernel/inductive/k1-inductive-suite-still-green
- spec: `14 §6`, `§8.4`, `§8.5`
- given: the full existing K1 inductive corpus — `../seed-k1.md`
  `positive-{nat,list}-admitted`, `negative-{bad,under-pi}-rejected`,
  `nested-negative-in-application-rejected`, `d-in-own-indices-rejected`,
  `elim-{nat,vec}-iota-*`, `elim-vec-type-checks` — re-run under K1.5
- expect: **all unchanged-green** — K1.5 widens admittance by **exactly** the
  strictly-positive Π-bound class and nothing adjacent
- why: `§8.4` — no change to `§8.1`/`§8.2` is needed; only the separate blanket
  Π-bound gate is removed. Negative occurrences, nested occurrences (`§8.5`,
  `List (Rose A)`), and mutual families stay **rejected**; the structural
  eliminator and its ι are unchanged for direct (non-W-style) recursion. The
  regression gate for the trust-root extension.

### kernel/inductive/s2-erratum-allowed-reconciled
- spec: `14 §2`, `§2.1`, `§8.4`, `§8.5`
- given: the chapter prose after K1.5 — `§2`'s "Allowed" list entry `(Nat → D) →
  D` annotated "**admitted in K1.5**, §2.1", and no in-chapter K1.5-deferral
  note remaining for the W-style class
- expect: the chapter's **prose** and the kernel's **admittance** now **agree**
  for the strictly-positive Π-bound class (`§2` "allowed" is true), with
  nested/mutual correctly split out as still-deferred (`§8.5`)
- why: closes this WP's origin — **positivity ≠ admittance**. Pre-K1.5 `§2`
  said `(Nat → D) → D` was "allowed" (positivity) while the kernel *rejected* it
  (admittance), the stale-prose hazard. K1.5 makes the prose true and removes
  the deferral note (`§8.4`/`§8.5` split admitted-W from deferred
  nested/mutual).
  Documentation-vs-behavior reconciliation case: a checker reading `§2`'s
  "allowed" must find the kernel actually admits it.

---

## AC5 — `elim_ITree` exists, unblocking L5

Spec: `14 §3.1`, `36 §2`.

### kernel/inductive/elim-itree-unblocks-l5
- spec: `14 §3.1`, `36 §2`
- given: the generated `elim_ITree : (M : ITree E R → Type ℓ') → ((r : R) → M
  (Ret r)) → ((e : E.Op) (k : E.Resp e → ITree E R) (ih : (x : E.Resp e) → M (k
  x)) → M (Vis e k)) → (t : ITree E R) → M t`, used to define a structural fold
  (the shape of L5's `bind`/handlers/denotation)
- expect: `elim_ITree` is **generated and computes** (per `wstyle-elim-itree-
  iota`); a `bind`-shaped fold over it type-checks and reduces — L5's denotation
  half is buildable
- why: `§3.1` — "Generating `elim_ITree` is the concrete deliverable that
  unblocks L5's denotation half"; `bind`/handlers are structural folds on this
  eliminator (`36 §2`), total by `§9.4` without SCT. The kernel-side counterpart
  of L5's surface seed `eff-bind-is-tree-grafting`. AC5 (the high-fan-out
  unblock the frame's risk level rests on).
