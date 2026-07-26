---
id: LIB-GATE-DECOUPLE
title: "main is red on two library documentation-census gates: the currency gate the operator decoupled from merges still fires from inside CI, and a doc-only merge invalidated the ledger unreported"
status: ready
owner: verify
size: S
gate: none
depends_on: []
blocks: [KW-ORACLE-REMOVE]
github: null
origin: Surfaced 2026-07-26 when PR #1035 (KW-ORACLE-REMOVE, exact 68c3d870, Architect-approved) failed CI on two tests in a different crate. Steward measured the failure at origin/main=11b21039 WITHOUT the candidate — 29 passed, 2 failed — proving the red pre-exists. Root cause is Steward's own doc-only merge 95bc855c (PR #1031). Steward-filed per COORDINATION §2.
---

> ## ⛔⛔ `main` IS RED AND THE BLOCKED CANDIDATE IS INNOCENT
>
> `KW-ORACLE-REMOVE` exact `68c3d870` is **correct, Architect-approved
> (`dec_200k7z471z9x6`, `resolved`), and cannot land** until this clears. ⛔ Do
> not send its ring to repair anything.

## 1. The measurement — the red PRE-EXISTS the candidate

Run at `origin/main = 11b21039`, which does **not** contain the deletion:

```
scripts/ken-cargo test -p ken-cli --test library_documentation_gates
test result: FAILED. 29 passed; 2 failed; 0 ignored
```

| failing test | asserts |
|---|---|
| `registered_record_validation_gates_run` (`:1048`) | shells `scripts/gen-doc-status.sh --check`; fails on **12 cited sources drifted from their attestations** |
| `agent_library_manifest_schema_contract_and_measurements_hold` (`:3440`) | `library/agents/tasks/author-package.md` **`measured_tokens` declares 480, recomputes 459** |

⭐ **This is the positive control the diagnosis rests on:** the same suite, at a
tree with no deletion in it, already fails these two and passes the other 29. ⇒
The candidate cannot be the cause.

## 2. ⛔ THE CAUSE IS ONE MERGE, AND IT IS MINE

`95bc855c` (PR #1031, `DOC-CATALOG-CONTENTS`, published **`--doc-only`**):

- edited **11 cited catalog sources** + `docs/program/07-catalog-style-guide.md`
  without regenerating `library/SOURCE-ATTESTATIONS` (last written at
  `4c2d9529`, which is **earlier**);
- edited `library/agents/tasks/author-package.md` without recomputing its
  `measured_tokens`.

⛔ **`--doc-only` skips CI polling, so neither was reported.**

### ⭐⭐ THE DEFECT THAT MATTERS: I diagnosed ONE of three consequences

That merge broke **three** things. I found the one that happened to surface —
the `kw_theorem` source oracle — because an implementer measured it while
blocking an AC of mine. I then wrote into `KW-ORACLE-REMOVE`: *"Every
non-doc-only merge is now blocked behind this deletion."*

⛔ **True and INCOMPLETE. The deletion is necessary, not sufficient.** Two other
consumers of the same tree broke silently and I never enumerated the consumer
set at all — I treated the failure I could see as the failure there was.

⭐ **The generalizable shape: a diagnosis keyed on ONE reporting mechanism is
blind to every other consumer of the same change.** The oracle reported; the
attestation ledger and the token census did not, because nothing runs them on a
doc-only path. ⇒ ⛔ **After any `--doc-only` merge, enumerate consumers of the
touched paths — do not wait to be told.**

## 3. ⛔ THE PUBLISHER PRINTED A FALSE GREEN, TWICE

The operator removed the library currency gate from the publisher on 2026-07-26
(*"no remove it. it's just friction… including it as a CI-type system induces
coupling that causes just the sort of slowdown and waffling"*). The **check** was
removed. The **success sentence** was not:

> `Post-merge verification: … and the currency checker is green on origin/main.`

⇒ From that moment the publisher asserted a check **that no longer ran**, on
every publish. It printed green for **#1031 while #1031 was breaking the
ledger**, and again for **#1034**. I acted on that sentence both times.

✅ **FIXED — `b4cf8df5`.** The message now states only the tree-OID comparison it
performs and says explicitly that no currency check ran.

⭐ **The lesson is not "the removal was wrong" — it was the operator's and it was
right. It is that a message is part of a gate's surface.** Deleting a check and
leaving its success line converts a real signal into a false one, which is worse
than having neither. ⛔ **Remove the claim with the check, in the same edit.**

⚠ **And the publisher had already predicted this exact incident.** Its own
comment records `a5d3a13b` on **2026-07-22**: a doc-only merge that broke a
citation gate, left `main` red ~25 minutes, and *"surfaced on the next `crates/`
PR, where it read as that PR's own failure."* ⇒ **Second occurrence, four days
later, same shape.** The guard that would have caught it is the one that was
removed; the comment describing it survived and I did not read it before
publishing.

## 4. ✅ RULED — OPERATOR, 2026-07-26: **"remove the CI coupling."**

⇒ **Option (a). The fork below is closed**; it is retained because it records
what was weighed. ⛔ **Do not regenerate the ledger** — that was option (b) and
it was not chosen.

### ⛔ SCOPE IS THE WHOLE COUPLING, NOT THE TWO TESTS THAT FIRED

The two failures in §1 are the two CI **happened to name today**. ⛔ Removing
only those leaves the coupling live and the next doc edit reopens this node.

### ⭐⭐ THE MEASUREMENT — and read the trap before you trust any of it

`crates/ken-cli/tests/library_documentation_gates.rs` — **4061 lines, 31 tests.**

| set | count | standing |
|---|---|---|
| **confirmed coupled** — failed at a tree with no candidate in it | **2** | ✅ measured |
| **reach `repo_root()`** (upper bound on the coupling) | **24** | ⚠ **over-broad, see below** |
| **provably fixture-only** | **7** | ✅ no live-tree read on any path |

⛔ **`reaches repo_root()` IS NOT `asserts a fact about the live tree`.** Several
of the 24 — `ledger_rejects_a_duplicate_path_row`,
`content_currency_gate_rejects_a_drifted_cited_source_and_recovers`, the
`shallow_clone_self_heals_*` pair — read the live tree only to **seed a scratch
fixture** (`build_currency_fixture`, `build_synthetic_origin`) and then test the
generator's *behaviour* on that fixture. Those do **not** break when a document
changes. ⇒ **24 is an upper bound; 2 is the confirmed floor. The true set is
between them and must be measured, not inferred.**

### ⛔⛔ THE TRAP THAT COST ME TWO WRONG ANSWERS — do not repeat it

I built a static call-graph classifier twice, and **both runs mis-classified
`registered_record_validation_gates_run` as fixture-only** — the very test whose
CI failure had already named it. Its body is:

```rust
#[test]
fn registered_record_validation_gates_run() {
    for gate in VALIDATION_GATES { (gate.run)(); }
}
```

⇒ It reaches the live tree through an **11-row static function-pointer table**,
so there is **no textual call site** for any classifier keyed on `name(` to find.
⛔ **This is not a regex bug — a static call graph is structurally blind to
indirect dispatch.** ⭐ **I only caught it because the CI failure gave me a case
whose answer I already knew.** With no known-answer case, both wrong lists would
have looked complete and I would have handed this ring a frame built on one.

### ▶ USE A BEHAVIOURAL DISCRIMINATOR, NOT A STATIC ONE

⭐ The operator's own rule *is* the discriminator, and it is directly executable:

> *"Does an edit that changes nothing about how any program behaves make this
> test fail?"*

⇒ In a scratch copy, **perturb a documentation line** — a cited catalog source,
and separately a `measured_tokens`-bearing agent module — and run the suite. **The
tests that flip red are the coupling.** That is a positive control by
construction, and it answers the question the static analysis cannot.

⛔ **Report the count as a floor, with the perturbations you ran named.** A test
coupled to some *third* doc-shaped input neither perturbation touches will not
flip, and your set will look complete when it is not.

## 4a. ▶ THE FORK AS IT STOOD — retained for the record, NOT open

⛔ **The gate the operator decoupled from merges is still firing, one layer
down.** `registered_record_validation_gates_run` shells out to
`gen-doc-status.sh --check` — the same currency gate, running **as a CI test**.
⇒ Removing it from the publisher did not remove the coupling; it moved the
firing from *before* the merge to *after*, where it lands on an innocent PR.

⚠ Both failing tests also assert facts about **documentation lines and token
counts**, which the operator's standing test policy puts out of bounds: *"Test
oracles that assert facts about source code, catalog, or documentation lines are
an invitation for failure and delay. Tests should focus on behavior."*

| option | cost |
|---|---|
| ✅ **(a) remove the CI coupling** — **CHOSEN**; consistent with **both** operator rulings | the coupled set must be *measured*, not guessed — §4 |
| ⛔ **(b) regenerate the ledger** — **NOT chosen** | needs a **real Librarian revalidation**, which is exactly the *"full re-validation round on the publish path"* the operator called friction |

⛔ **DO NOT run `scripts/gen-source-attestations.sh` to clear this.** It would
turn the gate green while asserting a corpus revalidation **that never
happened** — the ledger's claim is that citing library docs were checked against
the new source content, and a generator run checks nothing. ⭐ Under ruling (a)
the ledger's staleness stops being a gate at all, so there is nothing to clear.

### ⚠ What genuinely goes away with the gate, stated so it is a decision

The generators are **kept** (`gen-source-attestations.sh`, `gen-doc-status.sh`) —
the operator's position is that the ledger is produced **at version release
points**, not enforced per-merge. ⇒ Between releases, a `library/` page may cite
a source that has since changed and **nothing will say so.** That is the accepted
cost of decoupling, ⛔ not an oversight to re-litigate inside this node.

## 5. ⚠ THE HONEST RESIDUAL

⛔ **I have not enumerated the full consumer set of `95bc855c` even now.** I found
three consequences; the surface that bounds "three" is unwritten. ⇒ **Treat 3 as
a floor.** Whoever takes this node should ask what else reads `catalog/`,
`docs/program/`, or `library/agents/` and is not exercised on a doc-only path —
and ⛔ should not assume the answer is empty because these two tests were the
ones CI happened to name.
