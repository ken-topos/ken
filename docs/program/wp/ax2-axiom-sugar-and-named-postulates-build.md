# AX-2 — `axiom` sugar + named postulates: **the build**

**Owner:** Team Language · **Size:** L · **Gate:** G2 (surface) + TCB surface
**Depends on:** **AX-1 — MERGED, `origin/main @ 2b0a05a5`** (the spec is law)
**CI:** ⛔ **FULL CI. NEVER `--doc-only`.**
**Operator ruling:** Pat, 2026-07-14 — *"sugar plus named postulates… adding a
label to kernel data is low risk. approved."*

---

## 1 · Objective

Build what AX-1 specified:

1. **`axiom N : T`** — surface sugar for `lemma N : T = Axiom`. Mechanical.
2. **Named postulates** — every `Decl::Opaque` carries a **required** name, so
   `trusted_base()` becomes **readable**, not merely countable.

**The spec is on `main` and is normative.** Read it first; it settles every
question this frame does not:
`spec/10-kernel/11 §4` · `spec/10-kernel/18 §4.2` · `spec/30-surface/32` ·
`spec/30-surface/39` · `spec/60-security/64`.

---

## 2 · ⛔ SETTLED — do not reopen

- **`Axiom` STAYS.** The sugar **adds** a declaration form and **removes
  nothing**. `instance Ord Int { refl = Axiom; … }` remains legal, undeprecated,
  unrestricted. `Axiom` stays legal in **every term position that has an expected
  type**.
- **The name is a LABEL, not a KEY.** **No uniqueness invariant.** Two postulates
  minted by one declaration **legitimately share a label** and keep distinct
  `GlobalId`s. Identity lives in the `GlobalId`; the label is provenance.
- **⛔ NO POSITIONAL DISCRIMINATOR.** Nothing may derive from a term position,
  allocation order, occurrence index, counter, or gensym. A positional name is
  **unstable across edits** and would **churn every `trusted_base_delta`
  assertion in the DS-suite.**
- **The name is KERNEL-INERT.** The kernel **MUST NEVER READ IT.**

---

## 3 · ★★ GROUNDING — the producer inventory, derived FROM THE GATE

**`declare_postulate` is the gate. Every postulate in Ken is minted through it.**
I enumerated it mechanically — **do not trust any inventory you did not derive
the same way**:

```sh
grep -rn "declare_postulate(" crates/ --include=*.rs | grep -v "fn declare_postulate"
```

**127 call sites · 27 files · 4 crates.** A **required** name parameter breaks
**every one of them at compile time — which is the point.**

| where | sites | what they need |
|---|---|---|
| **`ken-elaborator/src`** (8 files) | **17** | `elab.rs` ×8 (incl. the `SUGAR_AXIOM` intercept), `capabilities.rs` ×3, `bytes.rs` ×3, `foreign.rs` ×2, `prover.rs`, `numbers.rs`, `modules.rs`, `lib.rs`, `decimal_char.rs`, `conversions.rs` — **honest derived or literal names** |
| **`ken-interp/src/lib.rs`** | **8** | honest literal names |
| **⚠ `ken-kernel/src/check.rs`** | **2** | **the kernel mints its own postulates** (`sound_ty`, `complete_ty`) |
| **tests** (16 files) | **95** | mechanical literals (`k2c_series2.rs` alone has **41**) |

### ⚠ MY OWN AX-1 FRAME GOT THIS WRONG — READ THIS BEFORE YOU TRUST ANY LIST

**My AX-1 frame named THREE producer files** (`bytes.rs`, `conversions.rs`,
`capabilities.rs`). **There are twelve in `src` alone — and two of them are inside
the kernel itself.** I enumerated from the sites I had happened to read, **not
from the gate**, and then stated the result with confidence.

> **This is the third time in one day the same error has shipped from me** —
> KTR-1's population came from the symptom, KTR-2's tests came from the frame, and
> AX-1's producer list came from my reading notes. **In every case the reasoning
> was sound, ran on the wrong universe, and came back clean.**
>
> **⇒ The population of a mechanism is defined by the NARROWEST GATE EVERY MEMBER
> MUST PASS THROUGH. Here that gate is `declare_postulate`. Derive from it.**

**Treat every current-state claim in this frame as perishable.** If a pin is false
against the landed code, **say so and escalate — do not quietly build around it.**

---

## 4 · Deliverables

### D1 — Parser: the `axiom` production

Add `axiom N : T` per `spec/30-surface/32`. It desugars **mechanically** to
`lemma N : T = Axiom`; there is **no new elaboration rule**.

**⚠ A new declaration form needs a FORMATTER arm.** A production the formatter
cannot round-trip **reds the corpus fmt gate** — see AC6.

### D2 — Kernel: the required label

```rust
Opaque { id: GlobalId, name: String, level_params: Vec<LevelVar>, ty: Term },
pub fn declare_postulate(env: &mut GlobalEnv, name: String,
                         level_params: Vec<LevelVar>, ty: Term) -> KernelResult<GlobalId>
```

**`String`, NOT `Option<String>`. A required positional parameter, NOT a defaulted
setter or a builder.**

> **⛔ AND NO ESCAPE HATCH.** Do **not** add a `declare_postulate_unnamed`, a
> `Default`, or a test-only convenience wrapper. **That would rebuild the
> anonymous half of the trust base this WP exists to eliminate** — and it would
> do it in exactly the place nobody looks. **95 test call sites will not compile
> until each supplies a name. That is the feature.**

### D3 — Elaborator: thread the semantic owner

The `SUGAR_AXIOM` intercept lives in the **generic `check()`**
(`crates/ken-elaborator/src/elab.rs:474`, arm at `:502`), which today has **no
idea which semantic owner it is serving**. Thread it: the elaboration context
carries a required owner label. A declaration body uses its own name; an
instance field uses `Class.HeadType.field`; and both public standalone-expression
entrypoints require their callers to supply a stable semantic audit owner.

> ### ★★★ THE GUARDRAIL THAT MATTERS MOST — AND IT HAS ALREADY BITTEN US TWICE
>
> **If the owner label is `Option<String>` and you fall back to `"<unknown>"`
> / `""` / `"axiom"` / `"?"` when it is `None` — YOU HAVE REBUILT KTR-2's BUG,
> FOR THE THIRD TIME.** KTR-2 existed to stop the elaborator fabricating a
> placeholder **sort**; its own first diagnostic fabricated a placeholder
> **attribution** (`"<unknown>"`, `index: 0` → rendered as `#1`). **The Architect
> blocked it. A sentinel is a fabricated placeholder wearing different clothes.**
>
> **⇒ The honest shape is STRUCTURAL: give the context a NON-OPTIONAL label,
> established when elaboration enters a declaration or supplied through a
> required public standalone-expression API argument, so THERE IS NO `None`
> BRANCH TO FABRICATE INTO.** Then the type system enforces the honesty, not your
> care.
>
> **Every `Axiom` receives its label from a semantic owner:** either its
> enclosing named declaration or the explicit caller of a public standalone-
> expression API. The label is never inferred from the expression. There is no
> ownerless overload, optional/default owner, source/module fallback, session
> counter, gensym, or generic sentinel.

### D4 — Every producer supplies an honest name

All **127** sites (§3). The **17 elaborator-src** and **2 kernel-src** sites are
the ones that need thought — each mints a *specific known thing* (`retract_id`,
the two `bytes` round-trip laws, the capability authorities, the prover's goal),
so each has a **real name already**; use it. The **95 test sites** are mechanical.

### D5 — `trusted_base()` becomes readable

`crates/ken-kernel/src/env.rs:472` returns `Vec<GlobalId>`. Make the audit surface
**carry the names**, so an author can ask *"what does Ken assume?"* and get an
answer they can read.

**⚠ Existing `trusted_base_delta` count assertions MUST NOT churn.** They are the
DS-suite's trust ratchet. (This is *why* no positional index — see §2.)

### D6 — One conformance fixture

`axiom N : T` **parses**, **elaborates**, and the postulate appears in
`trusted_base()` **under its derived name**. Plus a test that
`lemma foo : T = f Axiom Axiom` mints **two** postulates that **share a label**
and hold **distinct `GlobalId`s** (§2).

---

## 5 · Acceptance criteria

- **AC1 — Enforced by TYPE, not convention.** `Decl::Opaque.name` is `String`
  (not `Option`); `declare_postulate`'s name is a **required positional**
  parameter. **A reviewer must confirm this from the type, not from a comment.**
  If a producer *can* skip the name, D2 has failed.
- **AC2 — Kernel-inertness, checked STRUCTURALLY.** Enumerate **every read of the
  name inside `crates/ken-kernel/src`** and show each is a reporting/audit path.
  **ZERO reads in conversion, typing, admission, positivity, universe checking, or
  elimination.** *Grep the emission, not the name.*
- **AC3 — The negative-attribution AC** (KTR-2's carry; mandatory in every frame
  I write now). Show structurally that every `Axiom`-capable entrypoint obtains
  its owner from an enclosing declaration or a required public API argument.
  Demonstrate there is no ownerless overload and no `None`/default/sentinel
  branch; compile-time call-site closure is the discriminator.
- **AC4 — No positional discriminator.** Grep the naming code for
  index/counter/gensym/allocation-order. **Zero.**
- **AC5 — Label sharing is legal.** The `f Axiom Axiom` test (D6) passes: same
  label, distinct `GlobalId`s. **If this test is uncomfortable to write, re-read
  §2 — it is the spec.**
- **AC6 — ⚠ CORPUS ORACLES: enumerate them, name each in the PR.** A new
  declaration form and a new fixture must satisfy **every corpus-wide oracle**,
  and they live in crates this WP never touches — so **targeted per-crate testing
  cannot see them** and they surface as **red CI at publish**, the most expensive
  place to find them. **Grep for them** (`rg 'collect\(.*catalog|examples/rosetta'
  crates/*/tests/`) and name each. **The formatter gate is rarely the only one.**
- **AC7 — Workspace-green IN CI.** ⛔ **Never a local `cargo test --workspace`**
  (COORDINATION §12, operator hard rule — it OOMs the box). Targeted `-p` locally;
  the publisher gates the merge on GitHub's full `--locked` run.
- **AC8 — No corpus migration.** Do **NOT** rewrite existing
  `lemma … = Axiom` sites to the new `axiom` form. The sugar is **additive**.
  A corpus migration is a separate WP if we want one.

---

## 6 · Guardrails — do not reopen

- **Do NOT** remove, deprecate, hoist, or restrict expression-position `Axiom`.
- **Do NOT** make labels unique, positional, or gensym'd.
- **Do NOT** add an unnamed/defaulted `declare_postulate` escape hatch.
- **Do NOT** let the kernel read the name.
- **Do NOT** mass-replace blind: 127 sites, but each src site names a *specific
  known thing*. **The tests are mechanical; the 19 src sites are not.**

## 7 · Ownership note

**Language owns this WP.** The `ken-kernel` delta is a **kernel-inert label**
(spec-normative), not a soundness change — but it *is* the kernel crate. **If
Language and Kernel leaders prefer to split the kernel commit between them, that
is a leader↔leader call, not mine.** The **Architect** terminal-reviews the
kernel-inertness bar and the TCB surface, as usual.
