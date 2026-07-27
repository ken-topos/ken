---
id: STR-NFC-CONSTRUCTION
title: "NFC-at-construction is normative and unimplemented: all three `EvalVal::Str` ingresses store the raw string, so `char_length`/`byte_length`/`s2l`/`==` observe unnormalized values and the interp carrier disagrees with the runtime carrier"
status: active
owner: language
size: L
gate: none
depends_on: [STR-BIJ-TEST-CARRIER]
blocks: []
github: null
origin: "Architect Decision dec_ppakqc11kffh disposition (b), 2026-07-27: NFC-at-construction REMAINS Ken's normative String contract; 37 §9's 'deferred behavior — currently stubbed' is an honest staging disclosure, not authority to weaken it. Measured by the Steward at origin/main a8b632f0. Blocked on STR-BIJ-TEST-CARRIER, whose AC-C7 tripwire this WP is required to trip."
---

> ## ⛔ READ FIRST — THE OBVIOUS FIX IS THE ONE THE RULING FORBIDS
>
> ⛔ **Do not patch `list_char_to_string` alone.** From `dec_ppakqc11kffh`:
>
> > ⭐⭐ *"an `l2s`-only normalization would make `l2s (s2l s) ≡ s` FALSE for a
> > decomposed literal `s`, contradicting the landed retraction axiom."*
>
> ⇒ A partial fix **falsifies the very axiom `STR-BIJ` just corrected the corpus
> to describe.** `String` values enter through **three** ingresses (§1); normalize
> one and the others keep minting decomposed `String`s that the normalizing one
> can no longer reproduce.
>
> ⭐ **The required shape is a representation change, not a call-site patch:** one
> **non-bypassable NFC constructor** — the ruling prefers a **sealed
> normalized-string carrier with a private raw field** — with every semantic
> ingress routed through it, landing **atomically** (§6).

## ⭐ §1 — The ingress census (measured at `origin/main = a8b632f0`)

**Exactly three sites construct an `EvalVal::Str` in `crates/*/src/`. None
normalizes.** Re-run the census at point of use; ⛔ do not trust this list
without re-deriving it.

```sh
grep -rn 'EvalVal::Str(' crates/*/src/ | grep -v 'EvalVal::Str(s)\s*=>'
```

| # | site | ingress | what it stores |
|---|---|---|---|
| **I1** | `ken-elaborator/src/elab.rs:2707` → carried by `ken-cli/src/lib.rs:449` | **source literal** | `NumericLitVal::Str(s.to_owned())`, raw source bytes |
| **I2** | `ken-interp/src/eval.rs:1770` | **successful `bytes_decode`** | `EvalVal::Str(string.to_string())` straight from `str::from_utf8` |
| **I3** | `ken-interp/src/eval.rs:1606–1608` | **`list_char_to_string`** | `list_char_to_evalval_string(v, &ids).map(EvalVal::Str)` |

⚠ **`I1` already CLAIMS to be normalized and is not.**
`ken-elaborator/src/numbers.rs:43` documents the variant as *"NFC-normalized
UTF-8 string literal (`37 §2.1`, VAL1-surface)"*. ⛔ That is a **doc comment, in
a position nothing executes** —
the constructor two lines away stores `s.to_owned()`. Treat every "is normalized"
comment on this path as a claim to verify, not a fixed input.

## ⭐⭐ §2 — What is ACTUALLY broken, and what is already right

⛔ **This is narrower than "Ken does not do NFC", and the difference decides the
scope.** Two carriers exist and they **disagree today**:

| carrier | equality | normalization | status |
|---|---|---|---|
| `ken-runtime::Value` (`values.rs`, blob at §5) | ⭐ **NOT derived** — exposed only on `CanonicalWitness`, i.e. **the canonical bytes themselves** | `ken-runtime/src/canonical.rs:406` and `:593` normalize **at encode time** (`s.chars().nfc().collect()`) | ✅ **NFC-aware already, definitionally** |
| `ken-interp::EvalVal` (`eval.rs:176`) | ⛔ `#[derive(Clone, Debug, PartialEq)]` — **raw byte equality** | ⛔ **none, at any of I1/I2/I3** | ⛔ **the defect** |

⇒ ⭐ **The blast radius is the interpreter-visible surface, not the content-addressed
store.** K3 hashing/storing is already correct because `ken-runtime`'s encoder
normalizes on the way out. What is wrong is every consumer that reads the
`EvalVal::Str` payload directly:

- `char_length` (`eval.rs:1360`) — `s.chars().count()` over the **raw** string.
  On a decomposed `"e"+U+0301` this returns **2**, and `37 §2.1` requires **1**.
- `byte_length` (`eval.rs:1358`) — `s.len()`. `37 §2.1` is explicit: *"reports
  the NFC byte buffer's length, **not the pre-normalization source**."*
- `bytes_encode` (`eval.rs:1347`) — `s.as_bytes()`. `37 §2.3`: *"the stored NFC
  UTF-8 buffer."*
- `string_to_list_char` → `build_list_char` (`eval.rs:~1990`) — decomposes the raw
  string.
- **`==` on `EvalVal`** — raw derive, so two NFC-equivalent `String`s compare
  **unequal** in the interpreter while comparing **equal** on the runtime carrier.

⭐ **The elegant consequence, and it is a scope instruction:** normalize at the
three ingresses and the **derived** `PartialEq` becomes correct on its own.
⛔ **Do NOT hand-write an NFC-aware `PartialEq` for `EvalVal`** — that installs a
second definition of identity, which is exactly what `ken-runtime`'s `D3` note
removed as unsound.

## ⭐ §3 — The spec pre-declares the flip you are about to cause

`spec/30-surface/37-strings-collections.md §9` (`DS-AC4`) already says what
changes when this lands, and it is a **fixed input, not a surprise**:

> *"`String` is NFC-normalized at construction (`§2.1`, a deferred behavior —
> currently stubbed), so **once real NFC lands the two literals merge to one
> value and `eq` on them is `True`**; a literal-level pin would falsely fail then
> (the over-pin-a-deferred-behavior trap)."*

⇒ Two hard boundaries fall straight out, and they point in **opposite**
directions:

- ✅ **The `String` layer normalizes.** Two NFC-distinct `String` **literals**
  merge to one value; `eq` on them becomes `True`.
- ⛔ **The `List Char` layer stays NFC-BLIND, unconditionally.** `DS-AC4` pins
  `list_eq eqChar` on a precomposed-vs-decomposed pair **built directly as
  `List Char`** to evaluate **unequal**, *"pinning that NFC-eq was not smuggled
  in (ADR 0010 §3)."* ⛔ **Do not normalize inside `build_list_char`, inside
  `List Char` equality, or anywhere below the `String` boundary.** If your change
  makes that pin go green-by-merging, you have normalized one layer too deep.

## ⭐ §4 — Acceptance criteria

The ruling's nine controls, plus the two this census added (`AC-N10`, `AC-N11`).

⛔ Every row needs a control that **can fail**. Where a row says *"must redden"*,
report the redden evidence, not the green run.

| AC | claim | control |
|---|---|---|
| `AC-N1` | A decomposed and a precomposed **source literal** of one grapheme evaluate to **one and the same** `String` value. | `eq` on the two literals is `True`. ⭐ `37 §9 DS-AC4` pre-declares exactly this flip — cite it |
| `AC-N2` | **`bytes_decode` normalizes independently of `l2s`.** | Decode decomposed UTF-8 bytes; the resulting `String` is precomposed. ⛔ Must hold with the `l2s` normalizer **disabled** — the ingresses may not be inductive on one another |
| `AC-N3` | **`list_char_to_string` normalizes independently of `bytes_decode`.** | `l2s [101, 769]` → the `String` `"é"`. ⛔ Must hold with the decode normalizer disabled |
| `AC-N4` | ⭐⭐ **`s2l (l2s [101, 769]) ≡ [233]`** — the concrete value this whole WP exists to move. | The direct `List Char` path in `l3_strings_roundtrip_acceptance.rs`. ⚠ This is the **flip of `STR-BIJ-TEST-CARRIER`'s `AC-C7` tripwire** (§6) |
| `AC-N5` | Consumers observe **only** the normalized value: `byte_length` = **2**, `char_length` = **1**, `bytes_encode` yields the **NFC** bytes, for the decomposed-input grapheme. | Assert all three result values. ⛔ A shape assertion does not discharge this |
| `AC-N6` | ⭐⭐ **The String-side retraction holds for EVERY constructible `String`** — `l2s (s2l s) ≡ s`. | ⛔ This is the axiom an `l2s`-only fix falsifies. Exercise it on a `String` obtained from **each** of I1, I2, I3 — not just a literal |
| `AC-N7` | The **reverse** identity `s2l (l2s cs) ≡ cs` **fails** on `cs = [101, 769]`, and that is correct. | ⭐ `l2s` is not injective on `List Char` — it is a **retraction**, per landed `STR-BIJ`. Pin the failure deliberately so a later "fix" cannot restore a false bijection |
| `AC-N8` | ⭐⭐ **Bypassing ANY ONE ingress normalizer reddens its OWN control.** | ⛔ **Per-ingress, three separate mutations** — neuter I1, then I2, then I3, and record which control reddens for each. A single "disable normalization" mutation does **not** discharge this: it cannot distinguish three working normalizers from one normalizer three paths happen to funnel through |
| `AC-N9` | **Kernel delta zero and trusted-base delta zero.** | `trusted_base()` unchanged; no new primitive, no `Axiom`, no kernel `Term` variant. ⛔ If the sealed carrier appears to need a kernel change, **stop and re-raise** |
| `AC-N10` | ⛔ **The `List Char` layer is still NFC-blind.** | `37 §9 DS-AC4`'s `list_eq eqChar` pin on a directly-built precomposed/decomposed `List Char` pair still evaluates **unequal**. ⚠ **This is the control that catches normalizing one layer too deep**, and it must stay green *unchanged* |
| `AC-N11` | ⛔ **`ken-runtime/src/canonical.rs`'s encode-time `.nfc()` at `:406`/`:593` is RETAINED.** | ⭐ It is now redundant-and-agreeing, and that is the point: **removing it would make K3's canonical identity inductive on the constructor you are adding.** Report that both sites are unchanged |

## ⛔ §5 — Fixed inputs

Blobs at `origin/main = a8b632f0`. ⛔ Re-derive every one at point of use.

| path | blob | why it is pinned |
|---|---|---|
| `crates/ken-interp/src/eval.rs` | `a62c24f4` | I2, I3, and every consumer in §2 |
| `crates/ken-elaborator/src/elab.rs` | `5e9d94bc` | I1 — `:2707` |
| `crates/ken-cli/src/lib.rs` | `33f92d70` | `:449` carries I1 into `EvalVal` |
| `crates/ken-runtime/src/canonical.rs` | `bc2579a9` | the encoder that **already** normalizes (`AC-N11`) |
| `crates/ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs` | `0eed416d` | the carrier to flip (§6) — ⚠ **will move** when `STR-BIJ-TEST-CARRIER` merges |
| `spec/30-surface/37-strings-collections.md` | `51695439` | `§2.1`, the `§2.3` conversion table, `§9 DS-AC4` |

**Dependency available:** `unicode-normalization = "0.1"` is declared by
**`crates/ken-runtime/Cargo.toml:23` only**. The ingresses live in
`ken-elaborator` and `ken-interp`, so this WP **adds a manifest edge**.

> ⛔ **CORRECTED 2026-07-27 by `language-leader` at `2ebe232c`.** The root
> `Cargo.toml` has **no `[workspace.dependencies]` table** and no
> `unicode-normalization` entry, so ⛔ **a `.workspace = true` edge is invalid
> and fails before the build.** ⇒ Add a **direct** `unicode-normalization =
> "0.1"` to each ingress-owning crate's own manifest. This does not change the
> ruled architecture or the scope.

⚠ Either way it changes `Cargo.lock`; commit the updated lockfile or CI's
`--locked` gate fails, and that failure will look like an unrelated PR's.

## ⛔ §6 — Atomicity, and the ONE dependency

**From the ruling, verbatim:**

> *"Assemble implementation, final test-carrier flip, and the narrow `37 §9`
> staging-status reconcile on **one branch/Decision** so no intermediate tree
> falsifies the retraction or leaves the spec's landedness claim stale."*

⇒ **One branch, one Decision, three parts:**

1. the sealed carrier + all three ingresses;
2. the **flip** of `STR-BIJ-TEST-CARRIER`'s `AC-C7` tripwire in
   `l3_strings_roundtrip_acceptance.rs` — that test expects `[101, 769]` **solely
   as a loud transition tripwire** and is **built to fail when this WP lands**.
   ⭐ Its failure is the intended signal, not a regression; replace the
   expectation with `[233]` and retire the tripwire wording;
3. the **narrow** `37 §9` staging reconcile — drop *"a deferred behavior —
   currently stubbed"*. ⛔ Narrow means narrow: `§2.1`, `§2.3`, and `AC1` are
   **already correct** and are not touched.

> ### ⛔ DEPENDENCY — `STR-BIJ-TEST-CARRIER` MUST MERGE FIRST
>
> It is in flight with Team Language **right now** and touches
> `l3_strings_roundtrip_acceptance.rs` — **the same file, the same lines.** This
> WP flips the tripwire that WP installs. ⛔ Do not cut this branch until
> `STR-BIJ-TEST-CARRIER` is on `main`, and re-derive the `0eed416d` pin then.

## §7 — Scope

**IN:** the three ingresses (I1/I2/I3), the sealed normalized-string carrier,
the consumers in §2, the test-carrier flip, the narrow `37 §9` reconcile, the
`Cargo.toml`/`Cargo.lock` edge.

⛔ **OUT — and each exclusion has a reason you should check before overriding it:**

- ⛔ **`crates/ken-foundation/src/canonical.rs:110`** — a **second** K3 canonical
  encoder whose `String` arm does **not** normalize (*"NFC normalization would go
  here in production. For F4 bench, we assume input strings are already
  normalized"*). ⚠ You will find it and it will look in-scope. It is not: **no
  crate in the workspace depends on `ken-foundation`** (`grep -rln ken-foundation
  crates/*/Cargo.toml` returns only its own manifest), so it feeds nothing but
  its own `store.rs`/`testing.rs`. It is F4-bench lineage and belongs to the
  separate F4 re-cut. ⭐ **Report it, do not fix it** — and note the shape: its
  correctness is *inductive on an assumption about its input*, which is the same
  fragility `AC-N11` refuses to introduce upstream.
- ⛔ **Normalizing at or below the `List Char` layer** — `AC-N10`. `37 §9 DS-AC4`
  and `ADR 0010 §3` pin that blindness unconditionally.
- ⛔ **A hand-written NFC-aware `PartialEq` for `EvalVal`** (§2) — a second
  definition of identity.
- ⛔ **Removing `ken-runtime/src/canonical.rs`'s `.nfc()`** — `AC-N11`.
- ⛔ **Any kernel or trusted-base change** — `AC-N9`. If the carrier seems to need
  one, that is a finding about the design, and it goes to the Architect.
- ⛔ **Grapheme-cluster anything.** `37 §2.1` is about **NFC**, a normalization of
  scalar sequences. Grapheme segmentation is a different (and unrequested)
  problem.

## §8 — Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). Use
`scripts/ken-cargo`, scoped: `-p ken-interp`, `-p ken-elaborator --test
l3_strings_roundtrip_acceptance`, plus the named suites your change touches.
Workspace, `--locked`, and conformance run **in CI** — "no-regression" means
**green in CI**, never a local `--workspace` run.

⚠ `ken-cargo` is a single machine-wide `flock`, slots == 1 — request and return
the build turn **in-thread**; ⛔ never sample `ps` to decide it is free.

## §9 — Reporting

Return exact SHA/tree/base, and specifically:

- the **re-run ingress census** (§1) showing every `EvalVal::Str` construction now
  routes through the carrier;
- **`AC-N8`'s three separate per-ingress mutations** and which control reddened for
  each — ⛔ three results, not one;
- the measured values for `AC-N5` (`byte_length`, `char_length`, encoded bytes);
- `AC-N6`'s retraction evidence **per ingress**;
- confirmation that `AC-N10`'s `List Char` pin is **green and unmodified**, and
  that `AC-N11`'s two `.nfc()` sites are **unchanged**;
- `trusted_base()` delta for `AC-N9`;
- the `Cargo.lock` delta.
