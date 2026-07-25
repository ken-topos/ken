# `RT-FNSPLIT-B2F` — functionization and authority switch-over

**Owner:** Team Runtime · **Size:** L · **Depends on:** `RT-FNSPLIT-B2A-S`
(merged, `origin/main` = `145fe915`) · **Closes:** `RT-NATIVE-FNSPLIT`
symptom-inventory **entry 2** — the last open entry.

**Every anchor below was re-derived on `origin/main` = `0aa9e53f`** (the tree
that carries `B2A-C` + `B2A-S`). ⛔ Do not trust a line number from the retired
`RT-FNSPLIT-B2A` frame or from the `B2F` issue file's draft prose — `lower_expr`
alone has moved `:3847 → :4255 → :4333` across three re-frames.

---

## ⛔⛔ HELD AT HARD-STOP #9 — 2026-07-25, `evt_197xpdavdyrn0`. COUNT = 9

**`D1`/`D2`/`D4`/`D6` are jointly unsatisfiable inside this frame's boundary.**
Raised by `runtime-implementer` **before writing any code** — tree clean at
`3891b7aa`, nothing committed, nothing to unwind on a re-slice. ⭐ *That is the
cheapest possible form of a frame-boundary defect; compare `#7`, which cost a
re-slice after `D1–D3` had landed.*

**The obstruction, measured and line-anchored (not inferred from a failed
build):**

`Lowered` (`lowering/mod.rs:415-507`) is a **compile-time specialization
lattice, not a value representation.** Only scalar variants hold `ir::Value`;
`String`, `Bytes`, `Constructor { args: Vec<Lowered> }`, `Record { fields }`, and
`Closure { captures: Vec<Lowered>, … }` carry **host Rust data with no emitted
representation at all**. A capture is an arbitrary `lower_expr` result
(`core.rs:4783`), so it may be any aggregate, including a nested closure. The
emitted signature is `(pointer) -> i64` (`core.rs:44-46`).
`CaptureSlot { ordinal: u32 }` carries no type or width; `PredeclaredFunction`
(`semantic_ir.rs:449`) carries **no signature**. Retained-body lowering is
**fused with its call site** (`core.rs:626-643`): the body's environment is
`captures ++ producer_env` — the *call site's* whole env — and the strategy is
chosen by inspecting the body's **syntax** and the argument's **shape**.

⇒ **One closed function per static origin requires configuration-independent
compilation of that body.** Specialization erases aggregates, so two call sites
applying the same origin in different configurations legitimately require
different code. Compiling once per origin therefore requires a **uniform runtime
representation** — layout, ownership, lifetime for constructors, records,
strings, bytes, closure environments — plus a call ABI wider than
`(ptr) -> i64`. ⛔ **Constructing that is not among `D1`–`D8`, and is unowned.**

**Robust across both readings of shape (a):** aggregates cross the ABI at runtime
⇒ needs the object model outright; or specialization still erases them ⇒ one
function per origin serves only **one** configuration, giving per-*specialization
-instance* units, which is neither "one per origin" nor `D8`'s Θ(n).

⭐ **Why atomicity converts "hard" into "unsatisfiable as framed":** the
increment that *is* buildable — functionize the origins whose parameters are all
scalars, keep specialization for the rest — is **precisely what `AC-1` and `D6`
forbid**, because it leaves two live emission strategies. **The
sound-subset-with-a-conservative-guard idiom is unavailable here by
construction.** ⚠ This is **not** a defect in atomicity (which exists to prevent
two live authorities — a real hazard) and **not** a defect in the increment. It
is a **genuine tension between two correct requirements**, which is what makes
it a ruling rather than an adjudication.

**⛔ NOT CLAIMED, and the distinction is the whole point:** shape (a) is **not**
asserted wrong, `b077eb7a` is **not** to be revisited, and the goal is **not**
unreachable. The claim is narrower: **the prerequisite is missing and unowned.**
The construction was **not attempted** — so per `pin-a-property` §7 (a strong
negative must be demonstrated, not tallied) the stop ships **falsifiers**, any
one of which dissolves it:

1. A uniform runtime encoding for `Lowered`'s aggregate variants somewhere
   unexamined ⇒ Reading 1 is buildable.
2. A corpus measurement showing **every reachable retained body has only scalar
   params/captures** ⇒ one-function-per-origin is configuration-independent for
   the whole population and `D6` is satisfiable as written.
3. A ruling that shape (a) admits **per-specialization units** ⇒ a different
   scaling claim, and `D8` changes shape.

### The two options — Architect authority, not the Steward's

- **(i) A prerequisite unit** constructing the native value representation +
  calling convention, with `B2F`-proper rebased on it.
- **(ii) Bounded coexistence** — a mechanically pinned boundary between
  functionized and specialized origins. ⛔ **This requires `AC-1` and `D6`
  AMENDED**, since as written they forbid exactly that.

### ⛔ Sequencing: RESEARCH FIRST, RULING SECOND

**Research dispatched `evt_63wjmry61vd89` BEFORE routing to the Architect** —
that is what the armed `#9` trigger means on this chain, and it is why the count
was armed at 9 specifically. The advisory is an **input** to the ruling, not a
review of it; asking for the ruling first is how `#6` got re-litigated. The
deciding question is prior-art: **is a permanently-bounded two-strategy backend
known-sound with a pin that cannot silently widen, or a known trap?**

### Steward rulings issued at the stop — these did NOT wait for the Architect

- ✅ **The `D5`/`D6` narrow reading is CORRECT.** Remove the **inlining across
  the retained-body boundary**; keep ordinary traversal within one function's own
  body. The operative words are *"the recursive **whole-configuration
  body-emission** authority"* — the target is whole-configuration re-emission,
  not recursion as a code shape. ⭐ **The settling test:** converting traversal to
  a worklist would remove "recursion" while doing **nothing** for entry 2, and a
  reading under which the deliverable is satisfiable without touching the defect
  is the wrong reading. **Population that matters: 7 of the 58 sites consume a
  retained body — `core.rs:327, 605, 620, 764, 4817, 4829, 4954`; the rest derive
  from `child_occurrence`, i.e. ordinary sub-expression traversal.**
- ✅ **Two ruling-independent deliverables proceed while held** (facts about the
  current tree, surviving any re-slice): `AC-G0`'s denominator — measure and pin
  `native_int_clif`'s per-native-module constant — and the **full 58-site
  disposition table**. ⛔ Nothing else: no representation design, no scaffold, no
  speculative construction.

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

**Measured on `0aa9e53f`:** `core.rs` holds **59** production calls into
`lower_expr`, spanning **`:188`** to `:6743`. There is **one** definition, at
`:4333`.

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

---

## The landed surface you are building on — measured, with anchors

### The production Cranelift surface today

| what | where (`crates/ken-runtime/src/cranelift_backend/`) | count |
|---|---|---|
| root `FunctionBuilder::new` | `lowering/core.rs:152` | **1** |
| root `define_function` | `lowering/core.rs:225` | **1** |
| `declare_function` — entry point | `lowering/core.rs:53` | 1 |
| `declare_function` — **imported** host dispatch (`ken_host_dispatch_v1`) | `lowering/core.rs:84` | 1 |

⇒ **2 declarations, of which one is an import, not a definition.**

### Entry 1's dispatcher — do not break it

| what | where |
|---|---|
| sole `origin -> expression` consumer | `lowering/core.rs:4176` `retained_body_occurrence` |
| the plan-side accessor | `planning/static_transition.rs:1009` `source_occurrence`, `pub(in crate::cranelift_backend)` |
| the single write site | `planning/static_transition.rs:452` `record_source_occurrence` |
| the retained-closure carrier | `lowering/mod.rs` — `Lowered::Closure` / `DeclarationClosure` hold `body: StaticOriginId`, **no term** |

⛔ **`B2A-S`'s AC-4 pins the `origin -> expression` lookup count at EXACTLY
ONE.** If `B2F` adds a second consumer, that pin reddens **correctly** — it is
not a false positive. Either route the new consumer through
`retained_body_occurrence`, or re-baseline AC-4 **explicitly in the frame
amendment** with the new count stated and justified. Do not quietly bump it.

### The plane already has what the bundle needs — evidence of FIT, not of existence

`planning/static_transition/semantic_ir.rs` already carries
`PredeclaredFunction` (`:449`), `PredeclaredFunctionId` (`:38`),
`functions: Vec<PredeclaredFunction>` (`:483`), and keys them by planned node —
`PredeclaredFunctionId(planned_node.0)` at `:536`, cross-checked at `:850-853`.

⚠ **These records are evidence that the function bundle is the *smaller*
construction. They are NOT proof that functions already exist.** Nothing here
declares or defines a Cranelift function. Do not read the plane's
`PredeclaredFunction` rows as an existing emitted-unit population.

---

## ⛔⛔ THE PIN YOU WILL BREAK FIRST — re-baseline it deliberately

`lowering/core/tests/control.rs:3336`
`correspondence_adds_no_emitted_unit_to_the_production_census` asserts an
**exact** census over five production files:

```
lowering/core.rs                              builders 1  definitions 1  declarations 2
lowering/mod.rs                               0  0  0
planning.rs                                   0  0  0
planning/static_transition.rs                 0  0  0
planning/static_transition/semantic_ir.rs     0  0  0
```

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
>   6 `Linkage::Local` (`ken_native_int_{resolve,intern,binop,compare,narrow,export}_local`, `:83-88`)
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
`BACKEND_PRODUCTION_SOURCES` (`control.rs:3580`, 12 files).**

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

## Deliverables

**D1 — the target code-unit population.** One closed Cranelift function per
static planned function/origin. Forward-declare the whole bundle
(`Module::declare_function` for every signature/ID first), then define each
body. Derive the population from the plane's existing static origins — do not
invent a parallel numbering.

**D2 — the fixed explicit activation frame (the ABI).** One explicit frame
layout through which dynamic environment/control/store state crosses into a
target function. It is **fixed** and **explicit**: code identity is static,
and every dynamic value travels through the frame, never through
capture-by-construction. Document the layout where the layout lives, not in a
comment far from it.

**D3 — persistent-store transport.** The store crosses the ABI. State the
ownership and lifetime contract at the boundary.

**D4 — static dispatch / call edges.** Call sites reference target functions by
their **static** identity. No indirect dispatch on a dynamic property, and no
runtime lookup that re-derives which code to run from a value.

**D5 — switch-over of EVERY live consumer.** All **59** production calls into
`lower_expr` (`core.rs`, **`:188`**–`:6743`) are accounted for, **including the
root call at `:188`.** ⛔ A count that does not reach 59 is an incomplete
switch-over, not a partial success — enumerate, do not sample. ⛔ **Derive the
population with a TOKENIZED census (`identifier_occurrences`,
`control.rs:3529`), never `grep 'self.lower_expr('`** — that spelling misses the
root, which is the one site that must become the call into the root target
function.

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
from 58 sites. Removal means the recursion is gone, not flag-disabled, not
`#[allow(dead_code)]`, not retained "for the differential."

**D7 — behaviour-equivalence evidence.** The five-category differential suite
(old `AC-1` of the retired frame lands here). Categories must be independently
falsifiable, and the differential must run against a **pre-change baseline whose
recipe is in the tree** — see the recipe requirement below.

**D8 — the growth verdict.** The Θ(n)-units / bounded-per-function claim, stated
in the Architect's exact shape, with `AC-G0`'s denominator named.

---

## Acceptance criteria

**AC-1 — one production authority at the landed point.** No feature flag,
runtime branch, optional callback, function pointer, or alternate entry can
reach a second body-emission path. Pinned structurally, verified in **both**
`cfg(test)` configurations.

**AC-2 — the emitted-unit census is re-baselined to a PREDICTED number.**
`correspondence_adds_no_emitted_unit_to_the_production_census`
(`control.rs:3336`) carries the new counts, the prediction, and the reason. The
pin still reddens on an unplanned declaration or definition.

**AC-3 — the four D3 width invariants**, each independently falsifiable (old
`AC-3`). Each gets its own assertion and its own positive control; a single
composite assertion does not satisfy this.

**AC-4 — the `origin -> expression` lookup count is stated.** Either it stays
**exactly 1** through `retained_body_occurrence`, or the new count is
re-baselined with justification in-source. `B2A-S`'s AC-4 pin must be left
truthful either way.

**AC-5 — all 59 calls into `lower_expr` enumerated and dispositioned** (D5),
**under the amended five-class taxonomy**, with the 14 caller-dependent sites
dispositioned per `(site × reaching path)` and the root at `:188` present. The
enumeration is committed, not asserted in a handoff message.

⛔ **Two withdrawals, both Steward defects — do not reinstate either:**
1. **The two-way migrated/not-a-body-emission classification is UNSOUND** for the
   14 caller-dependent sites (D5 amendment block).
2. **The `self.lower_expr(` population is SPELLING-SCOPED** and misses the root.
   **AC-5 must specify the census MECHANISM — tokenized, via
   `identifier_occurrences` — not just the number 59.** ★ *The number is the
   symptom; the mechanism is the fix:* a reader handed only "59" re-derives 58
   from the obvious grep and the root goes missing again.

**AC-6 — `lower_expr`'s recursive-descent inliner is gone** (D6), pinned so its
reintroduction reddens.

**AC-7 — the FULL runtime suite, unfiltered:**
`scripts/ken-cargo test -p ken-runtime`. ⛔ **Workspace, `--locked`, and
conformance are CI's — never run them locally** (`agent/COORDINATION.md §12`).
⚠ Also run **`-p ken-cli`**: its integration tests live in a different shard,
and that is exactly how `B2A-C` went red after a green targeted run.

**AC-8 — `AC-G0`, the named denominator** (above). The growth verdict states its
population and justifies every exclusion, `native_int_clif` included.

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

---

## ⭐ Pin discipline — this chain has spent 8 hard-stops on exactly this

**Load `agent/playbooks/tools/pin-a-property.md` before writing any assertion.**
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
   `the_backend_production_surface_inventory_is_closed` (`control.rs:3605`)
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

## Hard-stop protocol — ✅ #9 HAS FIRED; ITS RESEARCH PULL IS CONSUMED

**Count of record: 9** — `#9` raised 2026-07-25 (`evt_197xpdavdyrn0`), see the
**HELD** block at the top of this file. ✅ **Its research pull is CONSUMED:
dispatched `evt_63wjmry61vd89`, before the Architect ruled**, as the armed
trigger required.

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

## Carried in from the adversary's post-merge hunt on `145fe915`

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
