# SEC4-TCB — bind the authored trust-model seed to an executing suite

**`conformance/security/trust-model/seed-trust-model.md` is a fully authored
21KB seed with five AC groups and fifteen named cases, normatively grounded in
`64`. Nothing executes it. Sec1, Sec1ct, and Sec2 each have an acceptance
suite bound to their seed; Sec4 does not.**

**Owner:** Team Verify. **Branch:** `wp/SEC4-TCB`. **Size:** M.
**Risk:** low — additive test file; the substrate under test is landed.

**Status:** Steward frame, shovel-ready. ⛔ **Not blocked.** Sec4's only DAG
dependency is `K-api`, which is landed.

---

## 1. Fixed inputs — measured at `origin/main = d6df571e`

| path | blob |
|---|---|
| `spec/60-security/64-trust-model.md` | `f3811a6992a5cbb122d67bbd650abb247f4aec37` |
| `conformance/security/trust-model/seed-trust-model.md` | `31e23f7a258fa25d9718a953273595ab00a19a4e` |
| `crates/ken-kernel/src/env.rs` | `5fede058d0ef68ff3e6b680a7ad68f41c37d3fdc` |
| `crates/ken-elaborator/tests/sec2_acceptance.rs` | `09ab813b193645a27d84e4123698f1a642ca87b7` |
| `crates/ken-elaborator/tests/km_literal_trust_accounting.rs` | `9c9d34060046a521f7722082a96518f0f5adaf99` |

⚠ `sec2_acceptance.rs` and `km_literal_trust_accounting.rs` are **read-only
references** — the shape to follow and the coverage to not duplicate.
⛔ This WP does not edit them.

⚠ **`origin/main` will have moved** (`CAT-CAPEX` was publishing when this was
written). Re-derive your base at pickup; the intersection is expected empty.

---

## 2. The measurement

Zero of the seed's case identifiers appear anywhere in `crates/`:

| seed case | hits in `crates/` |
|---|---|
| `verified-term-has-empty-trusted-base` | **0** |
| `single-postulate-lists-exactly-itself` | **0** |
| `foreign-signature-surfaces-in-delta` | **0** |
| `discharged-hole-empties-delta` | **0** |
| `user-assumption-never-prelude-hidden` | **0** |
| `registered-primitive-surfaces-in-delta` | **0** |
| `kernel-has-no-ken-generated-dependency` | **0** |
| `check-signature-exposes-no-provenance-channel` | **0** |

And the precedent is exact — each sibling suite opens by naming its seed:

```
sec2_acceptance.rs:4   //! **11 cases, AC1–AC6** per `conformance/security/capabilities/seed-capabilities.md`.
sec1ct_acceptance.rs:2 //! `conformance/security/ct/seed-ct.md` (AC1–AC7, CT-A through CT-E).
sec1_acceptance.rs:2   //! `conformance/security/ifc/seed-ifc.md`.
```

⇒ **The gap is a missing binding, not missing substrate.** `trusted_base()`
(`env.rs:492`), `trusted_base_delta` (`foreign.rs:256`), and the
`declare_postulate` choke point (`check.rs:1126`) are all landed.

---

## 3. ⭐ Steward-discharged design judgments — do not re-litigate these

### 3a. ⛔ The published external audit report is OUT OF SCOPE

`64 §6` splits Sec4 explicitly. The **machine-checkable substrate** is
delivered here. The **external, published, independent kernel-audit *report***
is deferred, and `§6` states why: it is T4-class public documentation **and a
governance call — external auditor vs. internal reviewer, and the publication
decision — that is the operator's, not an autonomous one.**

⛔ Do not write, draft, or stub that report. ⭐ A lightweight agent-context TCB
inventory MAY accompany the seed per `§6`; that is the ceiling.

### 3b. ⛔ AC5 (honest limits) is NOT an executable row — leave it to the seed

Group **E**, `honest-limits-stated-normative-not-buried`, asserts a property of
**documentation prose**. ⛔ Per operator test policy (2026-07-26) — *"test
oracles that assert facts about source code, catalog, or documentation lines
are an invitation for failure and delay; tests should focus on behavior"* — it
does **not** become a CI test. ⚠ The weak "reports drift" form is still a gate
if it can go red, so that is banned too.

⇒ AC5 stays a seed row graded by the conformance validator. **`D5` must name it
as excluded, with this reason.** ⛔ An unexplained omission reads as an
oversight.

### 3c. AC4 (Invariant TT) is admissible — assert the DEPENDENCY GRAPH, not a file

`kernel-has-no-ken-generated-dependency` is discharged against the **build's
resolved dependency graph** (`cargo metadata` for `ken-kernel`), which is build
behavior, not a source-line census. ⚠ That is the boundary: asserting
"`ken-kernel` does not depend on `ken-elaborator`" from the resolved graph is
behavior; grepping `Cargo.toml` for a string is not. Use the former.

### 3d. ⚠ The seed's code locators are STALE — re-derive, ⛔ do not repair

Measured at `d6df571e`, every locator the seed's grounding block cites has
moved:

| the seed cites | actually at |
|---|---|
| `env.rs:383` — `trusted_base()` | **`env.rs:492`** |
| `check.rs:1055` — `declare_postulate` | **`check.rs:1126`** |
| `prover.rs:367` — unproved-goal postulate | **`prover.rs:494`** |
| `env.rs:256` — `is_prelude` | **`env.rs:330`** |

⭐ **The seed's *claims* are sound; only its *locators* drifted.** ⛔ Do not read
a moved line as a semantic change, and ⛔ do not edit the seed to repair them —
`conformance/` is the enclave's, and editing a cited source moves its OID.
**Report them under `D4` and I route it.**

---

## 3e. ⚖️ AMENDMENT 2026-07-27 — the C1/C2 pair is stale; re-scoped

**Ruled `evt_ff4m551h40fz` on the ring's hard stop (`a78a7dae`). Durable here
because a ruling that lives only in the channel is not an input.**

Verified independently at `crates/ken-kernel/src/obs.rs:113`,
`eq_at_registered_literal` (ADR-0013 Layer 2):

```
Eq ty (IntLit m) (IntLit n)  ⇝  Top     if m == n
                             ⇝  Bottom  if m != n
```

### ⛔ C2 is unreachable — and that is not a kernel bug

`Eq Nat 0 0` ⇝ `Top`, so `check.rs:434`'s `Term::Eq(a_ty, x, y)` arm never
fires and `Refl` cannot be offered at it. **The seed's C2 operand can never
accept.** ⛔ Do not "fix" the kernel; the reduction is ADR-0013 as designed.

### ⭐⭐ C1 is the worse half — it is GREEN and measures the wrong mechanism

`Eq Nat 0 1` ⇝ `Bottom` — also not `Eq`-shaped. So C1 rejects, but the seed's
`expect` claims *"conversion fails, `0 ≢ 1`"* and **conversion is never
reached**.

⇒ ⭐ **C2 fails loudly; C1 passes silently while measuring something else.** A
rejection control that passes for any reason is not a control, and the green
arm is the one that would have shipped unexamined.

⇒ **The defect is the pair, not the row.** The seed's own `why` asserts *"the
only difference is the proposition's truth"*; in landed behavior both arms
bypass `Refl` entirely and differ only in which constant the reducer picks.

### ✅ Authorized re-scope

`eq_at_registered_literal` returns **neutral** when either operand is not a
literal, so abstract binders keep the goal unreduced, and
`ds6c_intlit_elaborator_emission.rs::refl_still_accepted_on_a_genuinely_abstract_eq_shaped_goal`
proves the accept arm is reachable.

- **accept** `Equal Int x x`; **reject** `Equal Int x y` (distinct binders).
  Both reach `check.rs:434`'s `Term::Eq` arm and flip on `convert` alone.
- ⭐ **Retain** the closed `Id Nat 0 1` arm, asserting what it actually does
  (rejects via the `Bottom` collapse) — real landed behavior, just not under a
  `why` that names conversion.

### ⛔ And report the difference — this is checked

⚠ `Equal Int x y` is **unprovable, not false.** The seed is framed on truth;
the re-scoped pair flips on *convertibility at a genuinely Eq-shaped goal*.
⛔ Do not gloss that. **`D4` must carry all three:** (1) C2's operand is
unreachable, naming `obs.rs:113`; (2) **C1 passes via `Bottom`, not
conversion**; (3) how the re-scoped pair's claim differs from the seed's.

⭐ The substitution is authorized; **concealing it was what was forbidden.**

⇒ Seed defect filed as `CONF-SEC4-REFL-PAIR` (owner `spec-enclave`).
⛔ `conformance/` stays theirs; its absence does not gate this WP's close.

---

## 4. ⛔ Banned shapes

- ⛔ **No new CI gate or test asserting facts about source, catalog, doc, or
  spec lines.** See `§3b`. This includes a "reports drift" checker.
- ⛔ **Do not edit `conformance/`.** The seed is the enclave's artifact.
- ⛔ **Do not edit `crates/**/src/**`.** This WP observes landed behavior; it
  does not change it. If a seed row cannot be discharged because the substrate
  genuinely lacks a capability, ⛔ **stop and route to the Steward** — that is
  an implementation gap and a different WP.
- ⛔ **Do not edit or absorb `km_literal_trust_accounting.rs`,
  `k5_absurd_trusted_base.rs`, `b1_acceptance.rs`, or `i8_clock_effect.rs`.**
  They have other owners and consumers. See `D2`.
- ⛔ **Do not hand-insert a delta.** `64 §1.2` names this exact failure: the
  seed's force *"is that the conformance corpus can drive a **real**
  `foreign`/hole admission and observe it surface — the omission net — rather
  than accepting a hand-inserted delta."* ⭐ **This is the highest-severity way
  to fail this WP**, and it is the same defect Verify already fixed in
  `DOC-GATE-NEEDLE` (a control asserting on a needle the test itself supplied).

---

## 5. Deliverables

- **`D1`** — `crates/ken-elaborator/tests/sec4_acceptance.rs`, binding the
  seed's groups **A–D** (AC1–AC4), opening with the `//!` seed reference in the
  sibling suites' shape.
- **`D2`** — ⭐ a **prior-coverage census**: for each seed row, whether an
  existing test already discharges it, named as `file.rs::test_name`. Rows
  already covered are **cited, not duplicated**. ⚠ Known overlap to check
  first: `km_literal_trust_accounting.rs` (`foreign_axiom_and_open_obligation_
  trust_entries_still_count`, `literal_classification_is_the_only_primitive_
  accounting_exclusion`), `k5_absurd_trusted_base.rs`, `b1_acceptance.rs`,
  `i8_clock_effect.rs:139-155`.
- **`D3`** — each newly-bound row's **non-vacuity control** (see `AC-3`).
- **`D4`** — the stale-locator report from `§3d`, re-derived at your base.
  ⛔ Report only; do not repair.
- **`D5`** — a **closed** report: which of the seed's five AC groups this suite
  executes, which it does not, **and why**. ⛔ Must name AC5 (`§3b`) and the
  deferred audit report (`§3a`) as deliberate exclusions.

---

## 6. Acceptance criteria

- **`AC-1`** — `D1` executes and is green, and you name the command.
  **Control:** `scripts/ken-cargo test -p ken-elaborator --test sec4_acceptance`,
  output shown. ⛔ A test file nothing runs is not a binding.

- **`AC-2`** ⭐ **(load-bearing)** — **every assumption observed in
  `trusted_base()` got there by a real admission.** **Control:** for each
  surfacing row, the postulate/`foreign`/hole is introduced by *driving the
  elaborator*, and the test asserts the entry appears in `trusted_base()`
  **after** an admission that the same test performed through the production
  path. ⛔ Constructing a `GlobalId` and asserting it is present measures
  nothing. `64 §1.2` calls this the omission net; a hand-inserted delta fails
  this WP.

- **`AC-3`** — every newly-bound row has a **discriminating twin**.
  **Control:** for each row, the near-miss case gives the **opposite** result
  through the **identical** harness — a verified term yields an **empty**
  base where the postulate-bearing twin yields exactly one entry; a discharged
  hole **empties** the delta where the open one populates it. ⛔ A one-sided
  assertion passes for any reason, including a harness that silently skipped
  the program.

- **`AC-4`** — `D2` is complete and **honest about what it found**.
  **Control:** report rows in three buckets — newly bound, already covered
  (cited), and not covered with the reason. ⚠ ⛔ **A row that was already green
  elsewhere and is now green here is the case most likely to be silently
  double-counted; name those explicitly.** An empty "already covered" bucket
  is a failed census, not a clean result — the overlap in `D2` is measured and
  real.

- **`AC-5`** — `D5` is closed and names its complement, per `§3b`/`§3a`.
  ⛔ An empty exclusion list is a failed measurement.

- **`AC-6`** — `trusted_base()` is **unperturbed by this WP**. **Control:**
  no file under `crates/**/src/**` and nothing under `conformance/` is
  modified. ⭐ This WP's whole claim is that it *observes*; a source change
  invalidates that framing.

⛔ **No CI checker asserting source/catalog/doc facts** (operator policy) —
`AC-1`–`AC-3` are discharged by **elaboration and enumeration behavior**, which
is the behavioral form the policy asks for.

---

## 7. Contention and sequencing

**One new file under `crates/ken-elaborator/tests/`, additive.** ⚠ Re-measure
at pickup, not from this frame. `CAT-CAPEX` landed `cat_capex_authority.rs` in
the same directory — a **different file**, no overlap. No other ring holds
`conformance/security/trust-model/` or `ken-kernel/src/env.rs`.

⛔ Verify holds no other releasable node — `SEC1-IFC-R3` is blocked (see `§8`).

---

## 8. Hard stop

⛔ Route to the Steward if:

- a seed row cannot be discharged without editing `crates/**/src/**` — that is
  a substrate gap and a different WP; **or**
- discharging a row appears to require a source/doc-line assertion — ⛔ do not
  write one, and do not weaken the row to fit. Report it and I will re-scope;
  **or**
- the prior-coverage census (`D2`) shows a seed row is discharged by an
  existing test **incorrectly** — that is a finding worth more than this WP,
  and it should not be quietly overwritten.

⚠ **Unrelated and already ruled:** `SEC1-IFC-R3` stays `draft`. Its
`AC-R3b`/`AC-R3c` need a refutation over product-program obligations, and the
only production route to `Verdict::Disproved` is `attempt_d`'s hardcoded
`Eq(IntLit, IntLit)` arm. `z3` is not a workspace dependency; that decision is
the operator's. ⛔ Do not pick it up alongside this.
