# SUB-1 — The `Bytes` structural view (`bytes_to_list` / `list_to_bytes`)

**Owner:** Team Language · **Size:** M · **Base:** `origin/main @ c5f73b9c`
**Branch:** `wp/sub1-bytes-structural-view`
**Driver:** **Operator ruling, 2026-07-14** (see §1). **This is a deliberate,
bounded trusted-base extension. It is not a soundness compromise — it is the
principled fix, and it REDUCES total trust.**

## 1. The ruling this WP implements

The operator ruled the long-open `Bytes → Nat` substrate decision. **His
reasoning, verbatim — it is now `docs/PRINCIPLES.md` #15:**

> *"`String → List Char` is an essential reasoning tool that needs to be part of
> the trusted base; `Bytes` is parallel. In general, features of the
> implementation which we guarantee by our implementation should have the
> corresponding propositions which enable reasoning from those guarantees. It is
> preferable for us to extend the trusted base by a fixed number (per built-in)
> in order to prevent unbounded `Axiom`s added by consumers of Ken."*

**The defect being fixed.** `Bytes` is opaque and `bytes_length : Bytes → Int`
returns an **opaque `Int` with no `Int → Nat`** — so a byte length can be
*named* but never *used*. **You cannot write a terminating structural fold over
a bare `Bytes`.** Every consumer therefore carried a **cached `Nat` + an `Axiom`
that the cache agreed with the real length**: CAT-5's `Source`
(`SourceLength … = Axiom`), CC3's `ArgBytes` (`ArgByteLength … = Axiom`). **Four
consumers paid that tax.** And a surface `Axiom` is `declare_postulate` →
`Decl::Opaque` → **a real `trusted_base()` entry** — so the "zero-TCB" workaround
was never zero-TCB. **It was unbounded TCB, paid per call site, reviewed by
nobody.**

**`String` already has the fix.** `string_to_list_char : String → List Char` and
`list_char_to_string` are landed trusted primitives (`prelude.rs:1100–1110`,
`37 §2.3`). **`Bytes` never got the parallel view. That asymmetry is the bug.**

## 2. Deliverable

### 2.1 The primitive pair

```
bytes_to_list : Bytes → List UInt8
list_to_bytes : List UInt8 → Bytes
```

Mirror the landed `String` pair in every respect: registration, totality,
interpreter reduction, and audit posture. `UInt8` already exists (`bytes_at :
Bytes → Int → Option UInt8`; `ExitCode = … | Failure UInt8`), so `List UInt8` is
constructible today.

**Home them where the other `Bytes` destructors live** —
`crates/ken-elaborator/src/bytes.rs::register_safe_bytes_ops` — **unless** the
`List`/`UInt8` type ids are not reachable at that layer, in which case
`prelude.rs` alongside the `String` precedent is correct. **Ground it and tell
me which; do not guess.**

### 2.2 The reasoning propositions — the OTHER HALF of the ruling

**A primitive without its propositions is exactly the defect we are fixing.**
Ship the round-trip laws so consumers can *reason from* the guarantee rather
than re-postulate it:

```
list_to_bytes (bytes_to_list bs) ≡ bs
bytes_to_list (list_to_bytes l)  ≡ l
```

Mirror whatever posture the landed `String` round-trip uses (`37 §2.3`). **See
§4 — read it before you design these.**

### 2.3 The derived byte surface — ZERO further TCB

**Follow the `String` precedent exactly** (`37 §2.5`, Architect ruling
`evt_4k1yqah3yvpds`, "Approach A — do **not** native-ize"). The everyday byte
operations are **derived** — ordinary `view`s over the `List UInt8` view, routed
through the round-trip — **not** new native primitives. They add **zero**
`trusted_base()` delta.

**Do not native-ize a byte surface.** The whole point of the ruling is a **fixed
per-builtin** cost. A sprawl of native byte ops would betray it.

### 2.4 Spec + registry + conformance

- A `Bytes` section parallel to `37 §2.3/§2.4/§2.5` (the view, the round-trip,
  the derived surface).
- The primitive-registry entries (`spec/10-kernel/18a-primitive-registry.md`).
- A conformance seed. **State the trust posture HONESTLY** (PRINCIPLES #8) — see
  §4.

### 2.5 The non-vacuous consumer (this is the AC that proves it works)

**One worked structural fold over a `Bytes` that TERMINATES and needs NO
`Axiom`** — e.g. split a byte string on `/`, or count occurrences of a byte.
**This is the thing that was impossible before.** It must be a real
`match`/recursion over the `List UInt8` view, checked by the real termination
checker, with **no cached-`Nat` carrier and no postulate at the call site.**

## 3. Acceptance criteria

- **AC1 — the fold that was impossible is now possible.** §2.5's structural fold
  type-checks, passes SCT, and runs. **Zero `Axiom` in it.**
- **AC2 — the trusted-base delta is EXACTLY the fixed cost, enumerated.** Assert
  `trusted_base()` grows by **precisely** the primitives + propositions you
  registered — no more. **Name each one in the test.** *(The delta is the
  point: it must be bounded, visible, and reviewed.)*
- **AC3 — the derived surface adds ZERO further delta.** Any byte op you derive
  over the view is `trusted_base()`-neutral (`37 §2.5`).
- **AC4 — the round-trip is netted**, in the same posture as the `String` pair.
- **AC5 — no regression.** CAT-5, CC3, CC7 and both corpus oracles stay green.
  **Do NOT retire the cached-`Nat` carriers in this WP** — `Source` and
  `ArgBytes` keep working. Their retirement is **SUB-2**, a separate WP, so this
  one stays a clean, reviewable trust delta. *(Land the capability; collapse the
  consumers next.)*
- **AC6 — corpus oracles.** `ken_fmt` + `kenfmt_c_capstone` + the live
  fixed-point. **No `FRAME_LINE_COUNTS` row.**

## 4. ★ A SPEC DISCREPANCY YOU MUST NOT BUILD ON TOP OF — escalate, don't absorb

**`spec/30-surface/37-strings-collections.md §2.4` says:**

> *"A primitive op carries a **registered reduction** (`41`), so e.g.
> `byteLength "abc" ≡ 3` holds **definitionally** and proofs can compute over
> string literals."*

**I believe that is FALSE of the landed kernel, and it is load-bearing for your
§2.2.** My evidence, at `origin/main @ c5f73b9c`:

1. **`conv.rs::unfold_const` unfolds ONLY transparent constants** — it calls
   `env.transparent_body(id)`. `Decl::Primitive` has **no body** and is never
   unfolded in conversion.
2. The kernel's own comment on `PrimReduction::Op` is *"a primitive operation
   **awaiting** its registered reduction (K3)."*
3. **Ken's own guide ships the non-reduction as a REJECT:**
   `catalog/guide/proof-techniques.ken.md:133` —
   `lemma prim_eq_refl : Equal Bool (eq_int five five) True = Refl` **fails**,
   *"`eq_int five five` never reduces to `True` under conversion, even though
   `five` is concrete."*
4. `prim_reduce` / `prim_reduce_elaborated` live in **`ken_interp::eval`** — the
   **interpreter**. Primitive ops **compute at runtime** and are **opaque to
   conversion**.

**⇒ A proof CANNOT compute over a literal `Bytes`/`String`, and `Refl` cannot
discharge a round-trip law.** That is precisely why CAT-5 and CC3 discharge
their length certs with `Axiom` — **it was the only route available.**

**WHAT THIS MEANS FOR §2.2:** design the round-trip propositions as **registered
postulates** (part of the fixed per-builtin cost the ruling authorizes) — **not**
as lemmas you expect `Refl` to close. **If you find `Refl` closes them, STOP and
tell me** — that would mean §2.4 is true, my reading is wrong, and the laws get
*stronger*. Either answer is a good outcome; **guessing is not.**

**I have escalated §2.4 to the Spec enclave and the Architect in parallel.** I
have **not** asserted it as a spec bug — `prim_reduce_elaborated` exists and I
have not traced every path, and I have burned four false-positive greps in this
program already. **Build on the postulate posture; the adjudication can only
improve it.**

## 5. Guardrails

- **The trusted-base delta must be FIXED and ENUMERATED.** That is the entire
  bargain of the ruling. **If you find yourself adding a third, fourth, fifth
  primitive, STOP and escalate** — the bargain was *bounded* growth.
- **Do not native-ize the derived byte surface** (§2.3).
- **Do not retire the cached-`Nat` carriers here** (AC5). That is SUB-2.
- **Do not change kernel semantics.** No new reduction rule, no conv change. If
  §4's adjudication says K3's registered reductions should land, **that is its
  own WP, not this one.**

## 6. The clause that has now caught SEVEN bad pins of mine

**Treat every anchor as perishable.** Two frames in the last hour carried a false
fixed input; both were caught **pre-edit** by the implementer who refused to
build around them, and both made the WP *better*. **If a fixed input is false
against the landed code, say so with exact tree anchors and escalate. Do not
quietly build around it. I would rather be corrected than believed.**

Scope/layering routes to the **Steward**. Soundness routes to the **Architect**.
