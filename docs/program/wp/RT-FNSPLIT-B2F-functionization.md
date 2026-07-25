# `RT-FNSPLIT-B2F` — functionization and authority switch-over

**Owner:** Team Runtime · **Size:** L · **Depends on:** `RT-FNSPLIT-B2A-S`
(merged, `origin/main` = `145fe915`) · **Closes:** `RT-NATIVE-FNSPLIT`
symptom-inventory **entry 2** — the last open entry.

**Every anchor below was re-derived on `origin/main` = `0aa9e53f`** (the tree
that carries `B2A-C` + `B2A-S`). ⛔ Do not trust a line number from the retired
`RT-FNSPLIT-B2A` frame or from the `B2F` issue file's draft prose — `lower_expr`
alone has moved `:3847 → :4255 → :4333` across three re-frames.

---

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

**Measured on `0aa9e53f`:** `core.rs` holds **58** production `self.lower_expr(`
call sites, spanning `:310` to `:6743`. There is **one** definition, at `:4333`.
`core.rs` is production in its entirety — `mod tests;` at `:11-12` puts the
tests in a sibling directory, so there is no `#[cfg(test)]` region to partition
out of this file.

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
un-gated at `lib.rs:23` — and holds **5** `FunctionBuilder::new` sites
(`:190, :294, :534, :695, :790`) with its own `declare`/`begin`/`finish`
helpers, emitting exact-`Int` support "into every native module."

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

**D5 — switch-over of EVERY live consumer.** All **58** production
`self.lower_expr(` call sites (`core.rs`, `:310`–`:6743`) are accounted for.
"Accounted for" means each is either migrated to the function-call path or
explicitly classified as not-a-body-emission with the reason recorded. ⛔ A
count that does not reach 58 is an incomplete switch-over, not a partial
success — enumerate, do not sample.

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

**AC-5 — all 58 `self.lower_expr(` call sites enumerated and dispositioned**
(D5). The enumeration is committed, not asserted in a handoff message.

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

## Hard-stop protocol — ⚠⚠ #9 IS ARMED AND IT FIRES A RESEARCH PULL

**Count of record: 8.** ⛔ **The next hard-stop on this chain is #9, and #9
dispatches the research agent BEFORE the Architect rules, not after.** Raise a
hard-stop the moment a deliverable is unsatisfiable **inside this frame's own
boundary** — that is what `#7` was, and raising it early is what produced the
correct re-slice instead of a third cosmetic scan.

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
