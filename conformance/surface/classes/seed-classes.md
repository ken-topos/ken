# Typeclasses/constraints conformance — seed cases (Lc)

Format: `../../README.md`. These pin **typeclasses as subobjects of the
universe** (`spec/30-surface/33 §5` + the resolution algorithm `39 §6`,
impl-ready Lc): a `class` elaborates to a **record** (a Σ of operations *and*
their law propositions), an `instance` to a **record value** (including the law
proofs), and a constraint `where C A` to an **implicit instance argument** the
elaborator discharges by **instance search**. Coherence is **ADR 0008**. **No
new kernel feature** — a class is a `13 §3` Σ+η record, a law is a `16 §1` Ω
proposition, search termination is the **landed** `17 §4` SCT on a reified
dictionary group. These cases net the **eight discriminating ACs**, each a
non-degenerate pair/flip on a **structural** discriminator.

Grounding (landed `§`-bodies + landed code on `origin/main`, content-reconciled
— not the frame): `33 §5.1`–`§5.7` (the two kinds, the sort-is-the-discriminant,
class→record, instance→value+orphan, coherence policy, named-instance escape,
derive); `39 §6.1`–`§6.8` (instance env, search, sub-constraints, SCT
termination + reification-faithfulness, named passing, derive, diagnostics);
`13 §2`/`§3`/`§4` (Σ-Intro/record/η + the **both-components-keyed**
`sort_sigma`); `16 §1` (Ω proof-irrelevance); `17 §4.2` (SCT structural subterm
order); `21 §3` (`law`/`verify` proof obligations); `25` (T1 diagnostics).
**Landed code the soundness-critical ACs bottom out in:** `sort_sigma`
(`check.rs:192`, **both-components-keyed**, regression-guarded by
`ken-kernel/tests/sigma_sort.rs` — the Σ-sort erratum); `sct_check`
(`sct.rs:414`) run on a group by `declare_recursive_group` (`check.rs:983`,
`sct_check` at `:1033`) and on a single def by `declare_def` (`check.rs:944`,
`sct_check` at `:962`, guard-tested by `declare_def_sct_rejects_self_loop`); the
record encoding `Term::Sigma`/`Pair`/`Proj` (`13 §2`); Ω-PI (`16 §1`). **Net-new
(the Lc build creates it):** the class/instance desugaring, the orphan + overlap
checks, the constraint insertion, and the search.

## Reading these cases — the Lc-specific disciplines

**Two trust faces — kernel-backed ACs vs elaborator-convention ACs (the
producer-grep split).** The soundness-critical behaviors bottom out in **landed
kernel producers** the kernel re-checks: the property-vs-structure discriminant
is the real `sort_sigma` (AC4), search termination is the real `sct_check` on
the reified group (AC6), every instance + every `derive`d candidate rides the
real `declare_def` re-check (AC7), a law field is a real Σ-Intro-checked proof
(AC8). The coherence **convention** — canonicity, orphan, overlap, ambiguity,
explicit-passing (AC1/AC2/AC3/AC5) — is an **elaborator** gate the **kernel does
not backstop** (it constrains *where* a well-typed value may be declared, not
well-typedness): so for those, **conformance is the sole net** (the
erased-before-kernel posture, as with the security seams' N1) and the cases
assert the convention **structurally**. Every case drives the **real** producer
(the net-new elaborator path the Lc build creates, or the landed kernel
producer) — never a synthetic "class" literal or a trusted `env`-insert.

**The sort IS the discriminant — never an author flag (`33 §5.1`, AC4,
soundness).** A class elaborates to a right-nested Σ; the kernel's `sort_sigma`
is **both-components-keyed** — `Ω` **iff every** field is Ω (a **property**
class, coherence-free by Ω-PI), else `Type` the moment **any** field is relevant
(a **structure** class, resolver convention). Forcing a structure class into Ω
to "get coherence free" fires **Ω-PI** on the whole record and makes its
computational content proof-irrelevant — two observationally-distinct
dictionaries become definitionally equal, collapsing the content the prover's
lemmas depend on. **This is the Σ-sort trap** (`13 §4`); the both-components-
keyed `sort_sigma` is what prevents it, so AC4's discriminator is the **real
kernel sort computed over all fields** — a mis-keyed (codomain-only) sort is the
soundness bug (the erratum `sigma_sort.rs` already guards at the kernel; AC4 is
its surface-class image).

**Reification faithfulness is a trusted step — the sole net for AC6's soundness
(`39 §6.4` NORMATIVE, N2-class).** `sct_check` re-checks the group it is
**handed**; it does **not** check that the reified group **faithfully mirrors**
the resolution recursion. So the reification is the **trusted** step: a group
that **dropped an edge** or **keyed a node** on anything but the resolution
sub-goal makes `sct_check` bound the **wrong** recursion — a non-terminating
search could slip an **accepting** SCT verdict (an omission the kernel cannot
catch). AC6 therefore pins **both** faces: the reject reaches `sct_check` on the
**real reified group** (grep the producer —
`declare_recursive_group`/`sct_check`, not an elaborator-side proxy that could
diverge), **and** the group is the **exact image** of the resolution graph (one
node per distinct sub-goal, one edge per `dischargeSubConstraints` call,
head-type per node).

**Discriminating pairs — each AC must flip on a structural discriminator (not
green-vs-green).** Every case pins the two states that *should* bucket
differently on a shared input, keyed on a structural fact (the class record's
kernel sort, `(class, head-type)` key membership, `trusted_base()` membership,
the SCT verdict on the real group), never a self-reported string. A single
positive case is vacuous under the silent swap.

**The hand-feed net (AC7 derive, `39 §6.6`).** The `derive` case must **drive
the generator → the real `declare_def`** and observe the kernel's verdict; a
test that **inserts a ready-made dictionary** and re-checks a downstream
consumer is **green-vs-green** (it re-tests the consumer, not derivation). Grep
the derive path emits its candidate through `declare_def`/kernel-check, never a
trusted `env`-insert.

**Diagnostics are T1-protocol with spans (`39 §6.7`, `25`).** The failure cases
name the exact diagnostic (`OrphanInstance` / `OverlappingInstances` /
`AmbiguousInstance` / `NoInstance` / `NonTerminatingInstances`) and its carried
spans — never a silent pick, never a hang.

**Tags.** `(soundness)` = a real trust commitment that must never regress — the
sort discriminant (AC4), the SCT-bounded termination + reification faithfulness
(AC6), the derive-is-kernel-re-checked (AC7). `(verification)` = the lawful
instance is a genuine prover-citable proof (AC8). Elaborator-convention cases
(AC1/2/3/5) are netted **solely** by conformance (no kernel backstop).

## CL-A. Coherence — same key, same canonical (AC1)

### classes/same-class-head-type-resolves-same-canonical (AC1)
- spec: `39 §6.2` (coherence), `33 §5.5`, ADR 0008
- given: one registered structure instance `Ord Int`; a program that resolves an
  implicit `where Ord Int` at **two distinct sites** (`39 §6.2` `resolve`)
- expect: **both sites resolve to the same canonical instance** — the identical
  registered dictionary (same `GlobalId`/record `Term`), because the search key
  is `(class C, head(A))` and the registry holds **one** entry per key
- why: AC1 — coherence: "the `Ord Int`" is a **function of the head-type**,
  stable program-wide (the property the law-carrying prover relies on).
  Structural discriminator: the two resolved dictionary `Term`s are
  **identical**. The bug — resolving by import-order / by-site rather than by
  `(class, head-type)` — yields **different** dictionaries at the two sites →
  the identity assertion fails. Sole net (elaborator convention; kernel does not
  enforce canonicity).

## CL-B. Orphan check — the declaration-locus pair (AC2)

### classes/instance-with-class-or-head-accepted-orphan-rejected (AC2)
- spec: `33 §5.3`, `39 §6.1`/`§6.7`
- given: two otherwise-identical `instance C T` declarations differing **only in
  declaration locus** — (a) declared in the module of its class `C` **or** its
  head-type `T`'s constructor; (b) an **orphan** declared in a module naming
  **neither**
- expect: (a) **accepted** (registers a canonical entry); (b) **rejected at the
  declaration site** with `OrphanInstance` (carrying the decl span + class +
  head-type) — the syntactic, per-module predicate of `33 §5.3`
- why: AC2 — the orphan check is the structural precondition that keeps
  canonicity per-module-decidable + un-break-able by accident. Non-degenerate
  pair: the only difference is the locus; the verdict **flips** accept↔reject.
  Sole net (elaborator gate, not a kernel rule).

## CL-C. Overlap / ambiguity — never a silent pick (AC3)

### classes/single-canonical-resolves-two-overlapping-error-naming-both (AC3)
- spec: `39 §6.1` (overlap at registration), `39 §6.2`/`§6.7` (ambiguity at
  search), `33 §5.5`
- given: (a) a **single** canonical structure instance for `(C, h)`; (b) a
  **second** instance registering under the **same** `(C, h)` structure key
- expect: (a) **resolves** to the one canonical dictionary; (b) a **compile
  error naming both** candidates — `OverlappingInstances` at registration (both
  decl spans) or `AmbiguousInstance` at search (the use-site span + **all**
  candidate spans) — **never a silent pick**
- why: AC3 — the anti-guessing commitment (`39 §3`, ADR 0008). Structural
  discriminator: the count of registered entries under `(C, h)`. The verdict
  **flips** resolve↔error, and the error **enumerates all** candidates (a silent
  default is the bug). Sole net (elaborator convention).

## CL-D. Property vs structure — the sort discriminant (AC4, soundness)

### classes/property-class-two-instances-clean-structure-errors (AC4, soundness)
- spec: `33 §5.1` (sort discriminant), `13 §4` (`sort_sigma` both-keyed),
  `16 §1` (Ω-PI), `39 §6.2`
- given: (a) a **property** class — every field Ω-valued (e.g. `Decidable p`) —
  with **two** registered instances on the same head-type; (b) a **structure**
  class — at least one **relevant** operation field (`eq : A → A → Bool`), so
  `Type`-valued — with **two** instances on the same head-type. Drive the real
  class→record desugar → the real `sort_sigma`
- expect: (a) the record sorts **Ω** (every field Ω, `sort_sigma(Ω,Ω)=Ω`) → the
  two instances are **definitionally equal** (Ω-PI) → resolution returns either,
  **no ambiguity** (clean); (b) the record sorts **`Type`** (a relevant field,
  `sort_sigma(Type,_)=Type`) → the two instances are the AC3 **overlap/ambiguity
  error**
- why: (soundness) AC4 — the discriminant is the **kernel-computed sort**, not a
  flag. The verdict **flips** on the sort: property→clean, structure→error. The
  soundness direction: a **codomain-only (mis-keyed) `sort_sigma`** would
  classify the **structure** record as Ω → its two relevant dictionaries
  collapse definitionally (Ω-PI) → the structure case wrongly reads "clean"
  (content collapsed) — the **Σ-sort trap**. The both-components-keyed landed
  `sort_sigma` (`check.rs:192`, `sigma_sort.rs`) is the net; AC4 is its
  surface-class image. Bottoms out in a landed, regression-guarded producer.

## CL-E. Named-instance explicit escape — distinct paths (AC5)

### classes/explicit-named-instance-used-implicit-selects-canonical (AC5)
- spec: `33 §5.5`, `39 §6.5`, `39 §6.2`
- given: a **non-canonical** `byLength : Ord String` (an ordinary record-typed
  `let`/`view` value, distinct from the canonical `Ord String`); one call
  passing it **explicitly** (`f {d = byLength} x` / positional dictionary
  application), and one call at the **same type** using the **implicit**
  `where Ord String`
- expect: the explicit site **binds `d := byLength` directly** (the passed
  value; `resolve` **not** called) **while** the implicit site runs `resolve`
  and selects the **canonical** `Ord String` — the two dictionaries are
  **different**
- why: AC5 — explicit passing is ordinary value application at the dictionary Π;
  it **bypasses search** and **does not perturb** implicit canonicity.
  Structural discriminator: the dictionary `Term` at the explicit site is
  `byLength`; at the implicit site it is the canonical registered entry.
  Non-degenerate (byLength ≠ canonical). The bug — explicit passing perturbing
  the registry, or implicit picking `byLength` — blurs the two paths. Sole net
  (elaborator convention).

## CL-F. Search termination — SCT on the real reified group (AC6, soundness)

### classes/wellfounded-chain-resolves-cyclic-rejected-by-sct (AC6, soundness)
- spec: `39 §6.4` (reify + `sct_check` + **reification faithfulness**),
  `17 §4.2` (structural subterm order), `39 §6.7`
- given: (a) a **well-founded** parameterised chain
  `instance Ord (List a) where Ord a` resolved for `Ord (List Int)` — sub-goal
  `Ord a`'s head-type `a` is a **strict structural subterm** of `List a`; (b) a
  **cyclic / non-decreasing** set (`instance C A where C A`, or `C a ⇒ C (F a)`
  with `F` non-decreasing). Drive the real reification → the real
  `declare_recursive_group`/`sct_check`
- expect: (a) the reified dictionary group **strictly descends / bottoms out** →
  **`sct_check` accepts** → resolves; (b) a node **transitively references
  itself with no head-type decrease** → **`sct_check` rejects at admission** →
  `NonTerminatingInstances` (the offending span + the cycle) — a
  **declaration/admission-time** error, **never a search-time hang**
- why: (soundness) AC6 — termination is the **landed** `sct_check` on the
  reified group (`declare_recursive_group`, `check.rs:983`/`:1033`), not a
  second checker. The verdict **flips** accept↔reject on the `17 §4.2`
  structural-subterm metric; the reject is **guard-gated** by that metric (not a
  coincidental hang). **Plus the reification-faithfulness net (trusted step,
  N2):** the reject must reach `sct_check` on the **real reified group** (grep
  the producer path), and the group must be the **exact image** of the
  resolution graph — one node per distinct sub-goal, one edge per
  `dischargeSubConstraints` call, head-type per node. A **dropped edge /
  mis-keyed node** would make `sct_check` bound the wrong recursion → a
  non-terminating search slips an **accepting** verdict (the omission the kernel
  cannot catch); faithfulness is the **sole net** for that step, asserted
  structurally on the emitted group's shape.

## CL-G. `derive` — a kernel-re-checked candidate (AC7, soundness)

### classes/derive-candidate-kernel-rechecks-malformed-rejected (AC7, soundness)
- spec: `33 §5.6`, `39 §6.6`, `check.rs` `declare_def`
- given: `derive DecEq` for a `data` type — driving the **real generator** → the
  candidate instance **emitted through `declare_def`** — in two states: (a) the
  well-formed structural candidate; (b) a **deliberately-malformed** candidate
  (a wrong op body / an **unprovable** law proof)
- expect: (a) `declare_def` **re-checks and admits** the ops **and** the law
  proofs; (b) the malformed candidate **fails the kernel check** (`declare_def`
  rejects) — **never admitted by a trusted insertion**
- why: (soundness) AC7 — `derive` generation is **untrusted**: the candidate is
  admitted only by the **real `declare_def`** re-check, so `derive` is a
  convenience that **cannot widen the trusted base**. The verdict **flips**
  admit↔reject on the kernel check. **Producer-grep (the hand-feed net):** the
  case must drive the generator → `declare_def`; a test that **inserts a
  ready-made dictionary** + re-checks a downstream consumer is
  **green-vs-green** (re-tests the consumer, not derivation). Bottoms out in
  landed `declare_def`.

## CL-H. Lawful instance — a prover-citable proof (AC8, verification)

### classes/monoid-law-proof-is-real-cited-not-stub (AC8, verification)
- spec: `33 §5.2`/`§5.3` (law fields, Σ-Intro-checked), `13 §2` (`Proj`/η),
  `21 §3`
- given: a `Monoid Int` instance whose `assoc`/`unit` **law fields carry real
  proofs** (each checked at its Σ-Intro position `B[a/x]`, `13 §2`); a prover
  goal that **cites `d.assoc`** (the projected law field)
- expect: `d.assoc` projects (definitional η) to a **genuine kernel proof** of
  the associativity proposition the prover uses as a lemma — **absent** from
  `trusted_base()`, kernel-re-checked at the Σ-Intro position; **not a stub**
- why: (verification) AC8 — the lawful-by-construction dictionary is the
  verification win: the law field is a **real proof**, not a postulate/hole.
  Discriminator (proved↔assumed, the B1/`21 §5.4` axis on the law field): a real
  proof is **absent** from `trusted_base()` and carries a certificate `check`s;
  an instance whose law field is a **hole/postulate** would still typecheck (the
  field type is the proposition) but its "proof" would **ride `trusted_base()`**
  — so the prover's citation would rest on an **assumption**, not a discharge.
  Assert the law proof is the discharged half. Bottoms out in landed
  `Term::Proj` + the `declare_def` re-check of the proof.

## Coverage map

| Case | AC | Spec | Discriminator | Trust face |
|---|---|---|---|---|
| `same-class-head-type-resolves-same-canonical` | AC1 | `39 §6.2` | `(class, head-type)` key → same entry | elaborator (sole net) |
| `instance-with-class-or-head-accepted-orphan-rejected` | AC2 | `33 §5.3` | declaration locus (per-module) | elaborator (sole net) |
| `single-canonical-resolves-two-overlapping-error-naming-both` | AC3 | `39 §6.1` | count of entries under `(C, h)` | elaborator (sole net) |
| `property-class-two-instances-clean-structure-errors` | AC4 | `33 §5.1` | kernel `sort_sigma` (both-keyed) | kernel-backed (landed) |
| `explicit-named-instance-used-implicit-selects-canonical` | AC5 | `39 §6.5` | explicit (value app) vs implicit (search) | elaborator (sole net) |
| `wellfounded-chain-resolves-cyclic-rejected-by-sct` | AC6 | `39 §6.4` | `sct_check` on the real reified group + faithfulness | kernel-backed + trusted reification |
| `derive-candidate-kernel-rechecks-malformed-rejected` | AC7 | `33 §5.6` | real `declare_def` re-check | kernel-backed (landed) |
| `monoid-law-proof-is-real-cited-not-stub` | AC8 | `33 §5.3` | `trusted_base()` membership of the law field | kernel-backed (landed) |

## Cross-case sweep (mechanism consistency)

The **sort discriminant** governs two mechanisms that must agree: it classifies
the class (AC4) **and** it is the search branch (`39 §6.2` — Ω→property/any,
Type→structure/canonical), so AC1/AC3 (structure: canonical, overlap-errors) and
AC4-property (any instance, never conflicts) are the **same** switch read at
classification and at search — a case that let a property class error on two
instances, or a structure class silently pick, would contradict the sort split.
The **coherence key `(class, head-type)`** is consistent across AC1 (same key →
same entry), AC2 (orphan = the precondition making the key
per-module-decidable), and AC3 (a second entry under the key = overlap). The
**landed trust root** is consistent across the kernel-backed ACs: AC4 rests on
`sort_sigma`, AC6 on `sct_check`, AC7 on `declare_def`, AC8 on `Term::Proj` +
`declare_def` — none introduces a kernel rule (`33 §5.7`/`39 §6.8`). Two
boundary facts pinned: the elaborator-convention ACs (AC1/2/3/5) have **no
kernel backstop** (conformance is the sole net); the reification step (AC6) is
**trusted** (faithfulness is the sole net for it, the kernel bounds only the
group it is handed).

## Subsumed, not duplicated

These cases **compose** landed producers; they do not re-net upstream meaning:
the **both-keyed `sort_sigma`** itself is `ken-kernel/tests/sigma_sort.rs` (the
Σ-sort erratum, `13 §4`) — AC4 is its surface-class image, not a re-test of the
kernel sort; the **`sct_check` size-change bound** is the kernel SCT suite
(`17 §4`) — AC6 nets the *reification* + the *admission-time reject*, not the
SCT algorithm; **`declare_def`** re-check is K1 — AC7/AC8 net the *derive
candidate* / the *law-proof-is-real*, not `declare_def` itself; **Ω-PI** is the
observational seed (`16 §1`) — AC4-property rides it. Lc nets only the
**class/instance/search/ derive** logic and the **coherence convention** — the
net-new surface, over an unchanged landed trust root.

## Build-sequencing note (Team Language — net-new logic, landed trust root)

Lc adds **no** kernel rule, judgment, or "class" former (`33 §5.7`/`39 §6.8`);
the build creates the **net-new** class/instance desugaring, the orphan +
overlap checks, the constraint insertion, and the search, and **reuses** the
landed kernel producers the soundness-critical ACs bottom out in (`sort_sigma`,
`sct_check` via `declare_recursive_group`, `declare_def`, `Term::Sigma`/`Pair`/
`Proj`, Ω-PI). The producer-grep QA gate greps the **real** elaborator/kernel
path — the class desugar feeding the real `sort_sigma` (AC4), the reified group
admitted through the real `declare_recursive_group`/`sct_check` (AC6), the
derive candidate through the real `declare_def` (AC7) — never a synthetic
"class" literal or a trusted `env`-insert. Unblocks L3b (`Map`/`Set` need
`DecEq`/`Ord`), L8 stdlib, `Ord`-sorting, and prover-facing lawful instances.
