# `RT-FNSPLIT-B2F` `AC-5` — the `lower_expr` call census, re-derived on this base

**Author:** `runtime-implementer` · **Branch:**
`wp/RT-FNSPLIT-B2F-functionization-live` at `9fe97ea4` · **Base:** `origin/main`
= `6534e4a6` · **File:** `crates/ken-runtime/src/cranelift_backend/lowering/core.rs`

⛔ **This is the committed enumeration `AC-5` requires.** The frame says in terms
that it must be committed rather than asserted in a handoff message.

---

## 1. ⛔ THE FRAME'S NUMBER IS STALE — 59 → **61**

| | frame | measured here |
|---|---|---|
| calls into `lower_expr` | **59** (at `bd24422b`) | ⛔ **61** |
| definitions | 1 (`:4333`) | 1 (**`:5211`** — moved) |
| the root call | `:188` | **`:286`** — moved |
| span | `:188`–`:6743` | `:286`–`:7717` |

⭐ **The frame anticipated exactly this and told me to re-derive** — *"Re-derive
it again on your own base."* ⚠ The population grew by **two** on `main` between
`bd24422b` and `6534e4a6`; ⛔ **none of my own commits added a call** (the census
is `61` at my branch base and `61` at my tip, unchanged).

⇒ ⭐ **This is why `AC-5` pins the MECHANISM and not the number.** A reader
handed "59" and no tokenizer would have concluded the switch-over was
over-complete by two.

## 2. ⭐ THE NAIVE GREP MISSES EXACTLY ONE SITE, AND IT IS THE ROOT

```
tokenized `lower_expr` occurrences   62   (61 calls + 1 definition)
grep -c 'self.lower_expr('           60   lines
```

⇒ ⛔ **One call is invisible to the spelling the frame warns against, and it is
`:286` — the root**, spelled `compiler.lower_expr(` because the root is called on
the `Lowering` value before `self` exists. ⭐ **The frame predicted precisely this
failure and it reproduces on this base**: a reader deriving the population from
`self.lower_expr(` loses the one site that must become the call into the root
target function.

⚠ **`grep -c` counts LINES, not occurrences**, so the two numbers are not even
the same kind of quantity — a second reason not to derive the population that
way.

## 3. The census mechanism

⛔ **Tokenized**, per `identifier_occurrences` (`control.rs:3925`): comments
stripped at `//`, split on every non-`[0-9A-Za-z_]` character, compared as
**whole tokens**. ⇒ A call split across lines is counted; a mention in prose is
not; `lower_expr_into` would not be conflated with `lower_expr`.

Reproduce from this worktree:

```sh
python3 - <<'PY'
import re
src = open('crates/ken-runtime/src/cranelift_backend/lowering/core.rs').read()
print(sum(1 for line in src.split('\n')
            for t in re.split(r'[^0-9A-Za-z_]', line.split('//')[0])
            if t == 'lower_expr'))
PY
```

⛔ **This census is NOT committed as a test.** A test whose subject is repository
text is barred by operator policy (2026-07-26), and `AC-2`'s ruling already
records that the existing text census is **fail-open** and retained only as a
tripwire. ⇒ The enumeration lives here, as a document, and the *behavioural*
claim about the switch-over is `S6`'s to pin.

---

## 4. The 62 occurrences, enumerated

| # | line | kind | text |
|---|---|---|---|
| 1 | `:286` | call — INVISIBLE to `self.lower_expr(` | `let lowered = compiler.lower_expr(` |
| 2 | `:476` | call | `self.lower_expr(builder, arg, argument_env)` |
| 3 | `:573` | call | `return self.lower_expr(builder, occurrence, producer_env);` |
| 4 | `:576` | call | `let value = self.lower_expr(builder, occurrence, producer_env)?;` |
| 5 | `:604` | call | `let value = self.lower_expr(builder, occurrence, producer_env)?;` |
| 6 | `:746` | call | `let value = self.lower_expr(builder, value_occurrence, producer_` |
| 7 | `:761` | call | `let callee = self.lower_expr(builder, callee, producer_env)?;` |
| 8 | `:829` | call | `self.lower_expr(builder, arg, producer_env)` |
| 9 | `:950` | call | `self.lower_expr(builder, arg, producer_env)` |
| 10 | `:1005` | call | `self.lower_expr(builder, arg, producer_env)` |
| 11 | `:1110` | call | `self.lower_expr(builder, arg, producer_env)` |
| 12 | `:1207` | call | `self.lower_expr(builder, arg, producer_env)` |
| 13 | `:1225` | call | `let selected = self.lower_expr(builder, scrutinee, producer_env)` |
| 14 | `:1472` | call | `let selected = self.lower_expr(builder, scrutinee, producer_env)` |
| 15 | `:1527` | call | `let value = self.lower_expr(builder, occurrence, producer_env)?;` |
| 16 | `:1758` | call | `return self.lower_expr(builder, case_body, &case_env);` |
| 17 | `:1813` | call | `self.lower_expr(builder, body, &case_env)` |
| 18 | `:1909` | call | `self.lower_expr(builder, zero_body, &zero_frame_env)?` |
| 19 | `:2033` | call | `self.lower_expr(builder, suc_body, &suc_env)?` |
| 20 | `:2132` | call | `let lowered = self.lower_expr(builder, field, deferred.producer_` |
| 21 | `:2537` | call | `value: self.lower_expr(` |
| 22 | `:4786` | call | `let lowered = self.lower_expr(builder, body, &case_env)?;` |
| 23 | `:5122` | call | `self.lower_expr(builder, body, &case_env)?` |
| 24 | `:5211` | DEFINITION | `fn lower_expr(` |
| 25 | `:5233` | call | `let result = self.lower_expr(builder, body, env);` |
| 26 | `:5245` | call | `let result = self.lower_expr(builder, body, env);` |
| 27 | `:5262` | call | `let result = self.lower_expr(builder, body, env);` |
| 28 | `:5268` | call | `self.lower_expr(builder, body, env)` |
| 29 | `:5277` | call | `let value = self.lower_expr(builder, body, env)?;` |
| 30 | `:5289` | call | `let lowered_value = self.lower_expr(builder, value, env)?;` |
| 31 | `:5299` | call | `self.lower_expr(builder, body, &body_env)` |
| 32 | `:5309` | call | `let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)` |
| 33 | `:5321` | call | `self.lower_expr(builder, then_expr, env)` |
| 34 | `:5323` | call | `self.lower_expr(builder, else_expr, env)` |
| 35 | `:5334` | call | `let lowered = self.lower_expr(builder, arm, env)?;` |
| 36 | `:5359` | call | `self.lower_expr(builder, arg, env)` |
| 37 | `:5418` | call | `let lowered_scrutinee = self.lower_expr(builder, scrutinee_occur` |
| 38 | `:5535` | call | `return self.lower_expr(builder, body, env);` |
| 39 | `:5551` | call | `let lowered = self.lower_expr(builder, body, env)?;` |
| 40 | `:5596` | call | `self.lower_expr(builder, body, &case_env)` |
| 41 | `:5620` | call | `Ok((name.clone(), self.lower_expr(builder, expr, env)?))` |
| 42 | `:5635` | call | `let lowered_record = self.lower_expr(builder, record, env)?;` |
| 43 | `:5720` | call | `self.lower_expr(builder, capture, env)` |
| 44 | `:5742` | call | `let lowered_callee = self.lower_expr(builder, callee, env)?;` |
| 45 | `:5799` | call | `self.lower_expr(builder, arg, env)` |
| 46 | `:5814` | call | `self.lower_expr(builder, body, &call_env)` |
| 47 | `:5891` | call | `self.lower_expr(builder, arg, env)` |
| 48 | `:5983` | call | `self.lower_expr(builder, argument, env)` |
| 49 | `:6071` | call | `self.lower_expr(builder, capability_value, env)?` |
| 50 | `:6881` | call | `let zero_lowered = self.lower_expr(builder, zero_body, &zero_env` |
| 51 | `:6951` | call | `let next = self.lower_expr(builder, suc_body, &suc_env);` |
| 52 | `:6996` | call | `self.lower_expr(builder, arg, producer_env)` |
| 53 | `:7079` | call | `self.lower_expr(builder, body, &call_env)` |
| 54 | `:7182` | call | `self.lower_expr(builder, body, &call_env)` |
| 55 | `:7266` | call | `let result = self.lower_expr(builder, declaration_body, &[]);` |
| 56 | `:7336` | call | `return self.lower_expr(builder, body, &arm_env);` |
| 57 | `:7382` | call | `let lowered = self.lower_expr(builder, body, &arm_env)?;` |
| 58 | `:7445` | call | `let lowered = self.lower_expr(builder, body, &arm_env)?;` |
| 59 | `:7502` | call | `let lowered = self.lower_expr(builder, body, &arm_env)?;` |
| 60 | `:7574` | call | `let lowered = self.lower_expr(builder, body, &arm_env)?;` |
| 61 | `:7666` | call | `self.lower_expr(builder, body, &arm_env)?` |
| 62 | `:7717` | call | `self.lower_expr(builder, arg, env)` |

---

## 5. ⛔ NOT YET DONE — the disposition

⛔ **This document enumerates the population; it does not disposition it.** The
amended `AC-5` requires the **five-class** taxonomy with the caller-dependent
sites dispositioned per `(site × reaching path)`, because — per the frame's own
amendment, found at `evt_1vz8pmztgtye9` — **disposition is a function of the
PATH, not of the site** for at least 8 of them. ⇒ A table keyed by site alone is
unsound, and producing one here would be the exact defect the amendment
withdrew.

That disposition is `S6`'s work and joins the switch-over it authorizes.
