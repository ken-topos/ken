# B2W-1 — Repair the B2 `Temporal` kernel-admission witness

**Owner:** Team Language (elaborator) · **Size:** S · **Risk:** ★★ ·
**Stream/gate:** WS-B / G-Ward-seam evidence · **Blocks:** KTR-1's 89-site sweep

**Status:** RELEASED.

> **Chain:** KTR-1's new constructor-universe gate rejected an existing caller
> (`evt_4pmjt0hda4zej`) → **Architect ruling `evt_2a5p6fp42e5xp`: gate stands,
> repair the caller, separate WP** → this frame. **Kernel is HELD at
> `wp/ktr-1-constructor-universe-gate @ 54db124e` until this lands.**

---

## 0 · What is actually wrong — and what is NOT

`crates/ken-elaborator/src/temporal.rs:252` `temporal_inductive_spec` declares
the `Temporal` family at `Level::Zero` with **`params: vec![]`**, and stubs its
two deferred carriers as **the SORT itself**:

```rust
let pred = Term::ty(Level::Zero);  // (oracle): `Pred Σ` spelling deferred
let var  = Term::ty(Level::Zero);  // (oracle): `Var`  spelling deferred
```

A constructor argument whose **type is `Type 0`** synthesizes to **`Type 1`**.
The family is at **`Type 0`**. ⇒ `ConstructorUniverseViolation { argument: suc 0,
family: 0 }`, first at `atom`.

> ### ⛔ SCOPE — read this before you touch anything
>
> **This is the kernel-admission WITNESS. It is NOT the production carrier.**
> The **production** `Temporal` is a plain **Rust `enum`** (`temporal.rs:71-90`);
> production obligations flow **`TemporalObligation → TEntry → T/delegated`** and
> **never pass through `declare_inductive`, `Q`, or the trusted base.**
> `temporal_inductive_spec` is called **only from
> `crates/ken-elaborator/tests/b2_acceptance.rs`.**
>
> **⇒ NOTHING in production is broken. No `Q` certificate, no persisted export, no
> checked-core proof, no trusted-base entry.** *(The Steward initially claimed
> otherwise. He was wrong; the Architect grounded the call graph. Do not inherit
> the louder framing.)*
>
> **⛔ DO NOT TOUCH, and these are not up for discussion:** the public `Temporal
> Σ` (`spec/70-behavioral/72-temporal.md`), the Rust `enum Temporal`, `Pred`,
> `Var`, `TemporalObligation`, `TEntry`, the export schema, `emit_export`.
> **This is a witness repair, NOT a public redesign.**

## 1 · Why it still matters — the defect is VACUOUS EVIDENCE

**The witness's entire job is to demonstrate §72's claim** that `Temporal` is
admissible as an **ordinary** inductive — *"K1 admits it **without** the K1.5
W-style path"* — i.e. **no kernel enlargement.**

**And it demonstrated that with a declaration the correct kernel REJECTS.**

> **The claim is almost certainly TRUE** — `Temporal` admits fine once `P`/`V` are
> real parameters. **But it was never actually demonstrated.**
> ***The witness that proves the datatype is safe was itself unsafe.***
>
> **A green test that "proves" admissibility by admitting something inadmissible
> proves NOTHING — and it would have gone on proving nothing forever, because it
> passed.** *This is the CC6b/LET-1 family: a gate that cannot fail is not a gate.*

**⇒ Your deliverable is not "make the test pass." It is "make the test MEAN what
it claims."**

## 2 · The repair (Architect-directed — a FIXED INPUT, not a design fork)

**Give the witness two genuine type parameters `P V : Type 0`**, abstracting the
deferred `Pred Σ` / `Var` spellings.

- `params: [Type 0, Type 0]`. The family stays **non-indexed**, at `Level::Zero`.
- **`atom` takes `P`** · **`mu`/`nu` take `V` then `D P V`** · **`var` takes `V`**
  · `not`/`and`/`or`/`next`/`until` take `D P V` operands as today.
- **Every recursive occurrence must be FULLY APPLIED `D P V`** — today it is
  `Term::indformer(d_id, vec![])`, i.e. **unapplied**.
- **Every constructor targets the same parameter instance.**
- **The HOAS foil gets the SAME parameters.** Its body argument becomes
  `D P V → D P V`, with the **codomain shifted beneath the `Pi` binder**.
- **TE-A1** must instantiate **concrete small `P`,`V`**, pass them to
  `method_type`, call `recursive_args(..., 2)`, and assert the **two-parameter
  former** and the **fully-applied recursive** shapes.

### ★★ TRAP 1 — DE BRUIJN SHIFTING THROUGH PRECEDING CONSTRUCTOR BINDERS

**This is where this WP will actually go wrong.** `P` and `V` are now **variables
bound by the parameter telescope**, so their indices are **relative to the
current depth** — and **every preceding argument binder in a constructor's
telescope shifts them by one.**

```
and : (_ : D P V) → (_ : D P V) → D P V
       ^ P,V at depth d          ^ P,V at depth d+1   ^ at depth d+2
```

**The second operand's `P`/`V` are NOT the same term as the first's.** Build them
per-position, at the correct depth. **A copy-paste `d.clone()` across argument
positions — which is exactly what the current code does, and is CORRECT today
because `d` is unapplied and closed — becomes WRONG the moment `D` carries
parameter references.**

> **The existing code's `d.clone()` idiom is safe only because `indformer(d_id,
> vec![])` is CLOSED. Parameterizing it silently makes that idiom a bug.** *This is
> the whole difficulty of the WP; the rest is typing.*

**Same for the HOAS foil:** `Pi(D P V, D P V)` — **the codomain sits under the
`Pi` binder and its `P`/`V` shift by 1.**

### ★★★ TRAP 2 — THE FOIL CAN GO VACUOUS, AND EVERY TEST WILL STILL BE GREEN

**`temporal_hoas_inductive_spec` exists to prove the positivity checker REJECTS a
negative occurrence** — it is the **negative arm** of a non-degenerate pair
(`72 §3.1`).

**If you parameterize it wrong, it will fail with a UNIVERSE error instead — at
the universe gate, BEFORE `check_positivity` ever runs.** The test asserting
"the HOAS variant is rejected" **still passes.** And **positivity is now
completely untested.**

> **You would have replaced one hollow witness with another, and the suite would
> be green both times.** *That is the exact failure this entire WP exists to
> correct — reproduced, one level down, by the repair itself.*

**⇒ AC3 is therefore the most important AC in this frame: assert the SPECIFIC
error variant `PositivityViolation` — NOT `is_err()`, NOT "some error."**

---

## 3 · Acceptance criteria

**AC1 — the witness is ADMITTED by the repaired kernel.**
`declare_inductive(temporal_inductive_spec)` succeeds against **KTR-1's gate**.
*(Coordinate with @kernel-leader: KTR-1's branch `54db124e` carries the gate. You
may develop against it, but **B2W-1 lands on its own** and Kernel rebases after.)*

**AC2 — it is admitted WITHOUT the K1.5 W-style path.** §72's actual claim.
Assert positivity accepts it as **first-order/direct**, i.e. the thing the
witness was always supposed to show.

**AC3 — ★ the HOAS foil still fails with `PositivityViolation`, ASSERTED BY
VARIANT.** Not `is_err()`. **If it fails with a universe error, the foil is
vacuous and the WP is NOT done, even though the test is green.** *Make the test
name say so.*

**AC4 — TE-A1 re-derives the shapes** with concrete small `P`,`V`:
two-parameter former; `recursive_args(..., 2)`; **fully-applied `D P V`** in every
recursive position.

**AC5 — ZERO public-surface delta.** `git diff` touches **only**
`crates/ken-elaborator/src/temporal.rs` and `crates/ken-elaborator/tests/b2_acceptance.rs`.
**No** change to `spec/`, the Rust `enum Temporal`, `Pred`, `Var`,
`TemporalObligation`, `TEntry`, `export.rs`, the kernel, or `Cargo.lock`.
**If you find yourself editing anything else, STOP and route to the Steward.**

**AC6 — targeted gates only.** `scripts/ken-cargo test -p ken-elaborator`
(`b2_acceptance`), plus the affected nets. **Never `--workspace` locally**
(`COORDINATION §12`); CI owns the whole-repo gate.

## 4 · Guardrails

- **Do not lift the family to `Type 1`.** It accommodates the stub rather than
  fixing it, and §72 promises **no kernel enlargement**.
- **Do not weaken KTR-1's gate.** It is correct; it found this.
- **Do not redesign `Temporal`.** The `(oracle)` deferral of `Pred Σ` / `Var`
  spellings **stays deferred** — `P`/`V` are exactly the abstraction of that
  deferral, which is what the doc comment at `:248` already said they were.
</content>
