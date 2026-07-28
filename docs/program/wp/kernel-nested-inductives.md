# WP `KERNEL-NESTED-IND` — nested strictly-positive inductives in the kernel

> ## ▶ READ THIS FIRST
>
> **Node:** [`docs/program/issues/KERNEL-NESTED-IND.md`][n] — carries the
> five-point contract and `AC-K1`–`AC-K12`. This frame is the executable
> artifact: measured substrate, slicing, controls, validation.
>
> **Sequence:** `SPEC-NESTED-IND` → **`KERNEL-NESTED-IND`** → `DS-9`.
> ✅ **`SPEC-NESTED-IND` MERGED 2026-07-27 and the node is `active`** — `D1a` has
> landed and the `D1b` gate is lifted. The governing chapter is
> `spec/10-kernel/14-inductive.md`; ⛔ re-derive its blob at point of use.
> ⚠ **This banner previously said the node was `draft` pending that merge** —
> edited, not annotated. ⛔ **Un-gating `D1b` did NOT reorder the work:** §4 puts
> `D3a`, then the atomic `D3b`+`D4`, ahead of it.
>
> ⚠ **This node changes the TCB.** `docs/PRINCIPLES.md`, small auditable trusted
> base. `K1p5-wstyle-inductives.md` is the direct precedent — the last time this
> exact surface was widened — and it was rated ★★★ for the same reason.
>
> ⭐ **What is NOT in doubt:** the current rejection is *sound*. Architect,
> verbatim: *"a safe, deliberate completeness/staging boundary, **not an unsound
> kernel result**."* You are adding capability, not fixing a bug.

## §1 Fixed inputs — settled, ⛔ not reopenable in this WP

| input | status |
|---|---|
| **Option B, nested-only.** Preserve DS-9's ordinary six-constructor `Json`; lift the kernel restriction. | Architect `dec_13af1mercv2m0`, resolved |
| ⛔ **Mutual inductives are OUT.** *"A distinct extension, not required by DS-9, would enlarge the trusted change without present demand."* | ruled |
| ⛔ **No `List` name allow-list.** Positivity is keyed on **declared parameter polarity**. | ruled |
| ⛔ **No new axiom, postulate, trusted escape, or library-side workaround.** | ruled |
| DS-9's carrier is `List Char` (`dec_3n1pp559pxrrw`). | ruled — ⛔ not yours, do not revisit |

⚠ If a slice finds mutual machinery falling out for free, that is **not**
authorization to land it. Bring it to the Steward as a separate node.

## §2 Measured substrate — at `origin/main = 10b2f56a`

⚠ Re-derive at kickoff; these line numbers move. Every citation below was
verified to resolve at that SHA.

### 2a The line that rejects nesting is one predicate

`crates/ken-kernel/src/inductive.rs:86`, `check_pos_arg`'s non-`D`-head arm:

```rust
Term::IndFormer { .. } | Term::Const { .. } | Term::Constructor { .. } | Term::Var(_) => {
    // `C u` with a non-`D` head: recurse into the (atomic) head
    // and `occurs`-guard every argument.
    check_pos_arg(d, pol, &head) && args.iter().all(|x| !occurs(d, x))
}
```

For `List Json` the head is `List`, the args are `[Json]`, `!occurs(Json, Json)`
is `false` ⇒ rejected. Reached from `check_positivity` (`:137`), called at
`check.rs:956`.

### 2b ⭐ The machinery contract point 1 requires DOES NOT EXIST

The ruling requires positivity *"structural through **declared**
strictly-positive type-parameter positions."* To honour that the kernel must be
able to ask *"is `List`'s first parameter declared strictly positive?"* — it
cannot:

| measured | consequence |
|---|---|
| `InductiveDecl` (`env.rs:144-159`) carries `params: Vec<Term>` — parameter **types** only, no polarity | nowhere to read a declared polarity from |
| `Pol` (`inductive.rs:43-46`) is a **private** two-valued enum, used only inside one `check_pos_arg` traversal | polarity is a transient of the check, not a property of a declaration |

⇒ `D1a` exists. This is why `D1` is not a local edit.

### 2c ⭐⭐ `recursive_args` IS THE SEAM — and its SILENCE is the inert outcome

**This is the most important measurement in the frame.** `recursive_args`
(`inductive.rs:183`) is the single producer of *"which of this constructor's
arguments are recursive, and what IH does each one need."* It returns
`Vec<(arg_position, branching_telescope, index_exprs)>` and works by peeling Π
binders, then peeling the application spine, then testing whether the head is
the family:

```rust
if let Term::IndFormer { id, .. } = head {
    if id == d && args.len() >= m { out.push((j, pis, args[m..].to_vec())); }
}
```

For `JsonArray (List Json)` there are no Π binders, the body is `List Json`, and
the head is **`List`** — not `Json`. **So the arm never fires and
`recursive_args` returns `[]`.**

⛔⛔ **`[]` is not an error. `[]` is the correct answer for `JsonNull`.** That is
the whole hazard, and it makes contract point 2 mechanically precise.

### ⛔⛔ CORRECTED 2026-07-27 — the inert outcome is PRESENT, not future

⚠ **This section originally described the zero-IH outcome as something relaxing
`check_pos_arg` *would* create. It already exists on `main`**, for a shape the
kernel **already admits**. Measured by `kernel-implementer`, independently
confirmed:

```rust
// crates/ken-kernel/src/inductive.rs
:90  Term::Pi(dom, cod)    => check_pos_arg(d, pol.flip(), dom) && check_pos_arg(d, pol, cod),
:91  Term::Sigma(dom, cod) => check_pos_arg(d, pol,        dom) && check_pos_arg(d, pol, cod),
```

⭐ **`Pi` flips its domain; `Sigma` does not — both components are checked at the
same polarity.** So `Σ (_ : D). D` is **strictly positive and admitted today**,
while `recursive_args` returns `[]` for it (a `Sigma` is not `IndFormer`-headed
after `peel_app`). ⇒ **A declaration with two recursive occurrences and zero
induction hypotheses is reachable on `main` right now**, with no foreign former
involved.

⚠ **It is not currently a *bug*** — Architect: the method type and the reduct
**agree** on zero IHs, so it is *"an already-admitted **eliminator-completeness
gap** … internally type-correct."* ⛔ **Changing only `method_type` is what turns
it into a subject-reduction defect.** That is the entire reason `D3b` and `D4`
must be atomic (§4).

⇒ **Primitive `Sigma` is MANDATORY in the exhaustive recursive-shape
population** — Architect: it belongs through §8.5 clause 2's native-former rule,
and *"§3.2's lift is structural over the argument type, not name-scoped to
foreign formers."* For `Σ (_ : D). D`: preserve the dependent `Sigma` topology
and expose **exactly two motive leaves**. ⛔ Omitting it preserves an
already-demonstrated zero-IH gap.

Original text, still accurate as to mechanism — if you only relax
`check_pos_arg`:

| step | what happens |
|---|---|
| the declaration | **admitted** |
| `method_type` (`:211`) asks `recursive_args` | gets `[]` ⇒ generates the `JsonArray` method with **zero IHs** |
| `check.rs:555` checks that method type | **passes** — it is a well-formed type |
| `iota_reduct` (`:339`) fires on `JsonArray` | **succeeds** — applies a method with zero IHs |
| every existing test | **green** |

⇒ You get a `Json` you can declare, construct, and pattern-match on, but
**cannot induct over** — and *nothing goes red anywhere*. The TCB has grown and
the capability is absent. ⭐ Structurally identical to hard-stop `#11`'s
inertness clause: *the producer → validator → eliminator edge must be real and
executable.* Here that edge is **declaration → eliminator + lifted IH + iota → a
writable theorem**, and `AC-K3` is what observes its far end.

⚠ **The return type itself cannot express a nested occurrence.** The triple says
*"arg `pos` has type `Π branching_tel. D params idxs`."* A nested occurrence is
*"arg `pos` has type `List Json`"* — the recursive occurrences are **elements of
a container**, and the IH must be **lifted through** the container. ⇒ `D3`
requires widening this return type (or adding a sibling producer), which is a
**public API change with consumers in three crates**. Size accordingly.

### 2d The consumer census — 2 delegators, 3 re-derivations

⛔ **A `recursive_args` change does NOT propagate everywhere.** Measured:

| consumer | how it learns recursive-arg structure | follows a `recursive_args` change? |
|---|---|---|
| `ken-kernel` `method_type` (`inductive.rs:211`) | calls it | ✅ |
| `ken-kernel` `iota_reduct` (`inductive.rs:339`) | calls it | ✅ |
| `ken-kernel` `check.rs:555` | via `method_type` | ✅ |
| `ken-elaborator` `elab.rs:1241` (dependent-match IH) | calls it — the comment says it deliberately uses *"the kernel's own producer … rather than re-deriving"* | ✅ |
| `ken-elaborator` `compiler_driver.rs:1786` | calls it | ✅ |
| ⛔ **`ken-kernel` `sct.rs:241` `is_recursive_field`** | **re-derives** — peels `Pi`, tests the head | ❌ |
| ⛔ **`ken-interp` `eval.rs:557` `recursive_arg_arity`** | **re-derives** — peels `Pi`, tests the head, returns `nb` | ❌ |

⭐ **The elaborator half of `D5` may largely fall out of `D3`** — it was
deliberately built to delegate. ⛔ **The termination checker and the evaluator
will not.** Both return the same silently-wrong answer as `recursive_args`
(`false` / `None` = *"not a recursive position"*), so a nested `Json` reaches
SCT and the evaluator as a **non-recursive field**. ⚠ Neither will error.

### 2e ⛔ SAME NAME, TWO CONCEPTS — the trap waiting for `D1a`

`D1a` records a structural property on the declaration. **That has been tried on
this exact struct and left inert.** There are two fields named
`recursive_positions`:

| field | state |
|---|---|
| `ken-kernel/src/env.rs:137` (`ConstructorDecl`) | ⛔ **DEAD — never populated.** `check.rs:933` sets `Vec::new()`; every fixture sets `vec![]`. `env.rs:498` still documents a contract that it *"must already be computed"* — **stale**. `sct.rs:232-241` documents why it refuses to trust it: *"would silently under-count every constructor's IH slots as zero."* `sct_completeness_repro.rs:17` calls it *"a separate, pre-existing latent bug."* |
| `ken-elaborator/src/checked_core.rs:301` + `ken-runtime/src/ir.rs:148` | ✅ **LIVE and load-bearing** — populated at `compiler_driver.rs:3606`, consumed throughout erasure and the cranelift backend |

⚠ **Grepping `recursive_positions` returns 100+ hits dominated by the live
one**, so it is easy to conclude the recorded-property approach already works.
⛔ In the kernel it does not.

⇒ **`D1a` must not repeat this.** Whatever polarity notion is recorded has to be
**populated at admission and actually consumed by the positivity check**, with a
control proving the consumer reads the *recorded* value rather than recomputing
it. `AC-K11` exists for that.

### 2f The blast radius reaches native lowering

The **live** `recursive_positions.len()` is the **IH slot count** for match
lowering: `lowering/core.rs:1486`, `lowering/mod.rs:3990-4020`,
`static_transition.rs:891` all compute binder arity as
`argument_binders + recursive_positions.len()`.

⚠ A nested occurrence contributes a **lifted** IH, so the
one-IH-per-recursive-position relationship those sites assume no longer holds by
construction. ⛔ Do not assume this is out of scope because it is not the kernel
— `AC-K12` covers it, and a source-level census cannot see it (⭐ the ABI-S3
lesson: only a crate that exercises a **built artifact** caught that class).

### 2g ⛔ ADDED 2026-07-27 — the producer must be TOTAL, and this frame did not say so

**Two consecutive Architect rejections of `D1a` landed on the same axis**, and it
is an axis §2b–§2e never named. Recorded here so the third attempt is measured
against a written bar rather than discovering it again.

| rejected SHA | Decision | the defect |
|---|---|---|
| `83d6a7c3` | `dec_3g5qg6f9hzge5` | unknown/unsupported positions were encoded as `Pol::Minus`. `Minus` is **ordinary contravariance, not absorbing** — `Minus.flip() == Plus` — so a nested `Pi` laundered the fail-closed marking positive. `List (A -> Bool)` recorded `[StrictlyPositive]` where the contract demands `[NonPositive]` |
| `6103d321` | `dec_2r7xykp0aswe5` | the producer is **not total**. `declare_inductive` **panics** with `attempt to subtract with overflow` on an accepted field type. Two coupled causes: the fallback traverses every `Term::Let` child at one `local_depth` although `body` binds index 0; and `(r < n).then_some(n - 1 - r)` evaluates the subtraction **eagerly**, underflowing before the condition can yield `None` |

⭐ **The generalisation, which is what makes this a frame defect and not two bugs.**
`D1a` as originally written asked for a polarity notion *"derived at admission,
recorded on the declaration, and readable when checking a nested occurrence."*
All three properties are about **the record**. `AC-K11` then guards **consumption**
(recorded-then-ignored). ⛔ **Nothing guarded the producer.** A producer can
satisfy every one of those and still be non-absorbing on one lattice element, or
partial on one `Term` form — and both failures are silent until someone declares
the shape that hits them.

⚠ **Two distinct lessons, both worth carrying past this node:**

1. ⛔ **A fail-closed state must not be an ordinary value of a lattice that has a
   flip operation.** `Unknown` has to absorb `flip()`; encoding "I don't know" as
   "contravariant" means the type system cannot tell the two apart, and one of
   them is safe to invert.
2. ⛔ **Enumerate the producer's domain by `Term` form, not by example.** The
   Architect's instruction is the right standard: *"audit every child traversed
   by the fallback whose de Bruijn depth differs, rather than fixing one named
   arm only."* ⭐ Same discipline Runtime applied to `first()` on `ABI-S3` the
   same morning — fix the **class**, because a re-review scoped to the named
   trace passes either way.

## ⛔⛔ §2h STOP ENUMERATING SPELLINGS — `AC-K15`, added after FOUR blocks

⚠ **Four consecutive Architect blocks have landed on this producer, and the last
two are the same defect at different depths.** That is a signal about the
**acceptance criterion**, not about the ring — which has repaired correctly and
fast each time.

| # | Decision | missed case |
|---|---|---|
| 1 | `dec_3g5qg6f9hzge5` | `Pol::Minus` used for "unknown" is not absorbing |
| 2 | `dec_2r7xykp0aswe5` | producer not total — `Term::Let` depth, eager `then_some` panic |
| 3 | `dec_4ph659v1q4cq2` | transparent `Alias := F` head not unfolded before classification |
| 4 | `dec_56wzhbds88jad` | lambda-bodied `Alias A := F A` → `(λA. F A) D`; no β/weak-head step, unresolved head |

⛔ **`AC-K13`'s "enumerate by `Term` form, not by example" was NECESSARY AND NOT
SUFFICIENT, and my wording sent the ring down an enumeration path.** Enumerating
binder forms exhausts *syntax*. It says nothing about whether an application head
**resolves**, and nothing about whether it resolves **after β**. ⇒ Against a
reviewer who can always write one more definitionally-equal spelling,
enumeration converges one block at a time.

⭐⭐ **A structural pin that enumerates SPELLINGS is not a proof of the
PROPERTY.** Architect, verbatim: *"topology classification is still
spelling-dependent."* That is the defect, stated exactly.

| `AC-K15` | ⭐⭐ **CLASSIFICATION IS INVARIANT UNDER DEFINITIONAL EQUALITY.** Any two definitionally equal constructor field types must classify the same. | ⛔ **Not an enumeration**, and ⛔ **not raw structural identity** — see the amendment below for the three-way split, which is binding |

⇒ **The test that this AC is met is not a case list. It is:** *given a set of
spellings the kernel already considers definitionally equal, is every
classification equal?* ⛔ If the answer depends on which spelling was written,
the producer is not done regardless of how many arms it has.

⚠ **`AC-K13` still stands for what it actually proves** — producer totality and
### ⛔⛔ `AC-K15` AMENDED — THREE PLACES (`dec_rn8krqyyeq33`)

⚠ **Raised by the Steward as a placement concern after FIVE blocks**, with `D3a`
at `+793/−1` and carrying a **conversion-quotient comparator over dependent
telescopes** inside a slice defined as *inert preparation*. Architect: *"yes on
placement, no on deferring the whole property."*

⛔ **My proposed rationale was FALSE and is withdrawn.** I argued *"classify the
normal form ⇒ invariance holds by construction."* Architect: **full `Term`
normalization is NOT a canonical representative of Ken's complete definitional
equality** — already proven by `Level::equiv` (`0 ≡ max 0 0`) and by proof
irrelevance. ⇒ ⛔ **Raw `RecursiveShape` structural identity cannot be the
contract either.** The invariance genuinely needs a **quotient**; only its
*oracle* moves.

| where | what, transcribed |
|---|---|
| **`D3a` production** | derive the recursive **lift skeleton** through the established terminating δ+β path. Definitionally equal spellings must classify to the same **variants, former identity, recursive/nonrecursive argument positions, arities, and leaf topology**. ⭐ Retained `Term`/`Level` values are **semantic payloads, not Rust identity**. ⛔ No `PartialEq`/`Eq` on descriptors. ⛔ **No unused production equivalence function** — an unconsumed comparator in a kernel module grows the audited surface *for a test oracle* |
| **`D3a` `AC-K15` evidence** | the quotient checker is **TEST-LOCAL**: topology structurally, levels via `Level::equiv`, retained telescope prefixes **in lockstep**, retained fields via context-aware `convert_type`. ⛔ Must include **both** the **positive convertible** dependent-prefix pair **and** the **negative `Type 0` vs `Omega 0`** pair |
| **atomic `D3b`+`D4`** | ⭐ the load-bearing semantic consequence: for definitionally equal field spellings, **generated method/lift terms must be definitionally equal**, the **matching iota terms must inhabit them**, and **the kernel must check the pair**. ⛔ Not claimable by `D3a` |

⭐⭐ **The generalisation worth keeping past this node:** an invariance can be
*correct* as a property and still be **misplaced as a deliverable**. Asking a
slice to *prove* an invariance can force it to build machinery its production
path does not need — here, a comparator with **zero production consumers** living
in the TCB to satisfy a test. ⇒ **Ask where the property is OBSERVED, not just
whether it holds:** the oracle belongs where it is tested, the semantic
consequence belongs where consumers exist.

⚠ **And the boundary that caught my error:** I raised *placement* and explicitly
did not rule *mechanism*. Had I ruled it, I would have prescribed a structural
comparison that **cannot express the property** — because normalization does not
quotient by level-equivalence or proof irrelevance. **Second time today that
division saved a wrong instruction from reaching a ring.**

`AC-K13` still stands for what it actually proves — producer totality and
non-panicking over syntactic forms. `AC-K15` is the exhaustiveness criterion it
was mistaken for.

## §3 Deliverables

Node text is authoritative; this is the slicing view.

| id | deliverable | notes |
|---|---|---|
| `D1a` | per-parameter polarity for an inductive family — derived at admission, **recorded** on the declaration, readable when checking a nested occurrence | ⚠ does not exist (§2b). ⛔ Read §2e before choosing where to record it |
| `D1b` | structural positivity through those declared positions, replacing the blanket rejection at `check_pos_arg`'s non-`D`-head arm | ⛔ keyed on polarity, never on a name. ⛔⛔ **GATED — read the block below the table before any admission widening** |
| `D2` | fail-closed for unknown and non-positive parameter positions | ⛔⛔ **over ALL FOUR positions** — constructor **arguments**, constructor **target indices**, **inductive indices**, **dependent parameter types**. The landed producer covers only the first. See the block below the table |
| `D3a` | ⭐ **inert preparation**: the exhaustive recursive-shape descriptor, its producer, and API plumbing. Population **must** include direct, Pi/W-style, `D`-free, declared-positive-former nesting (test-only construction), and **primitive `Sigma`** | ⛔ Must leave admission, `method_type`, `iota_reduct`, and **every observable eliminator signature and reduct behaviour unchanged**. If it replaces `recursive_args`, legacy consumers **project only** the legacy direct/Π-bound class. ⚠ Does **not** discharge `D3` or `AC-K3` |
| `D3b` | **semantic consumption**: a generated method binder actually carries the structured lift | ⛔⛔ **ATOMIC WITH `D4`** — see `AC-K14`. ⛔ Not separable by any commit |
| `D4` | the matching iota reductions; the kernel **checks** the generated eliminator | ⛔⛔ **ATOMIC WITH `D3b`.** *"Generated method binders and the terms supplied by ι are one semantic unit."* Both land before `D1b` |
| `D5` | surface consumability: matching, elaboration, structural-recursion/termination | ⚠ §2d — the elaborator delegates, `sct.rs` and the evaluator do not |
| `D6` | the four conformance rows of contract point 4 | |
| `D7` | `trusted_base()` delta as a **number**, with what grew and why | ⛔ not a zero — this node grows the TCB |

### ⛔⛔ `D1b`/`D2` GATE — THE POLARITY PRODUCER ON `main` IS FAIL-OPEN

**Recorded here 2026-07-28 on `kernel-leader`'s routed ask.** The node has
carried this since the `D3b`+`D4` merge; the frame did not, and **the frame is
the artifact a `D1b` implementer slices from.**

**Authority:** Architect ruling `evt_3edf99cq5mrka` and merge Decision
`dec_b1hj6th3363a` (resolved APPROVE), on adversary finding
`evt_79m7a5y9d1b4g`.

> ⛔ **`D1b` MUST NOT open production nested admission until polarity derivation
> is FAIL-CLOSED over all four positions:** constructor **arguments**,
> constructor **target indices**, **inductive indices**, and **dependent
> parameter types**.

**What is wrong on `main`:** `derive_parameter_polarities` scans only
`constructor.args`, while `derive_recursive_shape` admits a nested recursive
`Former` **only** on a recorded `StrictlyPositive`. ⇒ A negative occurrence in
any of the other three positions is **recorded positive**, and the adversary
demonstrated that **target-index placement flips the permissive gate from reject
to accept**. ⭐ `D1a` shipped the *record*; it never shipped coverage of the
positions the record claims to summarise.

⚠ **`D3b`+`D4` was audited against this and cleared — narrowly.** The clearance
rests on two facts **`D1b` destroys**: the nested-`Former` controls are reachable
only through the test-only `install_test_only_nested_family` fixture, and the
production-live primitive-`Sigma` path never consults another former's polarity
record. ⛔ **Do not read "`D3b`+`D4` is staging-safe" as "the record is sound."**
It is unsound now and merely unreachable — the semantic consumers already exist;
only the admission route is missing. `D1b` **is** that route.

⛔ **AC, not advice — a `D1b` candidate is incomplete without it.** Establish
coverage over all four positions **and one control that discriminates each**,
before any admission widening, and record the per-position result. ⚠ A control
exercising only constructor arguments **passes on the landed producer** and says
nothing about the other three — that is the exact shape this gate exists to
reject.

## §4 Slicing guidance — ⭐ `D1a` FIRST, and the order is not arbitrary

> ## ⛔⛔ CORRECTED — THE ORDER BELOW WAS UNSOUND (`dec_351mz4r239398`)
>
> **The order this section originally gave — `D1a` → `D3` → `D1b`/`D2` → `D4` —
> is unsound and is withdrawn.** Architect, verbatim: *"The current order `D3
> semantics → D1b → D4` is unsound."*
>
> **Why:** ⭐ *"In the TCB, generated method binders and the terms supplied by ι
> are **one semantic unit**."* Giving a method a structured lifted binder while
> `iota_reduct` still supplies none is not an incomplete step — it is a
> **subject-reduction defect**: the generated method expects a lifted value the
> reduct never constructs. Separating `D3`'s semantics from `D4` by *any* commit,
> let alone by `D1b`, passes through that broken state.
>
> ⇒ **`D3` splits, and `D3b` fuses with `D4`:**
>
> **`D1a` ✅ → `D3a` (inert prep) → `D3b`+`D4` ATOMIC → `D1b`/`D2` → `D5` →
> `D6`/`D7`.**
>
> ⚠ **`D3a` does NOT discharge `D3` or `AC-K3`.** *"The frame phrase 'eliminator
> generation extended' is not satisfied until a generated method consumes the
> structured lift."*

**Superseded order (kept only so the correction is legible): `D1a` → `D3` →
`D1b`/`D2` → `D4` → `D5` → `D6`/`D7`.**

### ⭐ RELEASED AS A `D1a`-FIRST SLICE — Steward sequencing call

> **`D1a` is releasable now, ahead of `SPEC-NESTED-IND`. `D1b` onward is not.**
>
> **Why `D1a` is safe:** the ruling already fixes what polarity *means* —
> *"declared strictly-positive type-parameter positions,"* with unknown and
> negative positions failing closed. `D1a` derives and **records** that property.
> ⭐ It admits nothing new, so it **cannot** produce the inert outcome: the
> nested declaration stays rejected for the whole of `D1a`.
>
> **Why the rest is not:** `D1b` opens admittance, and that is the rule the spec
> enclave is writing right now. Implementing it first is implementing a contract
> that does not exist.
>
> ⚠ **Expect `SPEC-NESTED-IND` to land while you are inside `D1a`.** Re-read it
> from the object when it does — `git show origin/main:` on the chapter path —
> ⛔ not from your worktree copy — a branch cut before the merge carries the old
> text, and that reads as complete and self-consistent.
>
> ⚠ If the landed spec defines the polarity notion differently from §2b's
> reading, **stop and report** rather than reshaping `D1a` silently. That is a
> finding about the frame, and the Steward owns it.

⭐ **The point of putting `D1b` third is to make the inert outcome unreachable.**
`D1b` is the one-line-shaped change that makes the blocked declaration
type-check. If it lands first, the node *looks* done while `AC-K3` is
undischargeable, and there is no red test to say otherwise (§2c). Build the
polarity notion and the IH machinery **while the declaration is still rejected**,
then open admittance last.

⚠ If you must open admittance early to test anything, do it behind a test-only
path — ⛔ never as a landed intermediate state on the branch head.

## §5 Acceptance criteria

`AC-K1`–`AC-K12` in the node. `AC-K11` and `AC-K12` come from §2e and §2f — the
substrate census found two consumers the original ten could not see.

⛔ **`AC-K3` is what discharges this node**, not `AC-K1`. `AC-K1` (declaration
admitted) is *necessary and not sufficient* — a guard deletion passes it.

⛔ **`AC-K3` and `AC-K8` are the pair that matters:** `AC-K3` proves the new
capability is usable, `AC-K8` proves the old one is undamaged. Greening one while
quietly weakening the other widens the TCB for nothing.

⚠ **Report `AC-K5`–`AC-K7` as three separate rows.** Three distinct rejection
reasons; an aggregate *"negatives still rejected"* pass hides one defecting.

## §6 Validation — ⛔ TARGETED ONLY
> ### ⛔⛔ ADDED 2026-07-27 — `-p <crate>` IS NOT A BLAST-RADIUS ORACLE
>
> ⚠ **This section conflated two different questions under one instrument, and a
> ring paid for it.** Measured by `runtime-implementer` on `P2`:
>
> `scripts/ken-cargo check -p <crate> --all-targets` builds that crate and **its
> own** targets. ⛔ **It does not rebuild dependent crates** — so a downstream
> derive that can no longer be satisfied is **invisible** to it. On `P2` a
> `-p ken-runtime` check reported clean while `ken-elaborator` was left red; the
> failure surfaced only under `-p ken-interp` / `-p ken-verify`, which pull
> `ken-elaborator` in. The measured claim *"entirely inside `ken-runtime`"* was
> produced by an instrument that could not see outside `ken-runtime`.
>
> ⇒ **Two questions, and `-p` answers only the first:**
>
> | question | instrument |
> |---|---|
> | *does this crate still build?* | ✅ `-p <crate>` |
> | *does anything that DEPENDS on it still build?* | ⛔ **not `-p <crate>`** — build a **dependent** crate, or state that CI answers it |
>
> ⭐ **Trigger:** any change to a **public type**, or to a **derive on one**. A
> derive is part of a type's public contract, so removing one is a breaking change
> to every dependent crate — and its cost is carried by **reachability**, not by
> call sites.
>
> ⛔ **This does NOT relax the resource rule.** `--workspace` remains forbidden
> (operator, `COORDINATION §12`). The correct move is **one more targeted build of
> a chosen dependent crate**, not a wider one. ⚠ And if you leave the question to
> CI, **say so** — an unmeasured blast radius reported as "targeted run green" is
> the defect this note exists to prevent.
>
> ### ⛔ THIRD INSTRUMENT FINDING — a cargo re-run is NOT idempotent for REPORTING
>
> Measured by `runtime-implementer` mid-census, 2026-07-27: a per-crate loop
> reported **2** sites for `ken-elaborator`; the raw uncached run showed **4**.
> **Cargo had cached the failed `lib test` target and did not re-emit its
> diagnostics on the second invocation.**
>
> ⇒ ⛔ **A second `cargo`/`ken-cargo` invocation can report FEWER failures than
> the first while nothing changed**, because diagnostics are not replayed from
> cache. **A census must read raw first-run output — never a re-filtered second
> pass.**
>
> ⚠ This defeats the ordinary habit of *re-run and grep*: the grep is correct, the
> input is silently truncated, and the result looks like a smaller problem rather
> than a broken measurement. ⭐ If you must filter, `tee` the first run and grep
> the file.
>
> ---
>
> ⭐⭐ **Three instruments, three different blind spots, all found by measuring the
> measurement rather than trusting it:**
>
> | instrument | cannot see |
> |---|---|
> | `-p <crate>` | dependent crates |
> | `--lib` | test targets |
> | a re-run | diagnostics the first run already consumed |
>
> ⇒ **State what each named build ANSWERS and what it CANNOT SEE.** A validation
> list that names commands without naming their blind spots is a list of
> instruments, not a measurement plan.


⛔ **NEVER `--workspace`** (operator hard rule, `agent/COORDINATION.md §12`).
**The full build, `--locked`, and the conformance suite run in CI on GitHub.**
"No regression" means **green in CI** — ⛔ never a local workspace run.

Through `scripts/ken-cargo`:

| scope | why it is in the set |
|---|---|
| `-p ken-kernel` | `D1a`–`D4`; includes the `k1p5_wstyle.rs` suite `AC-K8` must leave untouched |
| `-p ken-elaborator` | `D5` — `elab.rs` and `compiler_driver.rs` both consume `recursive_args` |
| `-p ken-interp` | ⚠ §2d — `eval.rs` **re-derives**, so nothing else will catch it. Run the **full** crate suite (a reifier/eliminator change has cross-suite reach) |
| `-p ken-verify` | ⭐ the cheap **built-artifact** oracle — only ~24 tests, and it is what caught the ABI-S3 class of regression a source census could not see |

⚠ `-p ken-runtime` is the §2f consumer and is **expensive** (cranelift). ⛔ Do
not run it as a matter of course — name it to CI and let `ken-verify` be your
local signal. If `AC-K12` needs it directly, run **one** `--test`, not the crate.

⚠ **A kernel change is where the local/CI split bites hardest** — the blast
radius is every crate. ⛔ Do not conclude *"no regression"* from a green targeted
run; state what you ran and let CI answer the rest.

## §7 Contention

⚠ **This is the widest blast radius in flight.** It changes the kernel's
admittance surface, so it is **not** contention-free the way DS-9 and `ABI-S3`
are with each other.

At the time of writing: Runtime holds `crates/ken-host`, `crates/ken-runtime`,
`crates/ken-interp`, `crates/ken-elaborator/src` for `ABI-S3` (candidate
`e60ab364`, in QA). ⚠ **That overlaps `D5`'s `ken-elaborator/src` and `D2`'s
validation reach into `ken-interp`.**

⛔ **Re-derive contention at kickoff** against whatever is then active in
`crates/`. This section will be stale. If `ABI-S3` has not merged when you are
ready to touch `ken-elaborator/src`, raise it to the Steward — ⛔ do not
serialize yourself silently, and ⛔ do not rebase over another ring's branch.

## §8 Reporting

Report to `kernel-leader` with: the exact SHA, the deliverables closed, and
`AC-K1`–`AC-K12` **row by row** with each control's result. For each control,
say what you **mutated** and that it **fired** — ⛔ an assertion that passes is
not evidence a control can fail. State the `D7` number.

⭐ **If the polarity design turns out differently than §2b implies, say so.**
The substrate was measured by the Steward, not by the kernel ring; a correction
is a finding, not a problem.

[n]: ../issues/KERNEL-NESTED-IND.md
