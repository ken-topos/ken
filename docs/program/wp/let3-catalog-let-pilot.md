# LET-3 · The catalog rewrites — a PILOT, reviewed, then family by family

**Owner:** Team Foundation · **Size:** M (pilot) → L (staged rollout)
**Branch:** `wp/let3-catalog-let-pilot` · **Base:** current `origin/main`
**Gate:** Foundation QA + **Steward** (the pilot review is a real gate, not a
formality)

## ▶ DEPENDENCIES ARE MET — this WP is RELEASED (Steward, 2026-07-14)

**Both blockers have landed.** The original frame named them "LET-1" and
"LET-2"; both were **subsumed and delivered under different IDs**, so read this
table, not the old names:

| Original dep | What actually landed | Where |
|---|---|---|
| **LET-1** — `kenfmt` layout | **LET-4** (`26527c5a`) — multi-binding `let` grammar, scope, desugaring, effects, **and canonical layout**. LET-1b was explicitly **subsumed** by it. | merged |
| **LET-2** — the convention | **LET-2b** (`ce6f0718`) — the guides, `write-ken`, the catalog style guide, and all three Foundation overlays now **teach the rule and carry a checked example**. | merged |

**⇒ The hazard each dep guarded against is closed:** the formatter no longer
shreds a binding group (LET-4 fixed the layout), and there **is** now a rule to
apply and an example to imitate (LET-2b wrote it, and the three Foundation
overlays you work from carry it).

### ⚠ The syntax you are applying is LET-4's, not the old frame's prose

The landed form is a **`;`-separated binding group** — sequential and
non-recursive, earlier names visible to later right-hand sides, duplicates
rejected. **Maximal coalescing applies at two or more bindings; a single binding
stays a plain `let`.** That last clause is load-bearing here: **do not turn a
one-binding `let` into a group.** Read `catalog/guide/surface-reference.ken.md`
and your own team overlay before the first edit — they are current as of
`ce6f0718`.

## 0. ★ What this WP is, and what it must NOT become

**It must not become a corpus-wide `let`-insertion sweep.** I want that stated
before anything else, because the temptation is structural: there are **27,404
tangled Ken lines with ZERO local bindings**, and a mechanical pass could "fix"
all of them in an afternoon.

**That would be the worst possible outcome.** The rule (LET-2 §1) is a **judgment
rule** — *does this name state a concept the reader would otherwise infer?* — and
**a blind sweep answers "yes" everywhere, which is the same as answering it
nowhere.** You would replace visible syntax with a namespace of `tmp1`, and the
corpus would be *less* readable while every gate stayed green.

**⇒ Pilot. Review. THEN staged rollout. The review between them is a real gate
and I will hold it.**

## 1. Phase 1 — the pilot (this WP's deliverable)

**Scope, exactly:**

- **`catalog/packages/Data/Text/StringBijection.ken.md`** — all 32 Ken
  lines / 2 declarations. **The whole file.** It is small, and its injectivity
  proof is the report's own worked example — **already checked against the current
  elaborator**, with the local aliases **definitionally equal** to the originals
  (so the existing `same_chars` proof is accepted **with no transport lemma**).
  *If you find you need a transport lemma, something is wrong — STOP AND REPORT.*
- **`Derived.ken.md` — `slice` (`:1321`) and one `sort_bool` proof. Those two
  only.**

`slice` today is a single nested application whose four domain concepts — the
character sequence, the suffix, the saturating width, the selected window — are
visible only as application nesting:

```ken
fn slice (i : Nat) (j : Nat) (s : String) : String =
  list_char_to_string (take Char (nat_sub j i) (drop Char i (string_to_list_char s)))
```

**⛔ And the control case, which matters as much as the rewrite:
`char_at`, `eq`, and `compare` sit immediately below `slice` and MUST BE LEFT
ALONE.** They are already clear pipelines with a familiar operation at the head.
**A WP that "improves" them has misunderstood the rule** — and I will read that
diff specifically.

## 2. Phase 2 — staged rollout (a LATER WP; do not start it here)

`Map.ken.md` is **14,723 Ken lines / 421 declarations / 143 spans ≥ 40 lines / 38
spans ≥ 80 / max 238.** It is where the value is, and it is where a blind sweep
would do the most damage.

**Family by family**, largest `trans` chains and repeated accumulator/lookup
states first. The report's exemplar: `union_from_list_acc_lookup_assoc_hit`
(`:10218`, **238 lines**) repeats `insert_with_fold_step …` **13×**, `lookup k v
leq query acc` **9×**, `assoc k v leq query xs2` **5×**. **Those are not
incidental strings** — they denote the updated accumulator, the original lookup
result, and the tail's association. **Name them and the proof stops being a tree
of syntax and becomes a chain of semantic states.**

First pass names **stable semantic states only** — `entry_key`, `entry_value`,
`updated_acc`, `tail_assoc`, `lookup_before`. **Proof evidence gets named after,
and only where a nested `trans` still forces the reader to reconstruct its role.**

**This phase does not start until the pilot review passes. It will be its own WP,
scoped by proof family. Do not pre-empt it.**

## 3. Acceptance criteria

- **AC1 — public surface is BYTE-IDENTICAL in meaning.** Every public
  declaration, result type, and proof claim preserved. **A `let` refactor renames
  nothing the outside can see.**
- **AC2 — ZERO `trusted_base()` delta.** Fail-closed set-equality assertion, same
  shape SUB-1/SUB-2 used. **No new `Axiom`.**
- **AC3 — the REAL ordered dependency harness runs** (DS-7/8), not a standalone
  `ken check`. *(`Collections` is dependency-bearing; a standalone check is not
  the gate — this has bitten a catalog WP before.)*
- **AC4 — ★ VERIFY the conservativity; do not ASSUME it.** *"A `let` rewrite in a
  pure proof is normally definitionally conservative"* — **normally is not a
  proof.** The report says to verify rather than classify it as whitespace, and I
  am making that an AC: show the elaboration still closes and the behavior is
  unchanged. **This is an AST change wearing whitespace's clothes.**
- **AC5 — INSPECT THE FORMATTED OUTPUT.** Run `ken fmt`, then **read it**.
  **`ken fmt --check` returning "canonical" is NOT a readability verdict** — that
  is precisely the defect LET-1 fixes and precisely the trap that makes a green
  gate meaningless here. **Paste the formatted `slice` and the formatted
  injectivity proof into your handoff so I can read them.**
- **AC6 — the control case is UNTOUCHED.** `char_at`, `eq`, `compare` byte-for-
  byte identical. **Show me that in the diff.**
- **AC7 — the pilot is REVIEWABLE AS PROSE.** Your handoff must answer, per
  binding: **what concept does this name that the reader would otherwise have had
  to infer?** If you cannot answer for a binding, **delete that binding.** *That
  question IS the rule, and it is the whole acceptance test.*

## 4. ⛔ Guardrails

- **⛔ NO corpus-wide sweep. NO `Map.ken.md` in this WP.** Three targets:
  `StringBijection.ken.md` (whole), `slice`, one `sort_bool` proof. **That is the
  entire scope.**
- **⛔ Do NOT bind for length.** `tmp`, `x2`, `value2`, `intermediate`,
  `step_result` — **if the name carries no more vocabulary than its RHS, the
  binding is a NET LOSS** and must not land.
- **⛔ Do NOT hoist across a branch or an effect.** Ken is **call-by-value**; an
  effectful `let` sequences its RHS before its body. **Bind at the narrowest scope
  containing every use.** A binding that changes evaluation placement is a **bug**,
  not a style — and it is the one way this WP could do real damage.
- **⛔ Do NOT add a top-level helper** for a concept that only narrates one body.
  *(And conversely: if a long binding preamble appears, that is a decomposition
  signal — **report it**, do not build a local namespace.)*
- **⛔ Do NOT touch `crates/`.** If the formatter misbehaves, that is a **LET-1
  regression — STOP AND REPORT IT.** Do not work around it by hand-formatting.
- Targeted gates only. **⛔ NEVER `--workspace`** (operator hard rule,
  `COORDINATION.md §12`).
