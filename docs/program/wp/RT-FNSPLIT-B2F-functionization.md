# `RT-FNSPLIT-B2F` — functionization and authority switch-over

**Owner:** Team Runtime · **Size:** L · **Closes:** `RT-NATIVE-FNSPLIT`
symptom-inventory **entry 2** — the last open entry.

**Depends on — all three merged and verified on `main`:**

| dependency | landed | PR | what it supplies to this node |
|---|---|---|---|
| `RT-FNSPLIT-B2A-S` | `origin/main` = `145fe915` | #944 | retained-body **selection** by static origin, one closed consumer |
| `RT-FNSPLIT-B2O` | `origin/main` = `e470ab65` | #963 | the validated **`SemanticOwner` partition** — which occurrences belong to which function unit |
| `RT-FNSPLIT-B2R` | `origin/main` = `c986d0a3` | #967 | the **representation and call-ABI contract** — `abi.rs`, inert |

**Every anchor below was re-derived on `origin/main` = `bd24422b`**, the tree
carrying `B2A-C` + `B2A-S` + `B2O` + `B2R`. ⛔ Do not trust a line number from
the retired `RT-FNSPLIT-B2A` frame, from the `B2F` issue file's draft prose, or
**from an earlier revision of this file** — this frame's own anchors were
previously stated at `0aa9e53f`, four merges back, and several had moved.
`lower_expr` alone has moved `:3847 → :4255 → :4333` across three re-frames.

---

> ## ✅✅ RELEASED 2026-07-28 — NO HARD-STOP HOLDS THIS NODE. Read this first.
>
> ⛔ **Three statements below this block are STALE and say `#10` is OPEN with
> the Architect. They are wrong.** They are left in place as append-only history
> per this chain's convention; **this block is the operative state.**
>
> | stop | state | ruled by | what it produced |
> |---|---|---|---|
> | `#9` | ✅ ruled | `evt_842spc7t6js1` + `evt_t4fykh52ncb` | the re-slice → `B2O`, `B2R` |
> | `#10` | ✅ **ruled** | `evt_28cnmxf6ncghn` | inserted **`B2V`** — ⛔ *not* open |
> | `#11` | ✅ **ruled** | `evt_7ay6s5s79awz8` (`dec_45aa2gngjc79z`) | retired `B2E`, produced **`C1`** |
>
> ⭐ **Every prerequisite those rulings named has now MERGED:** `B2O`, `B2R`,
> `B2V`, and **`C1` (PR #1156, `origin/main = feab3cb5`, blob-verified)**.
> ⇒ `B2F`'s release gate — **a closed `C1` carrier artifact** — is satisfied.
>
> **Hard-stop count of record = 11.** ⛔ The stale text below says `10`.
>
> ⚠ **NEXT RESEARCH PULL = `#15`.** ⛔ The text below says `#12`; that value was
> **corrected away from** on 2026-07-25 in favour of the operator override
> *"#11, then #15, #18, #21"*, and `#11` has since been raised and ruled. The
> operative anchor is `docs/program/issues/RT-NATIVE-FNSPLIT.md`'s
> **"ARMED §5a RESEARCH-CONSULT TRIGGER"** line, ⭐ **which this frame already
> designates as winning on any disagreement.** Read it at the point of a stop;
> ⛔ do not act on a count transcribed into this file.
>
> ### ⚠ ONE GENUINELY OPEN QUESTION — and it does NOT block the build
>
> ~~`#10`'s ledger entry left a live item: **whether `#9` and `#10` are one
> predicate or two instances.**~~ ✅ **ANSWERED — the Architect named it**
> (`evt_55bzwnhjpwjrs`): the predicate is **`executable-boundary closure`**, and
> `#9`/`#10` are **two observations of it**, `#10` being `#9` recurring one
> representation layer down. ⛔ **The ruling names the ALREADY-EXECUTED recut and
> explicitly does not reopen `#9` or `#10`, add a prerequisite, reset the count,
> or hold this build.** ⇒ No recut frame is owed; the count stays **11**. Durable
> record: `docs/program/issues/RT-NATIVE-FNSPLIT.md`.

---

> ## ⛔⛔ FRAME CORRECTED 2026-07-28 AT THE RING'S BOUND BASE — FIVE CORRECTIONS
> ## AND ONE NON-CORRECTION. Read this before you trust any anchor below.
>
> **Source:** `runtime-implementer`'s grounding (`evt_6q69zjx5fnr6j`,
> `evt_j39jgesq7mfz`) and `runtime-leader`'s routing (`evt_39mwgr1jrp5x7`,
> `evt_xh7q436j6r4v`), all re-derived on `origin/main = 6534e4a6` against frame
> blob `aa798ca9`. ⛔ **This block is the operative state where it disagrees with
> anything below.** Nothing here splits the atomic boundary or changes scope.
>
> ### ⭐ THE STANDING RULE THIS BLOCK INSTALLS — read it before the rows
>
> ⛔ **Every line number and count in this frame is a MEASUREMENT AT A NAMED SHA,
> never an authority. Re-derive on your own base and yours wins.** This file's
> anchors have now gone stale **four times** (`0aa9e53f` → `3891b7aa` →
> `bd24422b` → `6534e4a6`), and ⭐ **each fix produced a fresher copy that then
> rotted on the next merge.** ⇒ The numbers below are dated evidence that the
> stale ones moved — ⛔ **they are not a new pin, and a future reader must not
> treat them as one.** Where a deliverable needs a population, the **derivation
> mechanism** is the pin and the number is its output.
>
> | # | the frame says | measured at `6534e4a6` | what actually changes |
> |---|---|---|---|
> | 1 | the seam `B2F` must close is `let _boundary_value_abi = …` at `core.rs:87`, result **discarded** | ⛔ **that line does not exist.** `core.rs:92` binds it **without** the underscore, consumed 9× at `:128–137` into `BoundaryCarrierRefs`, wired to `Lowering.boundary_carrier` at `:188` | ✅ `C1` **already made the handle live.** `B2F` inherits a live carrier instead of making one live. ⛔ **This subtracts the REACHABILITY seam ONLY — it does NOT discharge `AC-11`**, whose clause 1 wants a producer-tracing walk per `Parameter`/`Capture`/`Result`. ⛔ Do not cite the live handle as that proof |
> | 2 | `lower_expr`: **59** calls, definition `core.rs:4333`, span `:188`–`:6743`, root `:188` | **61** calls, definition **`:5149`**, span **`:237`–`:7655`**, root **`:237`**; tokenizer `identifier_occurrences` now `control.rs:3698` | ⛔ **`D5`/`AC-5`'s ENUMERATED SITE LISTS ARE ALL DEAD** — the 8 caller-dependent, the 6 untraced, the 3 synthesized, `:4878`, `:4454`. ⭐ Re-derive the five-class taxonomy from scratch; ⛔ **do not port line numbers.** ⚠ The naive `self.lower_expr(` grep still returns **60** and still misses the root — the spelling-scoped defect is live at the new numbers |
> | 3 | `D1`/`D2` **consume** `B2O`/`B2R` | ⛔ **the types are not reachable.** Every ABI type is `pub(super)` in `abi.rs` (`AbiCarrier:60 · AbiSlot:294 · AbiFrameHeader:319 · AbiDescriptor:354 · AbiPlane:368 · build_abi_plane:431`), as are `SemanticOwner` (`semantic_ir.rs:154`) and `PredeclaredFunction` (`:634`). The emitter is in a **different subtree** and none of `static_transition.rs`'s 11 `pub(in crate::cranelift_backend)` items is an ABI accessor | ⭐ **`D1`'s "consume, don't construct" subtraction is SMALLER than this frame claims:** the structure exists but is **unreachable**, and making it reachable is this node's work. ✅ Ruled **mechanical** (leader, `evt_2g2pyhz7jesg5`): **one narrow lowering-reachable projection**, ⛔ not wholesale type promotion, ⛔ no second derivation. ⚠ **`AC-1` is re-argued against the WIDENED surface**, not the current one — see the closure note below |
> | 4 | `AC-2`'s five census rows are a subset of the 13-file `BACKEND_PRODUCTION_SOURCES` (`control.rs:3686`); add `abi.rs` | population is at **`control.rs:3749`**, and ⛔ **`boundary_value_clif.rs` — 8304 lines, live, emitting a Θ(1) helper population — is in NEITHER list** | ⛔ **`abi.rs` was not the only omission.** `AC-2` states the population and gives **every** excluded emitter an explicit row or a reasoned exclusion. ⭐ Still binding: an absent row and a `0/0/0` row read identically and **only one is a claim** |
> | 5 | `D2`: *"`AbiPlane::shape` / `shapes` is the accessor"* | ⛔ **both are `#[cfg(test)]`** (`abi.rs:342`, `:388` — the file's only two `cfg(test)` items). **Production emission cannot call either** | ⛔ **`D2` cannot be built as written.** Production consumes `AbiDescriptor` + its declared slot run via the **`EmittableUnit`** projection (private fields, no constructor, sole producer `emittable_units()`) |
> | 6 | `AC-3`: the four `D3` width invariants, each independently falsifiable | ⚠ every `AbiCarrier` variant is 8/8 bytes ⇒ a pairwise-carrier width check is non-discriminating | ✅ **NO AMENDMENT — `AC-3` ALREADY SAYS THIS**, in its own words: *"every `AbiCarrier` variant is currently 8/8, so a width assertion that compares carriers to each other passes on a mechanism that carries no layout information at all"*, and its subject is already **the emitted code's agreement with the declarations**. ⇒ ⭐ **The ring re-derived a hazard the frame had already pinned. Nothing changed here; read `AC-3` as written.** ⛔ A pairwise-carrier control still does not count |
>
> ### ⛔ WHY ROW 5 IS THE MOST EXPENSIVE ONE, AND IT IS NOT THE BIGGEST
>
> Rows 1, 2 and 4 are **stale anchors** — they misdirect, and the ring's own
> re-derivation corrects them, which is exactly what happened. **Row 5 is a
> different class: the frame named a `#[cfg(test)]` item as a production
> accessor.** ⇒ ⭐ **A deliverable pinned to a test-only symbol is unbuildable,
> not merely mis-anchored**, and no amount of careful re-measurement of *line
> numbers* surfaces it — you find it only by asking whether the named item is
> reachable from production at all. ⚠ Same family as this frame's own
> reachability/visibility failure class, one layer in: row 3 is *"the type is not
> visible here"*, row 5 is *"the accessor does not exist in a production build."*
>
> ### ✅ `AC-9`'s BASELINE — the RECIPE is the binding, not the run
>
> The implementer stated it would capture the `AC-9` baseline before any
> production edit, then reasoned it through and did not (`evt_j39jgesq7mfz`).
> ✅ **That is correct and it is recorded here so no reviewer demands a
> timestamped capture.** `AC-9` asserts equality against **committed constants**,
> and the recipe names `git worktree add --detach 6534e4a6` — a **pristine
> detached tree**. ⇒ A capture's *timing is unobservable*, so timing cannot be
> the evidence; the **SHA-anchored recipe** is what distinguishes a genuine
> baseline from a re-recording.
>
> ⛔ **The pristine-detached-SHA clause is LOAD-BEARING — do not "simplify" the
> recipe to run in the current worktree.** The moment the baseline is regenerated
> in an edited tree, timing becomes observable again and `AC-9` becomes
> unprovable. ⭐ The clause is the whole reason a post-edit capture is honest.
>
> ### ⚠ WHAT THE `S1` WIDENING DOES AND DOES NOT CLOSE — for `AC-1`'s re-argument
>
> The measured split (`evt_j39jgesq7mfz`): the **authority** stays `pub(super)`
> (`AbiPlane`, `AbiDescriptor`, `AbiDescriptorShape`, `build_abi_plane`,
> `AbiPlane::validate`, `AbiBoundarySignature`); the **inert layout data** and the
> 5 closed vocabulary enums are promoted to `pub(in crate::cranelift_backend)`.
> ⇒ The emitter can **read** a unit's declared layout; it cannot construct a
> plane, mutate a descriptor, or reach the pre-emission validator to bypass it.
> ⭐ `PredeclaredFunctionId`'s inner field stayed `pub(super)` — the emitter can
> key and compare an id but ⛔ cannot mint one or do arithmetic on it.
>
> ⚠ **The stated residual, which is NOT discharged by `S1`:** `AbiSlot` /
> `AbiFrameHeader` fields are now readable in `cranelift_backend`, so `lowering`
> **can** spell a *local* `AbiSlot` literal — ⛔ Rust cannot forbid struct-literal
> construction within a crate. What closes it is that a forged slot has **no route
> into a unit** (`EmittableUnit` has private fields, no constructor, and a single
> producer). ⇒ ⛔ **That closure is `AC-12`'s to control, and `S1` does not
> establish it.** ⭐ Recorded as a residual rather than a pin because it is an
> unenforced-by-the-compiler property; ⛔ do not read the `EmittableUnit`
> argument as a discharged control.

## ✅ THE #9 HOLD IS DISCHARGED — the re-slice landed. ⛔ ~~BUT #10 IS NOW OPEN~~

⛔ **STALE HEADING — `#10` was ruled `evt_28cnmxf6ncghn`; see the release block
above.**

**This block used to read `⛔⛔ HELD AT HARD-STOP #9`. It is retained as the
reason the two prerequisites exist, rewritten to the state on `bd24422b`.**

⛔ **Read this before anything else in the file:** if you are looking for the
"missing and unowned prerequisite" this frame used to describe, **it is
`RT-FNSPLIT-B2R` and it merged.** A stale *"what's broken"* is worse than a
stale *"what's done"* — it sends a ring to rebuild what just landed.

### What #9 said, and what answered it

`runtime-implementer` raised `#9` at `evt_197xpdavdyrn0` **before writing any
code** — tree clean at `3891b7aa`, nothing to unwind. `D1`/`D2`/`D4`/`D6` were
jointly unsatisfiable inside the frame's then-boundary, because one closed
function per static origin requires **configuration-independent compilation**,
and at that base:

- `Lowered` was (and still is) a **compile-time specialization lattice, not a
  value representation** — only scalar variants hold `ir::Value`;
- the emitted signature was (and still is) `(pointer) -> i64`;
- `CaptureSlot` carried no type or width, and `PredeclaredFunction` carried
  **no signature**.

⇒ Compiling once per origin needed a **stable executable representation
contract** — layout, ownership, lifetime, and a call ABI — that did not exist.

✅ **RULED `evt_842spc7t6js1`, addendum `evt_t4fykh52ncb`, gated behind research
advisory `evt_531c4k52mshrn` as the armed `#9` pull required: option (i),
PREREQUISITE-FIRST.** Bounded coexistence (option ii) was **rejected**;
**`AC-1` and `D6` are NOT amended.** The advisory supplied the framing that was
adopted — the prerequisite is a stable **executable representation contract for
every value crossing a generated-function boundary**, ⭐ *not* one universal
boxed `Value`.

### The prerequisites are landed code, not a plan

| the #9 obstruction | what closed it | where it lives now |
|---|---|---|
| "which occurrences belong to one function unit?" | **`B2O`** | `SemanticDescriptor.owner: SemanticOwner` + `validate_function_units`, `…/planning/static_transition/semantic_ir.rs` |
| "what layout/ownership/convention does a value cross the boundary in?" | **`B2R`** | `…/planning/static_transition/abi.rs` — `AbiDescriptor`, `AbiSlot`, `AbiFrameHeader`, `AbiCarrier`, `AbiOwnership`, `AbiStorageOwner` |

**Ownership preceded representation** because the ownership mapping *defines the
cut*, and "every value that crosses a generated-function boundary" cannot be
enumerated before the boundary is known.

⛔ **Both prerequisites are INERT.** Neither emits. `abi.rs` measures **0
builders / 0 definitions / 0 declarations**, and `build_abi_plane` is called
once during planning (`planning/static_transition.rs:1010`). **This node is
where the contract first becomes executable**, which is exactly why it is the
atomic boundary.

### The two Steward rulings issued at the stop — both still binding

- ✅ **The `D5`/`D6` narrow reading is CORRECT.** Remove the **cross-owner
  whole-configuration re-emission**; keep ordinary traversal within one
  function's own body. The operative words are *"the recursive
  **whole-configuration body-emission** authority"* — the target is
  whole-configuration re-emission, not recursion as a code shape.
  ⭐ **The settling test:** converting traversal to a worklist would remove
  "recursion" while doing **nothing** for entry 2, and a reading under which the
  deliverable is satisfiable without touching the defect is the wrong reading.
  **Re-measured on `bd24422b`: 7 of the 59 sites consume a retained body —
  `core.rs:327, 605, 620, 764, 4817, 4829, 4954` — unchanged from `0aa9e53f`.**
  ⚠ `grep -c 'self.retained_body_occurrence('` returns **eight**; the eighth is
  `:4208`, the internal composition by `machine_body_occurrence`, which is a
  *caller* of the lookup rather than a further consumer. `core.rs:4160-4170`
  states that window in-source. **Do not report 8 consumption sites.**
- ✅ **`AC-G0`'s denominator is ANSWERED** — see the `AC-G0` block below. The
  answer is **6 definitions / 8 declarations per native module**, Θ(1), and the
  `6` is *already pinned* as `LOCAL_HELPER_COUNT`. Do not re-derive it.

### ⚠ The armed research trigger — unchanged by the re-slice

⛔ **STALE — superseded by the release block at the top of this file.
`#10` was RULED (`evt_28cnmxf6ncghn`, which inserted `B2V`), `#11` was ruled
(`evt_7ay6s5s79awz8`, which produced `C1`), the count of record is `11`, and the
next research pull is `#15`. Retained append-only.**

~~**Count of record = 10 — `#10` raised 2026-07-25 and is OPEN with the
Architect. NEXT RESEARCH PULL = hard-stop `#12`, unchanged and armed; `#10` is
not a pull stop.**~~ The count of
record lives in `docs/program/issues/RT-NATIVE-FNSPLIT.md` under
*"ARMED §5a RESEARCH-CONSULT TRIGGER"*; **on any disagreement that line wins.**
`B2O` and `B2R` both closed with **no hard-stop**, so the count did not move —
a clean WP never advances it.

```text
SYMPTOM INVENTORY (Architect appends one line per hard-stop; never rewritten)
NEXT PREDICATE CHECK = 6th entry   (3rd is CONSUMED — answered at entry 2)
ENTRIES = 3   ← the live recut chain's inventory is held in
              docs/program/issues/RT-NATIVE-FNSPLIT.md; append THERE, not here,
              so one chain has one inventory.
```

⛔ **The pre-recut chain is FROZEN at 33 hard-stops — do not resume that count**,
and do not read a `#36` anchor out of any older prose.

## ⛔ READ FIRST — THIS IS A CONSTRUCTION, NOT A PORT

The retired `RT-FNSPLIT-B2A` frame called this a behaviour-preserving **port**
and carried a `Retain` list of emitted units to re-key. **That list described
`b077eb7a`, a branch that never landed.** On the real base there is **one**
production Cranelift function in the lowering path and **no** emitted-unit
population to retain.

⇒ **The target code units are NEW. Describe them as new.** There is nothing to
port, nothing to re-key, and no prior implementation whose structure carries
authority. If you find yourself asking "how did the old path do this?", the
answer is: *there is no old path for this* — there is one monolithic function
and you are constructing the population that replaces it.

> ⚠ **Still true after `B2O` + `B2R`, and the distinction is easy to lose.**
> Those two landed a **declared, validated description** of the units and of
> the values crossing between them. They landed **no emitted unit** — the
> production census is still 1 builder / 1 definition / 2 declarations. So the
> *population* is now **derivable** rather than invented, and the *code* is
> still entirely new. ⛔ **A descriptor is not a function.** Reading `abi.rs`'s
> `AbiDescriptor` rows as an existing emitted-unit population is the same
> "carrier exists ⇒ property holds" gloss that cost this chain hard-stops #5
> and #8, one substrate later.

## The defect this closes — symptom-inventory entry 2, in one paragraph

`lower_expr` (`crates/ken-runtime/src/cranelift_backend/lowering/core.rs:4333`)
is a **recursive-descent inliner**: it re-lowers each retained body **per call
site, in that call site's whole configuration**. `B2A-S` fixed *which* body is
selected (by static origin, one closed dispatcher); it did **not** change *when
or how often* a body is emitted, and it says so in-source at
`lowering/mod.rs`:

> ⚠ This does **not** change *when* a body is lowered. Each call site still
> re-lowers the resolved term in its own whole configuration — that is
> symptom-inventory entry 2, and it stays open for `RT-FNSPLIT-B2F`.

⚠ **STALE ANCHORS — re-measured at `6534e4a6`: 61 calls, definition `:5149`,
root `:237`, span `:237`–`:7655`. Correction **2** at the top governs; re-derive
on your own base.**

~~**Measured on `0aa9e53f`:** `core.rs` holds **59** production calls into
`lower_expr`, spanning **`:188`** to `:6743`. There is **one** definition, at
`:4333`.~~

> ⛔ **CORRECTED 2026-07-25 — THE FRAME'S OWN COUNT WAS SPELLING-SCOPED
> (Steward defect, `runtime-implementer` at `evt_79xg7hvfktp3a`).** This read
> **58** sites spanning `:310`–`:6743`, derived from `grep -c 'self\.lower_expr('`.
> **`self.lower_expr(` is a claim about the RECEIVER's spelling; the property is
> "calls into `lower_expr`."**
>
> **The missed call is `core.rs:188` — `let lowered = compiler.lower_expr(`** —
> un-gated production inside `compile_expr_into_module`, receiver named
> `compiler` because the `Lowering` value is not yet a method receiver.
> ⛔ **AND IT IS THE ROOT**: it takes
> `SourceOccurrence { expr, static_origin: root_static_origin }` and is the entry
> into the entire recursive descent. **It is the one call site any functionization
> must convert into the call into the root target function** — a switch-over that
> migrated all 58 and stopped would have left the program's entry point on the old
> authority.
>
> ⭐ **The span corroborated the miss and I did not notice:** `:188` is *below*
> `:310`, so the stated range excluded it **by construction**. The count and the
> span were consistent with each other and both wrong — mutual consistency is not
> evidence.
>
> **Verified independently on `3891b7aa`** before accepting: whole-token
> `lower_expr` occurrences in `core.rs` = **65**, of which **5** are in comments
> ⇒ **60 code tokens = 1 definition + 59 calls** (58 `self.`, 1 `compiler.`).
> Exact agreement with the ring's tokenized `identifier_occurrences`
> (`control.rs:3529`).
>
> ⇒ **`D5`'s census MUST be tokenized, not substring-matched.** ★ This is failure
> class 1 of this frame's own pin-discipline section — *a needle scoped to layout,
> not to the property* — committed **in the frame's specification of the work**,
> and caught by the very tokenizer the ring built for `B2A-S`'s AC-4 after the
> Architect defeated its line-oriented predecessor. **The mechanism this chain
> built caught the frame that specified it.**
>
> ⚠ **Residual, stated rather than passed over:** `identifier_occurrences` does
> **not expand macros**, so "no further call reachable through a macro" is **not
> claimed**.
`core.rs` is production in its entirety — `mod tests;` at `:11-12` puts the
tests in a sibling directory, so there is no **test-module** region to partition
out of this file.

> ⛔ **CORRECTED 2026-07-25 (Steward defect, caught by `runtime-implementer` at
> `evt_197xpdavdyrn0`).** The sentence above originally read *"no `#[cfg(test)]`
> region to partition out of this file"* — **narrowly true of the test module,
> and it reads much wider than it is.** `core.rs` carries **22 inline
> `#[cfg(test)]` attributes** gating live mutation hooks and trace recorders
> **inside production functions** (`:56, :146, :148, :200, :1454, …`).
> ⇒ **`AC-1`'s "verified in both `cfg(test)` configurations" has real
> conditional surface inside the very file being rewritten.** ★ A measurement
> that is true but does not entail what the reader takes from it is the exact
> class this chain keeps paying for.

---

## Mechanism — RULED, not open (transcribed from the issue file)

The Architect ruled shape **(a)** on merits at `evt_6h5gw5c503n5z`, amended at
`evt_25ynt8615r9sk`. **Do not re-open on taste; do not re-derive from
`b077eb7a`, which contributes no authority.** The full four-merit rationale is
in `docs/program/issues/RT-FNSPLIT-B2F.md` and is binding:

> **One closed Cranelift target function per static planned function/origin,
> forward-declared as a bundle, with the fixed explicit activation frame.**

**The scaling claim, in the exact shape the Architect required:**

> Total units may be **Θ(n)** while **each function is bounded by its own static
> body/transition contract.**

⛔ **State it that way. Never as a blanket bound.** A claim that "the backend no
longer grows" is false and will be blocked.

## ⛔ ONE atomic boundary — the live split was REJECTED

Functionization **+** live switch-over **+** differential equivalence **+**
removal of the old authority are **ONE** review/merge boundary
(`Q3`, ruled). The ring's proposed live `ii`/`iii` split is rejected: it would
leave **two live production authorities**, which is the exact condition
"carrier and removal land together" exists to prevent.

**At every landed point there is exactly one production authority.**

The boundary includes the whole connected mechanism: target code-unit
population · declarations/signatures · the fixed dynamic-frame ABI ·
persistent-store transport · static dispatch/call edges · behaviour-equivalence
evidence · switch-over of **every** live consumer · **removal** of the recursive
whole-configuration body-emission authority.

### The one permitted escape, as a checkable graph property

A preparatory merge is acceptable **only** when unreachability is shown
mechanically by **all four** conditions in
`docs/program/issues/RT-FNSPLIT-B2F.md` (production still has exactly the
pre-existing one `FunctionBuilder::new` and one root `define_function`;
executable scaffold is `#[cfg(test)]`-only; no flag/branch/callback/pointer can
activate it; and a **committed** structural pin holds the zero edge). ⛔ If
preparation needs a production call edge, or emits even one callable target
unit, **it is not scaffold** and travels in the atomic boundary.

⚠ `cfg(test)` asymmetry cuts **both** ways — a `#[cfg(test)]`-only scaffold is
invisible to a production build, and a production-only path is invisible to a
test build. Whatever pins condition (4) must be verified in **both**
configurations.

> ### ⭐ THE ESCAPE HAS BEEN USED, AND IT IS SPENT
>
> **`B2O` and `B2R` ARE the preparatory merges this clause permits** — both
> landed under exactly these four conditions, and both discharged them: the
> production census never moved, and `abi.rs` measures 0/0/0 on all three
> needles at `bd24422b`.
>
> ⇒ **There is nothing further to prepare, and no third inert node to reach
> for.** `B2F` is the atomic live boundary. ⛔ If a deliverable here feels like
> it wants its own preparatory merge, that is a **hard-stop to raise** (the
> protocol below), **not** a fourth application of this clause — the clause
> permits inert scaffold, and everything remaining in this node is by
> construction the part that goes live.

## The landed surface you are building on — re-measured at `bd24422b`

⚠ **This whole section was rewritten on 2026-07-25 after `B2O` and `B2R`
landed.** The previous revision described a plane that *"already has what the
bundle needs"* and closed by saying `B2O` **must establish** the population.
`B2O` established it. Every table below is a re-measurement, not a carry.

### The production Cranelift surface today — unchanged by both prerequisites

| what | where (`crates/ken-runtime/src/cranelift_backend/`) | count |
|---|---|---|
| root `FunctionBuilder::new` | `lowering/core.rs:152` | **1** |
| root `define_function` | `lowering/core.rs:225` | **1** |
| `declare_function` — entry point | `lowering/core.rs:53` | 1 |
| `declare_function` — **imported** host dispatch (`ken_host_dispatch_v1`) | `lowering/core.rs:84` | 1 |
| the emitted signature `(pointer) -> i64` | `lowering/core.rs:47-50` | — |

⇒ **2 declarations, of which one is an import, not a definition.** ⭐ **`B2O`
and `B2R` moved none of these, and that is the point of their inertness** —
`abi.rs` itself measures **0 / 0 / 0** on the same three needles.

### `B2O`'s owner partition — the function-unit population, landed

| what | where (`…/planning/static_transition/`) |
|---|---|
| `SemanticOwner` — `Function(id)` · `Terminal` · `TrapTerminal`, **no `_` arm** | `semantic_ir.rs:62` |
| `PredeclaredFunctionId` | `semantic_ir.rs:38` |
| `PredeclaredFunction { id, planned_node, origin, program }` | `semantic_ir.rs:498` |
| `SemanticDescriptor.owner: SemanticOwner` — **the owning unit** | `semantic_ir.rs:508`, field at `:520` |
| `functions: Vec<PredeclaredFunction>` on `SemanticPlane` | `semantic_ir.rs:539` |
| `validate_function_units` — the four edge laws as `return Err` arms | `semantic_ir.rs:987` |
| `EdgeKind::StaticBody` | `static_transition.rs:105`, built at `:869`, `:895` |
| `TransitionKind::ClosureBody` | `static_transition.rs:858`, `:875` |

> ### ⛔⛔ THE SEED SET IS `plan.entries` ∪ EVERY `StaticBody` **TARGET**
>
> **The previous revision of this file said the unit set is "root ∪
> `ClosureBody` heads." THAT IS WRONG and `B2O`'s own frame says so in those
> words:** `TransitionKind::ClosureBody` is a body's **return successor**, not
> its head. Deriving the population from `ClosureBody` nodes seeds the wrong
> set.
>
> The landed, enforced equality is stated in-source at `semantic_ir.rs:997`:
>
> ```
> functions.len() == entries.len() + count(StaticBody edges)
> ```
>
> checked at `semantic_ir.rs:1006` (against both `functions` and the partition
> seeds) and cross-asserted at `static_transition.rs:2302`. The doc comment for
> the unit count is at `static_transition.rs:299`; `AC-4`'s statement of the
> same set is at `static_transition.rs:4085`.
>
> ⇒ **`B2F` consumes this set. It does not re-derive it, and it must not
> re-seed it from a transition kind.**

⚠ **Two shared exit sentinels sit OUTSIDE the exclusive partition** —
`SemanticOwner::Terminal` and `SemanticOwner::TrapTerminal`. They are **not**
function units and must not receive target functions; `shared_exits`
(`semantic_ir.rs:548`) locates them as a checked pair.

### `B2R`'s representation contract — landed, inert, and this node's input

Everything below is in `…/planning/static_transition/abi.rs` (1257 lines,
registered in `BACKEND_PRODUCTION_SOURCES`), built once per plan by
`build_abi_plane` at `planning/static_transition.rs:1010` into the
`abi: AbiPlane` field at `:235`.

| what | where (`…/planning/static_transition/abi.rs`) |
|---|---|
| `AbiCarrier` — the closed carrier language | `:60`, widths `:91`, alignments `:103` |
| `AbiOwnership` — the transfer discipline | `:208`, per-carrier at `:122` |
| `AbiStorageOwner` — **who owns the storage a carrier borrows from** | `:192`, per-carrier at `:168` |
| `AbiSlotKind` · `AbiSlot` · `AbiFrameHeader` | `:226` · `:294` · `:319` |
| `AbiDescriptor` / `AbiDescriptorShape` | `:354` / `:344` |
| `AbiCaptureProvenance` — `Lexical` / seed, carrier at `:260` (**takes no value**) | `:244` |
| `AbiUnitDefinition` | `:276` |
| `AbiPlane` + `shape` / `shapes` / `validate` | `:368` · `:391` · `:409` · `:910` |
| `build_abi_plane` | `:431` |
| `result_carrier` — exhaustive, **no `_ =>` arm** | `:582` |
| `unit_definitions` · `closure_provenance` · `declared_arity` | `:640` · `:711` · `:734` |
| `validate_boundary_layouts` · `AbiBoundarySignature` · `boundary_signatures` | `:1022` · `:1124` · `:1134` |
| the fixed convention slots | `CONVENTION_SLOTS` at `:381` |

⛔ **`B2R` DECLARED the contract; NOTHING ENFORCES IT AT RUNTIME, because
nothing runs.** Its own report states the limits, and two of them are directly
this node's work — quoted rather than paraphrased
(`docs/program/rt-fnsplit-b2r-abi-report.md`, *"Boundaries this node does not
cross"*):

> **6. Ownership modes are declared, not enforced.** … it does **not** verify
> any emitted code obeys them, because nothing is emitted. Enforcement is
> `B2F`'s.
>
> **7. Artifact-static seed material is DECLARED, not minted.** The seed carrier
> borrows from material that must exist before execution begins; **creating that
> material is `B2F`'s work and is deliberately absent here.**

★ And the value-independence claim is likewise **about the descriptor only**:
`build_abi_plane`'s inputs contain no `RuntimeGroundValue` and no `Lowered`, so
a *layout* is not chosen by inspecting a value — but **whether `B2F`'s
*emission* path stays value-independent is `B2F`'s obligation and is not
inherited.**

### Entry 1's dispatcher — do not break it

| what | where |
|---|---|
| sole `origin -> expression` consumer | `lowering/core.rs:4176` `retained_body_occurrence` |
| the plan-side accessor | `planning/static_transition.rs:1045` `source_occurrence`, `pub(in crate::cranelift_backend)` |
| the single write site | `planning/static_transition.rs:476` `record_source_occurrence` |
| the retained-closure carrier | `lowering/mod.rs` — `Lowered::Closure` / `DeclarationClosure` hold `body: StaticOriginId`, **no term** |
| the `AC-4` pin itself | `lowering/core/tests/control.rs:3425` `exactly_one_plan_origin_to_expression_lookup_exists` |

⚠ **Both plan-side anchors moved** — `source_occurrence` `:1009 → :1045` and
`record_source_occurrence` `:452 → :476`, displaced by `B2R`'s
`build_abi_plane` wiring. The lookup itself did not move.

⛔ **`B2A-S`'s AC-4 pins the `origin -> expression` lookup count at EXACTLY
ONE.** If `B2F` adds a second consumer, that pin reddens **correctly** — it is
not a false positive. Either route the new consumer through
`retained_body_occurrence`, or re-baseline AC-4 **explicitly in the frame
amendment** with the new count stated and justified. Do not quietly bump it.

---

## ⛔⛔ THE PIN YOU WILL BREAK FIRST — re-baseline it deliberately

`lowering/core/tests/control.rs:3337`
`correspondence_adds_no_emitted_unit_to_the_production_census` asserts an
**exact** census over five production files (`:3363`–`:3423`); the three needles
are `matches("FunctionBuilder::new(")`, `matches(".define_function(")`, and
`matches(".declare_function(")`:

```
lowering/core.rs                              builders 1  definitions 1  declarations 2
lowering/mod.rs                               0  0  0
planning.rs                                   0  0  0
planning/static_transition.rs                 0  0  0
planning/static_transition/semantic_ir.rs     0  0  0
```

> ### ⛔ THE CENSUS'S FIVE ROWS ARE NOW NARROWER THAN THE PRODUCTION SURFACE
>
> **New, measured at `bd24422b`, and it is a `B2F` decision rather than a
> defect in `B2R`.** `B2R` added `planning/static_transition/abi.rs` to
> **`BACKEND_PRODUCTION_SOURCES`** (`control.rs:3686`, now **13** files, was 12)
> — but **not** to this five-row census, which `B2R` never touched.
>
> Today that is invisible, because `abi.rs` measures **0 / 0 / 0** on all three
> needles (I re-measured it). **But `B2F` is the node that makes emission real**,
> and a census whose population is a hand-listed subset of the file set that a
> *sibling* pin closes is exactly the shape this chain keeps paying for: the
> inventory pin says "13 files or redden", the census pin says "these 5", and
> **nothing relates the two.**
>
> ⇒ **`AC-2` must state which population the census covers and why**, and either
> add the missing production file(s) as explicit `0/0/0` rows or record the
> exclusion with its reason. ⛔ **Do not leave it silent** — a row that is absent
> and a row that is zero are indistinguishable to a reader and only one of them
> is a claim.

**`B2F`'s whole job is to add declarations and definitions, so this pin WILL go
red — by design.** Two failure modes, and the frame forbids both:

- ⛔ **Do not delete or weaken the pin.** It is the only mechanical statement of
  the emitted-unit population, and it is what condition (1) of the escape
  clause rests on.
- ⛔ **Do not re-baseline it to whatever the new numbers happen to be.** A census
  re-fit to the observed output measures nothing.

⇒ **Re-baseline it to the numbers your DESIGN predicts, BEFORE you measure**,
and record the prediction in the test comment alongside the reason. If the
measured counts differ from the predicted ones, that is a finding to route —
not a number to update. ★ This is the `pin-a-property` discipline
(`agent/playbooks/tools/pin-a-property.md`): **predict, then measure; a
post-hoc baseline is not a control.**

### ⚠ AND THE CENSUS POPULATION IS NARROWER THAN "THE BACKEND"

> ## ✅ `AC-G0` IS ANSWERED — 6 definitions / 8 declarations, Θ(1). READ THIS FIRST.
>
> **The denominator question is settled and re-verified at `bd24422b`. Do not
> re-derive it, and do not re-measure `native_int_clif` from scratch.** What
> remains is a *pin*, not a *measurement*:
>
> | | state | action |
> |---|---|---|
> | definitions = **6** | **already pinned** — `LOCAL_HELPER_COUNT` at `…/cranelift_backend/artifact/tests.rs:56` | **cite it; do not duplicate** |
> | declarations = **8** | genuinely unpinned | **pin it** — 2 `Linkage::Import` + 6 `Linkage::Local` |
> | program-independence | **enforced by the signature** | **add no test** — record the signature |
>
> The narrative below is retained because it records *how* the number was got
> wrong and why the wrong one was plausible. ⭐ It is the reason `AC-G0` exists.

**Measured, and it matters for the growth verdict:**
`crates/ken-runtime/src/native_int_clif.rs` is **production** — declared
un-gated at `lib.rs:23` — and holds **5** `FunctionBuilder::new` **source sites**
(`:190, :294, :534, :695, :790`) with its own `declare`/`begin`/`finish`
helpers, emitting exact-`Int` support "into every native module."

> ## ⛔ AC-G0 ANSWERED — AND MY "5" WAS THE WRONG POPULATION
>
> **Measured by `runtime-implementer` at `evt_1vz8pmztgtye9`, prediction-first
> per this frame's own rule. The prediction (5 definitions, ≥5 declarations)
> was WRONG, and they reported it instead of updating it — which is exactly the
> required behaviour.** Per native module, `native_int_clif` emits
> **unconditionally**:
>
> - **8 declarations** — 2 `Linkage::Import` (`malloc` `:76`, `free` `:81`) plus
>   6 `Linkage::Local`
>   (`ken_native_int_{resolve,intern,binop,compare,narrow,export}_local`, `:84-89`
>   — ⚠ re-measured at `bd24422b`; this frame previously said `:83-88`). All six
>   route through the one `declare` helper, whose `declare_function(name,
>   Linkage::Local, &sig)` is at `:148` — **so a source census of
>   `.declare_function(` in that file returns 3, not 8.** The unit count and the
>   spelling count differ here too.
> - **6 definitions** — `define_{resolve,intern,compare,narrow,export,binop}`,
>   each called exactly once from `emit_native_int_local_graph` (`:107-112`)
> - **6 `FunctionBuilder::new` invocations from 5 source sites** —
>   `define_view_consumer` (`:695`) is the shared body of **both**
>   `define_narrow` and `define_export`
>
> ⭐ **So the emitted-unit constant is 6 definitions / 8 declarations. My "5
> `FunctionBuilder::new` sites" is a SOURCE-SITE COUNT, and I presented it where
> an emitted-unit population belongs — pinning 5 would pin the wrong
> population.** ⚠ That is precisely the AC-G0 hazard this section was written to
> catch, committed by the person writing the section: **a grep count answers "how
> many spellings", never "how many units."** One source site emitted two units
> and the count could not see it.
>
> ✅ **It IS genuinely a constant, so the orthogonality holds:**
> `emit_native_int_local_graph` is called **once per compiled module**
> (`core.rs:66`), takes no program-derived input (only `module` plus a
> `#[cfg(test)]` mutation flag), and the six defines are unconditional
> straight-line calls. ⇒ **Θ(1) per native module, orthogonal to `B2F`'s
> per-static-origin Θ(n)** — the shape predicted, with a different number.
> **Pin 6/8, not 5.**
>
> ### ⛔ AC-G0 NEEDS LESS NEW WORK THAN FRAMED — and one thing I did not ask for
>
> **The `6` is ALREADY PINNED behaviourally, and the `5` was explicitly retired
> in this repo on 2026-07-21.** `artifact/tests.rs:56` holds
> `const LOCAL_HELPER_COUNT: usize = 6`, with an in-source note recording that
> *"the bare `5` was unverified provenance"* and grounding 6 against
> `emit_native_int_local_graph`'s six `define_*` helpers (`Q-RESIDUE`).
> ⇒ **CITE that pin; do not duplicate it.** ⚠ I re-litigated a number this repo
> had already settled — the frame should have found the existing pin before
> asking for a new one.
>
> **So `AC-G0`'s remaining work is narrower than the section above implies:**
>
> - **Definitions (6): already pinned** — cite `LOCAL_HELPER_COUNT`.
> - **Declarations (8): genuinely unpinned** — 2 `Linkage::Import` + 6
>   `Linkage::Local`. This is the part worth adding.
> - **Program-independence: needs NO detector at all.**
>   `emit_native_int_local_graph(module, wrapping_mutation)` takes **no
>   program-derived parameter**, so the compiler already forbids the growth mode
>   `AC-G0` worries about — making it program-dependent requires a **visible
>   signature change**. ★ `pin-a-property` §1: prefer the mechanism that makes the
>   violation **unrepresentable** over one that detects it. **Do not add a test
>   here; record the signature as the guarantee.**

**It is NOT in the N1 census's five rows, and NOT in
`BACKEND_PRODUCTION_SOURCES` (`control.rs:3686` — ⚠ **13** files as of `B2R`,
which added `abi.rs`; this frame previously said `:3580`, 12).**

This does **not** invalidate the landed pins — they are explicitly scoped to
"the PRODUCTION lowering and planning sources" and say so. ⛔ **But `B2F` owns a
scaling verdict, and a verdict whose denominator silently excludes a sibling
production emitter is measuring the wrong population.** So the frame requires an
explicit answer, not silence:

> **AC-G0 — name the denominator.** State which production Cranelift emitters
> the growth verdict covers, and for each excluded one, state *why* the
> exclusion is sound. The expected shape for `native_int_clif` is **a fixed
> constant per native module**, orthogonal to `B2F`'s **per-static-origin
> Θ(n)** — but that must be **measured and written**, not assumed. If it is a
> constant, pin the constant.

★ A measured property can be **true** and still not entail what the mechanism
needs. "One production function in the lowering path" is true and is *not* "one
production Cranelift function in `ken-runtime`."

---

## ⛔⛔ INHERITED 2026-07-28 — `B2F` now owns `AC-C4`'s runtime INVOCATION half

**Steward decision on the Architect's mechanism ruling `evt_17fgr8nk6859c`.**
`RT-FNSPLIT-C1` built the carried induction hypothesis and proved it eliminates;
**invoking** it cannot be done there. ⇒ ⭐ **The invocation lands inside this
node's existing atomic target/switch boundary — ⛔ it does NOT get a preparatory
merge of its own**, which this frame already classes as a hard-stop.

⚠ **Why it could not stay in `C1`:** a *specialized* recursive elimination
terminates because its residual is a compile-time value that strictly shrinks. A
**carried** residual is a runtime word ⇒ nothing shrinks at compile time, so
emitting the recursive case emits its IH invocation, which re-enters the same
eliminator **without bound** (measured on `6bae122a`: a compiler stack overflow,
not an error). The only general vehicle is a callable target, and `C1`'s
`AC-C10` forbids target-function population.

### The required execution vehicle — ruled, ⛔ not open

**One closed, recursively callable Cranelift target per static
computational-eliminator origin.** Forward-declare its `FuncId`, define it once,
and make a zero-argument structural IH invocation emit a **direct call to that
same target** with the projected recursive child word. The call returns one
carried result word; lowering then continues the enclosing case body, so
**non-tail contexts use the machine call stack** — ⛔ never a new Ken
continuation VM.

⛔ **Rejected alternatives, on merits — do not re-open.** A plain **CFG
backedge** is correct only when the IH result *is* the case result; `Wrap(x) =>
Suc(IH(x))`, pairing two IH results, or using an IH under a later call must
retain a continuation across the recursive step, and `AC-C4` imposes no
tail-position restriction — so an in-function header jump is a strict subset.
`Lowered::RecursiveBackedge` does not rescue it: it represents the existing
tail-recursive declaration jump with a predecessor-free current block, and making
it a **carried** value reopens the phase identity `C1 §2g-i` closed. ⛔ And ⛔ **no
explicit heap continuation / work stack** inside the current root function — that
is a second runtime abstract machine (frame tags, saved case state, dispatch,
unwind), larger than the already-decided closed-function construction and needing
its own authority and coverage.

**Target contract:**

- **Static authority** — target identity comes **only** from the computational
  eliminator's `static_origin` and its checked frame/slot records. ⛔ Never derive
  a target, case, slot, or invocation identity from the carried word.
- **Dynamic ABI** — the carried scrutinee word plus the already-defined fixed
  activation/environment context the static body needs; the result is one carried
  word through the existing result/error convention. ⛔ No `Lowered`, inverse
  conversion, producer re-entry, or durable closure crosses this call.
- **Body** — the existing artifact-static constructor identities, arity checks,
  field projection, closed default, and `case_env` order. A declared recursive
  position calls **the same target** on that projected child; a non-recursive case
  returns normally. Non-empty source arguments stay refused **before** invocation
  installation.
- **Ownership** — the helper belongs to the **same `SemanticOwner`/static origin**
  as the computational-match body. Checked-frame, IH-slot, activation, cursor and
  producer-origin metadata remain **compiler authority**; the runtime word supplies
  value data only.
- **Termination premise** — ⭐ the call is made only after the existing
  producer→validator boundary has established a **finite acyclic carrier graph**,
  and only on a **declared recursive child edge**. The measure is **strict descent
  in that validated graph**, ⛔ not compile-time shrinkage. ⛔ A forged or
  unvalidated word may not enter this target.
- **Boundary** — the outer `ComputationalRecursorClosure` stays specialization-only
  and unconditionally non-transferable; `C1`'s controls 4 and 5 are unchanged.

### Required causal closure — ⛔ a tail-shaped fixture does not pass this

⭐ **Control 1 must become a NON-TAIL executable fixture**: depth **at least two**
and a case such as `Wrap(x) => Suc(IH(x))`, or an equally discriminating
constructor around the IH result. Assert the final **value/discriminator**. ⇒ This
proves the continuation **survives** the recursive call — a tail-only jump cannot
pass it, which is exactly why the backedge was rejected.

Additionally pin:

1. **exactly one** declared/defined target for the fixture's computational
   eliminator `static_origin`, with its self-call resolving to that same `FuncId`;
2. a **non-default recursive sibling position**, and a mutation passing the **wrong
   child word** reds the value/ownership control **without changing target
   selection**;
3. a mutation **selecting another static target** reds the static-origin control
   while the carrier value remains valid;
4. replacing the runtime call with the **old inline re-entry** reaches the named
   compile-time sentinel — ⛔ never an actual stack-overflow run;
5. existing specialized-recursion, outer-capsule refusal, and non-empty-argument
   refusal stay **green**.

### ⛔ Release gate — CORRECTED

`B2F` gates on **the closed `C1` carrier artifact**. ⛔ It does **not** gate on any
claim that the recursive call already exists — that claim is unsatisfiable in
`C1` and was the source of the jointly-unsatisfiable state this amendment
resolves.

---

## Deliverables

> ### ⭐⭐ WHAT THE PREREQUISITES SUBTRACTED — read this before `D1`
>
> **`D1`–`D3` were written when this node had to CONSTRUCT the population, the
> frame layout, and the store contract. It no longer does.** `B2O` and `B2R`
> landed all three as **declared, validated, inert** structure. This node's
> obligation on those three axes changed from **construct** to **consume and
> enforce**, and the wording below has been re-cut to say so.
>
> | axis | before the re-slice | on `bd24422b` |
> |---|---|---|
> | which occurrences form one unit | invent it | **`B2O` — consume `SemanticOwner`** |
> | frame layout, carriers, widths, ownership modes | design it | **`B2R` — consume `AbiDescriptor` / `AbiSlot` / `AbiFrameHeader`** |
> | store transport contract | state it | **`B2R` — consume `AbiStorageOwner` + `AbiOwnership`** |
> | **emit** target units, **obey** those modes, **mint** the artifact-static seed material | — | ⛔ **all still this node's, and none of it is inherited** |
>
> ⛔ **The subtraction is of DESIGN work, not of PROOF work.** `B2R` declared
> the modes and validated that each slot carries *its own carrier's*
> declarations; **it verified no emitted code obeys them, because nothing is
> emitted.** ★ A contract that is declared and validated is not thereby
> honoured — that gap is exactly what this node closes, and an AC that reads
> "the descriptor says so" discharges nothing.

**D1 — the target code-unit population, CONSUMED from `B2O`.** One closed
Cranelift function per **`PredeclaredFunction` in the validated `SemanticOwner`
partition**. Forward-declare the whole bundle (`Module::declare_function` for
every signature/ID first) from `B2R`'s validated descriptors, then define each
body.

⛔ **Do not re-derive the unit set, and do not invent a parallel numbering.**
The set is `plan.entries` ∪ every `EdgeKind::StaticBody` **target**; the
equality `functions.len() == entries.len() + count(StaticBody edges)` is
already a planner error at `semantic_ir.rs:1006`. ⛔ **The two shared exits
(`Terminal`, `TrapTerminal`) are NOT units** and get no target function.

**D2 — emit against `B2R`'s activation frame; do not redesign it.** Every
dynamic environment/control/store value crosses into a target function through
the **declared** `AbiFrameHeader` + `AbiSlot` layout for that unit, never
through capture-by-construction. ~~`AbiPlane::shape` / `shapes` is the
accessor.~~ ⛔ **FALSE — both are `#[cfg(test)]` (`abi.rs:342`, `:388`); production
cannot call either.** Consume `AbiDescriptor` + its declared slot run via the
**`EmittableUnit`** projection — correction **5** in the frame-corrected block at
the top of this file.

⚠ **"Fixed frame" does not mean equal byte size across origins** — `B2R` states
this explicitly, and reading it as one universal layout is the error that would
reintroduce a boxed `Value` nobody asked for.

⛔ **The value-independence obligation transfers but does not carry.** `B2R`
proved *the descriptor* is not chosen by inspecting a runtime value
(`build_abi_plane`'s inputs contain no `RuntimeGroundValue` and no `Lowered`).
**Whether the EMISSION path stays value-independent is unproven and is `D2`'s
to establish** — quoting `B2R`'s own limit statement: *"this says nothing about
whether `B2F`'s emission path stays value-independent. That obligation is
`B2F`'s."*

**D3 — persistent-store transport, and MINT the artifact-static seed
material.** The store crosses the ABI under `B2R`'s declared `AbiOwnership` /
`AbiStorageOwner` contract; this node makes emitted code **obey** it.

⛔ **The seed carrier borrows from artifact-static material that DOES NOT EXIST
YET.** `B2R` declared it and deliberately did not mint it. ★ And the reason
matters: `Lowering<'a>` holds `seed_env: &'a NativeSeedEnvironment` — a borrow
that lives only during *compilation* — while `CompiledModule<M>` has no
lifetime parameter, so **nothing borrowed can escape into the artifact** (the
compiler refuses it, and
`escaping_a_source_borrow_into_the_compiled_artifact_does_not_typecheck` pins
exactly that). ⇒ **A runtime activation cannot borrow the seed environment.**
Creating owned, artifact-static seed material that outlives every activation is
**new work in this node**, and it is the one piece of `B2R`'s contract with no
landed counterpart at all.

> ### ⛔ `D3` GROUNDED 2026-07-28 — "no landed counterpart" IS LITERALLY TRUE, AND
> ### ⛔ THIS IS NOT A NEW HARD STOP. Count of record stays **11**.
>
> **Measured by `runtime-implementer` at `evt_2we75javgbctw`** on
> `wp/RT-FNSPLIT-B2F-functionization-live = 001242a8`.
>
> ⭐ **The frame's claim is confirmed at the TOOLCHAIN level, not merely the
> design level:** `declare_data`, `define_data` and `DataDescription` have **zero**
> occurrences anywhere in `ken-runtime`. ⇒ There is no in-crate precedent to copy.
>
> **What is actually there today:**
>
> | fact | consequence for `D3` |
> |---|---|
> | `lower_seed_capture` resolves the symbol against `seed_env` **at compile time**; `lower_ground_value` emits `iconst(…)` for `Bool` and small `Int` | ⛔ the value is **baked into the instruction stream**. An `iconst` is owned by the frame the moment it exists — ⛔ **neither a borrow NOR artifact-static storage** |
> | `Bytes` / `String` / `Constructor` / `Record` map to `Lowered::…` holding the **compiler's own Rust values**; only `Lowered`'s scalar variants carry an `ir::Value` | ⛔ a seed capture of a non-scalar **exists only compiler-side and is specialized away** — it has **no runtime representation at all** |
>
> ⇒ ⛔ **`B2R`'s `GroundValueCarrier` declaration (`BorrowedForActivation` /
> `ArtifactStatic`) describes NOTHING THAT EXISTS, in both halves.** ⚠ Reading it
> as a description of current behaviour is the error `D3` invites, and it is
> wrong in **both** directions rather than merely incomplete.
>
> #### ⛔⛔ THE §5a BOOKKEEPING — do not let this become hard stop `#12`
>
> ⚠ **Finding 2 has hard-stop `#10`'s exact signature** — a dynamic value with no
> executable representation. ✅ **It was correctly NOT raised as a new stop, and
> the count of record stays `11`.** The reason is durable, not a judgement call:
>
> - `#10` was **discharged** by `B2V` + `C1` landing an executable carrier, and
> - ⭐ **this frame already assigns minting the durable seed material to THIS
>   node, in these words** — *"`B2R` declared it and deliberately did not mint
>   it… new work in this node."*
>
> ⇒ ⭐ **Resolvable from the frame ⇒ resolve from the frame** (`COORDINATION §6`).
> ⛔ **A seat re-deriving this measurement later WILL see `#10`'s signature. It is
> a known, assigned deliverable — not a new structural wall.** ⛔ Do not move the
> count to `12` on it, and ⛔ do not re-anchor the research cadence (next pull
> stays `#15`).

**D4 — static dispatch / call edges, derived from the graph.** Call sites
reference target functions by their **static** identity. No indirect dispatch on
a dynamic property, and no runtime lookup that re-derives which code to run from
a value.

⛔ **The boundary disposition is DERIVED from validated graph facts, never
hand-authored.** The classification and its load-bearing reject arm are ruled
and live in `docs/program/issues/RT-FNSPLIT-B2F.md` (*"RE-HOMED FROM `B2O`"*):
a `StaticBody` edge between **distinct** owners is a **cross-owner call**; a
same-owner ordinary edge is **local traversal**; a function edge to a
**terminal** owner is a **shared exit**; **anything else is a REJECT.**

> ### ⛔⛔ THOSE FOUR ARE INVARIANTS THIS NODE RELIES ON — NOT ACs IT DISCHARGES
>
> **`validate_function_units` (`semantic_ir.rs:987`) already enforces all four
> as `return Err` arms in the production bytes `B2O` landed** — I re-verified
> the arms at `bd24422b`: the `match` over `SemanticOwner` has **no `_ =>`**
> (so a new variant is a compile error), `Function(to_unit) if to_unit ==
> from_unit => {}` is the accepting arm with `Function(_) =>` rejecting, and the
> shared-exit rejects are explicit.
>
> ⇒ **Planning REFUSES TO CONSTRUCT a violating graph**, so a `B2F` control
> asserting one of these laws is green on **every input that can reach `B2F`**.
> **It would read as coverage and test nothing.** ⛔ Cite them to `B2O` as
> inherited invariants; do **not** re-assert them here.
>
> ★ **What actually survives the re-home is one-for-one consumption** — that the
> view is consumed without a second table — **which inert `B2O` could not check
> and never could.** When a claim moves between nodes, the part that survives is
> the part the source node was structurally unable to verify.

**D5 — switch-over of EVERY live consumer.** ⛔ **DERIVE THE POPULATION ON YOUR
OWN BASE; the numbers here are dated.** At `6534e4a6` it is **61** calls into
`lower_expr` (`core.rs:237`–`:7655`, definition `:5149`), root at **`:237`** —
⛔ ~~59, `:188`–`:6743`, root `:188`~~ (correction **2** at the top). **Every
call is accounted for, including the root**, which is the one site that must
become the call into the root target function. ⛔ A census short of the
**re-derived** count is an incomplete switch-over, not a partial success —
enumerate, do not sample, and ⛔ **do not port the enumerated site lists below;
they were taken at `bd24422b` and every line number in them has moved.**
⛔ **Derive the population with a TOKENIZED census (`identifier_occurrences`,
⛔ now `control.rs:3698`), never `grep 'self.lower_expr('`** — that spelling
misses the root. ⚠ At `6534e4a6` the naive grep still returns **60** and still
misses the root, so the spelling-scoped defect is live at the new numbers too.

> ⚠ **Re-measured at `bd24422b`, and every number in this deliverable held:**
> one definition at `core.rs:4333`, **59** calls spanning `:188`–`:6743`, the
> root still at `:188`. `B2O` and `B2R` touched neither the definition nor any
> call site. ⚠ The tokenizer's own anchor **did** move — `identifier_occurrences`
> is at `control.rs:3635`, not `:3529`. **Re-derive it again on your own base.**

> ## ⛔ D5/AC-5 AMENDED 2026-07-25 — THE TWO-WAY CLASSIFICATION WAS UNSOUND
>
> **This deliverable originally read: each site is "either migrated to the
> function-call path or explicitly classified as not-a-body-emission with the
> reason recorded." That is a STEWARD DEFECT — it presupposes each site has ONE
> static disposition, and at least 8 do not.** Found by `runtime-implementer` at
> `evt_1vz8pmztgtye9`.
>
> **MEASURED:** the 7 retained-body resolutions flow into `lower_expr` calls via
> shared parameters, and **those same parameters are also fed ordinary
> sub-expressions** (`lower_computational_producer_expr` is called with a
> non-retained occurrence from `:1575`, `:1671`, `:1795` and from its own
> descent). ⇒ Sites **407, 410, 438, 1330, 1518, 1573, 6108, 6207** are
> body-emission on some call paths and plain traversal on others.
> **THE GAP:** *a disposition table keyed by site is only sound if disposition is
> a function of the site.* For those 8 it is a function of the **path**.
> ★ **An AC taxonomy with no cell for the honest answer reads as complete** — the
> table could have been filled in fully and been wrong.
>
> ### The amended classification — five classes, matching measured provenance
>
> | class | count | derivation |
> |---|---|---|
> | structural: `child_occurrence` | 32 | positional syntax child |
> | structural: `case_body_occurrence` | 9 | match-arm bodies |
> | **caller-dependent** (parameter-fed) | **14** | provenance is the caller's |
> | synthesized occurrence | **3** | `:188` **root**; `:2288` source-machine fallback; `:6291` `declaration_body` |
> | direct retained body | 1 | `:4878`, from `:4829` |
>
> **32 + 9 + 14 + 3 + 1 = 59** ✓
>
> ✅ **RE-MEASURED AT `bd24422b`, AND THE WHOLE TABLE HELD.** I re-derived the
> call population after `B2O` and `B2R`: still **59**, still one definition at
> `core.rs:4333`, still spanning `:188`–`:6743`, and **every line number named
> in this amendment still resolves to a `lower_expr` call** — the 8
> caller-dependent sites (`407, 410, 438, 1330, 1518, 1573, 6108, 6207`), the 6
> untraced ones (`1669, 1793, 5920, 5990, 4538, 1892`), the 3 synthesized
> (`188, 2288, 6291`), the direct retained body (`4878`), and the hand-resolved
> `4454`. ⚠ **This is the exception, not the rule** — the plan-side anchors
> beside them moved by tens of lines. **Re-derive on your own base anyway; the
> fact that they held once is not a licence to trust them twice.**
>
> ⛔ **THE TOTAL WAS 58 IN THE FIRST CUT OF THIS AMENDMENT AND THAT WAS WRONG**
> (crossed with Finding 4; corrected by `runtime-implementer` at
> `evt_2jaqww2frbrba`, a correction to their own measurement). The 58 was taken
> over `self.lower_expr(` sites only — **the same spelling-scoped population
> Finding 4 shows is incomplete.** The root call at `core.rs:188` is a
> **`synthesized`** occurrence, demonstrably so: `core.rs:190-193` builds
> `SourceOccurrence { expr, static_origin: root_static_origin }` **inline at the
> call site** rather than deriving it from a parent.
>
> ⭐ **The class it lands in is the one that matters most for the pending fork:**
> the root is `synthesized`, so it is **not reachable from any parent
> occurrence** — it is where the whole descent is **seeded**. Under either (i) or
> (ii), that site is the entry into the root target function, and **it is the one
> call site that cannot be dispositioned as "traversal."**
>
> ⚠ **The number is the symptom; the mechanism is the fix.** `AC-5` must specify
> a **tokenized** census, or the next reader re-derives 58 from
> `self.lower_expr(` and the root goes missing again.
>
> ⇒ **For the caller-dependent class, disposition is recorded per
> `(site × reaching path)`, not per site.** A site in this class is discharged by
> enumerating its reaching paths and dispositioning each; it is **not** discharged
> by picking one of the two original labels.
>
> ⛔ **"Cannot determine" must not silently fall through to a class.** Six sites
> came back undetermined by mechanical resolution and were each resolved by hand
> (`pin-a-property` §4) — e.g. `:4454`, whose occurrence is a `for`-loop variable
> over `[(then_block, then_expr), (else_block, else_expr)]`, both
> `child_occurrence`.
>
> ⚠ **NOT CLAIMED:** that the other 6 caller-dependent sites
> (`1669, 1793, 5920, 5990, 4538, 1892`) can never be reached with a retained
> body. They were not traced to one; **unreachability is not asserted.**
>
> ⭐ **This corroborates hard-stop #9 from a second direction, and that is its
> real significance:** the body-emission authority is **not localized at a handful
> of call sites** — it is **diffused through the producer / eliminator-frame
> machinery**, with a retained body's `cases` and `default` travelling *inside* an
> `OrdinaryEliminatorFrame`. ⇒ **`D6`'s "removal" is not excising a function; it
> is dismantling the deforestation architecture that
> `lower_computational_producer_expr` + `EliminatorFrame` exist to implement.**
> Any ruling on the #9 fork must be read against that scope.

**D6 — REMOVAL of the recursive whole-configuration emission authority.** The
inliner goes. ⚠ **Re-scoped and this is the load-bearing warning:** the
"whole-configuration emission path" is **not a separable path you can delete** —
it is `lower_expr`'s entire recursive-descent structure at `core.rs:4333`, reached
from **every call site the `AC-5` derivation returns on your own base**. Removal
means the recursion is gone, not flag-disabled, not `#[allow(dead_code)]`, not
retained "for the differential."

> ## ⛔⛔ `D6` IS NOT A FOLLOW-ON TO THE SWITCH-OVER — IT IS THE SAME EDIT
>
> **Measured, not forecast** (runtime-implementer, `evt_39eq88mcs7n2k`, against
> exact `f465bae0`; runtime-leader concurred, `evt_6j07e092z8t4k`; Steward
> accepts the measurement and this block replaces any earlier sequencing).
>
> The plan's **per-occurrence atom records are single-consumption.** Emitting a
> body as a real unit **while the root still inlines that same body** consumes
> each record twice, and the second consumer finds it gone — 7 ×
> `PlannerInvariant("static origin has no atom of that kind at that occurrence")`.
> ⇒ **The root's recursive descent cannot coexist with real unit bodies.**
>
> ⭐ ⛔ **THE CONSEQUENCE, AND IT IS NOT OPTIONAL: `AC-6`'s removal pin and
> `AC-11` clause 3's invariant must both be IN PLACE AND GREEN ON THE
> PRE-REMOVAL BASE, BEFORE the combined edit lands.** ⚠ A pin authored *after* a
> removal cannot witness the removal, and **the tests a ban reddens on
> introduction never contain its witness** — those exercise the **success** path.
> ⭐ The frame already has one correct instance to copy:
> `an_unrepresentable_transfer_is_refused_before_any_unit_is_declared`
> was deliberately asserted **before** `D6` deletes `lower_expr`'s late arm.
>
> ⚠ **And a red intermediate is not a permitted resting point here.** A red
> baseline makes every subsequent mutation experiment uninterpretable in **both**
> directions, and the remaining work is mutation-heavy (the removal pin, clause
> 3's invariant, the destination control). ⇒ Keep the green checkpoint; land the
> combined edit once, green.

**D7 — behaviour-equivalence evidence.** The five-category differential suite
(old `AC-1` of the retired frame lands here). Categories must be independently
falsifiable, and the differential must run against a **pre-change baseline whose
recipe is in the tree** — see the recipe requirement below.

**D8 — the growth verdict.** The Θ(n)-units / bounded-per-function claim, stated
in the Architect's exact shape, with `AC-G0`'s denominator named.

**D9 — the sole carrier producer made TOTAL over the value classes a unit result
can take.** ⭐ **Added 2026-07-28 on the Architect's ruling `evt_69aedr4j844xd`;
Steward-scoped.** Extend `C1`'s producer to cover **spillable `Int`, `Bytes`,
`ProcessExitStatus`, `HostResult`, and `Trap`** — the five classes whose absence
the switch-over measured as **69 reds**, each of which says so in its own error
text (*"the carrier producer does not yet emit …"*).

> ### ⭐ WHY THIS IS `B2F` WORK AND NOT A NEW NODE — the scoping call, made
>
> **Steward, 2026-07-28.** `C1` is **merged**; this is a residual of a closed
> node that only functionization could surface. ⭐ **Under the inliner a closure
> body's result never crosses anything** — it stays a compile-time `Lowered` in
> the caller's own dataflow. A **unit's** result crosses a function boundary and
> so must be *produced into a carrier*. ⇒ Producer completeness stops being a
> `C1`-internal question and becomes this node's critical path.
>
> ⛔ **It does not touch the atomic boundary, and that is the test that decides
> it.** The clause above forbids a **second production authority**; this makes
> the **single existing** authority total. It declares **no target unit**, adds
> **no production call edge**, and installs **no second decoder** — so it is
> ⛔ **not** a fourth application of the spent preparatory-merge escape, and
> raising it as one would be a hard-stop against a clause it does not touch.
>
> ⇒ ✅ **Land it as a green increment on the pre-switch-over base**, the same way
> `ArtifactHelpers` and `AC-4`'s route pin already landed. ⚠ Those 69 tests are
> **green today** — they red only *under* a switch-over that is not landed — so
> this deliverable is behaviour-**additive**: new capability, no existing
> behaviour changed. ⭐ That is precisely what makes it safe to land alone, and
> ⛔ it is also what makes a green suite worth nothing as evidence for it.

---

## Acceptance criteria

**AC-1 — one production authority at the landed point.** No feature flag,
runtime branch, optional callback, function pointer, or alternate entry can
reach a second body-emission path. Pinned structurally, verified in **both**
`cfg(test)` configurations.

**AC-2 — the emitted-unit census is re-baselined to a PREDICTED number, AND
its population is stated.**
`correspondence_adds_no_emitted_unit_to_the_production_census`
(`control.rs:3337`) carries the new counts, the prediction, and the reason. The
pin still reddens on an unplanned declaration or definition.

⛔ **AND it must say which files it covers and why.** Its five rows are a
hand-listed subset of the **13**-file `BACKEND_PRODUCTION_SOURCES`
(⛔ now `control.rs:3749`); `B2R`'s `abi.rs` is in the second and not the first.
⛔ **AND `abi.rs` IS NOT THE ONLY OMISSION** — live `boundary_value_clif.rs`
(8304 lines, emitting a Θ(1) helper population) is in **neither** list
(correction **4**). Add every missing production file as an explicit `0/0/0` row
**or** record the exclusion with its reason in-source. ⛔ **Silence is not an
answer here** — an absent row and a zero row read identically and only one of
them is a claim.

> ### ⛔⛔ `AC-2` AMENDED 2026-07-28 — THIRD POPULATION DEFECT IN ONE NODE. THE
> ### CENSUS IS **FAIL-OPEN**, AND THAT — NOT ITS NEEDLE LIST — IS THE DEFECT.
>
> **Raised by `runtime-implementer` at `evt_2we75javgbctw`, third of three.**
>
> | # | defect | scope of the hole |
> |---|---|---|
> | 1 | five rows are a subset of the 13-file population | one file unmeasured |
> | 2 | live sibling emitters (`boundary_value_clif.rs`, `native_int_clif.rs`) absent | more files unmeasured |
> | 3 | ⛔ **the needle set omits `.declare_data(` / `.define_data(`** | ⛔ **NO file measured for that entire kind of emission** |
>
> ⭐ **3 is a strictly worse shape than 1 and 2 and must not be filed beside
> them.** A missing *row* leaves one file unmeasured and the census visibly has
> a gap. A missing *needle class* leaves the census reading **complete across
> every row** while `n` data objects sit in the artifact. ⇒ **`D3`'s
> artifact-static seed material — every byte of it — would be invisible.**
>
> #### ⛔ THE RULING: the failure DIRECTION is the property to fix
>
> The census's needles are `FunctionBuilder::new(`, `.define_function(`,
> `.declare_function(`. ⛔ **Its default branch is *"needle not found ⇒ nothing
> emitted"*, so it fails OPEN for every emission spelling nobody enumerated.**
> ⇒ ⭐ **Each of the three fixes added something it was not looking for, which is
> why there was a third: a needle-list census can only ever be repaired one
> discovery behind the code.** ⛔ Adding two more needles does **not** make it
> sound, and this frame does not claim it does.
>
> ⚠ **Repeated defeats of one check mean its DEFAULT branch is wrong, not that
> the check needs a longer list.** Three defeats is the evidence.
>
> #### ✅ What `AC-2` now requires — two instruments, different jobs
>
> 1. ⭐ **PRIMARY — a BEHAVIOURAL count of what the compiled module actually
>    contains.** It counts what is **there** rather than searching for what a
>    reader expected, so an unanticipated emission spelling **cannot** hide in it.
>    ⇒ This is the evidence for the population property.
> 2. ✅ **RETAINED — the source-text census, as a TRIPWIRE.** ⛔ **Do NOT retire
>    or weaken it** — that would trade a real (if partial) guard for nothing, and
>    a defeat count never licenses removing a gate. ⛔ **And do not let it be
>    read as the population claim**; ⚠ its evasions survive every needle added.
> 3. ✅ **Add `.declare_data(` / `.define_data(` to the needle set.** Prediction
>    `P6`, committed **before** the code exists: `units.rs` = 1 `declare_data` /
>    1 `define_data`, every other row `0/0`. ⭐ The prediction-before-existence
>    ordering is itself the evidence, as it was for `units.rs`'s `1/1/1`.
>
> ⚠ **State in-source which instrument carries the claim.** ⛔ Two counts sitting
> side by side with no stated division of labour reads as corroboration, and it
> is not — one is fail-open by construction.

**AC-3 — the four D3 width invariants**, each independently falsifiable (old
`AC-3`). Each gets its own assertion and its own positive control; a single
composite assertion does not satisfy this.

⚠ **`B2R` landed the widths as *declarations*, so an assertion that reads them
back out of `AbiCarrier` is circular.** `AbiCarrier::width_bytes`
(`abi.rs:91`) and `align_bytes` (`:103`) are `const fn`s over a closed enum;
`AbiPlane::validate` already checks each slot carries its own carrier's
declarations. ⇒ **`AC-3`'s subject is the EMITTED code's agreement with those
declarations, not the declarations' internal consistency.** ★ Measured and
worth knowing before you write the pin: **every `AbiCarrier` variant is
currently 8/8**, so a width assertion that compares carriers to each other
passes on a mechanism that carries no layout information at all.

**AC-4 — the `origin -> expression` lookup count is stated.** Either it stays
**exactly 1** through `retained_body_occurrence`, or the new count is
re-baselined with justification in-source. `B2A-S`'s AC-4 pin must be left
truthful either way.

**AC-5 — EVERY call into `lower_expr` enumerated and dispositioned** (D5),
**under the amended five-class taxonomy**, at **whatever count the tokenized
derivation returns on your own base**, with the caller-dependent sites
dispositioned per `(site × reaching path)` and **the root call present**. The
enumeration is committed, not asserted in a handoff message.

> ⛔⛔ **THE COUNT IS NOT PART OF THIS AC, AND THIS HEADING USED TO CARRY ONE.**
> ⚠ It read *"all **59** calls"* and *"the root at `:188`"*. **The population is
> `61`** as of Runtime's committed census
> (`docs/program/rt-fnsplit-b2f-ac5-lower-expr-census.md`, tip `e08efe6f`), and
> `:188` was never the root at any base this AC was read on.
>
> ⭐ **Correction 2 at the top of this frame recorded `61` hours before a ring
> read `59` here — which is the whole lesson: an appended correction block does
> NOT replace operative text.** A reader who goes to the AC to learn the AC finds
> the AC, not the banner. ⇒ ⛔ **Fix the operative line; a banner is a supplement,
> never a substitute.** *(Found by runtime-implementer, `evt_11saxqd8mht90`,
> doing exactly what this AC's own item 2 told it to do.)*
>
> ⇒ **The derivation is the pin, and it is the ONLY pin:** tokenize via
> `identifier_occurrences`; ⛔ never `grep 'self.lower_expr('`, which is
> spelling-scoped and **misses the root**. ⭐ **Your own count wins over every
> number written in this file, including `61`.**

⛔ **Two withdrawals, both Steward defects — do not reinstate either:**
1. **The two-way migrated/not-a-body-emission classification is UNSOUND** for the
   14 caller-dependent sites (D5 amendment block).
2. **The `self.lower_expr(` population is SPELLING-SCOPED** and misses the root.
   **AC-5 must specify the census MECHANISM — tokenized, via
   `identifier_occurrences` — not just the number 59.** ★ *The number is the
   symptom; the mechanism is the fix:* a reader handed only "59" re-derives 58
   from the obvious grep and the root goes missing again.

**AC-6 — `lower_expr`'s recursive-descent inliner is gone** (D6), pinned so its
reintroduction reddens — ⛔ **and the pin is authored, and green, on the base
that still HAS the inliner.** **Control:** show the pin passing at a commit
before the removal, then passing after it. ⚠ A pin first authored in the removal
commit witnesses nothing: it has never observed the thing it forbids. See the
`D6` atomicity block above — the removal and the switch-over are one edit, so
this ordering is the only place the pin can be established.

**AC-7 — the FULL runtime suite, unfiltered:**
`scripts/ken-cargo test -p ken-runtime`. ⛔ **Workspace, `--locked`, and
conformance are CI's — never run them locally** (`agent/COORDINATION.md §12`).
⚠ Also run **`-p ken-cli`**: its integration tests live in a different shard,
and that is exactly how `B2A-C` went red after a green targeted run.

**AC-8 — `AC-G0`, the named denominator** (above). The growth verdict states its
population and justifies every exclusion, `native_int_clif` included.

⚠ **The measurement half is DONE — 6 definitions / 8 declarations, Θ(1) per
native module.** What `AC-8` still owes is (a) **cite** `LOCAL_HELPER_COUNT`
(`artifact/tests.rs:56`) rather than duplicating the 6, (b) **pin the 8
declarations**, which are genuinely unpinned, and (c) **record the
`emit_native_int_local_graph` signature as the program-independence
guarantee** — it takes no program-derived parameter, so the compiler already
forbids the growth mode. ⛔ **Add no test for (c).**

**AC-9 — the differential's baseline recipe is IN THE TREE.** Record the base
SHA, the probe function names, and the `git worktree add --detach <sha>` +
test invocation. ★ **The deep reason this is required:** the asserted property is
**equality against committed constants**, so a post-change re-capture produces
byte-identical values — **no observation distinguishes a genuine pre-change
baseline from a re-recorded one.** Demonstrate the binding; do not testify to
it.

**AC-10 — the narrow claim, stated separately.** `B2F` closes symptom-inventory
**entry 2**. Entries 1 and 3 closed with `B2A-S` and `B2A-C`. State what is
**not** claimed as its own sentence.

> ### ⭐⭐ AC-11 WAS AMENDED BY ARCHITECT RULING `evt_7ggqdk61pxzzf` (2026-07-25)
>
> **The ruling CONFIRMED the scope split and WIDENED the obligation.** The
> repair of `reject_imported_capture_edges` stays in `RT-FNSPLIT-B2O-CHECK`, and
> `B2F` must not patch it — but the switch-over may land ahead of that repair
> **only if `B2F` independently and fail-closed establishes representability for
> every transfer it makes.**
>
> ⛔ **The Steward's original enumeration was incomplete, in the direction that
> matters.** It named `Capture`, `Result`, and the four convention slots and
> **omitted `Parameter`** — but `push_slots` lays `Parameter / ValueWord`
> **first**, and a caller argument can carry the same hidden imported result as
> a capture or a return. **The source-valued transfer set is `Parameter` +
> `Capture` + `Result`.**
>
> ⚠ **And it was wrong in the other direction too:** `Control`, `Trap`, and
> `Store` are **protocol-produced convention values**. `result_carrier` is not
> their producer, and presenting it as their representability proof is a false
> discharge.
>
> ⛔ **I also told the Runtime ring at kickoff that this answer "can only relax
> `AC-11`, never widen it." That was wrong** — it widened it. The corrected
> contract below is the binding one.

> ## ⭐⭐ RE-ANCHORED ONTO LANDED `RT-FNSPLIT-B2V` — 2026-07-26, Steward
>
> **`B2V` is merged** (`a5c8ba73`, PR #1014, retros in). This frame was written
> before `B2V` existed and, until this block, **did not mention it once** — while
> the `RT-FNSPLIT-B2F` node records that the hard-stop-#10 ruling
> (`evt_28cnmxf6ncghn`) **re-scoped `AC-11`**. The re-scoping lived in the node
> and not in the frame the implementer opens. That is corrected here.
>
> ### ⛔ WHAT THE RE-SCOPING CHANGES — the answer to an aggregate INVERTED
>
> `AC-11` becomes **enforcement of `B2V` on every `Parameter` / `Capture` /
> `Result` transfer.** It is **not** rejection of common aggregates, and **not**
> inheritance from `B2R`'s `C4`. The transfer set is unchanged. **What changed is
> that the correct response to an aggregate is now *represent it*, not *refuse
> it*.** ⚠ Read clause 1 below with that inversion in force: *"must either reject
> before emission or follow an explicitly represented dependency-linking path"*
> now resolves toward the **represented** branch wherever `B2V` supplies a
> carrier, and the reject branch is the exception rather than the default.
>
> ### ✅ The hard-stop-#10 blocker is DISCHARGED — measured, not inherited
>
> #10 stopped this node because `Constructor` and `HostResult` transfers **had no
> executable word representation at all**, so a fail-closed guard would reject
> most source-valued transfers, incompatible with `D6` and `D7`.
>
> ⛔ **The `41` / `29` / `~33 of 41` figures that used to appear in this block are
> HISTORIC and are no longer the operand.** They were taken pre-`B2V` against a
> top-level-shape proxy. The current census is below; bind that one.
>
> Verified on landed `main`:
>
> | check | result |
> |---|---|
> | node classes for the blocking cases | `Constructor = 4`, `Record = 5`, `HostResult = 6` present in `BoundaryNodeClass` |
> | emitted-graph entry point | `boundary_value_clif::emit_boundary_value_local_graph` |
> | called from a **live** (non-test) site | ✅ `cranelift_backend/lowering/core.rs:87` |
>
> ⇒ *"There is no executable word representation"* is now **false**. The
> representation exists and is emitted.
>
> ### ⛔ ~~THE EXACT SEAM `B2F` MUST CLOSE~~ — CLOSED BY `C1`. See correction **1**.
>
> ⛔ **The line below does not exist at `6534e4a6`.** `core.rs:92` binds the
> handle **without** the underscore and it is consumed 9× at `:128–137` into
> `BoundaryCarrierRefs`, wired to `Lowering.boundary_carrier` at `:188`. ⇒ `C1`
> already made it live; `B2F` inherits a live carrier. ⚠ **This subtracts the
> reachability seam ONLY — `AC-11`'s producer-tracing walk is untouched.**
>
> ```rust
> // ⛔ STALE — as it read at bd24422b, core.rs:87
> let _boundary_value_abi = crate::boundary_value_clif::emit_boundary_value_local_graph(
> ```
>
> **The underscore binding discards the result.** The ABI is emitted into the
> module and **nothing consumes it** — which is precisely the *"INERT but
> EXECUTABLE"* deliverable `B2V` was scoped to produce. ⇒ **`B2F` is the node that
> makes it live**, by routing `Parameter`/`Capture`/`Result` transfers through that
> value rather than dropping it.
>
> ⚠ **Residual, and it is yours to measure, not mine to assert:** I verified that a
> carrier *exists* for the blocking classes and that the ABI is emitted from live
> code. ⛔ **I did NOT verify that each transfer is representable end-to-end** —
> that is the content of `AC-11` clause 1 and it needs the producer-tracing walk,
> not a type-level existence check. **Do not cite this block as that proof.**
>
> ### ⭐ THE CENSUS IS MEASURED — 47 events / 10 positions, NOT 41 (2026-07-26)
>
> The re-derivation this block asked for **has been done and it disagreed.** Bind
> these figures; the historic `41` / `29` are dead operands.
>
> | class | measured | historic |
> |---|---|---|
> | `Constructor` | **31** | 29 |
> | `Int` | **8** | — |
> | `HostResult` | **4** | 4 |
> | `CapabilityToken` | **2** | — |
> | `BorrowedNativeValue` | **2** | — |
> | **total** | **47 events / 10 distinct positions** | ~41 |
>
> **Provenance, labelled:** measured by `runtime-implementer` against bound code
> `bb3e58ea` and this frame's prior blob `65d3fa25`; relayed by `runtime-leader`.
> Evidence was doc-only and **unpushed** when this amendment was written, so ⛔ do
> not treat the census as Steward-re-derived — it is a ring measurement recorded
> on a fetchable ref so terminal QA can bind an operand instead of a proxy. The
> discriminator that makes it better than the old figure: it is censused at the
> point where `call_env == args ++ captures`, i.e. the **actual transfer
> boundary**, not a top-level-shape proxy.
>
> Also reported with it: every observed transfer lands on an exhaustive
> `Represented*` `boundary_disposition` arm — **zero** `FailClosedForbidden`, zero
> `ProtocolOnly` ⇒ a fail-closed guard rejects **0 of 47**, against #10's ~33 of
> 41. That is why #10 is spent.
>
> ⛔ **THIS AMENDMENT DOES NOT NARROW `AC-11`.** A census of what the current
> corpus *happens to* transfer is not a proof about what `B2F` *may* emit.
> `AC-11`'s producer-tracing walk, its represented-aggregate requirement, and its
> fail-closed branch all remain in force at full strength. "0 of 47 unrepresentable
> today" is **not** a licence to omit the guard: see the corpus lesson that a
> negative check passes for any reason, and #10's own history — the same
> population was measured once already and the figure moved.
>
> ⚠ And if **your** measurement disagrees with 47/31, yours wins and you say so —
> the same standing rule that produced this correction.

> ## ✅ `S4` DISCHARGED CLAUSE 3 FOR TWO SHAPES — ⛔ AND `AC-11` IS STILL OPEN
>
> ⛔ **Do not total the partial populations below into a discharge.**
>
> **Steward, frame owner, 2026-07-28.** Runtime's `S4` result
> (`evt_7me5y92n8jhqv`, tip `1e905036`), under the Architect's ruling
> (`evt_1y43t24pnv9hz`) and acceptance (`evt_5gd19nrew6hat`).
>
> ### Two records that are mine and are binding
>
> 1. ⛔ **Clause 3 is NOT amended.** The wording below stands exactly as
>    transcribed. ⚠ A reader of the `S4` exchange must not infer an amendment was
>    pending, proposed, or granted — **route (a) was taken instead**, because the
>    late `Unsupported` lives in `lower_expr`'s `ImportedDeclarationRef` arm,
>    which is the authority **`D6` retires**. It cannot discharge a property of
>    the surviving boundary.
> 2. ⛔ **`AC-11` is NOT discharged.** `S4` closed the *mechanism* question for
>    the two **named** holes. Three partial results now exist and ⛔ **they must
>    join the SAME proof; they do not sum.**
>
> ### ✅ What `S4` proved — the differential, not the enabled column
>
> | fixture | walk enabled | only the walk gated off |
> |---|---|---|
> | `Hole A` — `If { true, imported, imported }` capture | `Some(0)` | `Some(2)` — **reds** |
> | `Hole B` — `LexicalClosure { captures: [], body: imported }` | `Some(0)` | `Some(2)` — **reds**, re-measured separately (rule 2 below) |
> | both intra-module positive controls | accepted | accepted |
>
> The three-valued epoch — `None` = never reached the emission seam (⛔ **not** a
> zero), `Some(0)` = reached it and refused **before** any unit was declared,
> `Some(n>0)` = units already declared — is stamped in `core.rs` immediately
> **before** the validator. ⛔ Stamping it inside `declare_unit_bundle` makes
> `Some(0)` unreachable by construction: observing the epoch would require
> declaring the very unit whose *absence* is the measurement.
>
> ⚠ **The superseded reading, so nobody re-derives it.** `holeA = 1, holeB = 1`
> was **stale recorder state** left by a successful sentinel compiled first and
> never reset by an early refusal — consistent with *both* outcomes. That
> `1`-valued assertion is **retired**, not kept: it pinned recorder state, so the
> event it claimed to announce could not have reddened it. Its replacement is a
> durable invariant,
> `an_unrepresentable_transfer_is_refused_before_any_unit_is_declared`,
> which ⭐ **`D6`/`S7`'s removal of `lower_expr`'s late arm must leave GREEN** —
> the reason to assert it *before* the deletion rather than after.
>
> ### ⛔ THE RESIDUAL, AS A PARTITION WITH A DISCRIMINATOR
>
> Clause 1's population is **every** source-valued transfer. To place one, ask
> **which derivation reaches its producer** — mechanical, and it partitions the
> whole population instead of listing the shapes anyone has thought of:
>
> | derivation reaching the producer | status |
> |---|---|
> | `child_occurrence` — positional child of the occurrence | ✅ within the walk's reach; proven for the two named shapes only |
> | `case_body_occurrence` — `Match` / `ComputationalMatch` case bodies | ⛔ **not traced by the walk, and covered by no control.** Positional layout in `plane.child_origins` is unestablished |
>
> And by slot class:
>
> | slot class | status |
> |---|---|
> | `Capture`, `Result` | ✅ clause 1 discharged for the two named binder-free import shapes — ⛔ **not for the shape class** |
> | `Parameter` | ⛔ **population is EMPTY until `S6`'s call emission** — ⚠ **not `S5`**, see the correction below. Vacuous is not passing |
>
> ⇒ ⛔ **`AC-11` closes only when the `Parameter` population is nonempty and
> non-vacuous, the `case_body_occurrence` path is either traced or rejected, and
> both join the same proof as the two named holes.**
>
> ### ⛔⛔ CORRECTED AT `S5` — THE `Parameter` POPULATION ARRIVES AT `S6`, NOT `S5`
>
> **Runtime measured this against exact `9fe97ea4` and my ordering claim was
> wrong.** `evt_3sfmpegm4kgxx` (implementer) and `evt_3sbrsh7g19prz` (leader),
> independently. ⭐ **Their measurement wins.**
>
> `D4` derives, projects and resolves `FuncRef` **edges**; it emits **no call
> instruction**. A unit body loads its result slot and returns, and body emission
> does not descend until `lower_expr`'s consumers switch over. ⇒ `D4` carries
> **rejection** authority, not emission authority, and **no `Parameter` transfer
> exists yet to populate.** That population is created by **`S6`'s
> switch-over**.
>
> ⚠ **Why this correction matters more than the fact it fixes:** the earlier text
> said the population is empty *"until `S5`"*, which reads as **`S5` will supply
> it** — ⛔ a stale *"what remains"*, the shape that sends a ring to build
> something and find nothing there. It would also have let `S6` inherit an
> `AC-11` clause 1 that looked one step closer to closable than it is.
>
> ### ⛔⛔ `D4` FINDING — WHICH UNIT AN EDGE RESOLVES TO IS UNPINNED (`M8`)
>
> **Runtime's `M8`: resolve every edge to the CALLER's function instead of the
> callee's ⇒ ⛔ the ENTIRE suite stayed GREEN** (498 + 26 + 14, zero failures).
> The `FuncRef` is declared in the caller's `Function` and never *called*, so a
> wrong target is a reference nobody follows.
>
> ⭐ **This is a finding against `D4`, not a missing control** — and ⛔ **it is
> NOT covered by the edge-population pin.**
> `the_resolved_call_edge_population_moves_with_the_program` pins the edge
> **COUNT** and is blind to the edge's **DESTINATION**. ⇒ ⛔ **`S6` must not read
> the resolved edge set as covered.** The identity-alias defect `UnitBundle`'s own
> doc comment warns against is undetectable by every test in the crate today.
>
> ⚠ `M7` (derivation yields no edges) reds **exactly 1 test of 498** ⇒ the
> population control is `D4`'s **sole** defender — the same single-defender shape
> as `M5`/`D3`.
>
> ### ⛔ RULED — DO NOT WIDEN THE PLANNER SURFACE TO MEASURE EDGE EXACTNESS
>
> Runtime correctly declined to decide this alone: exactness of the emitter's
> edge population against the planner's `StaticBody` set is **argument, not
> measurement**, because `SemanticOwner` and the edge list are planner-private by
> design, so no control in `lowering` can count the planner's edges
> independently.
>
> ⛔ **The widening is already forbidden by this frame, so this is not an open
> fork.** `D1`'s correction row rules **one narrow lowering-reachable projection,
> ⛔ not wholesale type promotion, ⛔ no second derivation** — and the pin
> `the_owner_classification_has_a_closed_production_naming_inventory` **already
> reddened** when `S5`'s first draft named the owner classification in a third
> production file. ⇒ ⭐ **That pin firing was the answer arriving early.**
> ⚠ An exact-count oracle over a private set is also the structural-oracle shape
> the operator's test policy deprioritizes: controls assert **behaviour**.
>
> ⭐ **AND THE TWO GAPS ARE ONE GAP.** `M8`'s unpinned destination and the
> unmeasurable population-exactness both close with **one behavioural control at
> `S6`: a program whose answer depends on which unit actually ran.** A wrong
> destination gives a wrong answer; a **missing** edge means a call is never
> emitted, which gives a wrong answer too. ⇒ ⛔ **Do not carry these as two
> residuals** — they are one `S6` deliverable, and it needs **no** surface
> widening.
>
> ### ⭐ Two instrument rules `S5`–`S8` inherit
>
> 1. ⛔ **`Some(0)` is also what a stamp firing beside a dead counter reports.**
>    The epoch therefore carries its own positive control: a **successful**
>    compile in its own stamped attempt must report `Some(n>0)`. Without that row
>    both rejection rows are satisfiable by a counter that never moves.
> 2. ⛔ **A mutation that reddens the FIRST row of a multi-row control leaves the
>    later rows UNMEASURED.** `S4`'s first gated run reddened on `Hole A` and
>    panicked, so `Hole B` was never reached and had to be re-measured under the
>    same mutation with `Hole A`'s row neutralized. ⇒ **Assertion order
>    short-circuits exactly the evidence a differential collects.** Every
>    remaining multi-fixture mutation in this node is measured **per row**.

**AC-11 — every boundary transfer emitted by B2F is representable, established
by B2F and not inherited from B2R `C4`.** *(Architect text, transcribed
verbatim — do not paraphrase it.)*

> 1. For every source-valued transfer — each `Parameter` argument, each
>    `Capture`, and each `Result` — derive the actual value-flow producer that
>    reaches the slot and prove that producer has an admitted carrier. Checking
>    only the immediate occurrence's top-level `RuntimeExprShape` does not
>    discharge this obligation. A direct imported reference and a binder-free
>    wrapped import such as `If { true, imported, imported }` must either reject
>    before emission or follow an explicitly represented dependency-linking
>    path; the corresponding intra-module values must remain accepted.
> 2. For the protocol slots `Control`, `Trap`, and `Store`, prove the emitter
>    writes exactly the fixed carrier declared by the ABI. Do not claim these
>    are supplied by `result_carrier`; they are protocol-produced, not
>    source-expression results.
> 3. The check/proof executes before the atomic authority switch-over can emit
>    or call a unit. No path may treat `AbiPlane::validate`, `C4`, or descriptor
>    existence as a substitute for this per-transfer proof.

⛔ **Front-end unreachability is NOT available to you as a premise.** Neither the
Adversary nor the Steward established it. `B2F` may use such an invariant **only
if it is grounded structurally over every accepted input**, with the wrapped
`Capture`/`Parameter`/`Result` cases as discriminators. **Otherwise it must
reject those flows.**

> ### ⛔⛔ WHY THIS IS AN AC AND NOT AN INHERITED GUARANTEE — the P1 hazard
>
> **Adversary report on the landed `B2R` merge (`c986d0a3`), and I re-read the
> code at `bd24422b` before writing this AC.** `abi.rs:500-503` states that
> `C4` *"excludes the position where an imported value would have to cross a
> frame boundary and be given a carrier."* **The implementation does not
> establish that.**
>
> `reject_imported_capture_edges` (`abi.rs:514`) iterates a lexical closure's
> **direct capture children** and calls `result_carrier(seed.source)` on each —
> which answers *"is this capture expression's own top-level shape
> `ImportedDeclarationRef`?"*, **not** *"can an imported value reach this frame
> slot?"* Two consequences follow from the code shape, and the Adversary
> measured both as plans that **planned green**:
>
> | | |
> |---|---|
> | **Hole A** | any wrapper defeats it — `If { Bool(true), imported, imported }` is **binder-free**, so no de Bruijn reading makes its result anything but the imported value, and it receives a full `Capture / ValueWord / OwnedByFrame` slot |
> | **Hole B** | needs no wrapper at all — `LexicalClosure { captures: [], body: ImportedDeclarationRef }`; the function iterates **capture children only**, so the unit's own **result** slot is never carrier-checked |
>
> ⚠ **Grounding, stated precisely (`pin-a-property` §4).** **I verified the code
> shape myself** — the capture-children iteration, the `result_carrier` call on
> each child's own `SemanticSourceKind`, and that no path carrier-checks the
> body. **I did NOT rebuild the Adversary's fixtures**, so "planned green, 2
> descriptors, 10 slots" is **their measurement, relayed**. ⛔ The ring
> re-measures before acting; this frame is not their corroboration.
>
> ⭐ **The shape is what makes it worth an AC.** `abi.rs:494-503` records that
> the first implementation rejected *every* occurrence with an unrepresentable
> result carrier, that this was strictly stronger than `C4`, and that a
> pre-existing property test caught it. The repair moved from *"any occurrence
> anywhere"* to *"the capture child's own node"* — **and skipped the correct
> middle: the set of occurrences whose value can reach a boundary slot.**
> Corrected past the target, on the same axis, and documented with more care
> than the original error was — **which is exactly why it reads as settled.**
>
> ### Steward disposition — and what it does NOT do to this node's scope
>
> 1. **`B2F` must not treat `C4` as a tight exclusion.** `AC-11` is discharged
>    by establishing representability **at the slots this node emits**, which is
>    a property of the emission boundary and therefore genuinely this node's.
> 2. **The repair of `C4` itself rides `RT-FNSPLIT-B2O-CHECK`, widened to
>    `abi.rs`** — together with the Adversary's `P2` (`AbiPlane::validate:922`
>    asserts `descriptors.len() == functions.len()`, after which the `:934`
>    orphan check can never be `None`, so the "both directions are asserted"
>    note at `:931-933` describes one direction and a restatement). **Both are
>    the same advertised-vs-enforced defect that node already owns.**
>    ⛔ **`B2F` does not absorb it.** This is an `L` node on an atomic boundary;
>    adding a checking-layer repair to it is how an `L` becomes unlandable.
> 3. ⚠ **Not claimed:** that either hole is reachable from a real Ken program.
>    That depends on front-end constraints nobody has traced. **The claim is
>    bounded to the layer measured** — at the plan layer these are buildable
>    plans that `C4` says it excludes and does not.
>
> ⚠ **`B2R` is not being re-opened and this is not a merge-blocking finding.**
> Nothing emits today, so nothing is wrong on `main`. It becomes live **the
> moment this node emits**, which is the reason it is written into this frame
> rather than filed and forgotten.

**AC-12 — the declared ownership modes are OBEYED by emitted code, with a
positive control.** `B2R` validated that each slot carries **its own carrier's**
declarations; it verified nothing about behaviour, because nothing ran. State
per `AbiOwnership` mode what the emitted code does, and give **at least one
control that reddens if the emission ignores the declaration.**

⛔ **An assertion that reads the mode back out of `AbiCarrier::ownership`
(`abi.rs:122`) discharges nothing** — it re-measures a `const fn` over a closed
enum. ★ *`pin-a-property`:* the needle must not be supplied by the thing under
test.

**AC-13 — the carrier producer is TOTAL, and totality is proved by the COMPILER,
not by a case list** (`D9`). **Control, and it has two halves that do not
substitute for each other:**

1. ⛔ **Structural.** The producer dispatches over the **closed** variant sum
   with **no `_` arm** — the same discipline `SemanticOwner` (`semantic_ir.rs:62`)
   already carries. ⭐ **Then the compiler is the oracle:** a variant added later
   cannot be silently unhandled, and no test has to remember to exist. ⛔ Five
   hand-written cases discharge nothing — they are satisfied by exactly the five
   classes you thought of, which is the population that was already known.
2. ⛔ **Behavioural, per class.** For each of the five, a fixture whose **answer**
   depends on the value surviving the transfer. ⚠ A test that produces a carrier
   and asserts on its tag re-measures the constructor.

⛔ **The `#[cfg(test)]` trap, and this frame has already been bitten by it once.**
Correction row 5 above records `D2` pinned to `AbiPlane::shape`/`shapes` — ⛔
**both `#[cfg(test)]`**, so the deliverable was **unbuildable, not merely
mis-anchored**, and no line-number audit would ever have found it. ⇒ ⛔ **A
producer arm reachable only from a test configuration does not discharge `AC-13`
for that class.** ⚠ Verify in **both** configurations — the `cfg(test)` asymmetry
clause above cuts both ways here too.

**AC-14 — the process-ambient pair crosses ONLY through declared slots**
(Architect ruling `evt_69aedr4j844xd`, §2 — ⛔ do not paraphrase the mechanism).
`BorrowedNativeValue` and `CapabilityToken` are **Ken source-valued bindings**:
parameters at root ingress, captures in a retained body that closes over them.
The root wrapper loads them from host activation context and produces their `B2V`
words into the root unit's **declared** process-input/capability slots;
descendants transfer only what their callee descriptor declares; each callee
builds its environment **solely** from those slots, in descriptor order.

⛔ **No caller-env append. ⛔ No callee-side reload of offsets `0`/`16` as a
substitute for a missing slot. ⛔ No new ambient-capture provenance or implicit
tail.**

**Control — the Architect's discriminator, and it is two-sided:** a separately
emitted process body whose answer uses **both** bindings, where **(a)** deleting
either declared slot, or substituting a callee-side host-context reload, **must
red**, and **(b)** an otherwise identical body that does **not** close over a
binding **must not acquire its slot**. ⚠ Without **(b)** the control is passed by
an emitter that hands every unit every slot.

⛔ **The nine traced failures are NOT the class.** They are the part that was
traced. Every genuinely free variable reaching past a unit's slots must likewise
be **declared or fail closed** — ⚠ if the plan does not name those slots, that is
the defect `S6` exposed: repair closure conversion, or reject before emission.
⭐ **The fixed host-context parameter stays structurally separate from the
program-derived frame schema** — it is permitted as uniform, non-program-derived
runtime service context, and ⛔ that permission does **not** extend to rebuilding
the semantic environment from it.

---

## ⭐ Pin discipline — this chain has spent 9 hard-stops on exactly this

**Load `agent/playbooks/tools/pin-a-property.md` before writing any assertion.**
⚠ It has grown since this frame was first written: **§2a** (a predicted
population must include registration-driven fan-out) and the **§6a witness-axis**
subsection both came out of `B2R`'s retro and both bind this node.
It exists because of this WP chain. The four failures it encodes, all committed
here:

1. **A needle scoped to LAYOUT, not to the property.**
   `line.contains(".source_occurrence(")` is a claim about *formatting*; the
   token split across lines and the pin passed with two lookups present.
   ⇒ Tokenize: strip comments, split on non-identifier characters, whole-token
   match, count the **identifier**.
2. **Enumerating what you FORBID.** Three `body:` spellings, four container
   spellings — any form the author had not imagined passed green.
   ⇒ **Pin the ALLOWED inventory** and redden on anything else, the way
   `the_backend_production_surface_inventory_is_closed` (`control.rs:3715`)
   derives its population from the `mod` declarations themselves.
3. **Bounding a closure on an incidental privacy.** *Field privacy bounds
   nothing; item visibility bounds callers.*
4. **A negative check that passes for ANY reason.** Every negative assertion
   needs a **positive control** proving it can fail.

⇒ **Per pin: attempt one compile-preserving evasion, and write
`MEASURED / NOT CLAIMED / THE GAP` as its own sentence.** The pin's **name** is
part of its claim — a name asserting more than the mechanism sees is the defect,
not a cosmetic issue.

---

## Hard-stop protocol — ✅ #9, #10 AND #11 ALL RULED · ⛔ NOTHING HOLDS THIS NODE

> ⛔ **CORRECTED 2026-07-28.** The paragraph below said `#10` was OPEN with the
> Architect and that the next pull was `#12`. **Both are false.** `#10` was ruled
> `evt_28cnmxf6ncghn` (it inserted `B2V`); `#11` was ruled `evt_7ay6s5s79awz8` /
> `dec_45aa2gngjc79z` (it retired `B2E` and produced `C1`); every prerequisite
> those rulings named has merged, `C1` last, at PR #1156. **Count of record =
> 11. Next research pull = `#15`.** ⭐ The operative anchor is the
> **"ARMED §5a RESEARCH-CONSULT TRIGGER"** line in
> `docs/program/issues/RT-NATIVE-FNSPLIT.md` — read it at the point of a stop.
>
> ⚠ **The jointly-unsatisfiable report was real and it was ANSWERED, not
> waived.** `D1`+`D2`+`D6`+`D7` were unsatisfiable because no executable
> runtime-value representation existed. `B2V` and then `C1` built one. ⇒ This
> node now consumes an edge that already executes.

~~**Count of record: 10.**~~ ⛔ ~~**`#10` was raised 2026-07-25
(`evt_71d2jg83z2yt4`, leader escalation `evt_r7797bd7bzk3`) and is with the
Architect**~~ — the ring reported `D1`+`D2`+`D6`+`D7` jointly unsatisfiable
inside this frame's boundary, on an executable runtime-value representation gap.
Evidence `49e24b59` is pushed to `origin`; `crates/` is byte-identical to
`1e09a30a`. ~~**`#10` is not a research-pull stop** — the next is `#12`.~~

**`#9`** — raised 2026-07-25 (`evt_197xpdavdyrn0`); see the
**discharged-hold** block at the top of this file. ✅ **Its research pull is
CONSUMED: dispatched `evt_63wjmry61vd89`, before the Architect ruled**, as the
armed trigger required. **`B2O` and `B2R` both closed with no hard-stop, so the
count did not move** — a clean WP never advances it.

⚠ **The next armed pull is `#12`.** Raise a hard-stop the moment a deliverable is
unsatisfiable **inside this frame's own boundary** — that is what `#7` and `#9`
both were, and raising it early is what produced a correct re-slice instead of a
third cosmetic scan. ⭐ **`#9` set the standard: raised with nothing committed,
claiming a missing prerequisite rather than impossibility, and shipping
falsifiers instead of a tally.**

⭐ **The discriminator, from the `B2A-S` implementer's retro, verbatim:**

> "N defeats of one detector ⇒ stop repairing the detector" is **not** "the
> property is unenforceable." **The discriminator is whether the failures share
> a cause you can name.**

`AC-4`'s three defeats shared a **line boundary** — repairable, and the property
stood. `AC-5`'s shared nothing repairable, so it genuinely needed a frame
correction. **Applying one verdict to both would have retired an enforceable AC
or papered over an unenforceable one.** Route the **authority boundary** before
spending another test round (`agent/playbooks/build/leader.md`).

## Carried in from the adversary's hunts — two, on different merges

⚠ **Read the SHA on each.** These are two separate reports and only the second
concerns code this node builds on.

### From the hunt on `c986d0a3` (`B2R`) — **`P1` is live and it is `AC-11`**

`C4`'s imported-edge exclusion is narrower than `abi.rs` says it is, and `B2F`
would inherit it as its calling convention. **The full disposition, the two
holes, my own grounding boundary, and the routing of the repair to
`RT-FNSPLIT-B2O-CHECK` are written into `AC-11` above — that is the durable
home, not this section.** ⛔ Do not read this heading as the whole treatment.

`P2` from that same hunt (`AbiPlane::validate`'s one-direction-plus-restatement)
is **routed to `RT-FNSPLIT-B2O-CHECK`**, widened to `abi.rs`. It does not ride
this node.

### From the hunt on `145fe915` (`B2A-S`)

**P2 — a CANDIDATE, for the Architect to rule at framing review. Not adopted.**
`B2A-S`'s AC-5 leaves residual **arm 1** — "no independently maintained
entry-keyed source-term store inside the two planner files" — **review-enforced,
not mechanically detected.** The adversary is right that review-enforcement
decays and that arm 1 is the arm that matters.

⛔ **But their proposed form — "no `BTreeMap`/`HashMap`/`BTreeSet`/`HashSet`
keyed by `StaticNodeId`" — is a forbidden-spelling census, the exact mechanism
class this chain just retired** (failure 2 above). `Vec<Option<_>>` indexed by
ordinal, a boxed slice, a newtype, or any third-party map evades it and the
census stays green. And it "passes as written" **because there are none today** —
a negative check passing for any reason, with no positive control.

⇒ **If arm 1 is mechanized, it takes the allowed-inventory form with a positive
control.** Their sound observation stands and should be reused: an
*identifier-count* form cannot work here, because `.entry` is legitimately read
a dozen times as an edge endpoint. **Architect rules whether this rides `B2F` at
all** — it is optional scope on an already-L boundary, and "leave it
review-enforced and say so" is an acceptable answer.

## Rebase and handoff discipline

- **On every rebase of a branch under review, publish the old→new SHA mapping
  plus a diff isolating what the rebase itself changed.** A rewrite silently
  invalidates every SHA-anchored finding in the thread. **Prove the rebase
  preserved content; do not testify that it did.**
  (`agent/playbooks/build/implementer.md`)
- **Verify the frame is FETCHABLE AT YOUR BASE**, not merely written. `B2A-S`
  lost a round to a frame that existed only on `steward/work` while the ring's
  base held a stale draft **reusing the same identifiers for different
  deliverables.**
- **Never edit source while a background shard run is in flight** — it trips
  `px4b`'s freshness guard, and that guard has caught the same seat in two
  consecutive units of this chain.
- ⛔ **Never `git stash`** — `refs/stash` is shared across ~70 worktrees.
