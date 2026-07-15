# LET-2 · The local-binding convention — and the guidance loop that lost it

**Owner:** Spec enclave (spec-author drafts · spec-leader reviews · CV validates
every checked example actually elaborates) · **Size:** M
**Branch:** `wp/let2-local-binding-convention` · **Base:** `origin/main @ 2c184550`
**Gate:** spec-leader + CV + **Steward terminal** (you edit `agent/playbooks/` and
`agent/teams/` — that corpus is mine and its routing is mine)
**Doc-only. Runs in PARALLEL with LET-1. Blocks LET-3.**

**Source:** operator-commissioned report,
`local/ken-let-authoring-style-report.md` (2026-07-14). **Every claim below is
re-verified at the emission. Build from THIS frame.**

## 0. ★ The finding, and why it is a GUIDANCE bug, not an authoring bug

**There are ZERO local `let` bindings in the entire catalog.** I counted:
**27,404 tangled Ken lines, 32 files, exact ```` ```ken ```` fences — 0 uses.**
Not "rare." **None.**

**Do not read that as 32 authors independently exercising bad taste.** Read the
loop that produced it — **I verified all three links:**

| Link | Status |
|---|---|
| `agent/playbooks/tools/write-ken.md` — the skill an agent reads immediately before authoring | **mentions `let` ZERO times** ✅ verified |
| `agent/teams/foundation/` — the catalog-authoring overlay the campaign promised, which `moot.toml` already loads "if present" | **`agent/teams/` DOES NOT EXIST AT ALL** ✅ verified |
| `catalog/guide/` + the landed catalog — the corpus agents learn practice from | **contains not one authored `let` to imitate** ✅ verified |

**⇒ The agents are faithfully imitating the examples and instructions they were
given. The corpus is the guidance's own reflection.** This is the project's
authoring model working *exactly as designed* — and pointed at a hole.

**That is why this WP is not "add a style rule." It is: close the loop that
would swallow the rule.** A convention landed in the style guide alone would be
invisible to the very agent it is written for, and we would be back here in a
month wondering why the corpus still has no `let`.

## 1. The rule — normative, and it is a JUDGMENT rule

**Pinned. Do not weaken it into a metric, and do not strengthen it into a quota.**

> Use `let` to give a local name to an intermediate term **when the name states a
> domain concept, proof endpoint, invariant, or stage that is otherwise visible
> only as nested syntax.** A useful local name should let the reader describe the
> remaining body **at a higher level than the RHS.**
>
> **Prefer a binding** when a non-atomic expression is repeated; when a proof
> combinator chain repeats its middle endpoints; or when a single-use stage has an
> important domain role. **Keep a familiar one-step expression inline** when a
> binding would only rename its syntax.
>
> **Bind at the narrowest scope containing every use.** Do not hoist a
> branch-specific computation before a `match`, and **do not move an effectful
> computation across a branch or another effect.** Local `let` is
> **non-recursive** — use a top-level helper for recursion or genuine reuse.
>
> **Name the ROLE, not the mechanism.** `sorted_tail`, `updated_acc`,
> `left_round_trip`, `lookup_after_insert` are informative. **`tmp`, `value2`,
> `x2`, `intermediate`, `step_result` are not** — they replace visible syntax with
> indirection and are worse than no binding.
>
> **A long preamble of bindings is itself a decomposition signal.** If a reader
> must retain many unrelated local names, **split a helper or a lemma instead** of
> building a local namespace. But do **not** mint a top-level helper for a concept
> that only narrates one body.

**⛔ THE COUNTER-RULE IS LOAD-BEARING AND MUST SHIP WITH IT.** The report is
emphatic and so am I: **the survey does NOT support "nested application implies
`let`."** Several direct forms are *clearer* staged-free — small exhaustive
matches like `list_eq`/`list_compare` expose the recursion directly; a single
constructor assembly reads better inline; a one-step `cong` is clearer with all
endpoints visible. **`char_at`, `eq`, and `compare` sit right below `slice` in
`Derived.ken.md` and need NO bindings.**

**⇒ Expression length is EVIDENCE, never the DECISION.** The question is always:
***does this binding name a concept the reader would otherwise have to infer?***

## 2. Mandated deliverable — five artifacts, and the `agent/` ones are the point

### 2.1 `docs/program/07-catalog-style-guide.md`
New normative subsection **"Local bindings as exposition"** (the §1 rule).
Cross-reference it from **Proof presentation** and **Naming**. Extend the author
and review checklists with the five review questions:

- Are repeated, non-atomic terms named once, where the name carries meaning?
- Do multi-step proof chains name their important endpoints or evidence?
- Are bindings scoped narrowly enough to preserve branch and effect order?
- Does each name improve vocabulary, or merely hide syntax?
- Has a long binding chain exposed a missing helper or lemma?

State plainly: **a style-only `let` refactor preserves public names, result types,
proof claims, and trusted-base delta — but it IS an AST change and takes the
normal elaboration and behavioral gates. It is NOT whitespace.**

### 2.2 `catalog/guide/surface-reference.ken.md`
A **first-class section for local `let`.** It is the language's local naming
construct and the surface guide's index currently **omits it entirely** — it is
discoverable only in the grammar and the formatter spec. **Checked examples**
(they elaborate — CV verifies): a single inferred binding · an annotated
binding · a short staged value pipeline · a **proof-valued** binding whose type
is used by the rest of the body · narrow placement inside a `match` branch ·
**a rejection example showing the binding is NOT recursive and is NOT in scope
in its own RHS.**

**⛔ It MUST state the operational caveat: Ken is call-by-value. An effectful
`let` sequences its RHS before its body. Authors must not hoist work out of a
branch merely to name or share it.** A convention that quietly changes evaluation
order is a bug, not a style.

### 2.3 `catalog/guide/proof-techniques.ken.md`
New subsection **"Name endpoints and evidence in proof chains."** Small example:
the string-injectivity certificate (report §"Worked example" — it was *checked
against the current elaborator*, and the local aliases are **definitionally
equal** to the originals, so `same_chars` is accepted with no transport lemma).
Larger example: one reduced `Map` bridge. Teach: **name the middle endpoints of
`trans`** · name a transported/congruent fact when its role is not obvious at the
call site · **keep the final combinator skeleton visible** · prefer a top-level
lemma when the fact is reusable or needs recursion.

*Context for why this matters: the three surveyed files hold **228 `trans`, 100
`cong`, 29 `J`** — and not one name for any endpoint.*

### 2.4 `agent/playbooks/tools/write-ken.md`  ← **the highest-leverage file here**
Inline a **short, high-salience** version of the rule — this is the skill an agent
reads *immediately before authoring*, and it is where the loop broke. Suggested
heading: **"Use local `let` to name meaning, not merely length."** Point to §2.2
and §2.3. **One small checked example.**

**It must also tell authors to run `ken fmt`, LOOK at the emitted binding layout,
and re-run `ken check`.** A canonical result from `ken fmt --check` is not a
readability verdict. *(The shredded-chain defect was fixed in LET-1 at
`ec980d76`; `kenfmt_let_layout.rs` asserts exact emitted text, and inspection
remains the rule.)*

### 2.5 `agent/teams/foundation/{implementer,qa,leader}.md`
← **create the directory**
**`agent/teams/` does not exist.** `moot.toml` and `CLAUDE.md` both already route
to `agent/teams/<team>/<role>.md` "if present." **Nothing has ever been present.**

- **`implementer.md`** — for **any** `catalog/**/*.ken.md` edit: **load
  `write-ken`**; plan a final **exposition pass after proof closure**; apply the
  local-binding convention; run formatting + exact package checks; **inspect the
  formatted source rather than treating `--check` as a readability verdict.**
- **`qa.md`** — review local naming and proof staging **independently**; **reject
  any binding that changes branch or effect placement**; request a named
  intermediate when a repeated semantic state or proof endpoint is otherwise left
  for the reader to reconstruct; apply positive formatter-layout checks.
- **`leader.md`** — catalog WPs must cite the authoring guide and carry
  **readability as an acceptance axis. Do NOT prescribe a binding count.**

**The generic build playbooks stay GENERIC.** Add **one line** noting that team
overlays may add source-language authoring rules. **⛔ Do not duplicate the
convention into `ken-build-*` — that is exactly how the two drift.**

## 3. Acceptance criteria

- **AC1** — every Ken example in `catalog/guide/**` **elaborates**. CV runs them.
  *A guide that teaches from an example that does not check is worse than no
  guide.*
- **AC2** — the rejection example genuinely **fails**, and fails **for the stated
  reason** (non-recursive / not in scope in its own RHS) — **assert the specific
  error, not `is_err`.**
- **AC3** — `agent/teams/foundation/` exists with all three overlays, and the
  routing in `moot.toml` / `CLAUDE.md` **actually resolves to them.** *Verify by
  loading, not by `ls`.*
- **AC4** — `write-ken` requires authors to run `ken fmt`, inspect the emitted
  binding layout, and re-run `ken check`; it contains **no live-defect caveat**.
  Assert the caveat text is absent by file content, not by requiring the
  legitimate historical name `LET-1` to be absent.
- **AC5** — the counter-rule (§1, "some terms should stay direct") appears in
  **every** artifact that states the rule. **A convention shipped without its
  brake becomes a quota in practice.**
- **AC6** — **DOC-ONLY.** Zero `crates/` changes. Zero `catalog/packages/` changes
  (the rewrites are LET-3). Zero `trusted_base()` delta.

## 4. ⛔ Guardrails

- **⛔ Do NOT rewrite catalog packages.** Not one. Not "just `slice` as a demo."
  **That is LET-3, it is Foundation's, and it is gated on LET-1's formatter fix.**
  A demo rewrite here would land un-formatted code and pre-empt the pilot.
- **⛔ Do NOT add the rule to `decomposition-abstraction.ken.md`.** That strand
  decides between *component-level mechanisms*; `let` is *local exposition*.
  Different question, different reader.
- **⛔ Do NOT retro-edit historical WP frames** to the new style.
- **⛔ Do NOT turn this into a linter, a quota, an AST-depth threshold, or a
  minimum count.** Explicitly out of scope, and the report explains why: it would
  breed meaningless `tmp` bindings and still miss short, conceptually dense
  proofs. **If you find yourself specifying a number, you have left the WP.**
