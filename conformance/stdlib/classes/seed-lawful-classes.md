# ES4 lawful structure classes (`Eq`/`DecEq`/`Ord`) conformance — seed cases

Format: `../../README.md`. These pin the **laws-PROVED discipline** of the first
`catalog/packages/` catalog tranche (`spec/50-stdlib/51-lawful-classes.md`, ES4-classes)
— the pattern every later ES4 package follows. The load-bearing property is
**AC3**: on an **inductive carrier** an instance's **law fields are real kernel
proofs**, so "lawful instance ≡ zero-`trusted_base()`-delta instance"; a
law-**less** dictionary (postulated laws it *could* have proved) must be
**rejected as unlawful**. A **primitive** carrier (`Int`/…) is the honest
exception — its ∀-laws are **unprovable** (no eliminator), so its instance
carries a **declared audited delta** (`§6`), not zero — the carrier axis is
load-bearing. (The zero-delta *real-proofs* path is realizable now for the
**live-`Eq`-conclusion** laws via **K4** (`3be0e30`): `refl`/`trans`/`total` +
`Eq`'s `refl`. The **concrete-equality-conclusion** laws
`antisym`/`sound`/`complete` (per-branch `Top`/`Bottom`) needed the **K5 + K7**
kernel capability (K5's `tt`/`absurd` terms, `1c84a30`; K7's `eq_at_inductive`
operand-`whnf`, `4ae2baf`, `16 §8.1`) — both now landed, so they are **real
zero-delta proofs on main** (ES4-lawproofs-remainder, `9a82745`); `Eq`'s
`sym`/`trans` are likewise real zero-delta via **case-split** (not K6). The
first real instances (`Ord Bool` `refl`/`trans`/`total`, `Eq Bool` `refl`) are
on main (ES4-lawproofs, `72e38a5`); `Ord Bool`'s `antisym` + `DecEq Bool`'s
`sound`/`complete` landed with the remainder (`9a82745`) —
`Ord Bool`/`DecEq Bool` now **complete** zero-delta lawful instances.)

## Grounding (content-verified against the landed targets)

- `51-lawful-classes.md` — the three classes as **structure classes**
  (`33 §5.1`): op fields `Type`-valued, **law fields `Ω`-valued** (`§3`);
  totality is the **Bool-equation** `IsTrue (leq x y || leq y x)` (Ω-clean, no
  truncation, `§3`); an instance is a `declare_def` **record value carrying real
  law proofs** (`§5`); **zero `trusted_base()` delta on an inductive carrier**,
  an **audited delta** on a primitive carrier (`§6`, the **two-axes rule**: law
  *sort* Ω-clean **and** carrier *provability* via an eliminator); `where Ord a`
  supplies the same `leq` the explicit `sort` threads (`§4`).
- `33 §5.1`/`§5.2`/`§5.3` — a class is a **record** (`13 §3` Σ+η); the sort is
  kernel-computed (both-components-keyed `sort_sigma`, `13 §4`); an instance is
  a record value of ops **+ law proofs**. **No new kernel former.**
- `37 §6` — the verified `sort`'s explicit `leq : a→a→Bool` (ES2-remainder
  `2358b4d`) that `Ord` subsumes; the `isSorted leq ys ∧ Perm ys xs` obligation.
- `25 §3` — `trusted_base_delta`; `16 §1` — `Ω`/proof-irrelevance;
  `../surface/taxonomy/minimality.md` (ES1) — the zero-delta / surface-TB-Sound
  invariant AC3 is the **law-side reading** of.

## Scope — the law-side of the ES1 zero-delta invariant

AC3 is not a new soundness mechanism: it is the ES1 **zero-`trusted_base()`
delta** invariant read from the law side (`51 §5`). A law field inhabited by a
`postulate` is an `Opaque` entry → **non-empty delta**; a holed field **fails
the kernel re-check**. So the corpus reuses the ES1/Sec4 `trusted_base()` net
(`../surface/taxonomy/minimality.md`, `../../security/trust-model/`) — it does
**not** re-pin the delta mechanism, it pins that a **class instance's law fields
are subject to it**. **The carrier axis (`51 §6`) qualifies the equivalence:**
"lawful ≡ zero-delta" holds **on an inductive carrier** (whose ∀-laws are
kernel-provable by case-split/induction); a **primitive** carrier's ∀-laws are
**unprovable** (no eliminator), so its instance is separately an
**audited-delta** one (a *declared*, visible delta — the honest primitive
posture, not a defect). The verified-`sort` obligation itself is homed in
`../surface/collections/` (`sort-emits-issorted-and-perm`); AC2 here references
it, does not re-pin it.

**Tags.** `(soundness)` — the laws-PROVED gate (a law-less instance admitted as
lawful is a verification-soundness hole: downstream lemmas *assume the laws*).
`(oracle)` — the literal field spellings (`leq`/`eq`/`refl`/`antisym`/… — the
`51` naming, finalizable). The **law-field `Ω`-sorts, the zero-delta lawfulness,
the same-obligation subsumption, and every verdict** are **normative**.

**Static vs runtime face.** These pin the **static face** — the spec discipline
(lawful ≡ zero-delta on an inductive carrier) and the discriminating shape. The
**runtime/build face** — the actual canonical instances in `catalog/packages/` carrying
real proof terms, producer-grepped for `declare_postulate`/holes on the law
fields — is the named **Team-Language build follow-on** (`51 §8`), not this WP
(`soundness-ac-static-vs-runtime-face`).

**K4 landed — the live-`Eq` fragment is zero-delta; the concrete-equality laws
need K5 + K7.** Proving a per-branch law needs to eliminate a `Type`-inductive
into an **Ω-motive** (`λx. P x : Bool → Ω`) — the **K4** rule (`14 §3`), on main
(`3be0e30`). So an **inductive** carrier proves — by finite case-split — the
laws whose per-branch obligation stays a **live `Eq`** (`Ord`'s
`refl`/`trans`/`total`, `Eq`'s `refl`): **zero-delta now**. The
**concrete-equality-conclusion** laws — `Ord`'s **`antisym`** and `DecEq`'s
**`sound`**/**`complete`**, concluding or hypothesizing the kernel `Eq a x y` —
have per-branch obligations that reduce to a concrete **`Top`** (the
trivially-equal case → `Top`-**intro** with `tt`) or **`Bottom`** (the
contradictory-hyp case → `Bottom`-**elim** / ex-falso with `absurd`). Closing
them needs **two** kernel capabilities, not one: **K5** (observational-fragment
completion, `16 §1.4`, the `tt`/`absurd` *terms*, **landed** `1c84a30`) **and**
**K7** (the `eq_at_inductive` **operand-`whnf`** completeness fix, `16 §8.1`,
**forward**). K5's `tt`/`absurd` fire only once the goal/hypothesis `Eq` has
actually **reduced** to `Top`/`Bottom`, and these three laws wrap the carrier
through the instance's **own operation** (`bool_leq`/`bool_eq`), so their
operands are **redexes**, not bare constructors: `antisym`/`sound`'s
contradictory hypothesis `IsTrue (bool_leq/eq True False)` **and** `complete`'s
**equal**-branch goal `IsTrue (bool_eq True True)` both stay a **neutral `Eq`**
until the operand is whnf'd — exactly the K7 gap; only the bare-constructor
equal branches (`antisym`/`sound`'s `Equal Bool True True`) reduce under K5
alone. So a **complete** zero-delta `Ord Bool`/`DecEq Bool` (`antisym` mandatory
for a total order, `sound`/`complete` for decidable equality) needed the **K5 +
K7** kernel capability (K5 `1c84a30`; K7's operand-`whnf` fix, `4ae2baf`,
`16 §8.1`) — both now landed, so the three laws are **real, kernel-checked,
zero-delta proofs on main** (ES4-lawproofs-remainder `9a82745`: `antisym` closes
via `tt`/`absurd`, `sound`/`complete` via `absurd`; **no `Axiom` remains** in
either instance); **none needed K6** (no swapped-`Eq` hypothesis-reuse across a
stuck congruence). The ES4-lawproofs build surfaced the K7 gap by pushing the
real proofs to a wall (Architect-ruled). **`Eq`'s `sym`/`trans` realize by
case-split — K6 is grounded-but-customerless.** A **full case-split** on the
carrier (`Bool`: `sym` splits `(x,y)`, `trans` splits `(x,y,z)`) closes every
branch concretely — `tt` on a reflexive-conclusion branch, `absurd` where the
hypothesis reduces to `Bottom` (`bool_eq` of mixed literals ⇝ `False` via K7
before the head-check) — so **no hypothesis is reused across a swap** and the
`conv_struct` `Eq`-congruence (**K6**) is **never exercised**; these are **real
zero-delta proofs**, no `Axiom` (the `Eq Bool` case-split WP, Architect-ruled
`evt_78ntsfnyjdtq6`). **K6 stands grounded-but-customerless:** a sound *forward
general-completeness* capability (comparing two genuinely-stuck `Eq` props) that
nothing here consumes — a **sound positional** K6 would **not** have closed this
pair (only an **unsound cross-wise** arm could, a hard **NO**), so K6 is
decoupled from `Eq`'s realization, not a blocker. Only `Eq`'s `refl` routes
through an unresolved `bool_eq x x` (a live `Eq`, `Refl` fires). A **primitive**
carrier still proves **no** ∀-law (no eliminator) → **audited-delta**.
**Realization status:** the real, kernel-checked, zero-delta instances are on
main (Team Language, `catalog/packages/Core/Classes/`): `Ord Bool`
`refl`/`trans`/`total` + `Eq Bool` `refl` (`72e38a5`), and now `Ord Bool`'s
`antisym` + `DecEq Bool`'s `sound`/`complete` (`9a82745`,
ES4-lawproofs-remainder) — so `Ord Bool`/`DecEq Bool` are **complete**
zero-delta lawful instances, **no `Axiom` remaining**. `Eq Bool`'s `sym`/`trans`
are likewise **real zero-delta** via case-split (the `Eq Bool` WP), not K6 — so
**all three `Bool` instances (`Ord`/`Eq`/`DecEq`) are complete zero-delta
lawful, no `Axiom` anywhere**; K6 remains a sound-but-**customerless** forward
capability. K4's rule conformance is
`../../kernel/inductive/seed-k4-omega-motive-elim.md`; K5's is
`../../kernel/observational/seed-k5-omega-fragment.md` (both on main).

---

## AC3 — laws PROVED, not postulated (the hard soundness gate)

`51 §5`: an instance carries **real** proofs of its law fields; a
law-less/postulated/holed instance is **rejected as unlawful** (non-empty
`trusted_base_delta` / re-check failure).

### stdlib/classes/law-fields-real-proofs-not-postulates (soundness)
- spec: `51 §5` (laws PROVED = zero-delta), `33 §5.3` (instance = record value +
  law proofs), `25 §3` (`trusted_base_delta`), `13 §2` (Σ-Intro re-check)
- given: two `Ord K`-shaped instances for a user carrier `K` (an **inductive**
  `data K` — it has an eliminator, so its ∀-laws are kernel-provable by
  case-split/induction; this is the precondition that makes a postulate a
  *defect*), **identical in their operation field** (`leq = k_leq`), differing
  **only** in the law fields: (a) a **canonical** instance whose law fields are
  **real kernel proofs** (a `declare_def` record value, re-checked) — the
  **live-`Eq`-conclusion** laws `refl`/`trans`/`total` provable on K4 **now**,
  and (for a *complete* total order) **`antisym`** — its per-branch obligation
  reduces to a concrete `Top`/`Bottom`, **proved via the K5 + K7 capability**
  (K5 `1c84a30`; K7 `4ae2baf`, `16 §8.1`) — now a **real zero-delta proof on
  main** (`9a82745`); (b) a **law-less** instance whose law fields are
  `declare_postulate`d (and, as further arms, holed / stubbed-absent)
- expect: **the verdict flips.** (a) **accepts as lawful** — every law prop is
  kernel-proved, so the instance's **`trusted_base_delta` is empty** (the law
  props ∉ `trusted_base()`); (b) **rejected as unlawful** — a **postulated** law
  field is an `Opaque` entry → **non-empty `trusted_base_delta`** (violating
  zero-delta lawfulness); a **holed** field **fails the kernel re-check**; a
  **missing** field leaves the record value **uninhabited** (`Ord K` cannot be
  constructed). Assert the **observable**: (a) empty delta / law props ∉
  `trusted_base()`; (b) non-empty delta or re-check/inhabitation failure —
  **not** a message string.
- why: (soundness) AC3 — the whole point of a *lawful* class. **The postulated
  arm is the discriminating green-vs-green:** a postulated-law instance
  **type-checks** (the `Opaque` postulate inhabits the field), so a test that
  merely asserts "an `Ord K` **resolves**" passes it vacuously
  ([[conformance-hand-feeds-the-deliverable]]) — yet the "carries the
  total-order laws" guarantee is **assumed, not proved** (the prop rides
  `trusted_base()`). The net is **structural**: the two instances share the same
  `leq`, so the flip is **solely** on the law fields' provenance — **grep the
  law fields for `declare_postulate`/holes; their *absence* is the guarantee**
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]). **Verdict-flip:**
  lawful-accepts vs unlawful-rejects on the same operation content — opposite
  observables, not both-accept. The predicate-definedness dual
  ([[lawful-class-instances-must-carry-law-proofs]]; the ES2 analog is
  `isSorted`/`Perm` **defined** not postulated). Does **not** re-pin the
  `trusted_base()` mechanism (ES1/Sec4) — pins that a class instance's law
  fields are subject to it. **Carrier precondition:** the postulate is a
  **defect here because `K` is inductive** (the laws *were* provable) — the same
  postulated field on a **primitive** carrier is the **honest audited-delta**
  (`primitive-carrier-declared-audited-delta` below), so the reject arm is
  conditioned on carrier provability, not on the postulate alone. **K4 landed
  (`3be0e30`):** the accept arm's **live-`Eq`-conclusion** laws
  (`refl`/`trans`/`total`) and the **postulate-is-a-defect** verdict over them
  are **realizable** now — an inductive carrier proves those ∀-laws by finite
  case-split via Ω-motive elimination (`14 §3`), so a postulate on `K` *is* an
  avoidable delta **today**. The **complete** accept arm — the
  **concrete-equality-conclusion** laws **`antisym`** (and `DecEq`'s
  `sound`/`complete`), whose per-branch obligation reduces to a concrete
  `Top`/`Bottom` — proved via the **K5 + K7** capability (K5 `1c84a30`; K7's
  `eq_at_inductive` operand-`whnf` fix, `4ae2baf`, `16 §8.1`) and now a **real
  zero-delta proof on main** (`9a82745`, `§6`, `16 §1.4`). The real
  proof-carrying instances are **on main** (the live-`Eq` fragment via the
  ES4-lawproofs build `72e38a5`, the complete arm via the remainder `9a82745`);
  the **holed** / **missing** / declared-vs-hidden arms were always live.

### stdlib/classes/absurd-subterm-postulate-counted-in-delta (soundness)
- spec: `51 §5`/`§6` (zero-delta lawfulness), `25 §3` (`trusted_base_delta`
  dependency-cone walk), `16 §1.4` (`absurd` / `Bottom`-elim), K5 seed
  `../../kernel/observational/seed-k5-omega-fragment.md`
  (`sct-rejects-recursion-through-absurd`, the SCT sibling)
- given: a **structural** pair — two definitions **identical** except one's
  `Absurd` subterm (motive **or** proof position) references a
  `declare_postulate`d constant `q` reachable **only** through that `absurd`:
  (a) `q` referenced inside the `absurd`; (b) a clean `absurd` referencing no
  postulate. The vehicle is the `absurd`-carrying **term** as walked by the
  delta accounting — **not** a full `antisym`/`sound`/`complete` instance:
  `trusted_base_delta` does **not** type-check the body (a producer property of
  `collect_consts_in_tb`, `63f3050`), so the injected `Absurd` need not be
  well-typed, and the net is **independent of the K7 operand-`whnf` gate** a
  real operation-wrapped law proof waits on
- expect: **the `trusted_base_delta` flips.** (a) **non-empty delta** — `q` is
  **counted** (the cone walk recurses into **both** `absurd` subterms), so the
  instance is **not** zero-delta (its lawfulness rides a hidden trusted const);
  (b) **empty delta**. Assert the **observable**: `q ∈` vs `∉` the computed
  `trusted_base_delta` — **not** a message string
- why: (soundness) the **elaborator-accounting sibling** of the K5 seed's
  kernel-SCT `sct-rejects-recursion-through-absurd` — the **same** `absurd`
  subterm position, the **trusted-const traversal** one over from the
  **termination traversal**. `collect_consts_in_tb`
  (`ken-elaborator/src/foreign.rs`) is the `trusted_base_delta` dependency-cone
  walker; a **no-op `Absurd` arm** would **silently exclude** a postulate
  laundered through an `absurd` subterm → **TCB undercount** → a law-less
  instance falsely reads zero-delta — a verification-soundness hole the kernel
  does **not** catch (the layer *omits* the const, it does not *supply* a bogus
  one; [[untrusted-layer-backstop-hole-for-omissions]]). **Structural,
  non-degenerate pair** on delta membership (`q ∈`/`∉`), keyed on the
  postulate's reachability-through-`absurd`, not a self-reported string. **Live
  now (K7-independent):** the `Absurd` traversal is exercised at the
  **structural** level — `collect_consts_in_tb` walks `Term::Absurd`'s motive
  and proof subterms over a term the delta accounting never type-checks (a
  `collect_consts_in_tb` property, `63f3050`), so the net fires on the landed
  kernel regardless of whether the operation-wrapped `Eq` collapses (K7). It
  does **not** wait on the complete `antisym`/`sound`/`complete` instances; now
  that K7 (`4ae2baf`) and the wiring (`9a82745`) have landed, the **same** net
  **additionally** covers their real `absurd` terms — the anticipated forward
  strengthening, never a gate. Producer: `collect_consts_in_tb`; reference tests
  `ken-elaborator/tests/k5_absurd_trusted_base.rs` (`63f3050`, producer
  flip-verified). Does **not** re-pin the `trusted_base_delta` mechanism
  (ES1/Sec4) — pins that its cone walk **traverses the `Absurd` variant**.

### stdlib/classes/primitive-carrier-declared-audited-delta (soundness)
- spec: `51 §6` (the carrier axis — a primitive carrier is **audited-delta**,
  not zero-delta), `25 §3` (`trusted_base_delta`), `30 §6` F2 (audited primitive
  ops), `../../security/trust-model/` (Sec4 TCB)
- given: the **same** postulated `total` law field, on two carriers: (a) an
  **inductive** carrier (`Bool` / user `data`) whose `total` is **postulated**
  though it is **provable** by finite case-split; (b) a **primitive** carrier
  (`Ord Int`: `int_leq` fires on literals, **opaque to δ on a variable**; `Int`
  has **no induction principle**) whose `total` is **postulated because it is
  unprovable**, and **declared** in `trusted_base_delta` (the package manifest
  names it)
- expect: **the verdict flips on carrier provability.** (a) **rejected as a
  lawful (zero-delta) entry** — the law *was* provable, so the postulate is an
  **avoidable hidden delta**, a defect; (b) **accepted as an audited-delta
  lawful entry** — `Int` **cannot** prove the ∀-law (no eliminator), so a
  **declared** postulate is the **honest** posture (the same trusted-by-audit
  surface as the primitive op `int_leq` itself), the delta **visible** in
  `trusted_base_delta`. **Sub-net:** a **hidden/undeclared** primitive delta (a
  primitive instance **mislabeled zero-delta**) is **rejected** — the honesty is
  in the **declaration**, not the mere presence of a delta
- why: (soundness) the **carrier axis** the `§6` erratum makes load-bearing.
  Zero-delta lawfulness needs **two orthogonal preconditions**: the law's
  **sort** (Ω-clean — `ord-total-law-is-omega-bool-equation`) **and** the
  carrier's **provability** (inductive / has an eliminator). So the **same**
  postulated `total` is a **defect on `Bool`/user-`data`** (provable) yet the
  **honest audited-delta on `Int`** (unprovable) — the reject is keyed on
  *carrier provability*, not the postulate alone. Ken **ships**
  `Ord Int`/`Eq Int` (you cannot simply lack them, `§6`), so this path is
  **real**, not hypothetical; its trust posture is a primitive op's
  ([[tested-not-trusted-posture-needs-reachability-precondition]]). **Ties to
  Sec4 TCB accounting:** the law postulates are `Opaque`, the op `Primitive` —
  **both** legitimately in `trusted_base()` (`../../security/trust-model/`); the
  audited delta is honest iff **declared**, an over-claim iff **hidden**
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]). Static face; the
  build lands the real instances + their manifest delta declarations. **K4
  landed (`3be0e30`):** arm (a)'s reject (an inductive postulate = defect) is
  **realizable** now — this case is keyed on **`total`**, a
  **live-`Eq`-conclusion** law (the Bool-equation `IsTrue (leq x y || leq y x)`)
  provable on K4 with **no K5 needed**, so `Bool`/user `data` prove it via
  Ω-motive elimination and the axis **separates** (inductive zero-delta vs
  primitive audited-delta) **today** — this is the live adjacent net; the
  `antisym`/`sound`/`complete` arms are now **real zero-delta proofs on main**
  (`9a82745`, via the K5 + K7 capability — K5 `1c84a30`, K7 `4ae2baf`). The
  **declared-vs-hidden** sub-net was always capability-independent — it is what
  keeps the primitive audited-delta posture enforceable regardless.

### stdlib/classes/ord-total-law-is-omega-bool-equation (soundness)
- spec: `51 §3` (law-field sorts; totality is the Bool-equation), `16 §1`/`§6`
  (Ω / truncation), `13 §4` (`sort_sigma`)
- given: the `Ord a` `total` law field, in two formulations: (a) the **landed**
  Bool-equation `total : (x y) → IsTrue (leq x y || leq y x)`; (b) a **bare
  propositional disjunction**
  `total : (x y) → IsTrue (leq x y) ∨ IsTrue (leq y x)`
- expect: (a) is an **admissible `Ω` law field** —
  `IsTrue (leq x y || leq y x) = Eq Bool (leq x y || leq y x) True : Ω`,
  proof-irrelevant, **no truncation**; (b) is **not** an admissible law field as
  written — a bare `∨` is **proof-relevant** (`Type`-valued: *which* side holds
  is content), so it either makes the field relevant (the record carries extra
  content the law must not) or requires the truncation `‖·‖` (`16 §6`) to reach
  `Ω`. Assert that `total`'s field type is the **Ω Bool-equation**, not a
  `Type`-valued sum
- why: (soundness) the law-field-sort no-regression net — a law must be
  **proof-irrelevant** (`Ω`), and totality is the one law that is a *disjunction
  in spirit*. The **decidable `Bool` `leq` sidesteps** the truncation
  obligation: totality becomes a Bool-equation, `Ω`-clean. A build that
  reformulates `total` as a bare propositional `∨` reintroduces proof-relevance
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]] — the ES1-`Perm`
  analog) — either leaking content into the "law" or silently needing
  truncation. This pins the corrected form forward (the same no-regression
  discipline as a trust-level erratum), so a later `Ord`-variant can't quietly
  regress the sort.

---

## AC-transport — `Ord Char` carries its laws via transport, not a stub
(re-homed from the Decimal/Char DEMOTE, `docs/program/wp/lawful-classes-lane.md`)

`51 §6`'s carrier axis, read for a **derived/refinement** carrier: `Char = {c :
Int | isScalar c}` erases to `Int` (`21 §6.3`), a canonical one-value-per-
codepoint carrier, so `Ord Char`'s laws **are** `Ord Int`'s laws — the sound
realization is **honest transport** (referencing `Ord Int`'s existing fields),
not a fresh proof and not a fresh postulate. **The discriminator here is
HONESTY, not zero-delta**: a transported instance is real and complete: no
field is fabricated or silently absent.

### stdlib/classes/char-ord-laws-carried-not-stubbed (soundness)
- spec: `51 §6` (the carrier axis, refinement/derived reading), `21 §6.3`
  (refinement erasure), `33 §5.2`/`§5.3` (instance = record value, `.`-field
  projection eta), `docs/program/wp/lawful-classes-lane.md`
- given: two `Ord Char`-shaped instances, identical in their `leq` field
  (both transport/wrap the same landed `leq_int` reduction path): (a) the
  **canonical, honest-transport** instance — `leq`/`refl`/`antisym`/`trans`/
  `total` all reference `Ord Int`'s own existing fields via `.`-projection
  (`(Ord_instance_Int).refl` etc, `catalog/packages/Core/Classes/LawfulClasses.ken.md`)
  — every field is present and is a real, kernel-checked term (a projection
  chain that reduces, exactly as honest as `Ord Int`'s own visible `Axiom`s);
  (b) a **deceptive/incomplete** instance attempt with a law field **omitted**
  from the `instance { … }` body entirely (not even an honest `Axiom` written)
- expect: **the verdict flips.** (a) **accepts** — the instance elaborates,
  every field position is filled by a real term (grep-confirm: no
  `declare_postulate`/hole introduced by `Ord_instance_Char`'s OWN decl —
  `trusted_base_delta` on `Ord_instance_Char` is empty, i.e. zero-**NEW**-delta;
  it correctly does **not** assert zero-delta outright, since `Ord Int`'s
  pre-existing `Axiom`s are still honestly there, one projection-hop away);
  (b) **rejected** — the record value is **uninhabited**: the class
  machinery requires every declared field present
  (`compute_ordered_field_values`, `ken-elaborator/src/elab.rs`) and errors
  rather than silently defaulting a missing field to some placeholder.
  Assert the **observable**: (a) elaborates + empty `Ord_instance_Char`-own
  `trusted_base_delta`; (b) elaboration fails (an `Internal`/missing-field
  error), not a message string
- why: (soundness) the **honesty-not-zero-delta** discriminator (`51 §6`
  erratum) applied to a **derived/refinement** carrier for the first time:
  Char is neither a bare inductive (case-split provable, the `Bool`
  exemplar) nor an unprovable primitive floor in its own right (the
  `Int` exemplar) — it is a **canonical alias** of one, so the sound
  realization is transport, and the discriminator is simply "is every field
  a real, present term" (never "is the whole instance zero-delta outright").
  A build that silently substitutes a hole, a fabricated proof of a false
  statement (the `DecEq Decimal` shape this WP's own build ruled out —
  `decimalEq`'s cross-representation equality is **not** `Decimal`'s
  structural `Equal`, so postulating `sound`/`complete` there would inhabit
  `Bottom` — that carrier is **not** canonical, hence **not** eligible for
  this same transport move; not delivered here) or an omitted field would
  either fail to kernel-check or fail to construct — this case pins that the
  **omitted-field** rejection path is real and enforced, alongside the
  accept arm's zero-NEW-delta observable. Complements
  [[lawful-class-instances-must-carry-law-proofs]] (predicate-definedness
  dual) for the transport case specifically, and records why `Ord Char`
  is sound where a same-shaped `DecEq`/`Num Decimal` transport is **not**
  (carrier canonicity is the load-bearing precondition, not the class
  mechanism).

---

## AC2 — `Ord` subsumes the explicit comparator (reflect-don't-extend)

`51 §4`: `where Ord a` supplies the **same** `leq` the explicit `sort` threads
(`d.leq`, `33 §5.4`) — same view, no second `sort`.

### stdlib/classes/where-ord-same-sort-obligation (AC2)
- spec: `51 §4` (`Ord` subsumes the comparator), `33 §5.4` (`where C a` →
  implicit dict + projection), `37 §6`/`34 §5` (the `sort` obligation)
- given: the verified `sort` invoked two ways on the same `List a`: (a) the
  **explicit** form `sort leq xs` (ES2-remainder surface, `37 §6`); (b) the
  **constrained** form `sort xs where Ord a`, where `where Ord a` desugars to an
  implicit `{d : Ord a}` and the body's comparator is `d.leq` (`33 §5.4`)
- expect: **both emit the structurally identical refinement obligation**
  `isSorted (leq) ys ∧ Perm ys xs` (`34 §5`, with (b)'s `leq` ≡ `d.leq`) — the
  **same view**, one `sort`, the comparator either passed or
  dictionary-supplied. There is **no second `sort`** and **no
  distinct/duplicated obligation**
- why: (AC2) reflect-don't-extend — `where Ord a` is ordinary
  implicit-dictionary insertion supplying the comparator, **not** new mechanism.
  **Discriminating:** a build that introduces a **second `Ord`-sort** (a
  distinct view whose obligation differs from, or duplicates, the explicit
  form's) **fails** — the emitted VC must be the **same shape** as the explicit
  form's (modulo `leq = d.leq`). Assert the **observable**: the two emitted
  obligations are the **same structural shape**, not "both type-check".
  References the base `sort-emits-issorted-and-perm` (`../surface/collections/`)
  — the `Ord`-supplied form emits **that same** conjoined obligation, re-pinning
  nothing. (The named-non-canonical escape hatch `sortBy byLength xs`,
  `33 §5.5`, still passes a non-canonical `Ord` value explicitly — canonical
  resolution unperturbed.)

---

## Coverage map

- **AC3** (laws PROVED, soundness): `law-fields-real-proofs-not-postulates`
  (live-`Eq`-conclusion accept arm + postulate-defect **realizable** since K4;
  the **complete** accept arm — `antisym`/`sound`/`complete` → `Top`/`Bottom` —
  are **real zero-delta on main** (`9a82745`, K5 + K7 landed); real instances =
  ES4-lawproofs build + remainder), `absurd-subterm-postulate-counted-in-delta`
  (the `absurd` subterm is traversed by the `trusted_base_delta` cone walk — the
  elaborator-accounting sibling of the kernel-SCT launder net),
  `primitive-carrier-declared-audited-delta` (carrier separation live on the
  live-`Eq` law `total`; declared-vs-hidden always capability-independent),
  `ord-total-law-is-omega-bool-equation`.
- **AC-transport** (honesty-not-zero-delta, a derived/refinement carrier):
  `char-ord-laws-carried-not-stubbed` — `Ord Char` by transport from
  `Ord Int` (Char's canonical, refinement-erased carrier), landed
  (`catalog/packages/Core/Classes/LawfulClasses.ken.md`, WP lawful-classes-lane).
  **`Num`/`DecEq Decimal` re-defer** (Steward ruling): `Decimal`'s
  non-canonical `(coeff, exp)` carrier makes `decimalEq` an `Eq`
  (equivalence), not a `DecEq` (decision procedure for the kernel's
  structural `Equal`) — postulating `sound`/`complete` there would inhabit
  `Bottom` (caught before landing, not a proof-difficulty gap); both sit
  behind a single decide-once design gate (canonicalize the carrier, or a
  setoid/quotient `Eq Decimal`), tracked in `90-open-decisions.md`, not
  delivered by this WP.
- **AC2** (`Ord` subsumes the comparator): `where-ord-same-sort-obligation`.
- **AC5** (un-defer): the two `../surface/collections/` cases
  (`user-ord-instance-drives-verified-sort`,
  `user-ord-sort-emits-both-conjuncts`) are re-pointed to the real `Ord` in
  `51`; they **un-defer on the build**, not here (edit made in
  `seed-collections.md`).

## Cross-case consistency sweep

- **Lawful ≡ zero-delta on an inductive carrier; audited-delta on a primitive
  one.** `law-fields-real-proofs-not-postulates` (an inductive carrier's
  postulated law → non-empty delta → **defect**) and the ES1
  `../surface/taxonomy/minimality.md` zero-delta invariant **agree**: on a
  **provable** (inductive) carrier nothing enters `trusted_base()` by the back
  door — a law proved-not-postulated is the law-side of zero-delta.
  `primitive-carrier-declared-audited-delta` completes it on the **carrier
  axis**: a primitive carrier's ∀-laws are **unprovable**, so its instance
  carries a **declared** audited delta (honest), not zero. The unifying rule: a
  delta is a **defect iff avoidable** (the carrier could have proved it) and an
  **honest audited-delta iff declared** (unprovable **and** visible). A case
  treating a primitive audited-delta as a defect (over-strict — you cannot lack
  `Ord Int`), an inductive-carrier postulate as lawful (under-strict — hides an
  avoidable delta), or a **hidden** primitive delta as honest, contradicts this
  class. **K4 + K5 + K7 landed (`3be0e30`, `1c84a30`, `4ae2baf`):** the
  **avoidability** half (hence the carrier *separation* and
  inductive-postulate-defect verdict) is **realizable** now for the
  **live-`Eq`-conclusion** laws (`refl`/`trans`/`total`) — an inductive carrier
  proves those via Ω-motive elimination, so the axis **separates today**; the
  **concrete-equality-conclusion** laws (`antisym`/`sound`/`complete` →
  `Top`/`Bottom`) are now **real zero-delta proofs on main** (`9a82745`, via the
  K5 + K7 capability) for a *complete* instance. The **declaredness** half
  (honest iff declared) always was capability-independent. The proof-carrying
  instances are **on main** (Team Language; the live-`Eq` fragment via the
  ES4-lawproofs build `72e38a5`, the complete arm via the remainder `9a82745`).
- **The `Absurd` subterm is traversed by *both* structural walkers.**
  `absurd-subterm-postulate-counted-in-delta` (the elaborator's
  `collect_consts_in_tb` trust-cone walk) and the K5 seed's
  `sct-rejects-recursion-through-absurd` (the kernel's `collect_calls`
  termination walk) agree: a def/proof whose only reference to a group member —
  a recursive call, or a trusted postulate — sits **inside** an `absurd` subterm
  must still be **seen** by the walker. A no-op arm in either is the same blind
  spot one position apart: a soundness hole (non-termination) or an honesty hole
  (TCB undercount). A case where either walker skips the `absurd` position
  contradicts this pair.
- **A law field is `Ω` (proof-irrelevant).**
  `ord-total-law-is-omega-bool-equation` and the `51 §3` law-field-sort pin
  agree: every law field lands in `Ω` (the record is a `Type`-sorted structure
  class **because** of its op fields, never **because** a law leaked to `Type`).
  A case with a `Type`-valued "law" (a proof-relevant `∨` untruncated) would
  contradict the structure-class sort discipline.
- **`Ord`-supplied and explicit `sort` are one view.**
  `where-ord-same-sort-obligation` and the base `sort-emits-issorted-and-perm`
  agree: the emitted obligation is identical whether `leq` is passed or supplied
  by the dictionary. A case where the `where Ord a` form emits a **different**
  obligation (a second mechanism) would contradict reflect-don't-extend.

## Subsumed / not-duplicated (one home per property)

- **The `trusted_base()` / zero-delta mechanism** is **ES1/Sec4's**
  (`../surface/taxonomy/minimality.md`, `../../security/trust-model/`). AC3
  **reads it from the law side** (a postulated law → non-empty delta); it does
  **not** re-pin the delta computation.
- **The verified-`sort` refinement obligation** (`isSorted leq ys ∧ Perm ys xs`,
  the `Perm`-conjunct-present net) is **`../surface/collections/`'s**
  (`sort-emits-issorted-and-perm`). AC2 references it (the `Ord`-supplied form
  emits the same obligation); it does **not** re-pin the emission completeness.
- **The two `user-ord-*-sort` user-instance cases** are
  **`../surface/collections/`'s** (`user-ord-instance-drives-verified-sort`,
  `user-ord-sort-emits-both-conjuncts`), currently `(deferred)`. This WP is
  their un-defer gate (AC5): the edit re-points them to the real `Ord` in `51`;
  the actual un-defer (making them live) rides the build. Not re-homed here.
- **The class mechanism** (record elaboration, `sort_sigma`, `instance_search`,
  `where`-desugaring) is **`33 §5`'s** (surface). This seed pins the **lawful
  content** (the law proofs are real), not the resolution machinery.
