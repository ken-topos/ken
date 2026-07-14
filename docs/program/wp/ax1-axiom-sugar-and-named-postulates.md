# AX-1 — `axiom` sugar + **named postulates**: the spec pin

**Owner:** Spec enclave (spec-author + conformance-validator)
**Size:** S · **Gate:** G2 (surface) · **Depends on:** nothing
**Blocks:** AX-2 (the Language build)
**Operator ruling:** Pat, 2026-07-14 — *"sugar plus named postulates… adding a
label to kernel data is low risk. approved."*

---

## 1 · Objective

Pin the spec for two changes that travel together:

1. **`axiom N : T`** — surface sugar for `lemma N : T = Axiom`. **Purely
   mechanical.**
2. **Named postulates** — every `Decl::Opaque` carries a **name**, so
   `trusted_base()` becomes **readable**, not merely countable.

**This is a transcription of a decided design, not a design fork.** Everything
in §2 is settled by the operator and must not be reopened.

---

## 2 · ⛔ SETTLED — do not reopen

- **`Axiom` STAYS** as an expression. The sugar **adds** a declaration form; it
  **removes nothing**. `instance Ord Int { refl = Axiom; … }` remains legal and
  is **not** deprecated.
- **The sugar is mechanical.** `axiom N : T` ≡ `lemma N : T = Axiom`. No new
  elaboration semantics.
- **Postulates get names.** The name is a **label on kernel data** — the
  operator has ruled this low-risk and approved it.

---

## 3 · Grounding — read this before writing a line of spec

### 3.1 · `Axiom` has **no kernel support**, because no `Axiom` term exists

It is **not** in the grammar, **not** in the prelude, and **not** a kernel term
(zero hits in `crates/ken-kernel/src`). It is a **string-matched intercept in the
elaborator** (`resolve.rs:648` `SUGAR_AXIOM`), fired in **checking mode**
(`elab.rs:508`):

```rust
RExpr::RCon(name, rspan) if name == SUGAR_AXIOM => {
    let id = declare_postulate(cx.env, vec![], expected.clone())?;
    Ok(Term::const_(id, vec![]))
}
```

**It mints a fresh `Decl::Opaque` whose type is the expected type**, and returns
a reference to it. The kernel only ever sees an **ordinary constant with a type
and no body**. `classify` still runs on the type, so an ill-formed proposition
cannot be postulated. **Nothing "terminates at an `Axiom`" — there is nothing to
check.** *(This is also why it is checking-mode only: it needs `expected` to know
what to postulate, and why it works in any term position.)*

### 3.2 · ★ THE TRUST BASE IS **ANONYMOUS — AND WIDER THAN THE SURFACE**

```rust
Opaque { id: GlobalId, level_params: Vec<LevelVar>, ty: Term },   // ← NO name
pub fn declare_postulate(env, level_params, ty) { … env.fresh_id() … }
```

**`trusted_base()` (`env.rs:472`) returns `Decl::Opaque` + non-literal
`Decl::Primitive`, minus `is_prelude`. And `is_prelude` excludes EXACTLY THREE
ids:**

```rust
fn is_prelude(&self, id) -> bool { top_id == id || bottom_id == id || tt_id == id }
```

> **⇒ It is NOT a prelude filter. Postulates minted from RUST are IN the trust
> base, and they are ANONYMOUS:** `bytes.rs:136` `bytes_round_trip_law`,
> `bytes.rs:260/275` the two round-trip laws, `conversions.rs:133` `retract_id`,
> `capabilities.rs:428/432/444` the authority postulates.
>
> **Naming only the surface `Axiom`s would name HALF the trust base and leave
> the half no Ken author can even see anonymous. That is backwards.**
> **⇒ The name goes on `declare_postulate`, so EVERY producer feeds it.**

### 3.3 · The surface population is **closed**, and both shapes are nameable

Enumerated at the gate over **all** Ken sources (not just `catalog/`). **Every**
real `Axiom` is exactly one of two shapes:

| shape | example | derived name |
|---|---|---|
| top-level declaration body | `lemma prim_eq_axiom : … = Axiom` | the declaration's own name |
| **instance method field** | `instance Ord Int { refl = Axiom; … }` | `Ord.Int.refl` |

**There is no third shape.** *(Sites: `LawfulClasses.ken.md:213-216`,
`StringBijection.ken.md:15`, `proof-techniques.ken.md:159,356`,
`conformance/challenge/C1,C6,C7`.)*

### 3.4 · ★ Uniqueness of the instance-derived name comes FREE from coherence

`Ord.Int.refl` is unique **because instance resolution already guarantees at most
one instance per (class, head type)** — if it did not, `Ord Int` would be
ambiguous at every call site.

> **The naming scheme borrows its uniqueness proof from an invariant Ken already
> enforces.** **⇒ No gensym, no collision counter, and — critically — NO
> POSITIONAL INDEX.** A positional name would be **unstable across edits** and
> would **churn every `trusted_base()` delta assertion in the DS-suite.**

---

## 4 · Deliverables (spec only — AX-2 builds it)

- **D1 — Grammar.** Add the `axiom` declaration production to
  `spec/30-surface/32-grammar.md`, alongside `lemma`. State the desugaring:
  `axiom N : T` ≡ `lemma N : T = Axiom`. **Mechanical; no new semantics.**
- **D2 — `Axiom` is specified.** It is currently **absent from the grammar
  entirely** while being a load-bearing trust primitive. **Fix that**: specify
  `Axiom` as the checking-mode postulate intercept (§3.1), including that it
  mints a typed, body-less opaque of the *expected* type, and that it is legal
  in **any** term position (hence instance fields).
- **D3 — Named postulates, normative.** Every postulate carries a name. Specify
  the **derivation rule**: declaration body → the declaration's name; instance
  field → `Class.HeadType.field`; and state the **coherence-uniqueness argument**
  (§3.4) as the reason no disambiguator is needed.
- **D4 — The trust-surface section.** Specify that `trusted_base()` is
  **readable**: an author can ask *"what does Ken assume?"* and get **names**.
  Record that this includes **elaborator-minted** postulates, not only surface
  `Axiom`s (§3.2).
- **D5 — ⛔ THE KERNEL-INERTNESS BAR (normative, and the reason this is safe).**
  **The name is a LABEL. The kernel MUST NEVER read it.** Nothing in
  conversion, typing, admission, positivity, universe checking, or elimination
  may branch on a postulate's name. **State this as a normative constraint**, so
  AX-2's review has a bar to check and the TCB argument is written down rather
  than assumed.

---

## 5 · Acceptance criteria

- **AC1** — D1–D5 land in `spec/`; `spec/SPEC-PROGRESS.md` updated.
- **AC2** — The grammar example **satisfies its own rules** (a spec example that
  violates the grammar it introduces is a recurring defect).
- **AC3** — **The naming rule is stated so that it cannot be implemented
  positionally.** If a reader could satisfy D3 with a term index, D3 is
  under-specified — **re-word it.**
- **AC4** — **No claim that `Axiom` is removed, deprecated, or restricted.** It
  is not (§2).
- **AC5** — **CV reconcile:** the spec's `Axiom` description matches the **landed
  elaborator mechanism** (§3.1) — *grep the emission, not the name*.

## 6 · Guardrails

- **Do NOT** design an `Axiom`-removal, an instance-field ban, or a hoisting
  requirement. **Pat asked for sugar + names. That is the whole scope.**
- **Do NOT** specify kernel *behavior* on names beyond D5's prohibition.
- **Do NOT** invent a name for a postulate with no derivable path — **there are
  none today** (§3.3). If AX-2 finds one, it **escalates**; it does not gensym.
