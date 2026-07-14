# LET-2b — Teach the `let` binding group

**Owner:** Team Language · **Size:** M · **Risk:** ★★
**Stream/gate:** WS-L / surface pedagogy · **Rides:** LET-4 (`26527c5a`)
**Unblocks:** LET-3 (catalog `let` pilot — Foundation)

**Status:** RELEASED.

> **Chain:** LET-4 made the multi-binding **group** the canonical `let`. It
> migrated the two `catalog/guide/*.ken.md` **fences** — because it *had to*, or
> `ken fmt --check` would have gone red on the frozen corpus — and **touched none
> of their prose.** This WP closes that gap.

---

## 0 · The defect, in one sentence

**The guides now SHOW group-form code that their own prose never explains.**

`catalog/guide/surface-reference.ken.md:476-486` renders a binding group.
Twenty-five lines above it, `:451-454` — the normative sentence that *introduces
the construct* — still says:

> *"`let name = rhs in body` gives an intermediate expression a local name…"*

**The `;` separator is never named. The group form is never named.** The section
is still titled *"Local `let`: naming an intermediate"* (`:25`, `:449`) —
singular.

**This is not a formatting problem. It is a teaching problem, and it is the one
thing LET-4 could not fix mechanically.** *A reader who learns `let` from this
guide learns a spelling the formatter will silently rewrite underneath them.*

---

## 1 · Fixed inputs — the spec is SETTLED. Do not reopen it.

LET-4 landed the normative text. **Cite it; do not relitigate it.**

- **`spec/30-surface/32-grammar.md:200-201`** — the production:
  ```
  let_expr    ::= "let" let_binding (";" let_binding)* "in" expr
  let_binding ::= ident (":" type)? "=" expr
  ```
- **`:209-213`** — one or more bindings; a `;` occurs **only between** two
  bindings; **a trailing `;` before `in` and a comma in place of a `;` are syntax
  errors.** ★ *"A one-binding `let` remains the same production with zero
  repetitions."*
- **`:215-221`** — bindings are **sequential and non-recursive**; duplicate names
  are rejected.
- **`spec/30-surface/31-lexical.md:228-231`** — the load-bearing sentence:
  > *"A directly nested sequential chain of **at least two** local lets has one
  > canonical surface form: the formatter **coalesces the maximal chain** into a
  > binding group. It does not retain the repeated `in let` spelling."*

---

## 2 · ⛔⛔ THE TRAP — AND IT IS AN OVER-APPLICATION TRAP, NOT AN OMISSION

**You are about to learn "the group is canonical." You will then want to convert
every `let` you see into a group. THAT IS WRONG, AND IT IS THE WAY THIS WP
FAILS.**

**A one-binding `let` is ALREADY CANONICAL.** The spec says so in as many words
(`32-grammar.md:212-213`): *the same production with zero repetitions.* And the
formatter only coalesces chains of **≥ 2** (`31-lexical.md:228`).

**⇒ Every one of these is CORRECT and MUST NOT BE TOUCHED:**

| Site | Why it stays |
|---|---|
| `surface-reference.ken.md:466, :468, :495, :504, :514, :522` | single-binding lets — canonical |
| `agent/playbooks/tools/write-ken.md:80` | single-binding — canonical |
| `examples/rosetta/gcd/gcd.ken:99` | the **only** `let` in all of `examples/` — canonical |

> **A "group of one" is not the canonical form of one binding. It is noise.**
> **The group is the canonical form of a CHAIN.** *If your diff converts a
> single-binding `let` into a group, you have inverted this WP.*

**There are ZERO surviving `let … in let …` chains in any teaching artifact** —
LET-4 already migrated the only two (`proof-techniques.ken.md:353-375`, `:396-407`;
`surface-reference.ken.md:476-486`). **Your job is the PROSE, not the code.**

### ★★ TRAP 2 — THE FROZEN ORACLE WILL TEMPT YOU TO RE-BASELINE IT

`crates/ken-elaborator/tests/kenfmt_c_capstone.rs:14-66` holds
`FRAME_LINE_COUNTS`, a **frozen** table that names your exact files:

```rust
("catalog/guide/proof-techniques.ken.md",  367),
("catalog/guide/surface-reference.ken.md", 535),
```

Your prose edits **grow** these files. The guard at `:163-166` is
`canonical_lines * 2 <= frame_lines * 9` — a **4.5× ceiling** (so
`surface-reference.ken.md` may reach **2407** lines; it is at 535). **Normal
prose expansion will not trip it.**

**But `:168` is `assert_eq!(frame_total, 32_594, "frame line-count oracle
drifted")` — and that assertion exists for exactly one reason: to catch someone
"fixing" the table.**

> **⛔ DO NOT EDIT `FRAME_LINE_COUNTS`. It is a frozen oracle, not a ledger to
> keep current.** *Re-baselining an oracle to match the artifact it is supposed
> to constrain converts a gate into a rubber stamp — it will then pass forever
> and mean nothing.* **If Gate C trips, STOP and route to the Steward.** That is
> a finding, not a chore.

---

## 3 · Deliverables

**D1 · `catalog/guide/surface-reference.ken.md` §8 — the primary artifact.**
Rewrite the §8 prose (`:449-454`, and the section title at `:25`/`:449`) to teach
the construct as it actually is: **one or more bindings, `;`-separated**;
sequential and non-recursive; duplicates rejected; **the formatter coalesces a
maximal chain of ≥2 into a group**; a one-binding `let` is the same production.
The migrated fence at `:476-486` is your worked example — **narrate the code that
is already there.**

**D2 · `catalog/guide/proof-techniques.ken.md` §6.** LET-4 rewrote the two proof
fences into groups and left the surrounding prose (`:331-341`, `:378-382`,
`:410-417`) narrating a nested chain that **is no longer on the page.** Re-narrate
it to match the code the reader can actually see.

**D3 · `agent/playbooks/tools/write-ken.md:57-98`.** The `let` section never
mentions the group. **Keep the single-binding example at `:80-85` — it is
canonical** — and **add** a group example. Refresh the pointer at `:92-94`.

**D4 · `docs/program/07-catalog-style-guide.md` §6.1 (`:182-216`).** Pure prose,
no examples. Give catalog authors the rule: **≥2 sequential bindings ⇒ write the
group.**

**D5 · `agent/teams/foundation/{leader,implementer,qa}.md` — ★ LOAD-BEARING.**
These overlays are silent on the group. **Foundation authors LET-3, the catalog
`let` pilot, next.** *If they author it in the single-binding spelling, the
formatter canonicalizes every one into a group and we churn the catalog twice —
which is the exact reason LET-3 was blocked behind LET-4 in the first place.*
**One clear line each is enough. Do not skip this because it is the smallest
item; it is the one with a consumer waiting.**

---

## 4 · Acceptance criteria

**AC1 — the prose teaches the construct.** Both `.ken.md` guides name the **`;`
separator**, the **group**, **sequential non-recursive scope**, and the **≥2-chain
maximal-coalescing** rule, citing `31-lexical.md:228-231` / `32-grammar.md:200-221`.

**AC2 — ★ ZERO Ken-fence changes.** LET-4 already migrated every fence. **Your
diff should touch prose only.** `ken fmt --check` is green on both guides **before
and after** — run it and show it. *If you believe a fence must change, STOP and
route: that means LET-4 left something non-canonical and it is a finding.*

**AC3 — ⛔ NO single-binding `let` becomes a group.** Enumerate **every** `let`
your diff touches and show each was a **≥2 chain**. *If that list is empty, that
is the correct and expected answer.*

**AC4 — the frozen oracle is UNTOUCHED.** `git diff` does **not** include
`crates/ken-elaborator/tests/kenfmt_c_capstone.rs`. **`FRAME_LINE_COUNTS` is not
edited. `32_594` is not edited.**

**AC5 — the Foundation overlays teach the group** (D5), so LET-3 is authored
canonically the first time.

**AC6 — targeted gates only** (`COORDINATION §12`; **never `--workspace`
locally**):
- `scripts/ken-cargo test -p ken-cli --test ken_fmt` (Gate A: strict frozen corpus)
- `scripts/ken-cargo test -p ken-elaborator --test kenfmt_c_capstone` (Gates B/C)
- `scripts/ken-cargo test -p ken-elaborator --test kenfmt_b1_lossless` (Gate D)

**AC7 — ★ FULL CI ON PUBLISH. NEVER `--doc-only`.** `catalog/**/*.ken.md` is
**CODE** — `ken_fmt.rs:89` formats it. *LET-2 landed two guide strands behind
`--doc-only`, went red on the frozen-corpus gate, and blocked the publisher for
every WP behind it. That does not happen twice.*

---

## 5 · Guardrails — do not reopen

- **The spec is settled and landed.** Do not edit `spec/`. Cite it.
- **Do not touch the Ken fences.** LET-4 owns them; they are canonical.
- **Do not re-baseline `FRAME_LINE_COUNTS`.**
- **Do not convert single-binding lets to groups.** (§2. This is the failure mode.)
- **`agent/`, `docs/`, `spec/`, `tooling/` are swept by no formatter gate** — only
  `catalog/**` and `examples/rosetta/**` are. Know which of your files is code.
