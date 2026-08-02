# WP frame — `DOC-AGENT-CITE`

Node: `docs/program/issues/DOC-AGENT-CITE.md`. Owner: doc ring. Authority:
`DOC-W6-AGENT-EVAL` `D5`, merged `723989ba`. Measured at `main = 579d369e`.

Wave 6 found one defect seven times: the core modules name normative
authorities as a reading list, so a cold agent asserts past what it loaded.
This WP binds each authority to the **claim class** that requires it.

## The four judgments this frame makes, so you do not have to

### 1. The obligation is CLAIM-TRIGGERED, and that constraint is load-bearing

`D2`'s rule is claim-sensitive: *a source need not be cited for a claim the
answer does not make.* Your edits must preserve that shape.

⇒ **A module that says "always load `spec/40-runtime/42-evaluation.md`" is a
regression, not a fix.** `unnecessary_loads` scored an exact `[]` on all seven
runs — that axis is clean and a blanket instruction is the one change most
likely to dirty it. You would trade a `partial` on one axis for a failure on
another and the suite would still not be ready.

Every rule you write has the form *if the answer claims X, load Y before
asserting it* — never *load Y*.

### 2. Routing rules, NOT new exposition

These five modules load into the context of every cold agent that selects a
pack containing them. `write-pure` alone pulls four of them.

⇒ **The modules must not grow materially.** If a module gains 40 lines of
explanation about why citation matters, you have made every pack more expensive
to buy a behaviour that one sentence in the right place would induce. Prefer
rewriting an existing authority section over appending a new one. `D3` records
the cost so this is visible rather than assumed.

### 3. This WP spends NO fixture and re-runs NOTHING

All seven fixtures are burnt. Minting semantically equivalent variants is the
Librarian's content judgment under `DOC-W6-AGENT-EVAL` hard stop 2, and it is
not this node's cost.

⇒ Verification is `D4`: a paper check against the **seven recorded answers**,
which the Wave 6 record preserves in full along with the exact authority each
one omitted. That is a real control and it is free. **A re-run proposal is hard
stop 1.**

### 4. `D2`'s rule is the target, not a thing to negotiate

The modules move toward `D2`. `D2` does not move toward the modules. If a
module cannot induce a citation `D2` requires without becoming unreasonable,
say so in `D5` as a finding about the rule — do not soften the module to meet
it halfway and do not edit `D2`'s recorded text.

## Fixed inputs

Measured at `main = 579d369e`.

| input | measured value |
|---|---|
| the finding | `docs/program/wave-6-agent-evaluation.md` `D5`, and the seven per-task `Independent score` paragraphs naming each omitted authority |
| the rule | that document's `D2` — five authority classes, claim-sensitive completeness |
| the verdict | `library/agents/evaluations/results-2026-08-02.toml`, `agent_core_ready = false`, `derived_library_substitutions = 0` |
| `core/read-ken.md` | 89 lines, blob `a4d9f82e` |
| `core/proof-and-trust.md` | 89 lines, blob `9032ad37` |
| `core/write-ken.md` | 114 lines, blob `eb66dd00` |
| `core/toolchain.md` | 89 lines, blob `f64437b0` |
| `tasks/effects-and-capabilities.md` | 87 lines, blob `68c9b421` |
| pack closures affected | all six; `write-pure` selects four of the five |

```sh
git rev-parse --short HEAD
wc -l library/agents/core/*.md library/agents/tasks/effects-and-capabilities.md
```

## Deliverables

- **D1 — the claim-class routing table.** For each of the five modules: the
  claim classes it governs, and for each class the exact authority `D2`
  requires. Derived from `D2` plus the seven score paragraphs, not invented.
  A claim class that no module governs is a finding, not a gap to paper over.
- **D2 — the module edits.** The five files, each stating its routing
  obligations in claim-triggered form. Rewrite in place where an authority
  section already exists (`read-ken.md` §9 is the clearest case).
- **D3 — the context-cost record.** Per module, lines before and after; per
  pack, the total line count of its resolved closure before and after. State
  the numbers plainly including any growth.
- **D4 — the paper check.** For each of the seven recorded Wave 6 answers, name
  the authority it omitted, and state whether the revised module would have
  obliged the agent to load it **before** making the claim it actually made.
  Quote the governing sentence you added. **A "no" is a valid and useful
  result** — it says the routing rule does not reach that claim class.
- **D5 — residual findings**, recorded only: any claim class no module can
  reasonably induce, and any place `D2`'s rule looks wrong rather than the
  module. No edit follows from `D5` in this WP.

## Acceptance criteria

- **AC-1 — every rule added is claim-triggered.** *Control:* read each added
  rule for an antecedent naming the claim class. A rule with no antecedent
  fails this AC — it is the blanket-load regression judgment 1 names.
- **AC-2 — `unnecessary_loads` cannot be inflated by construction.** *Control:*
  no added rule directs a load that is unconditional, and `D4` states for each
  of the seven answers whether the new rule would have caused a load beyond
  what its claims required. Any "yes" is a finding to resolve before candidate.
- **AC-3 — every omitted authority in the Wave 6 record is addressed or
  explicitly declined.** *Control:* `D4` against the seven `Independent score`
  paragraphs. An omitted authority absent from `D4` fails this AC.
- **AC-4 — no module grows more than is needed to carry its routing rules.**
  *Control:* `D3`'s per-module and per-pack numbers, stated. This AC is about
  visibility: unreported growth fails it, reported growth does not.
- **AC-5 — no pack `includes`, task file, fixture, or historical result is
  edited.** *Control:* the candidate's path list — nothing under
  `library/agents/packs/`, nothing in `library/agents/evaluations/`, no fixture.
- **AC-6 — `D2`'s recorded rule is byte-unchanged.** *Control:*
  `docs/program/wave-6-agent-evaluation.md` blob against the merge base.
- **AC-7 — no run, no fixture spent, no new results artifact.** *Control:* the
  path list and the absence of any `results-*.toml` change.
- **AC-8 — no normative claim is introduced into `library/`.** §4c: no
  `library/` page is normative. A routing rule points AT the normative source;
  it does not restate the rule the source contains. *Control:* each added
  sentence names a path rather than asserting a language fact.

## Banned scope

- **No re-run and no fixture.** Hard stop 1 (judgment 3).
- **No edit to `D2`, to `results-2026-07-24.toml`, or to
  `results-2026-08-02.toml`** (`AC-6`, `AC-7`).
- **No pack `includes` change.** `D5` may observe that a pack selects the wrong
  module; it does not act on it. Widening a pack with derived references is
  specifically what `D5` advised against.
- **No new core module.** Five modules exist and each already owns its claim
  classes; a sixth splits the authority the fix is trying to concentrate.
- **No normative content in `library/`** (`AC-8`).
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). `D3` is a recorded measurement, not a gate.

## Contention

`library/agents/` and `docs/program/` only. No `cargo`, no build lock, no
contention with the runtime ring.

## Sizing

**Size `M`.** Five files, and the routing table is the thinking. `D4` is
mechanical against a record that already names every omission. The risk is not
volume — it is judgment 1, writing a rule strong enough to induce the citation
and narrow enough not to induce a load the answer never needed.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. `D1` the routing table. **Stop here** — if the table binds an authority to
   the wrong claim class, every edit downstream inherits it.
2. `D2` the five module edits and `D3` the cost record.
3. `D4` the paper check and `D5` residual findings.

**Expect to end your turn at each checkpoint.** If any checkpoint runs past an
hour, stop and route.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A deliverable seems to need a cold run.** It does not. Say what you wanted
   to measure and stop.
2. **A claim class has no module that could reasonably govern it** — the
   authority is real, `D2` requires it, and no existing module is the right
   home. That is a structural finding and possibly a sixth module, which is
   banned here on purpose.
3. **A routing rule cannot be written without restating a normative rule** in
   `library/`. §4c forbids that, and the workaround is a pointer, not a
   paraphrase. If a pointer genuinely will not carry it, stop.
4. **`D4` shows the revised modules would not have changed a majority of the
   seven answers.** That means the routing model is wrong, not that the rules
   need strengthening, and strengthening them anyway will breach `AC-2`.
5. **`D2`'s rule looks wrong** for a specific claim class. Record it and stop
   rather than adjusting a module to fit a rule you believe is mis-stated.
