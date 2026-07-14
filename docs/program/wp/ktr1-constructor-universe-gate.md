# KTR-1 — Constructor-universe admission gate (trust-root repair)

**Owner:** Team Kernel · **Size:** M · **Risk:** ★★★ (TCB) ·
**Stream/gate:** trust root (G0)

**Status:** RELEASED.

> **Escalation** `evt_5d77tsdr2kyxz` (Steward) → **executed probe**
> `evt_69kdc7t1rynw4` (kernel-leader) → **ARCHITECT RULING: TRUST-ROOT SOUNDNESS
> GAP CONFIRMED** `evt_66d1p9bz1h621`. This frame implements the Architect's
> **technical** containment verbatim. **No design fork is open** — see §1.
>
> ### ⚖ OPERATOR RULING — **NO PUBLICATION FREEZE** (Pat, 2026-07-14)
>
> The Architect's ruling called for freezing `main` to all but this repair. **The
> operator has overruled that, and it is the right call.** *"No need for a
> publication freeze. No one is using this yet."*
>
> **A release freeze is a control that protects USERS from a defective artifact.
> Ken has no users.** Freezing `main` would therefore have bought **zero risk
> reduction** and stalled every other lane — a pure cost. The Architect's instinct
> is correct **for a shipped language** and mis-calibrated for a pre-release repo;
> the *severity* classification (trust-root) stands, the *containment* does not.
>
> **⇒ This is a normal WP on the normal path.** It is important because a trust
> root is important — **not** because anything is blocked behind it. Other lanes
> continue and publish as usual.

---

## 0 · The defect, established by execution (not by reading)

On `origin/main @ 26e9baed`, **both of these are ADMITTED, `ken check` exit 0**:

```ken
data D : Type where { C : (s : Type) → D }   -- kernel-leader, evt_69kdc7t1rynw4

fn decode (d : D) : Type =                   -- architect,     evt_66d1p9bz1h621
  match d { C s ↦ s }
```

Together they give a **same-level Tarski-style universe**: `D : Type 0` whose
constructor injects *every* `Type 0` — including `D` itself — with a decoder
satisfying `decode (C A) ≡ A`. That is a retraction of `Type 0` into a `Type 0`
inhabitant, i.e. an **impredicative inductive at a predicative level**. It is
the standing setup for **Girard's paradox**.

> ### ⚠ WHAT IS AND IS NOT CLAIMED — hold this line, and do not let anyone
> ### (including me) round it up
>
> **NOBODY HAS DERIVED `Bottom`.** Neither the Architect's ruling nor this frame
> claims an executed paradox. What is *established* is narrower and sufficient:
> **the checker admits a declaration its own normative spec forbids, and the
> spec's soundness argument for a feature we ship depends on that forbidden
> thing being rejected.** That classifies the checker as **outside its
> predicative soundness contract**. You do not need an executed `⊥` to justify
> repairing a trust root; you need exactly this, and we have it.

## 1 · Why there is NO design fork — the spec already decided

**This is a conformance defect in the TCB, not an open question.** Verified at
the producer (`origin/main:spec/10-kernel/14-inductive.md`), not inherited from
a citation:

- **§1 (the family/constructor telescope):** *"`ℓ` is the family's universe
  level; **constructor argument types must live at `ℓ` or below** (predicativity,
  `12 §2`)."*
- **§1 (admission):** *"The kernel admits the declaration only if it passes (a)
  ordinary type-checking of all constructor signatures in context `Δ_p`, (b) the
  **strict-positivity** check (§2), and **(c) universe-level checks**."*
- **§3 (large elimination):** *"The motive may land in any `Type ℓ'`, including a
  universe… **Predicativity keeps this sound; there is no special restriction
  beyond the universe-level checks.**"*

**Read that last sentence again.** The spec's *entire* soundness argument for
large elimination — a feature Ken ships and intends to keep — **is the
universe-level check.** It is not a belt-and-braces extra. **It is the load
bearer, and it was never implemented.**

`crates/ken-kernel/src/check.rs:918-966` (`declare_inductive`) runs
`check_positivity`, then `build_types`, then provisionally admits and calls
`synth_type` on the former type and each constructor's *whole* type. **Nothing
anywhere compares a constructor argument's level against the family level.**
And at `:945` sits this comment:

```rust
// Strict-positivity is the sole structural admission gate (`14 §8`, `14 §8.4`).
```

> **★ That comment is the bug, written down.** It asserts a completeness property
> of the admission gate that the spec directly contradicts (clause (c)). **A
> confident comment stating "this is the sole gate" is exactly how a missing gate
> survives review** — every reader who checked "is positivity called?" got a yes
> and stopped. **Delete it and replace it with an enumeration of all three
> clauses.** *(This is `contract-expressibility` in its purest form: the
> obligation had no in-code home, so it lived in a comment — and the comment was
> false.)*

---

## 2 · The repair — `declare_inductive` (Architect-directed, verbatim)

Insert the gate **after provisional self-reference is available** (so recursive
occurrences of `D` resolve) **but before admission commits**. For **each
constructor**:

1. Build the parameter context `Δ_p`.
2. Walk **only the constructor-local telescope `Δ_k`**, in order.
3. `synth_type` each argument's type in `Δ_p + (earlier Δ_k)`.
4. Require **`arg_level ≤ family_level`**, conservatively and symbolically,
   using the existing level algebra: **`max(arg_level, family_level) ≡
   family_level`**.
5. On **any** ill-formed or over-level argument: **remove the provisional
   declaration and reject** (the existing `env.remove_last()` rollback path).

### ★★ THE ONE TRAP THAT WILL BREAK THE ENTIRE BUILD — `Δ_p` IS NOT `Δ_k`

**This is the single highest-risk line in the WP. Read it twice.**

The rule is about **the types of constructor-local arguments** — *not* about the
types of the family's parameters.

| binder | its type | `synth_type` of that type | subject to the gate? |
|---|---|---|---|
| **param** `(A : Type u)` in `Δ_p` of `List (A : Type u) : Type u` | `Type u` | `Type (suc u)` — level **`suc u`** | ❌ **NO.** It is a **parameter**. |
| **c-local arg** `(a : A)` in `Cons : A → List A → List A` | `A` | `Type u` — level **`u`** | ✅ yes → `u ≤ u` ✓ **accept** |
| **c-local arg** `(xs : List A)` (recursive) | `List A` | `Type u` — level **`u`** | ✅ yes → `u ≤ u` ✓ **accept** |
| **c-local arg** `(s : Type)` in `C : (s : Type) → D` | `Type 0` | `Type 1` — level **`1`** | ✅ yes → `1 ≤ 0` ✗ **REJECT** |

> **If you sweep `Δ_p` into the walk, you compute `level(Type u) = suc u > u` and
> you REJECT `List`.** And `Maybe`, and `Either`, and `Vec`, and **every
> parameterized type in the prelude and the catalog.** The build will not merely
> fail — it will fail *everywhere at once*, which reads like "the gate is wrong"
> and invites weakening the gate. **It is not wrong. You swept the wrong
> telescope.**
>
> **The discriminator is structural, not level-arithmetic:** a parameter is a
> parameter *by position in the declaration*, not by anything about its level.
> Take `Δ_k` from the constructor, `Δ_p` from the family, and never confuse the
> binder's **own type** with the **type of the thing it binds**.

### Explicitly OUT of scope (Architect, and I am enforcing it)

- ⛔ **Do NOT restrict large elimination.** The defect is on the
  **formation/admission** axis. Large elimination is a **specified Ken
  capability** and stays, *once constructor universes fit*.
- ⛔ **Do NOT invent a `Prop` / impredicative sort / singleton-elimination
  exception.** Ken has no impredicative sort by design; adding one to contain
  this is a language redesign smuggled in as a bugfix.
- ⛔ **Do not weaken the gate to make an existing declaration pass.** See AC4.

---

## 3 · Acceptance criteria

**AC1 — the probe is rejected, at both layers, with the RIGHT error.**
- `data D : Type where { C : (s : Type) → D }` is **rejected by direct kernel
  admission** *and* by surface `ken check`.
- **Assert the specific `KernelError` variant and its message**, not `is_err()`.
  *A test that only asserts "some error" passes for a typo in the fixture, and
  then the gate can rot to nothing without a single test going red.*
- The Architect's decoder (`fn decode (d : D) : Type = match d { C s ↦ s }`)
  becomes **unreachable** because `D` no longer forms. Keep it in the fixture so
  the test documents the *composed* hazard, not just the declaration.

**AC2 — the positive arms still pass.** Each with a test:
- **`D : Type 1` with `C : (s : Type 0) → D` is ACCEPTED** — the lifted form is
  legal and must stay legal. *(This is the arm that proves the gate discriminates
  by LEVEL rather than by SHAPE. Without it, a gate that rejects every
  `Type`-taking constructor would pass AC1 and be wrong.)*
- **Parameterized families are ACCEPTED:** `List (A : Type u) : Type u` and
  friends — the `Δ_p`/`Δ_k` trap above, as an executable test.
- **W-style / recursive arguments at or below the family level** are accepted.
- **Ordinary large-elimination tests still pass, untouched.**

**AC3 — symbolic levels, conservatively.**
- `u ≤ max u v` **accepts**.
- **Incomparable `v ≤ u` REJECTS.** *The gate fails closed: when it cannot prove
  `≤`, it rejects. An admission gate that admits on "can't tell" is not a gate.*

**AC4 — ⚠ EVERY in-repo inductive declaration is run against the repaired
kernel, and they are ENUMERATED AT THE PRODUCER.**

> **★ I have NOT counted them, and I am NOT asserting a number. You must — and
> here is the trap that will make your count wrong:**
>
> **⛔ GREPPING `data ` IN `.ken` / `.ken.md` SOURCES WILL MISS THE PRELUDE.**
> **The prelude's inductives are EMITTED FROM RUST, not written in Ken.** A
> catalog/example sweep looks exhaustive, comes back clean, and has silently
> skipped the most load-bearing declarations in the entire system.
>
> **Enumerate BOTH:** (a) every `data` in `.ken`/`.ken.md` under `catalog/`,
> `examples/`, `conformance/`, and the test corpora; **and** (b) every inductive
> the **Rust prelude emitter** declares. Report **"there are N; here are all
> N"** — with **N a number you counted**, never one inherited from a citation.
> *(This frame's author got an inventory wrong by 3.6× two WPs ago by doing
> exactly what I am telling you not to do.)*

- **Any existing declaration the new gate rejects is a CALLER BUG** — lift its
  family level, or reject it as genuinely unsound. **It is NEVER a reason to
  weaken the gate.** If one trips, **STOP and route it to me**; do not repair it
  inside this WP without telling me.

**AC5 — transactional rollback is PROVEN, not assumed.** A failed admission
leaves `GlobalEnv` **byte-identical** to its pre-call state. Assert on the env,
not on the return value. *A gate that rejects but leaves a half-admitted `D`
behind has moved the hole, not closed it.*

**AC6 — the false comment at `check.rs:945` is gone**, replaced by an
enumeration of admission clauses (a), (b), **(c)**, each citing
`14-inductive.md`.

**AC7 — FULL CI GREEN.** Per `COORDINATION.md §12` this is **never** a local
`--workspace` run. Targeted `scripts/ken-cargo -p ken-kernel` locally; **the
whole-repo gate is CI's.** A kernel admission change can only be cleared by the
whole corpus rebuilding against it — that is precisely what CI is for.

---

## 4 · Guardrails

- **Do not reopen** the Architect's ruling, the spec's rule, or the choice to fix
  formation rather than elimination. All three are settled (§1, §2).
- **Do not extend scope.** This WP touches `declare_inductive` and its tests. If
  the enumeration in AC4 surfaces a second trust-root gap, **that is a separate
  WP — route it to me, do not absorb it.**
- **Nothing else is blocked on you.** There is no freeze (see the operator ruling
  above). Do not hold, hurry, or cut corners on account of other lanes — they are
  running normally. **A trust-root repair is exactly the wrong place to feel
  time pressure.**
</content>
</invoke>
