---
id: KERNEL-NESTED-IND
title: "admit nested strictly-positive inductives in the kernel — structural positivity through declared parameter positions, generated and checked dependent eliminators with one lifted IH per contained recursive occurrence, iota, and surface consumability"
status: active
owner: kernel
size: L
gate: none
depends_on: [SPEC-NESTED-IND]
blocks: [DS-9]
github: null
origin: Architect ruling evt_55k9f9efvd8jk, Decision dec_13af1mercv2m0 resolved. Demand-pulled by DS-9, which blocked at its first deliverable on `JsonArray (List Json)`; fork raised by the Steward as evt_1ykvpj7yvtg18. The five-point prerequisite contract below is the Architect's, transcribed verbatim in substance. Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ⭐ `D1a` + `D3a` LANDED. Next is the ATOMIC `D3b`+`D4`.
>
> | slice | PR | landed | evidence |
> |---|---|---|---|
> | `D1a` — per-parameter polarity | #1077 | `88196527` | `inductive.rs` `e37e906f`, `nested_inductives_d1a.rs` `280025f1` |
> | `D3a` — recursive-shape descriptor, inert | #1089 | `ac86b2d7` | `inductive.rs` `d6ab179c`, `nested_inductives_d3_shapes.rs` `33a3efbf`; +858/−1 |
>
> Both CI-green; both verified by **blob identity** with discriminating pre-merge
> controls. `D3a` took **six** candidates — five rejected objects preserved as
> ancestors on `wp/KERNEL-NESTED-IND-D3`, none rewritten.
>
> ⛔ **The node is NOT complete.** `D1a` and `D3a` are two of eight deliverables;
> `D3b`, `D4`, `D1b`, `D2`, `D5`, `D6`, `D7` remain. **A nested inductive is still
> rejected on `origin/main`** — ⭐ independently confirmed after `D3a` landed: the
> non-`D`-head `args.iter().all(|x| !occurs(d, x))` guard is **still present**.
> That is `D3a`'s inertness, verified rather than asserted.
>
> ⇒ **Next is the atomic `D3b`+`D4`** (`dec_351mz4r239398`), ⛔ **before** `D1b`.
> ⚠ `D1b`'s external gate lifted when `SPEC-NESTED-IND` merged; ⛔ that did **not**
> reorder the work.
>
> **`D1a` = per-parameter polarity, derived at admission and consumed by the
> positivity gate.** Candidate `e685570c1b8403c38af7ed0f45c205a6bc2eeb90`, **CI
> checks passed**, five `ken-kernel` paths, +463/−2. Verified by blob identity:
> `src/inductive.rs` `e37e906f`, `tests/nested_inductives_d1a.rs` `280025f1`, with
> discriminating controls at `b5c448d1`. Decision `dec_3k5rnnx0e04nz` read
> `resolved` from the object (07:39:48Z).
>
> ⭐ **`SPEC-NESTED-IND` merged (PR #1076), so the `D1b` gate is lifted.** The
> governing chapter is `spec/10-kernel/14-inductive.md` blob **`4dab9d0e`** on
> `origin/main` — ⛔ re-bind from the object, never from a worktree copy.
> ⚠ **Un-gating `D1b` does not reorder the work:** frame §4 still puts `D3` first,
> because `D1b` opens admittance and is the change that yields the inert outcome
> with nothing red to say so.
>
> ### ⚠ Three candidates, two rejections, and the fault was in this frame
>
> | candidate | Decision | outcome |
> |---|---|---|
> | `83d6a7c3` | `dec_3g5qg6f9hzge5` | **rejected** — `Pol::Minus` used for "unknown" is not absorbing (`Minus.flip() == Plus`), so a nested `Pi` laundered it positive |
> | `6103d321` | `dec_2r7xykp0aswe5` | **rejected** — the producer was not total: `declare_inductive` **panicked** on an accepted field type, violating `18 §4` |
> | `e685570c` | `dec_3k5rnnx0e04nz` | **resolved** — landed |
>
> ⛔ **Both rejections landed on an axis this frame never named.** `D1a` as
> originally written specified a polarity notion *derived, recorded, readable* —
> all three about the **record** — and `AC-K11` guarded **consumption**. Nothing
> specified the **producer**, so no control existed that could have failed.
> `AC-K13` is that specification, added after the fact.
>
> ⭐ **`AC-K13` was then discharged by closing the class, not the instance:** Pi,
> Sigma, Lam and Let are `Term`'s complete syntactic binder set, each given a
> depth-aware arm, and every fallback-traversed `children()` form adds no binder —
> *"thus no differing-depth fallback edge remains."* That is the standard for
> `D3`–`D5` as well.

> ## ▶ THE KERNEL HALF OF A TWO-STAGE PREREQUISITE
>
> **Frame:** [`kernel-nested-inductives.md`][f], under `docs/program/wp/`. The
> frame is the executable artifact — measured substrate, slicing order, control
> recipes, validation set, contention. This node carries the contract and the ACs.
>
> **Sequence:** `SPEC-NESTED-IND` → **`KERNEL-NESTED-IND`** → `DS-9`.
>
> ✅ **`D1a` is landed and `SPEC-NESTED-IND` has merged, so the `D1b` gate is
> lifted** (see the banner above for the evidence). `D1a` was released alone
> because it **admits nothing new** — the nested declaration stayed rejected
> throughout, making the inert outcome unreachable while the rule was still being
> written. ⚠ That gate is now discharged; ⛔ the **slicing order is not**. Frame §4
> still puts `D3` before `D1b`.
>
> ⚠ **This node changes the TCB.** Read `docs/PRINCIPLES.md` on the small
> auditable trusted base before slicing it.

## Why this exists

`DS-9` blocked at `D1` on `JsonArray (List Json)` — the `List (Rose A)` class that
`spec/10-kernel/14-inductive.md` §8.5 **deferred at the time** (it now states the
nested rule; `SPEC-NESTED-IND` merged 2026-07-27). The Architect ruled **B,
nested-only**: preserve DS-9's ordinary six-constructor `Json` and lift the
kernel restriction, rather than re-encode the value model.

⭐ **The rejection being lifted is sound, not broken.** Architect, verbatim: *"The
present rejection is a safe, deliberate completeness/staging boundary, **not an
unsound kernel result**."* This node adds capability; it does not fix a bug.

## ⛔ SCOPE — NESTED ONLY, and the exclusion is load-bearing

**Architect, verbatim:** *"Do **not** bundle mutual inductives. Mutual families are
a distinct extension, are not required by DS-9, and would enlarge the trusted
change without present demand."*

⛔ Mutual is **out**, and the landed spec now says so in its own place: `14 §8.5`
is *"Nested inductives — structural parameter polarity"* and **`14 §8.6` is
*"Mutually-defined inductives — still deferred"***, with its own reason
(simultaneous-block positivity, jointly generated eliminators, joint termination,
no present consumer). ⚠ Before `SPEC-NESTED-IND` merged, one §8.5 clause deferred
both, so "un-defer §8.5" read as both; that ambiguity is now removed in the text.
⚠ If a slice finds mutual machinery falling out for free, that is **not**
authorization to land it; bring it back to the Steward as a separate node.

## ⛔ THE FIVE-POINT CONTRACT — complete only when ALL FIVE hold

Transcribed from `evt_55k9f9efvd8jk`. ⛔ Not a summary — these are the completion
conditions.

1. **Positivity is structural through declared strictly-positive type-parameter
   positions**, sufficient for **both** `List Json` **and**
   `List (Pair String Json)`. Unknown and negative positions **fail closed**.
   ⛔⛔ **There is NO `List` name allow-list.**
2. **The kernel generates AND checks the dependent eliminator**, with **one lifted
   induction hypothesis for every contained recursive `Json`**, and the
   corresponding **iota reductions**. ⛔⛔ *"Merely deleting or relaxing the current
   `occurs` guard is **not delivery**: that would admit the declaration without
   supplying sound recursion/proof machinery."*
3. **Surface matching, elaboration, and structural-recursion/termination checking
   can consume those lifted hypotheses**, so that a theorem over the array and
   object branches is **actually writable**.
4. **Conformance** includes: a **positive** nested `List`/Rose-style declaration
   **with a real recursive computation or proof**; a retained **nested-negative
   rejection**; a retained **rejection through an unknown or non-positive
   parameter**; and evidence that **direct and existing W-style inductives are
   unchanged**.
5. ⛔ **No new axiom, postulate, trusted escape, or library-side representation
   workaround** enters the solution.

## ⭐⭐ The anti-pattern point 2 exists to forbid — read this before slicing

The cheap version of this node is: find the `occurs`-guard (§8.2, cited at
`14-inductive.md:569-570`), relax it so the declaration is admitted, watch
`data Json = ... | JsonArray (List Json)` type-check, and report success.

⛔ **That is explicitly not delivery**, and it is worse than nothing: the
declaration would be admitted with **no sound way to induct over it**, so the
first person to try proving anything about the array branch discovers the gap —
after the TCB already grew.

⭐ **This is structurally the same rule as hard-stop `#11`'s inertness clause**
(`RT-FNSPLIT-C1`): *a prerequisite may be inert only in the sense that production
routing has not switched to it yet; its producer → validator → eliminator edge
must nevertheless be real and executable.* Here the edge is **declaration →
eliminator + IH + iota → a writable theorem**. Point 3 is what makes the far end
of that edge observable.

⇒ **The AC that discharges this node is `AC-K3`**, not `AC-K1`.

## ⭐⭐ MEASURED SUBSTRATE — and it makes contract point 1 bigger than it looks

Measured at `origin/main = 10b2f56a`, every citation re-verified to resolve.
⚠ Re-derive before starting; these line numbers move.

**The single line that rejects nesting** — `crates/ken-kernel/src/inductive.rs`,
inside `check_pos_arg` (`:86`, the `14 §8.2` judgment):

```rust
Term::IndFormer { .. } | Term::Const { .. } | Term::Constructor { .. } | Term::Var(_) => {
    // `C u` with a non-`D` head: recurse into the (atomic) head
    // and `occurs`-guard every argument.
    check_pos_arg(d, pol, &head) && args.iter().all(|x| !occurs(d, x))
}
```

For `List Json` the head is `List` and the args are `[Json]`, so
`!occurs(Json, Json)` is `false` and the declaration is rejected. **That
`args.iter().all(|x| !occurs(d, x))` is the whole mechanism.**

⛔⛔ **And this is precisely why "relax that line" is not delivery.** Replacing it
with `args.iter().all(|x| check_pos_arg(d, pol, x))` would admit `List Json`
**today**, in one line, with no eliminator, no lifted IH, and no iota — the exact
inert outcome contract point 2 forbids. ⚠ **Expect this to be tempting: it is a
one-line diff that makes the blocked declaration type-check.**

### ⭐⭐ `recursive_args` returns `[]` for a nested arg — SILENTLY

**This is what makes contract point 2 mechanically precise.** `recursive_args`
(`inductive.rs:183`) is the single producer of *"which arguments are recursive
and what IH does each need."* It peels Π binders, peels the application spine,
and fires only when the head **is** the family. For `JsonArray (List Json)` the
head is `List`, so the arm never fires and it returns `[]`.

⛔⛔ **`[]` is not an error — it is the correct answer for `JsonNull`.** So with
`check_pos_arg` relaxed and nothing else: the declaration is **admitted**;
`method_type` (`:211`) generates the `JsonArray` method with **zero IHs**;
`check.rs:555` **accepts** that method type; `iota_reduct` (`:339`) **fires**;
and **every existing test stays green**. ⇒ A `Json` that can be declared,
constructed and matched but **not inducted over**, with the TCB already grown and
no red test anywhere.

⚠ **The return type cannot express a nested occurrence.** Its triple says *"arg
`pos` has type `Π tel. D params idxs`"*; a nested occurrence puts the recursive
occurrences **inside a container**, so the IH must be **lifted through** it. ⇒
`D3` widens a public API with consumers in three crates — and per the frame's
census, `sct.rs:241` and `ken-interp` `eval.rs:557` **re-derive** this test
rather than calling it, so they will not follow. Frame §2c–§2d.

### ⭐ The machinery contract point 1 requires DOES NOT EXIST YET

The ruling requires positivity *"structural through **declared** strictly-positive
type-parameter positions"*, with unknown and non-positive positions failing
closed. To honour that, the kernel must be able to ask *"is `List`'s first
parameter declared strictly positive?"* — **it cannot.**

| measured | consequence |
|---|---|
| `InductiveDecl` (`crates/ken-kernel/src/env.rs:144-159`) carries `params: Vec<Term>` — parameter **types** only, **no polarity** | there is nowhere to read a declared parameter polarity from |
| `Pol` (`inductive.rs:43-46`) is a **private**, two-valued enum used only *within* one `check_pos_arg` traversal | polarity is a transient of the check, not a recorded property of a declaration |

⇒ **A per-parameter polarity notion — computed at admission, recorded on the
declaration, and consulted when checking a nested occurrence — is a deliverable
of this node, not a given.** `D1` is written accordingly.

⚠ **This is also what makes `AC-K2`'s control meaningful.** Declaring a *new*
container and nesting `Json` in it must work with **no kernel change** — which is
only possible if polarity is derived from the container's own declaration. If
`AC-K2` requires a code change, the implementation has hardcoded a set of known
containers, which is the allow-list the ruling forbids.

## Deliverables

- **`D1a`** — ⭐ **the missing machinery**: a per-parameter polarity notion for an
  inductive family — derived at admission, recorded on the declaration, and
  readable when checking a nested occurrence. ⚠ Sizing input: this does not exist
  today (see the substrate section), so `D1` cannot be a local edit to
  `check_pos_arg`.
  ⛔ **AND the producer must be TOTAL over every accepted constructor field
  type** — a polarity record or a *rejection* for each, and **never a panic**
  (`18 §4`: the kernel contract is yes/no, never a crash). ⚠ This clause was
  **added 2026-07-27 after two consecutive Architect rejections landed on it**;
  the original three properties (derived / recorded / readable) are all about the
  *record* and say nothing about the *producer*. See `AC-K13`.
- **`D1b`** — structural positivity through those declared strictly-positive
  parameter positions, replacing the blanket nested rejection at
  `inductive.rs` `check_pos_arg`'s non-`D`-head arm. ⛔ Keyed on **declared
  parameter polarity**, never on a type-constructor name.
- **`D2`** — fail-closed handling for unknown and non-positive parameter
  positions.
- **`D3`** — eliminator generation extended: one lifted IH per contained recursive
  occurrence, extending §3.1's Π-abstracted-IH machinery.
- **`D4`** — the matching iota reductions, and the kernel **checks** the generated
  eliminator rather than trusting it.
- **`D5`** — surface consumability: matching, elaboration, and
  structural-recursion/termination checking accept the lifted hypotheses.
- **`D6`** — the four conformance rows of contract point 4.
- **`D7`** — a **`trusted_base()` delta report**, stated as a number with what
  grew and why. ⚠ This node *does* grow the TCB; the deliverable is an honest
  accounting, ⛔ not a zero.

## Acceptance criteria

Each names its positive control.

| AC | claim | positive control |
|---|---|---|
| `AC-K1` | `data Json = ... \| JsonArray (List Json) \| JsonObject (List (Pair String Json)) \| ...` is **admitted**. | ⚠ **necessary, not sufficient** — a guard-deletion passes this row. It is listed to be discharged, not to be relied on |
| `AC-K2` | Admission is keyed on **declared parameter polarity**, not on a name. | declare a **new** strictly-positive container of your own and nest `Json` in it → must be admitted **with no kernel change**. ⛔ If it needs one, an allow-list is hiding somewhere |
| `AC-K3` | ⭐ **A real theorem over the array branch is written and kernel-checked**, consuming a lifted IH. | delete the lifted IH from the generated eliminator → the theorem must **fail to check**. ⛔ If it still checks, `AC-K3` was never testing the IH |
| `AC-K4` | Iota reduces for nested occurrences; a **recursive computation** over `JsonArray` evaluates. | perturb one iota rule → the computation's result changes or it fails to reduce |
| `AC-K5` | Nested-**negative** rejection retained. | the known-bad `(D → Bool) → D` under a container must still be **rejected**, asserted as the specific rejection |
| `AC-K6` | Rejection through an **unknown** parameter retained. | nest `Json` under a parameter whose polarity is undeclared/unknown → **rejected**, not admitted-by-default |
| `AC-K7` | Rejection through a **non-positive** parameter retained. | as `AC-K6` with a declared-negative position |
| `AC-K8` | Direct and existing **W-style** inductives unchanged. | the K1.5 Π-bound suite (`(Nat → D) → D`, §2.1) runs green **untouched**; ⛔ a diff to those tests is itself a finding |
| `AC-K9` | ⛔ **Zero** new axiom, postulate, trusted escape, or library-side representation workaround. | grep the diff for `Axiom`/`postulate`/`sorry`/`unsafe` additions; a hit fails the row |
| `AC-K10` | `trusted_base()` delta reported **as a number**, with what grew. | ⚠ no mechanical control — discharged by the report. Listed so "grew by 0" and "never measured" cannot read identically |
| `AC-K11` | ⭐ `D1a`'s recorded polarity is **populated at admission and read by the positivity check** — not recorded-then-ignored. | perturb the **recorded** value for one parameter → admittance must change. ⛔ If it does not, the check recomputes and the record is inert — the `ConstructorDecl.recursive_positions` failure repeated (frame §2e) |
| `AC-K14` | ⛔⛔ **`D3b` and `D4` land in ONE commit, and the pair is kernel-checked.** No commit exists in which a generated method binder carries a structured lift that `iota_reduct` does not construct. | ⚠ **`Σ (_ : D). D` is the control, because it is admitted on `main` TODAY with zero IHs** (`inductive.rs:91` checks both Sigma components at the same polarity; `:90` flips only `Pi`'s domain). Exercise the eliminator on it: the method binder's lift and ι's constructed term must agree, and the kernel must check the pair. ⛔ A `method_type` change without the matching ι is a **subject-reduction defect**, not an incomplete step. Architect `dec_351mz4r239398` |
| `AC-K13` | ⭐ **The polarity producer is TOTAL over every accepted constructor field type** — every such field yields a polarity record or a rejection, ⛔ **never a panic** (`18 §4`). | ⚠ **Enumerate by `Term` form, not by example.** For each form the fallback traverses, exercise a field of that form that mentions the parameter. Two named controls, both from Architect rejections: (a) `Term::Let { ty: Bool, val: false, body: pi(var(1), Bool) }` — an accepted field reducing to `A -> Bool`, which must record `NonPositive`; its `body` binds index 0, so a fallback that traverses children at one depth reads `A` at the wrong index. (b) index selection must be non-panicking for an out-of-range relative index — ⛔ `bool::then_some` evaluates its argument **eagerly**, so `(r < n).then_some(n - 1 - r)` underflows *before* the condition can yield `None`; `then(\|\| …)` is the lazy form |
| `AC-K12` | A nested-IH constructor **lowers and evaluates**, not just type-checks. | the evaluator and native-lowering paths **re-derive** recursive positions (frame §2d, §2f) and one lowering site computes binder arity as `argument_binders + recursive_positions.len()`. Control: a recursive computation over `JsonArray` evaluates, and the built-artifact suite is green |

⛔ **`AC-K3` and `AC-K8` are the pair that matters.** `AC-K3` proves the new
capability is *usable*; `AC-K8` proves the old capability is *undamaged*. A node
that greens one and quietly weakens the other has widened the TCB for nothing.

⚠ **Report `AC-K5`–`AC-K7` as three separate rows.** They are three different
rejection reasons and an aggregate "negatives still rejected" pass would hide one
of them defecting.

## Validation — targeted only

⛔ **NEVER `--workspace`** (operator hard rule, `agent/COORDINATION.md §12`). Scope
to the crates you touch (`-p ken-kernel` and, for `D5`, `-p ken-elaborator`), plus
the kernel conformance suite. **The full build, `--locked`, and conformance run in
CI on GitHub.** "No regression" means **green in CI**.

⚠ **A kernel change is the case where the local/CI split bites hardest** — the
blast radius is every crate. ⛔ Do not conclude "no regression" from a green
targeted run; say what you ran and let CI answer the rest.

## Contention

⚠ **This is the one node in flight with a wide blast radius.** It changes the
kernel's admittance surface, so it is **not** contention-free the way DS-9 and
`ABI-S3` are with each other. ⛔ Re-derive contention at kickoff against whatever
is then active in `crates/` — this section will be stale.

## What this unblocks

`DS-9`, and with it Phase 3 of the catalog data-structures campaign. ⭐ More
broadly: nesting a `List` inside a recursive type is the shape of **every tree
with a list of children** — JSON, XML, S-expressions, ASTs, rose trees. ⚠ That
breadth is the argument for doing it properly, ⛔ not for widening scope past
nested-only.

[f]: ../wp/kernel-nested-inductives.md
