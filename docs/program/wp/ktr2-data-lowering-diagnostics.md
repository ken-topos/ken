# KTR-2 — data-declaration lowering: no fabricated sorts, actionable universe diagnostics

**Owner:** Team Language · **Size:** S/M · **Gate:** G2 (surface/elaboration)
**Depends on:** KTR-1 (merged `65d68cfc`) · **Blocks:** nothing
**Frame author:** Steward · **Design ruling:** Architect, `evt_28bw30t80dx7d`

---

## 1 · Objective

Repair a **production correctness defect** in the legacy `data` lowering path —
it fabricates a *sort* for an unknown type name — and turn KTR-1's kernel
rejection into an **actionable surface diagnostic**.

This is **implementation and diagnostic completion of an already-decided
design**, not a design fork. Everything in §2 is settled and **must not be
reopened**.

---

## 2 · ⛔ SETTLED INPUTS — do not reopen

The Architect ruled on all of these (`evt_28bw30t80dx7d`). They are **fixed
inputs**, not questions:

### 2.1 · The universe posture is already decided — and the escape already works

The explicit form **already carries a family universe end to end.** Verified
link by link on `origin/main`:

| link | file | behavior |
|---|---|---|
| surface AST | `ast.rs:596` | `TUniv(Option<u32>, Span)` — `Type 1` is a real type |
| resolve | `resolve.rs:516` | `TUniv(level, _) => (indices = [], level)` |
| elaborate | `data.rs:161` | `level.map(level_from_nat).unwrap_or(Level::Zero)` |
| kernel | — | `InductiveSpec { level: Succ(Zero) }` |

⇒ **`data D : Type 1 where { C : (s : Type) → D }` is admitted today**, under
KTR-1's gate: the argument's type is `Type 1`, the family is at level 1, and
`1 ≤ 1` passes. **This is the valid, existing escape. Build on it.**

### 2.2 · ⛔ DO NOT infer a family universe from its constructors

Inference would make **a declaration's TYPE depend on its own BODY** — adding a
constructor could silently lift `D`'s universe, changing `D`'s type and breaking
every downstream user at a distance. That is precisely what predicativity and
explicit annotation exist to prevent.

### 2.3 · ⛔ DO NOT add a universe slot to the legacy sugar

The sugar (`data D a = C A | …`, `32-grammar:33`) is **intentionally** the
bounded, non-indexed `Type 0` form; the explicit family form is the general one
(`32-grammar:121` says so in as many words). **Its inability to lift is not a
defect and is not yours to fix.** An author who needs a lifted family uses the
explicit form.

### 2.4 · ⛔ The KERNEL GATE remains authoritative

A surface preflight may improve *attribution*, but it **must not replace or
weaken KTR-1's admission check**. **Do not put surface-syntax advice into the
kernel TCB** — the kernel reports `ConstructorUniverseViolation`; the
*elaborator* translates it for humans.

---

## 3 · The defect (grounded, at the producer)

The two lowering functions in `data.rs` diverge exactly where it matters:

| function | unknown type name | used by |
|---|---|---|
| `rtype_to_kernel_checked` (`:433`) | `Err(ElabError::UnresolvedCon)` ✅ | explicit form (`:138`, `:151`, `:241`, `:248`) |
| **`rtype_to_kernel` (`:391`)** | **`Term::ty(Level::Zero)`** ⛔ | **legacy sugar (`:64`)** |

**`data.rs:391` fabricates a sort** — the *same* sort-as-placeholder idiom KTR-1
was built to catch in test fixtures, sitting in the elaborator itself:

```rust
} else {
    // Unknown name — produce a type-level placeholder.
    Term::ty(Level::Zero)
}
```

**It is production-reachable.** `resolve_type` (`resolve.rs:1589`) **never
rejects an unknown type name**: `TCon(name) → RCon(name)` unconditionally, no
scope check — and `TVar` *falls back* to `RCon` when not in scope. The name
survives resolution and lands on `globals.get(name) == None`.

```
data D = C Foo      -- Foo undefined, or declared LATER in the file
```

- **Before KTR-1:** silently declared `C : Type 0 → D`. **A typo produced a
  different, wrong declaration, with no error at all.**
- **After KTR-1 (now on `main`):** the same typo trips the `Δₖ` gate and reports
  **`ConstructorUniverseViolation`** — a *universe* complaint for what is really
  an *undefined name*.

> **KTR-1 did not cause this; it converted it from SILENT to
> LOUD-BUT-MISLEADING.** That is a strict improvement and exactly why the gate is
> right — **but the confusing diagnostic is now user-visible, and that is what
> this WP fixes.**

**Forward references are in scope.** The Architect confirmed file elaboration is
**in declaration order** (each declaration registered before the next), so a name
declared *later* is a forward reference and **must reject exactly like an
undefined name**. A name declared *earlier* still resolves and **must keep
working**.

---

## 4 · Deliverables

### D1 — Eliminate the fabricated-sort fallback

Route the legacy path through **one checked, `Result`-preserving lowering
boundary** (unify with `rtype_to_kernel_checked`, or an equivalent single
builder shared by both forms). **No placeholder term may reach the kernel.**

Unknown **and** forward type names must produce `ElabError::UnresolvedCon`.

### D2 — Actionable surface diagnostic

Translate a **genuine** constructor-universe rejection at the **elaborator**
boundary into a diagnostic that tells the author what to do:

- identify the **constructor argument** (name/span where available),
- report the **argument's level** and the **family's level**,
- **suggest the explicit form**: `data D : Type n where { … }`.

### D3 — Discriminating tests

Each of these must be a **separate** assertion on a **specific** error variant
(not `is_err()`):

1. undefined uppercase name → `UnresolvedCon`
2. unbound lowercase name (the resolver `TVar → RCon` fallback) → `UnresolvedCon`
3. **later** declaration → rejects; **earlier** declaration → **accepts**
4. ordinary legacy `Type 0` data → **still accepts** (no over-rejection)
5. genuine legacy universe overflow → **remains a universe error**
6. explicit `data D : Type 1 where { C : (s : Type) → D }` → **accepts** (§2.1)

---

## 5 · Acceptance criteria

- **AC1** — `git grep -n 'Term::ty(Level::Zero)' -- crates/ken-elaborator/src/data.rs`
  shows **no fabricated-placeholder site**. The `Δ_p` parameter uses (`:46`) and
  genuine surface-`Type` lowering (`:417`, `:490`) are **correct and stay** —
  only the **unknown-name fallback** dies. *Read each hit; do not mass-replace.*
- **AC2** — All six D3 cases pass, each asserting a **named error variant**.
- **AC3** — **No kernel change.** `git diff --name-only` touches **no**
  `crates/ken-kernel/**`. KTR-1's gate is untouched and remains authoritative.
- **AC4** — **No grammar/spec change.** The sugar grows no slot (§2.3).
- **AC5** — No regression: **green in CI** (never a local `--workspace` run).
  Local gates are **targeted only** (`-p ken-elaborator`, `--test <name>`).
- **AC6** — Full CI on publish. **Never `--doc-only`** — this is Rust.

---

## 6 · Guardrails

- **Do not touch `crates/ken-kernel/**`.** If you believe the gate is wrong,
  **stop and route** — do not weaken it.
- **Do not lift a family** to make an error go away.
- **Do not mass-replace `Term::ty(Level::Zero)`.** Most occurrences in the repo
  are **correct** (`Δ_p` parameters, genuine `Type` lowering). *The idiom is
  right in one position and wrong in another; read each site.*
- **Do not add inference or a sugar universe slot** (§2.2, §2.3).
