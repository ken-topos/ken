# SUB-1b — `DecEq UInt8`, and the lawful `DecEq Bytes` it unlocks

**Team:** Language · **Size:** S · **Base:** `origin/main @ 6e415f23`
**Branch:** `wp/sub1b-uint8-deceq` · **Unblocks:** CC8 (and every future
byte-comparing consumer)

## 0. In one line

**Add exactly ONE trusted entry — a retraction certificate for `UInt8` — and
derive lawful `DecEq UInt8` → `DecEq (List UInt8)` → `DecEq Bytes` from it at
zero further trust.**

## 1. ★ START HERE: why this WP exists, and what SUB-1 did *not* do

SUB-1 landed the `Bytes` structural view: `bytes_to_list : Bytes → List UInt8`,
`list_to_bytes`, and the two round-trip postulates. **It was correct, it was
audited, and it does not reach as far as we first claimed.**

**SUB-1 moved the wall from `Bytes` to `UInt8`. It did not remove it.**

- **SUB-1's own consumer is `bytes_nat_length` — and LENGTH IS A SPINE-ONLY
  OPERATION.** Folding a list never looks at an element.
- **A key comparison is an ELEMENT operation.** CC8's environment lookup compares
  keys ⇒ it needs `DecEq Bytes` ⇒ it needs `DecEq (List UInt8)` ⇒ it needs
  **`DecEq UInt8`**.
- **`UInt8` is `PrimReduction::OpaqueType`** (`numbers.rs:290`, via `reg_ty!`).
  **You cannot case on it.** So `DecEq UInt8` is **not writable as ordinary
  structural Ken** — it hits the *identical* wall `Bytes` hits.

**Grepped, at the emission, all absent:** no `eq_uint8` primitive · no `DecEqCert`
for `UInt8` · **no injectivity or retraction law for `uint8_to_int` anywhere** in
`crates/`, `catalog/`, or `spec/`.

> **This is the expressibility audit (PRINCIPLES #14, widened; audit (b‴)):
> *try to WRITE each obligation in the shape's own vocabulary and see if your pen
> has anywhere to land.* It had nowhere to land at `DecEq UInt8`.** Note what did
> **not** catch it: **everything green.** SUB-1's tests pass, three lanes approved
> it, CI is green. **They are all correct.** The gap is not in any *value* — it is
> in what the *shape can say*.

## 2. Settled inputs — the Architect's ruling, transcribed (`evt_2z09abj5eqx5n`)

**Route B (the one retraction postulate) is RULED. Do not relitigate it, and do
not reach for Route A.**

### 2.1 What was rejected, and why it is not yours to decide

**Route A** — mint an `eq_uint8` primitive + a kernel `DecEqCert` for `UInt8`
(mirroring `Int`/ADR-0013 Layer 1) — **buys conversion-level decidability**
(`Refl` computes over byte literals). **It was rejected here for two reasons:**

1. **It buys what no consumer asks for.** CC8's lookup compares keys **at
   runtime**. It needs a `DecEq Bytes` that *computes a decision* and is *lawful*.
   **It does not need `byte_literal ≡ byte_literal` definitionally.**
2. **★ Route A IS the K3 fork, and K3 is the OPERATOR's call.** *"Make the opaque
   primitive decidable in conversion"* is precisely the K3 / registered-reductions
   decision, and it belongs to the whole opaque-primitive family, not to `UInt8`
   alone. **Deciding it here would spend kernel-decidability trust ahead of Pat's
   ruling on the family.** If Pat later rules the definitional side, **Route A/K3
   SUBSUMES this postulate in one stroke** — that is a clean supersession, not
   waste.

**⛔ Do not add a kernel `DecEqCert`. Do not mint `eq_uint8`. Do not touch
`crates/ken-kernel/`.**

### 2.2 The ONE new trusted entry

Both primitives **already exist and are already in the TCB** (`conversions.rs:94`
and `:97` — `uint8_to_int : UInt8 → Int` and `int_to_uint8_raw : Int → UInt8`,
both `PrimReduction::Op`). **You add no primitive.** You add exactly one
postulate:

```
uint8_int_retract : (x : UInt8) → Equal UInt8 (int_to_uint8_raw (uint8_to_int x)) x
```

**It is TRUE** (the Architect grounded it): `uint8_to_int` widens exactly onto
`[0,255]`, and `int_to_uint8_raw` is the identity on that range. **It is
NECESSARY**: it is the one fact that cannot be proved by casing, because `UInt8`
cannot be cased on. **This is the textbook one-extensionality-anchor pattern — a
lawful `DecEq`/`Ord` for an opaque key always needs exactly one un-case-provable
injectivity/retraction certificate, homed at the bijection layer.**

## 3. The derivation chain — every link, and every link is ordinary Ken

**Nothing below this line costs trust.** Each step is a real, kernel-checked proof.

1. **Injectivity of `uint8_to_int`**, from the retraction — the *same cong-rewrite
   argument SUB-1 already uses* for `bytes_to_list`:
   given `h : Equal Int (uint8_to_int a) (uint8_to_int b)`, apply
   `cong int_to_uint8_raw h`, then rewrite both sides by `uint8_int_retract a` and
   `uint8_int_retract b`. ⇒ `Equal UInt8 a b`. ∎
2. **`DecEq UInt8`**, transported across that injection from the **already-lawful**
   `DecEq Int` (`LawfulClasses.ken.md:197` — whose `sound`/`complete` are the
   kernel `DecEqCert`, `numbers.rs` → `env.rs:436`; **genuine proofs, not
   opaques**):
   - `eq  = λa b. eq_int (uint8_to_int a) (uint8_to_int b)`
   - `sound` — from `int_eq_sound` then **injectivity** (step 1).
   - `complete` — from `cong uint8_to_int` then `int_eq_complete`.
   ⚠ **Spell the hypothesis with the raw `eq_int …` expression verbatim**, exactly
   as `DecEq Int` does — see `LawfulClasses.ken.md` §on `Int` for *why* (the
   same-literal-expression discipline that sidesteps the `conv_struct`/K6 operand
   congruence question). **Do not `fn`-wrap it into an alias.**
3. **`DecEq (List UInt8)`** — the landed generic instance
   (`instance DecEq (List a) where DecEq a`, `LawfulClasses.ken.md:2022`) applied
   to your `DecEq UInt8`. **Nothing to write but the application.**
4. **`DecEq Bytes`** — `eq = λa b. list_deceq_eq UInt8 decEqUInt8 (bytes_to_list a)
   (bytes_to_list b)`, with `sound`/`complete` closing via **injectivity of
   `bytes_to_list`**, itself derived from **SUB-1's own `bytes_list_roundtrip`** by
   the identical cong-rewrite argument. **Zero further trust.**

## 4. Mandated deliverables

1. **The postulate** — `uint8_int_retract`, registered exactly once, in the
   numeric/conversion registration path (`conversions.rs` is its natural home;
   `bytes.rs` is not — this is a `UInt8` fact, not a `Bytes` fact).
2. **The fail-closed guard — copy SUB-1's, it is the reason SUB-1 was trusted.**
   The registration must compute the `trusted_base()` set difference across itself
   and **refuse to proceed unless it equals exactly `{uint8_int_retract}`**. A
   second entry must **fail the set equality**, not merely be noticed in review.
3. **`DecEq UInt8`** — catalog, with real `sound`/`complete` proofs (§3.2).
4. **`DecEq Bytes`** — catalog, with real `sound`/`complete` proofs (§3.4).
5. **Acceptance test** — `sub1b_uint8_deceq.rs`.

## 5. Acceptance criteria

1. **AC1 — `trusted_base()` grows by EXACTLY ONE**, and it is
   `uint8_int_retract`. **Assert it as a SET EQUALITY** (SUB-1's `ac2_…` is your
   template). A fifth/second entry fails the test.
2. **AC2 — ZERO consumer `Axiom`s.** `DecEq UInt8`, `DecEq (List UInt8)` and
   `DecEq Bytes` all elaborate with **zero** `Axiom`/`declare_postulate`/
   `Decl::Opaque` beyond AC1's single entry. **This is the whole point of the WP.**
3. **AC3 — the laws are REAL, not postulated.** `sound` and `complete` on
   **both** new instances are kernel-checked proofs. **Grep the emission:** the
   catalog files must declare **no** `Axiom`. *(A `DecEq` whose laws are `Axiom`s
   is exactly the disease — it would make the WP a no-op that merely relocates the
   trust.)*
4. **AC4 — it DECIDES, non-vacuously, on real bytes.** `DecEq Bytes` must
   **evaluate**: equal byte strings → `true`; strings differing **only in the last
   byte** → `false`; **empty vs empty** → `true`; **prefix vs longer** → `false`.
   Include a **non-ASCII / invalid-UTF-8 pair** (e.g. `0xFF` vs `0xFE`) — *a String
   hop anywhere would mangle it, and that is the pin.*
5. **AC5 — `Refl` still CANNOT discharge it, and a test says so.** `uint8_to_int`
   and `bytes_to_list` remain `PrimReduction::Op`, **opaque to kernel conversion**.
   Assert that the propositional law **accepts** and the corresponding **`Refl`
   REJECTS** (SUB-1's paired accept/reject discriminator is your template).
   **This keeps the primitive-`Op` honesty erratum intact** — SUB-1b is the
   *propositional* answer, and it must not quietly imply the definitional one.
6. **AC6 — zero kernel change.** `crates/ken-kernel/` **absent from the diff**
   (a *path* probe — that asks about files, not meaning, so a grep is sound here).
7. **AC7 — no regression.** Green **in CI**, never a local `--workspace` run
   (`COORDINATION.md §12`).

## 6. Do-not-reopen guardrails

- **No `eq_uint8` primitive. No kernel `DecEqCert`. No `crates/ken-kernel/`
  change.** (§2.1 — ruled; that is the K3 fork and it is **Pat's**.)
- **No second postulate.** If you find yourself needing one, **STOP AND REPORT** —
  it means the chain in §3 is wrong, and I want to know that rather than have it
  papered over.
- **No `Axiom` in any consumer.** If a law will not close, **stop and report**.
  *An `Axiom` here would defeat the entire purpose of the WP.*
- **No SUB-2 work** (retiring the cached-`Nat` carriers) — separate WP.

## 7. The zero-X gate discipline (read this before you write a test)

**A "zero-X" gate must check what the artifact DECLARES — never whether the string
`X` occurs in it.** This frame, and your own acceptance test, will both spell
`Axiom` **while forbidding it**; a naive grep reads a prohibition as a violation.
**Extract the literate Ken first (`extract_ken_md(…).source`), then assert.**
*A raw-file grep took SUB-1's CI red on prose that said "no local `Axiom`". Do not
rebuild that trap.* See
`agent/memory/fleet/an-oracle-that-greps-a-name-fires-on-prose-that-denies-it.md`.
