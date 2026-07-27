# CAT-CAPEX — exhibit the capability discipline as checked catalog Ken

**The catalog has a whole `Capability/` package tree that exhibits effect rows
and not one capability token. The landed contract supports one today — the
proof is that four capability-typed `proc`s already elaborate and pass CI.
They just live embedded in the elaborator prelude instead of the catalog.**

**Owner:** Team Ergo. **Branch:** `wp/CAT-CAPEX`. **Size:** M.
**Risk:** low-medium — `catalog/` only, additive, no normative surface moves.

**Status:** Steward frame, shovel-ready. ⛔ **Not blocked on `ABI-R3`, the
membrane, or the spec.** See `§3` — I discharged that ordering question.

---

## 1. Fixed inputs — measured at `origin/main = e700b861`

| path | blob |
|---|---|
| `spec/60-security/62-authority.md` | `7b6b1b7299ee6438211690562167e5bf37e99316` |
| `spec/30-surface/36-effects.md` | `7fc26a22d4b6418fb001cf19e173c322cfd3c383` |
| `crates/ken-elaborator/src/prelude.rs` | `a9e887c4a94bcad6d5f2a5a4a7d6088f5f0f1152` |
| `crates/ken-interp/tests/i3_fs_floor.rs` | `279f1ead1c77023fb92246dac491c947c0d254db` |
| `catalog/packages/Capability/Filesystem/Errors.ken.md` | `a997783121fdc55d0021e5e907e1bd7cd20b1a06` |

⚠ The two `crates/` entries are **read-only references** — the landed surface
you write against. ⛔ This WP does not edit them.

---

## 2. The measurement

`catalog/packages/Capability/` has eight subpackages — `Console`,
`Diagnostics`, `Filesystem`, `Formatting`, `Parsing`, `Process`, `System`,
`Time`. Measured across the whole of `catalog/`:

| property | count |
|---|---|
| fragments exhibiting an effect row (`visits [...]`) | many |
| fragments taking a capability parameter | **0** |
| `Cap`-typed binders anywhere in `catalog/` | **1**, and it is unrelated |

⭐ **The one hit is a false positive and you should not chase it.**
`catalog/guide/decomposition-abstraction.ken.md:111` declares its **own**
`data Cap = MkCap Int` inside an illustrative block about representation
choice. It is a local type that happens to share a name. ⛔ It is not a stale
spelling of the real `Cap` and needs no repair.

Meanwhile the real thing is landed and green (`prelude.rs`):

```
proc read_bytes  (a : Auth) (cap : Cap a) (path : Bytes)
       : FS a (Result FileError Bytes)  visits [FS]
proc write_file  (a : Auth) (cap : Cap a) (path : Bytes)
       (policy : CreatePolicy) (contents : Bytes)
       : FS a (Result FileError Unit)   visits [FS]
proc append_file  (a : Auth) (cap : Cap a) …
proc file_metadata (a : Auth) (cap : Cap a) …
```

with `data Auth = ANone | APartial | AFull` and `Cap : Auth -> Type0` an
**authority-indexed opaque former**. Because opaque formers never δ-unfold,
`Cap APartial` and `Cap ANone` are **genuinely distinct** stuck-neutral types.

---

## 3. ⭐ Steward-discharged: the ordering question is ANSWERED

The `CAT-CAPEX` node parked itself on one question: *"is the exemplar blocked
on `ABI-R3` and the membrane implementation, or can a capability-typed fragment
be written against the landed contract today?"*

**It can be written today.** Three independent measurements:

1. `Cap` is registered in the elaborator's globals as a real surface type, and
   `Auth` is an ordinary checked inductive. Both are in scope for any Ken
   program.
2. Four capability-parameterized `proc`s **already elaborate** through
   `elaborate_decl` and are green in CI.
3. `crates/ken-interp/tests/i3_fs_floor.rs` already loads the **catalog**
   fragment `Capability/Filesystem/Errors.ken.md` and drives `read_bytes` with
   a real `Authority`. ⇒ The catalog-fragment path and the capability path
   **already meet** in a passing test.

⇒ ⛔ **Do not wait for `ABI-R3`, the membrane, or any spec change.**

### ⚠ And the census that surfaced this node searched for the wrong thing

The original grep looked for `Cap_FS`, `: Cap `, `CapParam`, `cap_set`, and
`attenuate`. Of those, `attenuate` is **required by the spec to be unbound**
(`38 §1.3.1`) and `Cap_FS` is a **retired** spelling. ⭐ A census that
enumerates spellings is not a measurement of the property — the zero it
returned was partly an artifact of the pattern. The gap is real; its stated
cause was not.

---

## 4. ⛔ Banned shapes — read this before writing a line

- ⛔ **Do not bind `attenuate`, `revoke`, or `strengthen` in Ken.** `38 §1.3.1`
  requires all three to resolve as `UnboundName`, and `62 §7` pins that as
  checked behavior. A Ken-callable one **falsifies a landed property** — this
  is the highest-severity way to fail this WP.
- ⛔ **Do not add a `Cap` constructor, producer, or wrapper.** `62 §2.2`: user
  code has **no constructor** for `Cap E`; it is minted only by a handler.
- ⛔ **Do not copy from `62 §7`.** ⭐ **This is the single most likely defect
  in this WP.** `§7` is the spec's only worked example of the authority
  discipline and it is **stale on three independent axes**: it uses the
  **retired `view` keyword** (operator SURF-1 retired `view` into
  `const`/`fn`/`proc`), the **retired `Cap_FS`** spelling rather than the
  authority-indexed `Cap a`, and `write_at` rather than the landed
  `write_file`. Copying it produces a fragment that cannot check. Write
  against the **prelude** signatures in `§2`, which are the landed truth.
- ⚠ **AMENDED 2026-07-27 (`evt_5h83ctgbajqk4`) — `crates/` is TEST-ONLY, and
  one new file only.** ⛔ ~~"Do not edit `spec/` or `crates/`."~~ — that flat
  ban blocked `AC-1`/`AC-2`, which demand a named harness. A frame cannot
  require "name the test" while banning every file a test can live in.

  ✅ **Authorized:** **one new** `crates/ken-elaborator/tests/cat_capex_*.rs`,
  following the shape of `cat3_collections_package.rs:15` — a `mk_env()`-style
  **fresh `ElabEnv` per test** (`ElabEnv::new()`, then
  `elaborate_ken_md_file`). ⭐ That pattern is already used by `cat1`, `cat3`,
  `cat5`, and `either_catalog_package_acceptance`; per-program env isolation is
  how catalog packages are covered here. Asserting a specific `ElabError`
  variant structurally is precedented at `surface_unicode.rs:35`.

  ⛔ **Not authorized:** editing `ken_md_literate.rs` or **any** existing
  shared harness — it discards `ElabError` and reuses one mutable `ElabEnv`
  (so a twin built on it is contaminated by the negative program's `main`
  registration), and it has other consumers. ⛔ **Nothing under
  `crates/**/src/**`.** If the diagnostic `AC-2` needs does not exist in
  `ElabError`, ⛔ **do not add it — stop and route to the Steward**; that is an
  implementation gap and a different WP.

- ⛔ **Do not edit `spec/`.** `62 §7`'s staleness is real and is a **separate
  enclave WP** (`SPEC-AUTH-EX`) — the Steward files it. Your job is the
  exemplar.
- ⛔ **Do not invent a new package.** Use the existing
  `catalog/packages/Capability/` tree.

---

## 5. Deliverables

- **`D1`** — a **checked** catalog fragment under
  `catalog/packages/Capability/` whose signature *is* its authority manifest:
  a `proc` taking an explicit `(cap : Cap a)` and declaring its effect row.
- **`D2`** — the **authority-index distinction** exhibited: a definition
  demanding a specific authority, shown to be a different type from one at
  another authority.
- **`D3`** — the **negative**: a program performing a world-action with **no
  capability in scope** is rejected, with the rejection exhibited.
- **`D4`** — an **honest boundary note** in the fragment's prose: `attenuate`,
  `revoke`, and `strengthen` are `UnboundName` **by design**, citing
  `62 §4`/`§3.2` and `38 §1.3.1`. ⛔ State the boundary; do not simulate it.
- **`D5`** — a **closed** report of what the catalog can and cannot exhibit of
  the `62` authority discipline, **naming the complement**: which properties
  are exhibitable as checked Ken today, and which are host/runner-side and so
  permanently outside the catalog.

---

## 6. Acceptance criteria

- **`AC-1`** — ⭐ **`D1` is actually checked, and you name the harness that
  checks it.** **Control:** name the test that elaborates the fragment and show
  it green. ⛔ A `ken` block that no test loads is **prose**, and prose is the
  exact thing this WP exists to replace. "It is in the catalog" is not
  evidence; "this named test elaborates it" is.

- **`AC-2`** ⭐ **(load-bearing)** — `D3`'s rejection is **attributable**.
  **Control:** the no-capability program is rejected **with the specific
  missing-capability diagnostic**, *and* the **same program with the
  capability supplied is accepted through the identical harness**. ⛔ A
  negative check passes for **any** reason — a typo, an unparseable fragment,
  a harness that silently skipped the file. Without the positive twin you have
  measured nothing.

- **`AC-3`** — `D2`'s distinction is causal. **Control:** supplying a `Cap` at
  the wrong authority index **fails**, and the correctly-indexed twin
  **succeeds**, through one harness. ⛔ If both pass, the index is not load-
  bearing in your fragment and `D2` is not delivered.

- **`AC-4`** — `D4` names the three unbound management names **and** cites the
  spec sections requiring them unbound. ⛔ A note that merely says "not shown
  here" fails: the point is that their absence is a **designed security
  property**, not a gap in the exemplar.

- **`AC-5`** — `D5` is closed and names its complement. **Control:** show what
  was examined **and** what was excluded, with the reason. ⛔ An empty
  exclusion list is a failed measurement — raw attenuation and revocation are
  trusted host actions over non-Ken-visible grant identities (`62 §4`) and
  therefore *cannot* appear. That exclusion existing is the honest result.

⛔ **No CI checker or gate asserting facts about catalog or spec lines**
(operator test policy). The weak "reports drift" form is still a gate if it can
go red. ⇒ `AC-1`–`AC-3` are discharged by **elaboration behavior**, which is
exactly the behavioral form the policy asks for.

---

## 7. Contention and sequencing

**`catalog/` only, and additive.** ⚠ Re-measure contention at pickup, not from
this frame. Known live: `CAT-C2` Phase 2 will touch
`catalog/packages/Data/Collections/` and `Core/Classes/` — **disjoint** from
`Capability/`, but it is gated behind `CAT-C2` Phase 1 and not released.

⛔ Ergo holds no other node. This is the ring's work.

---

## 7a. ⛔⛔ RUN THE FORMATTER ON ANY NEW `catalog/` FILE BEFORE HANDOFF

⚠ **Added 2026-07-27 after this WP's first candidate failed CI.** `d52611f5`
was AC-complete, QA-approved, and Architect-approved, and it went **red in CI**
on two shards:

```
crates/ken-elaborator/tests/kenfmt_c_capstone.rs:38
    canonical_live_corpus_is_a_fixed_point
    assertion `left == right` failed: …/Capability/Filesystem/Authority.ken.md
crates/ken-cli/tests/ken_fmt.rs:111
    strict_frozen_corpus_gate_is_green
```

⇒ There is a **corpus-wide formatter fixed-point gate**: every file under
`catalog/` must already be in canonical `kenfmt` form.

⭐ **Why no local run could catch it.** The gate lives in `ken-cli` and
`kenfmt_c_capstone` and is keyed on **every catalog file**, so it fires only in
a full-workspace run — which is **CI-only** by operator hard rule
(`COORDINATION §12`). A targeted `-p ken-elaborator --test <yours>` is correct
and cannot see it. ⛔ Do **not** respond to this by running `--workspace`
locally.

⇒ ✅ **Format any new `catalog/` file before handing the SHA to QA.** This
belongs in every frame that adds a `catalog/` file; its absence here was the
Steward's omission, not a ring error.

## 8. Hard stop

⛔ Route to the Steward if:

- a `Cap`-typed catalog fragment **cannot** be elaborated through the existing
  harness — that falsifies `§3`'s buildability finding and the whole frame
  rests on it; **or**
- delivering `D1`–`D3` appears to require binding `attenuate`/`revoke`, adding
  a `Cap` producer, or editing `crates/`/`spec/`. Any of those means the
  exemplar shape is wrong, not that the ban should bend.
