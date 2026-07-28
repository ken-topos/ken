# `RT-FNSPLIT-B2F` `AC-5` — the `lower_expr` call population, enumerated

**Base:** `origin/main` = `6534e4a6`, measured on branch tip `001242a8`
(`lowering/core.rs` after `S1`+`S2`) · **Author:** `runtime-implementer`

⛔ **`AC-5` requires this enumeration to be COMMITTED, not asserted in a handoff
message.** This file is that commitment.

⛔ **Derived with a TOKENIZED census**, per `AC-5`'s withdrawal 2 — comments
stripped, split on every non-identifier character, whole-token match. ⚠ The naive
`grep -c 'self\.lower_expr('` returns **60** on this tree and **misses the
root**, which is the same spelling-scoped defect the frame records against
itself. *The number is the symptom; the mechanism is the fix.*

---

## 1. The population

| | frame (`bd24422b`) | **measured (`6534e4a6`)** |
|---|---|---|
| definition | `core.rs:4333` | **`core.rs:5149`** |
| calls | 59 | **61** |
| span | `:188`–`:6743` | **`:246`–`:7655`** |
| root call | `:188` | **`:246`** |

⛔ **Every line number in the frame's `D5`/`AC-5` amendment is stale.** The
enumerated site lists there — the 8 caller-dependent, the 6 untraced, the 3
synthesized, `:4878`, `:4454` — ⛔ **do not resolve to `lower_expr` calls on this
base** and were not ported. This table is re-derived from scratch.

⚠ `:246` is the root and is **below** any span that starts at a later line, which
is exactly how the frame's own predecessor lost it. The root is
`compile_expr_into_module`'s `SourceOccurrence { expr, static_origin:
root_static_origin }`, built **inline at the call site**.

---

## 2. The five classes — 34 + 15 + 8 + 3 + 1 = **61** ✓

| class | count | derivation |
|---|---|---|
| structural: `child_occurrence` | **34** | positional syntax child |
| structural: `case_body_occurrence` | **15** | match-arm bodies |
| **caller-dependent** (parameter-fed) | **8** | provenance is the caller's |
| synthesized occurrence | **3** | `:246` root · `:2496` source-machine · `:7225` `declaration_body` |
| direct retained body | **1** | `:5773`, from `self.retained_body_occurrence(body)?` |

⭐ **The taxonomy survived; every number in it moved.** The frame measured
32/9/14/3/1; this base measures 34/15/8/3/1. ⚠ **The caller-dependent class
halved (14 → 8)** — that is a real structural change, not a re-count, and it
matters because that class is the one whose disposition is not a function of the
site.

⭐ The **three synthesized sites are the same three kinds** the frame names —
root, source-machine fallback, declaration body — at completely different lines.
⇒ The classification is stable under code motion even though every anchor is
not, which is the argument for re-deriving rather than porting.

---

## 3. ⛔ The four sites mechanical resolution could not decide

**`pin-a-property` §4: "cannot determine" is a third outcome and it must FAIL,
never fall through to a class.** Four sites came back undetermined and each was
resolved **by hand**, with its evidence:

| site | occurrence | resolved | evidence |
|---|---|---|---|
| `:1772` | `body` | **case_body** | tuple-bound at `:1752-1759` from `self.case_body_occurrence(eliminator.static_origin, case_index, &case.body)` |
| `:1868` | `zero_body` | **case_body** | tuple-destructured at `:1802`; both match arms (`:1821`, `:1840`) produce `case_body_occurrence` |
| `:1992` | `suc_body` | **case_body** | same destructure, `:1822` / `:1841` |
| `:5293` | `arm` | **child** | loop binder over `[(then_block, then_expr), (else_block, else_expr)]`; both bound at `:5266`/`:5267` by `self.child_occurrence(static_origin, 1\|2, …)` |

⚠ The mechanical pass missed all four for **one shared reason** — it matched
`let <name> = …` and these are bound by **tuple destructuring and a loop
binder**. ⇒ A shared granularity error, not four separate ones; recorded because
the same gap will recur in any successor scan.

---

## 4. Caller-dependent sites, dispositioned per `(site × reaching path)`

⛔ **A disposition table keyed by site is only sound if disposition is a function
of the site. For these 8 it is a function of the PATH.**

| site | parameter | enclosing function |
|---|---|---|
| `:532` `:535` `:563` `:1486` | `occurrence` | `lower_computational_producer_expr` |
| `:6840` `:6910` | `zero_body` / `suc_body` | `lower_unary_recursive_nat_fold` |
| `:7038` `:7141` | `body` | `lower_recursive_declaration_call` |

**Reaching paths measured — 33 in total:**

| callee | paths | of which carry a RETAINED BODY |
|---|---|---|
| `lower_computational_producer_expr` | **30** | ⭐ **6** — `:450`, `:793`, `:915`, `:5866` pass `retained_body_occurrence(…)` directly or by parameter |
| `lower_unary_recursive_nat_fold` | **1** | 0 measured (its own caller is itself caller-dependent) |
| `lower_recursive_declaration_call` | **2** | ⚠ both pass `&symbol`, not an occurrence — the body is resolved *inside*, via `declaration_occurrence_origin` |

⭐ **This confirms the frame's core structural claim on a fresh base:** the same
parameter is fed **both** retained bodies **and** ordinary sub-expressions, so
the body-emission authority is **not localized at a handful of call sites** — it
is diffused through the producer / eliminator-frame machinery. ⇒ **`D6`'s
removal is dismantling the deforestation architecture, not excising a
function.**

---

## 5. ⚠ NOT CLAIMED

Stated as a **partition with its discriminator**, not as an example — a residual
that names one instance reads as a boundary.

1. ⛔ **Path closure is NOT established.** The 33 reaching paths are the *direct*
   callers of three functions. **Any path reaching them transitively is
   unmeasured.** The discriminator: *does the path reach the parameter without
   passing through one of the three named callees?* — if yes, it is outside this
   measurement entirely.
2. ⛔ **`identifier_occurrences` does not expand macros**, so *"no further call
   is reachable through a macro"* is **not** claimed.
3. ⚠ **Two reaching paths remain genuinely undecided by mechanism** — `:1439`
   (`if known { then_expr } else { else_expr }`) and `:1455` (`branch`). Both are
   conditionals over already-classified children and neither was hand-resolved
   for this document. ⛔ They are recorded as **open**, not as `child`.
4. ⛔ **This is a census of the CURRENT tree.** It dispositions where each call
   gets its occurrence; it does **not** claim any of them has been switched over.
   `AC-5`'s switch-over half is `S6` and **nothing here discharges it.**
