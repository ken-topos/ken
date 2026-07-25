---
id: RT-FNSPLIT-B2V
title: "executable boundary-value ABI — one closed 64-bit tagged word for ValueWord/ResultWord plus the emitted-code interface to construct, discriminate and project it"
status: active
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2O, RT-FNSPLIT-B2R]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_28cnmxf6ncghn on hard-stop #10 of RT-FNSPLIT-B2F (raised by runtime-implementer evt_71d2jg83z2yt4, leader escalation evt_r7797bd7bzk3, evidence 49e24b59..1b789817 on origin wp/RT-FNSPLIT-B2F-functionization). The ruling found #10 VALID and STRUCTURAL and required exactly ONE new prerequisite inserted between B2R and B2F, delivering an INERT but EXECUTABLE boundary-value ABI. It explicitly forbade splitting the value contract from its access interface, on the ground that a second slot-only declaration would reproduce #9/#10 one layer down. Architect durable checkpoint at ARCHITECT-STATE.md commit 74b4f51b. Steward owns the ID, the frame, and the AC/control placement.
---

> ## ⛔ WHY THIS NODE EXISTS — `B2R` DECLARED THE SLOT; NOTHING DEFINED THE VALUE
>
> `B2O` and `B2R` give static code ownership, unit population, slot order and
> width, and declared ownership. **They do not define what the bits of
> `ValueWord` / `ResultWord` MEAN, nor how compiled code inspects a dynamic
> aggregate.**
>
> The landed lowering confirms the distinction: `Lowered` is a **compile-time
> specialization lattice**; `ground_value` exports only fully-constant
> aggregates, and it does so **through a Rust-side table**. `HostResult` and
> dynamic aggregates have **no executable word representation at all**. ⇒ A
> compiled-once callee cannot consume the measured `Constructor` / `HostResult`
> parameters.
>
> ⭐ **The measured escape hatch is closed, and this is the sharpest part of the
> evidence.** An aggregate result works *today* only because **the consumer is
> Rust**: the callee returns an `iconst` token and the caller decodes it through
> `ResultDecoder` + `result_table`, both compile-time Rust objects living in
> `CompiledModule`. Under functionization the consumer is **emitted code**,
> which holds no decoder and cannot read that table. **The existing
> aggregate-result path is a Rust-side decode at the artifact boundary, not a
> value representation** — so it does not generalize from the root boundary to
> an internal one.
>
> ⛔ **The conservative guard is sound and still insufficient.** A fail-closed
> guard rejecting the unrepresentable transfers would reject **~33 of 41**
> measured source-valued transfers — which cannot satisfy `B2F`'s `D6`
> (old-authority removal) or `D7` (equivalence).

## ⛔⛔ THE ONE THING NOT TO DO — do not split this node

**Architect, verbatim in effect:** *"Do not split the value contract from its
access interface: a second slot-only declaration would reproduce #9/#10 one
layer down."*

★ **That is the whole lesson of this chain, stated as a construction rule.**
`#9` produced `B2R` — a declaration. `#10` is `#9` again, one representation
layer below, because the declaration had no executable meaning. **A third
declaration-only node would produce `#11` in the same shape.** The value
representation and the emitted-code interface that reads it land **together or
not at all**.

## What the prerequisite must deliver — six requirements, from the ruling

1. **Give the word a meaning.** One closed 64-bit boundary-value representation
   used by `ValueWord` **and** `ResultWord`, reconciled explicitly with
   `GroundValueCarrier`. Because the plane has **no per-slot static type**, the
   permanent shape is a **tagged word**: immediate payload where lawful,
   otherwise an **opaque handle** into runtime-owned value storage. ⛔ **It must
   not specialize the representation from a JIT seed value or from caller
   depth.**
2. **Subsume existing runtime machinery.** Reuse the existing runtime
   value/store substrate for persistable Ken values; add only the
   **invocation-scoped** storage needed for borrowed ingress such as
   `HostResult`. ⛔ **Do not create a parallel permanent heap by default.** If a
   word is a handle, **name the referent owner and lifetime separately from the
   frame slot that stores the word** — `AbiStorageOwner::ActivationFrame` must
   not silently stand in for an invocation or persistent referent owner.
3. **Make the interface executable.** Supply the **constant-width emitted-code
   interface** to construct, discriminate and project: at minimum scalar
   extraction, constructor/result tag, field count/index, record field access,
   and the `HostResult` success/payload disposition. ⛔ **A Rust-side
   `result_table` token with no runtime lookup path does not count.** The
   helper/symbol population is fixed **Θ(1) per module** — never per origin and
   never per runtime value.
4. **Close the transfer population STRUCTURALLY.** One exhaustive **no-wildcard**
   disposition over every `Lowered` variant that can reach `Parameter`,
   `Capture`, or `Result`: *represented immediate* · *represented handle with
   explicit lifetime* · *protocol-only* · *fail-closed forbidden*. ⛔ **The
   41-transfer histogram is corroboration, not the population proof.**
   `Constructor` and `HostResult` are **required live arms**, not optional
   follow-ups.
5. **Prove runtime OBSERVABILITY, not round-trip serialization.** Controls must
   send a **non-constant** `Constructor` through a `Parameter` and let a
   **separately compiled** body inspect its tag/field; send a `HostResult`
   through a boundary and let the callee select the correct success/error
   payload; and cover **nested** aggregate `Capture`/`Result` flow. ⛔ Mutations
   must **redden** if the callee reads a compile-time template, a constant
   table, or the wrong referent owner. Borrowed ingress must **fail closed** if
   it escapes the native invocation.
6. **Remain INERT.** May add the representation, runtime support, declarations,
   pure codegen helpers, and isolated tests. ⛔ **Adds no production
   generated-function population, no production cross-owner call, no
   switch-over, and no second body-emission authority.** Existing
   root-function / definition / call censuses remain **unchanged**; any constant
   helper declarations are **predicted and re-baselined explicitly**.

## What this does to `RT-FNSPLIT-B2F`

`B2F` remains **the same atomic boundary**: consume the value ABI one-for-one,
define and call every static unit, switch the root and every cross-owner
transfer, prove equivalence, and remove recursive whole-configuration emission.

⭐ **`B2F`'s `AC-11` is re-scoped by this ruling.** It becomes **enforcement of
this prerequisite on every `Parameter`/`Capture`/`Result` transfer** — **not**
rejection of common aggregates, and **not** inheritance from `C4`.

## Classification — recorded exactly as ruled

- Hard-stop count is **10**. **Next research pull remains `#12`.**
- ⛔ **This is NOT a fourth symptom-inventory entry.** It is another missing
  prerequisite **under entry 2** — the same functionization obstruction one
  representation layer below `#9`. Record it under entry 2 / the prerequisite
  chain; **do not widen the headline symptom inventory.**
- **`B2R` is not defective within its declared slot-shape scope** — but the `#9`
  discharge text **must stop crediting it with the value half of
  representation.**

## Sequencing

`B2O` → `B2R` → **`B2V`** → `B2F`. Runtime is **held** and does not resume
`B2F` construction until this node's frame is fetchable on `origin/main` **and**
explicitly kicked (Architect condition, `evt_28cnmxf6ncghn`).

⚠ `RT-FNSPLIT-B2O-CHECK` remains sequenced behind `B2F` on **file contention**;
this insertion does not change that, but its anchors now have one more merge to
survive.
