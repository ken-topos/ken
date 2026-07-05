---
scope: enclave
audience: (see scope README)
source: private memory `abstraction-visibility-feature-soundness-gate`
---

# Soundness-gating a namespacing/visibility/abstraction build

**ES3-build gate (minimal modules, 2026-07-01, `dec_fjbm4s67sefg`).** A
module/namespacing/visibility/abstract-export feature elaborates AWAY to the
flat append-only Σ; its soundness is that it adds **nothing** to the kernel's
trust surface. Three checks, in order of load-bearingness:

1. **Zero-`trusted_base()`-delta is grounded by the KERNEL being untouched, not
   by a test.** Grep that the diff has **no `ken-kernel` file** and the `Decl`
   enum is unchanged (here: still 4 variants Transparent/Opaque/Inductive/
   Primitive — no `Module`/`Visibility`/`Abstract` variant). If the kernel
   admission vocabulary is unchanged, the mechanism *cannot* add a trust
   primitive regardless of how large the elaborator diff is. Resolution/scope
   failures must be **surface diagnostics** (`error.rs`), never kernel errors.
2. **Abstract export = the EXISTING opaque constant.** Verify the
   abstract-export path calls `declare_postulate` → `Decl::Opaque`
   (byte-identical to a hand-written `T : Type` postulate), **not** a new kernel
   abstract flag. AC: "abstract export IS the existing opaque constant." The
   ctors of an abstractly exported `pub data T` are never registered anywhere
   (unmatchable by the kernel too) — sound: an uninhabited opaque type former is
   a sound assumption, never `Bottom`.
3. **The opacity trigger must not over-fire (the caught bug).** A top-level
   `pub data T = MkT` must stay a **real `Decl::Inductive`** (out of
   `trusted_base()`); only an **in-module** `pub data` (a client to hide from)
   is abstract-exported. The fix here gated abstract-opacity on
   `is_pub && !prefix.is_empty()`. **Unfixed, an over-broad flag silently
   promotes a real inductive to an Opaque postulate** → breaks AC1 byte-identity
   (module `Opaque` vs flattened `Inductive`) **and grows `trusted_base()`**.
   Net it with a discriminating test that checks real-inductive-ness
   (`env.inductive(id).is_some()` + ctor registration + a value is
   *constructible-and-matchable*) and **flips** against the pre-fix commit.

**AC1 framing to keep straight:** "zero delta" means **module program ≡
flattened equivalent** (byte-identity), NOT "abstraction is free." An abstract
`T` is `Opaque` in *both* forms — the abstraction's trust cost is the intended
information-hiding semantics, present identically in the flattened form; it is
**not** the soundness ac static vs runtime face predicate-definedness / ES1
"derivable-thing-hiding-as-a-postulate" anti-pattern (an abstract type is a
genuine irreducible *from the client's view*). Recurs at ES4 (packages) and any
future visibility/sealing feature. Producer-grep sibling of kernel backed claim
grep the emission not the name.
