# LET-3 Phase 2 · The `from_list_acc_lookup` family — one proof, four times

**Owner:** Team Foundation · **Size:** L · **Branch:**
`wp/let3-p2-map-acc-lookup` · **Base:** current `origin/main`
**Gate:** Foundation QA + **Steward** · **CI:** **FULL** (touches `catalog/` —
never `--doc-only`)

**Predecessor:** the LET-3 pilot (`7071f919`), merged and reviewed. Its
prediction held: the local aliases were definitionally equal to the
originals and **no transport lemma was needed**. That stop-condition
carries forward unchanged.

## 0. The scope is a PROOF FAMILY, not a file

**Phase 2 is not "`Map.ken.md`."** The pilot frame said "family by family" and
then named the file, which is not a scope — it is 15,352 lines and 403
declarations, and a WP pointed at a file invites exactly the blind sweep the
pilot existed to prevent.

**The scope is the `*_from_list_acc_lookup_*` family: 34 lemmas, 3,442 lines,
22% of the file.** Derived by enumeration, not by reading around:

| sub-family | lemmas | first | the 100+ line members |
|---|---|---|---|
| `delete_from_list_acc_lookup_*` | 11 | `:6167` | `_other_assoc_miss` (160), `_locality_dispatch` (140), `_other_assoc_hit_survivor` (131) |
| `union_from_list_acc_lookup_*` | 5 | `:10218` | **`_assoc_hit` (237 — the exemplar)**, `_assoc_miss` (179), `_assoc_inner` (105) |
| `intersection_from_list_acc_lookup_*` | 9 | `:11363` | `_some_hit` (165), `_locality_dispatch` (140), `_none_dispatch` (117), `_some_miss_dispatch` (112) |
| `difference_from_list_acc_lookup_*` | 9 | `:12397` | `_keep_miss_dispatch` (206), `_keep_hit` (183), `_locality_dispatch` (140), `_none_dispatch` (117) |

Everything else in `Map.ken.md` — the other 369 declarations, including the
`law5_node_*`, `cat4_bool_or`, and `set_*_law` spans — is **out of scope and
must not be touched.** They are their own families and their own later WPs.

## 1. ★ Why this family, and what the deliverable actually is

**The four sub-families are the same proof.** Each walks an accumulator through
`from_list`, dispatches on a lookup, and relates the tail's association to the
updated accumulator. They differ in *which* dispatch and *which* update — and in
nothing else structurally.

**Today that is invisible.** The exemplar (`union_…_assoc_hit`, `:10218`, 237
lines) repeats `insert_with_fold_step …` **14×**, `lookup k v leq query acc`
**9×**, and `assoc k v leq query xs2` **5×**. Those strings are not incidental:
they denote **the updated accumulator**, **the original lookup result**, and
**the tail's association**. A reader re-derives each of those, at each
occurrence, in each of four sub-families, forever.

> **⇒ THE DELIVERABLE IS A SHARED VOCABULARY, NOT 34 INDIVIDUALLY NICER**
> **PROOFS.**
>
> If this WP lands 34 locally-improved proofs that each invented their own names,
> **it has failed** — even if every name is defensible and every gate is green.
> The whole return on naming here is that after it, **`delete`, `union`,
> `intersection`, and `difference` visibly read as one proof with four
> dispatches.** A per-lemma pass destroys precisely the thing worth having.

**So: one naming table for the family, declared before the first edit, applied
uniformly.** Where a sub-family genuinely lacks a concept (there is no
`updated_acc` in a proof that does not update), it **omits** that name — it does
not rename it.

## 2. ★★ THE PARALLELISM LICENSE — bounded, because I opened this door myself

The pilot review produced a calibration ruling I must now state as law, because
Phase 2 is where it gets abused. In the pilot I accepted a **one-use** binding in
a `True ↦` arm **solely so it would read in parallel with the `False ↦` arm**,
which had genuinely earned its bindings through repetition.

**At 34 lemmas across four parallel sub-families, "cross-family parallel
structure" would justify every binding anywhere.** That is the pilot frame's
"a blind sweep answers yes everywhere" failure, arriving through the one
door I left open. So the license is **derivative, and it can never originate
a binding**:

> **A name must be EARNED by repetition or by semantic weight in at least one
> member of the parallel set.** Once earned, it **may** be mirrored into the
> sibling position where it happens to be used once, so the set reads alike.
>
> **If NO member of a parallel set earns the name, the whole set stays
> unbound.** Parallelism propagates an earned name. It does not create one.

**The tell that this rule is being violated:** a binding whose justification
is *"the other three have it."* Trace it back. If the chain of "the other three
have it" never terminates in a member where the name is used more than once or
names a concept the reader would otherwise infer, **delete the entire set.**

## 3. Ordering — union first, and it is a CALIBRATION GATE

1. **`union_from_list_acc_lookup_*` (5 lemmas, incl. the 237-line exemplar).**
   Derive the naming table here. This sub-family is the report's worked example
   and has the densest repetition, so it is where the names are actually earned.
2. **STOP. Self-review against §5 AC7** — per binding, answer *"what concept
   does this name that the reader would otherwise have had to infer?"*
   Delete every binding you cannot answer for. **Then freeze the table.**
3. **Apply the frozen table to `delete`, `intersection`, `difference`** (29
   lemmas). This step is largely mechanical *by construction* — that is the
   point. **If it is not mechanical — if a sub-family keeps wanting a name the
   table does not have — that is a REPORT, not a local decision** (see §6).

First pass names **stable semantic states only** — `entry_key`, `entry_value`,
`updated_acc`, `tail_assoc`, `lookup_before`. **Proof evidence gets named
after, and only where a nested `trans` still forces the reader to reconstruct
its role.** The exemplar has 7 `trans` and 3 `cong`; expect some, not all, to
earn a name.

## 4. Fixed inputs — and every one of them is PERISHABLE

**Treat every anchor below as perishable. Re-derive it against the landed
code before you edit a line. If a fixed input is false, SAY SO AND ESCALATE —
do not quietly build around it.** (Language did exactly this on AX-2 today and
corrected my inventory before it cost anything. That is the clause working.)

- **The syntax is LET-4's `;`-separated binding group** — sequential,
  non-recursive, earlier names visible to later right-hand sides, duplicates
  rejected. **Maximal coalescing at two or more bindings; a single binding
  stays a plain `let`.** Read `catalog/guide/surface-reference.ken.md` and
  your team overlay first.
- **The line numbers and counts in §0 are as of `179efdc6`.** They will shift
  the moment you edit. Anchor on **names**, never on line numbers.

## 5. Acceptance criteria

- **AC1 — public surface is BYTE-IDENTICAL in meaning.** Every declaration,
  result type, and proof claim preserved. **A `let` refactor renames nothing the
  outside can see.**
- **AC2 — ZERO `trusted_base()` delta. No new `Axiom`.**
  `crates/ken-elaborator/tests/map_build_acceptance.rs` already asserts
  `trusted_base_delta` on `empty`/`to_list`/`fold` and must stay green
  unchanged. **Do not weaken or re-baseline that assertion.**
- **AC3 — the REAL ordered dependency harness runs.** `Map.ken.md`'s landed
  oracles are **`crates/ken-elaborator/tests/map_build_acceptance.rs`** and
  **`crates/ken-elaborator/tests/es2_acceptance.rs`**. A standalone `ken
  check` is **not** the gate.
- **AC4 — the two CORPUS-WIDE formatter oracles must pass.**
  **`crates/ken-elaborator/tests/kenfmt_c_capstone.rs`** and
  **`crates/ken-elaborator/tests/kenfmt_b4_splicing.rs`** both **glob
  `catalog/`** and therefore both read `Map.ken.md`. They live in crates this
  WP never touches, so **targeted per-crate validation cannot see them** —
  run them explicitly.
  (`crates/ken-cli/tests/ken_fmt.rs` is fixture-based and does **not** cover
  the catalog; it is not a gate here. Re-derive this — the oracle set is
  exactly the thing that has surfaced as red CI *after* review before.)
- **AC5 — VERIFY the conservativity; do not ASSUME it.** *"A `let` rewrite in a
  pure proof is normally definitionally conservative"* — **normally is not a
  proof.** Show the elaboration still closes and behavior is unchanged. **This is
  an AST change wearing whitespace's clothes.**
- **AC6 — INSPECT THE FORMATTED OUTPUT.** `ken fmt --check` returning
  "canonical" is **NOT a readability verdict.** Paste the formatted exemplar
  (`union_from_list_acc_lookup_assoc_hit`) and **one** `difference_*` member
  into the handoff so I can read them side by side. **The point of the pair
  is that I should be able to see they are the same proof.**
- **AC7 — the family is REVIEWABLE AS PROSE.** The handoff carries **the
  naming table** and, per name: *what concept does this name that the reader
  would otherwise have had to infer, and in which member did it earn its keep
  (§2)?* If you cannot answer for a name, **delete it everywhere.** *That
  question IS the rule, and it is the whole acceptance test.*
- **AC8 — no-regression is GREEN IN CI**, never a local `cargo test
  --workspace` (operator hard rule, `COORDINATION.md §12`). Locally: targeted
  only.

## 6. ⛔ Guardrails

- **⛔ SCOPE IS THE 34-LEMMA FAMILY.** No other declaration in `Map.ken.md`.
  No other file. If a neighbouring proof looks tempting, **it is a later WP.**
- **⛔ Do NOT bind for length.** `tmp`, `x2`, `value2`, `intermediate`,
  `step_result` — **if the name carries no more vocabulary than its RHS, the
  binding is a NET LOSS** and must not land.
- **⛔ Do NOT hoist across a branch.** Ken is **call-by-value**. **Bind at the
  narrowest scope containing every use.** A binding that changes evaluation
  placement is a **bug**, not a style.
- **⛔ Do NOT add a top-level helper** for a concept that only narrates one
  body. **But if the frozen table keeps failing to cover a sub-family — if you
  find yourself wanting a fifth, sixth, seventh name that union never needed —
  STOP AND REPORT IT.** That is not a naming gap; it is the family telling you
  it is **not actually one proof**, and it is a decomposition signal I want to
  hear about, not have worked around inside a `let`.
- **⛔ If you need a transport lemma, STOP AND REPORT.** The pilot predicted
  the aliases would be definitionally equal and it held. If that breaks here,
  something is wrong and I want to know before you build around it.
- **⛔ Do NOT touch `crates/`.** If the formatter misbehaves on a binding
  group, that is a **LET-4 regression — STOP AND REPORT IT.** Do not
  hand-format around it.
- **⛔ NEVER `--workspace`** locally (operator hard rule,
  `COORDINATION.md §12`).
