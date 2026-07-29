# RT-JOIN-DISPOSITION — separate join materialization from final semantic disposition

**The landed `RT-FNSPLIT-RECUR-PORT` invariant asserts that a planned join is
either *emitted* or *statically unselected*, never both. The Architect measured
that `consumed_join_origins` records **structural materialization**, not
semantic reachability — so a join materialized before its enclosing match selects
a case, whose case is later proved dead, is a legitimate shape the invariant
false-rejects. This node splits the two phases and supplies a CFG/SSA-grounded
materialized-but-dead proof in place of the blanket overlap error.**

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/RT-JOIN-DISPOSITION`. **Size:** M.
**Risk:** high — the repair **relaxes a landed fail-closed check**, which is the
direction that can silently lose a real defect.

**Status:** Steward frame, shovel-ready. ⭐ **FIRST in Runtime's queue.**

⛔ **[[NATIVE-HANDLE-CARRIER]] is bound behind this node** and its WIP
`8bc7556af024886a6db01679f35a2bb063166876` / tree `9bbce2f6` is held unchanged.
⛔ Do not edit, rebase, or resume it until this merges.

---

## 1. Fixed inputs

| path | blob at `origin/main = 29f15d93` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `b924db34df3be74421fa773132fe476a53503ecc` |
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `f9d7fc1025bfa80cb5eaf66284252d3bdd59c28c` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs` | `5f157879fd4b7a187d3465f452ac5b2ebfd288c1` |
| `crates/ken-cli/tests/rt_span_prov_native.rs` | `a590314024b7523d8f59683826c304f01df81c41` |

**Grounding:** Architect ruling `evt_2w62qa82fxyv`, measured in a detached
diagnostic-only worktree against the preserved WIP `8bc7556a` (tree `9bbce2f6`).
⭐ No candidate or production edit survived that worktree. Research grounding:
`docs/program/rt-join-disposition-research-advisory-21.md` (§5a consult #21).
⛔ Do not cite the `/workspaces/ken/local/` copy — it is untracked.

## 2. The measured mechanism

```text
consume join=StaticOriginId(1000)
  emission_reachable_match_cases has no entry for match 1055
select match=StaticOriginId(1055) case=Some(0) prior=None
select match=StaticOriginId(1055) case=Some(0) prior={0}
close match 1055; case 1 is dead; its subtree contains join 1000
```

`consume_join_plan(1000)` runs **before the enclosing `Result` match `1055` has
selected any case**; both later observations select `Err`. ⇒ This is **not** a
different recursive/specialization visit collapsed into a global fact — the third
reading is measured out. `consumed_join_origins` is a **materialization/token**
record, and the later dead-case fact does not contradict it.

## 3. ⭐⭐ THE CONFLATION IS ENFORCED AT FOUR SITES, NOT ONE

The failure text names one site. **Repairing only that site leaves the same
conflation live at three others**, and two of them reject in the opposite
direction (disposition-then-emission), which this fixture happens not to reach.

| # | site (`lowering/mod.rs` @ `b924db34`) | fn | rejects |
|---|---|---|---|
| 1 | `:1676-1680` | `consume_join_plan` | already **dispositioned**, now consumed |
| 2 | `:1709-1714` | `disposition_statically_unselected_source_subtree` | already **consumed**, now dispositioned — ⭐ **the one that fired** |
| 3 | `:1834-1838` | `enter_source_occurrence_plan` | re-entry into a **dispositioned** occurrence |
| 4 | `:1878-1886` | `validate_join_plan_consumption` | the two sets **intersect** at function closure |

Sites 1 and 3 share the message `"statically unselected source join reached
emission"`; site 4 is `"source join {origin:?} was both emitted and statically
unselected"`. ⭐ **All four encode `consumed XOR dispositioned`.** The phase
separation is not done until each has been re-derived against the new
three-fact model.

⚠ **Site 4 also carries obligations that are NOT being relaxed** — the
`covered ⊄ required` (wrong-owner) and `required ⊄ covered` (omission)
directions. ⛔ Those stay exactly as strict as they are.

## 4. ⭐⭐ A LANDED CONTROL PINS THE OVERSTRICT PROPERTY IN ITS OWN `CLAIMED` BLOCK

`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs:8531`,
`d8_every_required_join_plan_is_consumed_exactly_once`, promise class **durable
invariant**. Its `CLAIMED` block says, verbatim:

> *"every required planned FunctionizedUnits source join is consumed exactly
> once, and the retained active-recursor lane **cannot emit a case join and later
> disposition that case as statically unselected**."*

⇒ ⭐ **That second clause is precisely the property the ruling calls
phase-overstrict.** So this repair **necessarily edits a landed durable-invariant
control**, and the frame authorizes that edit rather than leaving the ring to
choose between blocking and weakening a test quietly.

**The edit is surgical and its shape is fixed:**

- ✅ **REPLACE** the "cannot emit then disposition" clause with the
  materialized-but-dead proof obligation, and update the `MEASURED` and
  `THE GAP` blocks to describe what the test now actually exercises. ⛔ A
  `CLAIMED`/`THE GAP` block left describing the old property is a false
  residual — nobody audits a confession, and this one would read as honest.
- ⛔ **RETAIN** every other direction the test proves: exactly-once consumption,
  omission closure, wrong-owner closure, and the recursive-revisit union edge.
- ⛔ **DO NOT DELETE THE TEST** and do not narrow it to the passing fixture.

## 5. ⚠ THE MUTATION THAT IS AT RISK FROM THIS REPAIR

Five `JoinConsumptionMutation` modes (`mod.rs:838`) currently red, and the repair
must keep all five red:

`SkipFirst` · `DuplicateFirst` · `IncludeStaticallyUnselected` ·
`OmitFirstStaticallyUnselectedMatchCase` ·
`OmitSourceMachineComputationalMatchSelection`

⭐ **`IncludeStaticallyUnselected` is the one this repair can silently defeat.**
It makes `disposition_statically_unselected_source_subtree` a **no-op**
(`mod.rs:1700-1705`), so joins in dead subtrees are never dispositioned. It must
red via site 4's **omission** direction (`required ⊄ covered`) — ⛔ **not** via
the overlap direction, which this node is relaxing. **Confirm which direction
actually reds it, before and after.** If it currently reds only on overlap, the
repair removes the mutation's only witness and the control becomes vacuous.

⚠ **This is the whole risk of the node in one paragraph.** Relaxing a
fail-closed check is the direction that loses defects, and a mutation that stops
reding is how you find out you lost one — **if you check.**

## 6. Deliverables

- **`D1` — Enumerate before repairing.** Record which of the four sites in `§3`
  each of the five mutations in `§5` currently reds at. ⛔ Do not edit production
  code until this is posted. This is the baseline that makes `AC-4` meaningful.
- **`D2` — Materialization fact.** A planned join token is consumed/emitted at
  most once, **owner-bound**, failing closed on duplicate or wrong-owner
  consumption. ⛔ Unchanged in strictness; only its *meaning* narrows to
  materialization.
- **`D3` — Final semantic disposition.** After the generated function's
  reached-case union closes, every planned join is classified **exactly once** as
  reachable or statically unselected, **in the same function/owner context**.
- **`D4` — Materialized-but-dead proof.** Overlap is permitted **only** when the
  emitted join/block is unreachable from the generated-function entry **and**
  retains no live predecessor input and no reachable use. Otherwise fail closed.
  Validate against the **completed Cranelift CFG/SSA** at the appropriate
  pre-seal or whole-IR boundary. ⛔ **A disposition bit is not CFG repair.**
- **`D5` — The four sites of `§3`**, each re-derived against `D2`/`D3`/`D4`.
- **`D6` — The landed control**, edited per `§4`.
- **`D7` — A causal control for this exact ordering**: a join materialized before
  its enclosing match selects, whose case is then proved dead — plus a mutation
  that makes the dead join **reachable**, or leaves a **live incoming edge or
  use**. ⭐ **That mutation must red.**

## 7. Acceptance criteria

- **`AC-1` (the row that opened the node).**
  `scripts/ken-cargo test -p ken-cli --test rt_span_prov_native` is **6/6**,
  including `sp_a_foreign_span_freeze_rejects_own_span_succeeds_on_both_engines`,
  on a tree carrying this repair. ⚠ Measure it on **this node's own branch
  first**; the WIP `8bc7556a` is held and is not this node's input.
- **`AC-2`.** `D7`'s causal control passes, and its mutation **reds**. A control
  that passes without a reding mutation proves nothing about a relaxed check.
- **`AC-3`.** All five `§5` mutations still red, and `D1`'s table records the
  site each reds at **before and after**. ⛔ A mutation that changes which site
  reds it is a reportable finding, not a detail.
- **`AC-4`.** The `§4` control's `CLAIMED` / `MEASURED` / `THE GAP` blocks
  describe the property the test now exercises. ⛔ Stale prose here fails this AC
  on its own.
- **`AC-5` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).

⭐ **Every AC above is behavioural** — a suite result, a mutation flipping red, a
compile. ⛔ **No AC asserts a source line, a symbol name, or a doc-comment
string** (operator: *"tests that assert facts about source code, catalog, or
documentation lines are an invitation for failure and delay"*). `AC-4` is a
**review** obligation on the QA seat, not a grep oracle.

## 8. ⛔ Banned scope

- ⛔ **A bare set flip.** The Architect's diagnostic differential reclassified the
  materialized join as dead and the fixture passed — **that is a diagnosis, not
  the repair.** It is explicitly not authorized as production code.
- ⛔ **Weakening owner validation.** Duplicate and wrong-owner consumption stay
  fail-closed.
- ⛔ **Keying a path-sensitive selection globally across generated functions.**
- ⛔ **Deleting the reached-case union discipline** (`mod.rs:1690-1700`,
  `:1750`, `:1785`). It defends a different hazard and it is not implicated.
- ⛔ **Touching [[NATIVE-HANDLE-CARRIER]]'s WIP** `8bc7556a`, its branch, or its
  six files. This node is repaired and merged independently of it.
- ⛔ **Widening into [[RT-DECL-CLOSURE-PORT]]** (`lowering/core.rs`'s residual
  selector). Different seam, separate node, and it is next in the queue.

## 9. Hard stop

Report and stop if `D4` cannot be discharged at a pre-seal or whole-IR boundary
without a rebuild whose cost exceeds this node's size, or if `AC-1` still fails
after `D5`. ⛔ Do not delete a mutation or narrow the `§4` control to reach green.

**Counter:** the §5a count of record is **21**, carried on
`docs/program/issues/RT-JOIN-DISPOSITION.md`; entries **12**; the next research
pull is **#24**. ⏳ The 12th-entry predicate check is owed by the Architect.

## 10. What landing this closes

[[NATIVE-HANDLE-CARRIER]] resumes **from the preserved WIP `8bc7556a`** and
re-runs the full 6/6 `rt_span_prov_native` module plus the already-named
CAP-41 / AC-5 / private-public controls and mutations. ⛔ **No honest partial is
authorized there** (standing Architect ruling). That closes
[[PX8-F-CAP-41]].

⭐ **It also removes a latent false-rejection from `main`** ahead of
[[RT-DECL-CLOSURE-PORT]], which moves whole objects onto the `FunctionizedUnits`
route whose per-function join accounting this repairs.
