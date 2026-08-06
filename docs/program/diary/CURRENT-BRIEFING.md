# Current briefing (live — read this first on every Steward resume)

> ## ⛔ HOW TO READ THIS FILE, AND WHEN TO DISTRUST IT
>
> **`origin/main` outranks this file, always.** ⛔ If anything below tells you to
> do something `git fetch origin` shows as landed, **this file is stale and the
> repository is right.** Re-read fresh, in this order:
>
> 1. `git fetch origin && git rev-parse origin/main`
> 2. the LIVE block below — ⛔ **only** the LIVE block
> 3. the open tasks (⛔ do not re-derive priority from memory)
> 4. for what is HELD, DEFERRED, or WHOSE it is: **the node**
>    (`docs/program/issues/*.md`), its operative block — ⛔ never this file
>
> ⭐ **This file is a resume POINTER, not an archive. Git is the archive.** When a
> window closes its block is **deleted**, not demoted to a "superseded" section —
> ⛔ a superseded block left in the file gets read by someone, eventually.

> ### ⚠ REWRITTEN 2026-07-26 ~19:4xZ — 2866 lines → this. Read the bound.
>
> The prior content (~2700 lines of windows stacked back to 2026-07-21) is at blob
> **`c26ee67f29d42690f461d43fe15e21c2202a31df`** — `git show c26ee67f`. Nothing was
> lost; it was archived to git with this pointer.
>
> ⛔ **HONEST BOUND ON THE AUDIT: I did not read all 2866 lines.** I read every
> heading, the blocks claiming to be authoritative, and then **scanned** the
> remainder for sole-source markers, decision ids, held items, and preserved refs.
> ⇒ **That is a scan, not an exhaustive audit**, and its surface was my own idea of
> what "load-bearing" looks like. A reader who needs something from before
> 2026-07-26 should assume it is in `c26ee67f`, not that it was considered.
>
> ⭐ **What the scan found is why the rewrite was worth doing: two blocks that
> advertised themselves as authoritative were WRONG** (see *Corrections*), and a
> hand-maintained list of 6 preserved refs when origin held **26**.

## LIVE — 2026-08-06 ~01:0xZ · `D8l` CLOSED and QA-approved; `D8f` witness in flight

**Verify `origin/main` before trusting anything below.**
`RT-CONTSRC-PRODUCER-LOCAL` is `active` in thread **`thr_6m43v75yndhtj`**.

### The one thing to do next

**Wait for the `D8f` witness continuation**, in flight on exact `df8bd057`.
Thread **`thr_6m43v75yndhtj`**.

**`D8l` is CLOSED and QA-approved.** Repair `1f9a2020`, evidence `abe46dda`,
QA verdict `evt_7kx4dtax3gqrv` — QA ran its own compile-preserving mutation
(payload worker returns `0`, saw `Returned(Int(Small(0)))` against expected
`41`, restored byte-clean), so that is an independent check. The composed
witness compiles and **executes** end to end; the causal ledger closes with a
non-empty composed half.

⛔ **`D8f` is NOT discharged at `df8bd057` — the mechanism is UNWITNESSED, not
proved.** The occurrence gate was restored unchanged from the preserved patch
and re-measured at `751/2/1`, identical to `abe46dda`. But mutating it to
**always admit — exactly the pre-`D8f` behaviour — leaves the whole suite
GREEN.** ⇒ **No landed fixture separates gated from ungated.** Same state
`D8d` was in before `D8e`'s witness, and the omission/duplicate/transplant/
wrong-occurrence refusals cannot be demonstrated until the witness exists.

**Two witness attempts already measured and recorded at the gate:** an ordinary
call on the selected recursive argument inside the `px8tr` checked wrapper
refuses in the declaration's own lowering (*"a source-machine call's callee is
a specialized-only surface"*), because that case body is lowered both there and
in the specialization while the binder run's static workers exist only in the
second. The same shape with the IH as inner callee refuses in plan validation
on checked/inferred frame mixing. **Do not re-derive these.**

### The sequencing lesson from this stretch, since it will recur

⛔ **An Architect hold condition can have several parts, and a fast ring will
satisfy the first and move.** `evt_2ffwkpgnmr8xd` held `D8f`/`D8g` until the
evidence was *"committed, QA-checked, and reviewed"*; `D8f` was released 40
seconds after the commit post, with QA never having posted in the entire `D8`
chain. **I held ACCEPTANCE rather than reversing the release** — the work was
test-side on a preserved branch and merged nothing, so killing a live turn to
re-issue the same instruction would waste it. QA was engaged in parallel and
approved in four minutes. **Hold the gate the condition names, not the turn.**

**Completed this stretch:** `D8h` `a435d180`, `D8i` `abfd67ff`, `D8j`
`8d0d6fca`, `D8k` `372097de`+`aaef1772`, `D8e` DISCHARGED, `D8l1` answered,
`D8l2` `1f9a2020`+`abe46dda`. **`D8g`, `D6b` closeout, `D6c`, candidate, `D6`
closure and downstream remain held.**

⛔ **`D5a-1` and `D5a-2` are SPENT LABELS — their text is refuted.** So is my
published *"new member at ordinal 2, not 3"*. So is the whole
continuation-input projection: `ContinuationInput(0)` **already is** the outer
success binder with exact case-body provenance, and a second input is the
banned fabricated availability. So is the `ContinuationOrdinaryEnvelopeRole` +
ABI `Parameter(ValueWord)` route — the selected recursive argument is a
**closure capsule** with no lawful `ValueWord` form.

⚠ **Each of those three would have passed a `Var(2)`-only positive.** At
ordinal 2, `Var(2)` still resolves to a success payload while `Var(1)` is
silently wrong. **Assert the whole run.**

**The corrected law:** `[IHs, ALL constructor arguments in source order, outer
frame]`. The functionized construction **replaces the selected recursive
argument with its IH** — that is the defect. Frame text is fixed; source
comments carry the same false law and are the implementer's to correct.

**`D6a`'s mechanism is ACCEPTED at exact `625b7860`** (Architect review
`evt_3hx267n11sm9k`): `730/7/1` to **`736/2/1`**, five rows repaired including
the governed `Var(2)`, run `[IH RawWorker, SelectedRecursiveArgument RawWorker,
ContinuationInput(BufferAllocate Ok)]`, `D4a` boundary reached. **Held for
fidelity only** — a comment-only child of `625b7860` is authorized, no
executable change.

⛔ **I wrote a FALSE ROUTE LAW into the frame and it is corrected.** The
conditional law: `SelectedRecursiveArgument` **always** carries `RawWorker`;
`InductionHypothesis` carries `GeneratedContext` **iff** the planner issued and
this unit resolved one, else it lawfully carries `RawWorker`. So the governed
fixture's **Raw/Raw is lawful** and **degenerate on route** — `D6b`'s
discriminating control must use the **mixed landed-object witness**
(`GeneratedContext`/`RawWorker`) and assert the exact pair directly.

⚠ Also retracted: *"a tail-appended member silently passes."* It does not — the
typed worker binding refuses in value position and five rows redden. **Keep the
whole-run control; the silent-pass rationale was wrong.**

⛔ **`D6b` IS MIS-SIZED and FROZEN** (Architect `evt_6azsr4xrch1he`). Exact
`d86be55d` is preserved as **accepted partial progress** — the route consumer
and both function-local tables are sound. `D6c`, QA, candidate, `D6` closure and
downstream are frozen with it.

**Another false claim of mine, corrected:** the two lowering paths do **not**
build the same segments at the recursive field. Composed/source-machine
(`core.rs:2929`, `:3467`, `:4649`) enters **every** constructor field as
`Value`; the functionized specialization **alone** substitutes the
`StaticWorker` (`units.rs:1696-1764`). ⇒ **A production representation/consumer
gap, not a missing fixture** — the released bounded witness repair could never
have discharged it, because the only source shape that exercises the raw table
is refused earlier. That is why its wrong-table mutation stays green.

**Next: WAIT for the Architect's causal-projection ruling.** Routed at
`evt_59dyjmzy6hqkw`. Runtime holds exact `89e36ec1`; `D8f`/`D8g`, `D6b`
closeout, `D6c`, QA, candidate, `D6` closure and downstream all held. ⛔ **The
mechanism fork is the Architect's, not mine** — (a) a claim seat reachable from
the composed path, (b) an exemption for composed producers from result-edge
projection, or (c) something unnamed. **I size and cut it once ruled.**

⛔⛔ **`D8e` WHOLE-NODE HARD STOP at exact `89e36ec1`.** The witness is **lawful
and the positive route is PROVED** — `745/2/1`, exactly three new rows, no new
reds, both profiles clean, test-only.

⭐⭐ **MY STATED WHOLE-NODE CONDITION DID NOT FIRE — a different one did.** I
wrote *"if the four cannot lawfully coincide."* **They DID coincide.** The
blocker is **what the conjunction NECESSARILY CREATES**: interning the
specialization projects a causal call onto the same emitting unit
(`continuation_result_edges_owned_by` is keyed on emission owner and admits
every projected call), so **the edge is the same act as satisfying fact 4.** Its
only two discharges are both impossible here — the sole
`claim_and_call_continuation` site sits on the ordinary producer branch the
composed path returns **before**, and a unit result that **is** the planned
producer constructor is the very thing the composed path **eliminates in
place**. ⇒ Refuses at the **`D5a` detached-result seat**; the outer raw-body
closure refusal my law named is **independently NOT reached**.

⛔ **The fabrication that would have gone GREEN, found and refused:** a
**different occurrence of the same constructor** as the unit result reaches the
second discharge — **the identity check is per-symbol** — while emitting a
specialization call for the **wrong occurrence**.

⚠ **`D8d`'s sentinel is RE-SCOPED, not deleted** — its "do not coincide
anywhere" wording is now false; ⛔ do not restore it. The surviving narrower fact
(neither of `D8d`'s two populations crosses over) is what keeps the witness a
**construction rather than an inheritance**. The refusal assertion is a
**labelled sentinel: it reds the moment a lawful discharge exists.**

⭐ **STANDING, third occurrence: satisfying a required fact can CREATE an
undischargeable obligation** (`D7a`→`D7b`, `D7a2`'s retention, now fact 4). Each
surfaced only when something **downstream** used it. ⇒ **A checkpoint owes a
statement of what its fact OBLIGES, not only what it provides.**

`D8a`/`D8b`/`D8d` discharged; `D8e` consumer + witness proved but **not
discharged**.

⭐ **`D8d` DISCHARGED at exact `c2e8314f`** (`742/2/1`). The target-derived
`StaticWorkerBinding` is installed at the selected recursive source-order
position; **not** a `Value(Closure)`, so value-position use stays fail-closed at
`value_at`. Owner-collision guard deleted; `emission_owner`'s selector role
re-measured live afterwards.

⚠⚠ **THE BINDING IS CORRECT AND MEASURABLY NEVER INSTALLED** — the two
preconditions coincide nowhere in the suite. Pinned as a **sentinel**, measured
with **two counters** because *"unreadable by design"* and *"never built"* are
indistinguishable from outside. **That sentinel is `D8e`'s inheritance and its
real work.**

⛔ **Third occurrence on this node** — `D6b`'s raw table, `D7a2`'s retention, now
`D8d`'s binding, all correct-and-unreached. **On this node the mechanism is the
cheap half and the WITNESS is the deliverable.** ⛔ **If `D8e` cannot build its
witness through the ordinary production planner/lowering path, that is a
WHOLE-NODE finding** (the composed path cannot exercise this substrate at all) —
hard-stop to me, do **not** hand-construct a plan to make the witness exist.

⛔ **`D8c` IS RETIRED — folded into `D8e`** (Architect `evt_nwgvvr4vaf7y`,
**outcome (c) at the CHECKPOINT boundary, not the whole-node boundary; the node
remains well-sized**). Order is now `D8d` (install the target-derived binding) →
`D8e` (consume at the callee seat **and** close the no-unit-boundary law) →
`D8f` → `D8g` → `D6b` closeout → `D6c`. ⛔ The label `D8c` is **not reused**.

**My error:** `D8c`'s consumption statement is an **integration property, not a
predecessor mechanism** — I read the mechanism family's three properties as
three checkpoints when the third is a property of the **composition** of the
other two. `D8d` owns binding, `D8e` owns consumption, and only together can
they establish it.

⛔ **Build neither returned form.** Resolving the target in `source_call_state`
from a threaded selector is a **second target-selection authority** and is not
self-authenticating; installing a target-derived `Lowered::Closure` as `Value`
is a **second callable representation** that lets the template enter value
positions. ⛔ **No temporary `Value(Closure)` bridge and no consumer-side lookup
that `D8d`/`D8e` later replace** — a scaffold the next checkpoint deletes is not
a checkpoint.

⭐ **`D8b` DISCHARGED at exact `e4b4c26c`** — `ComposedCallTarget` minted,
withdrawn retention machinery removed, no fork returned. The raw-body-versus-
route-resolved-callee question **dissolved** (the view already carries route
eligibility, so the target is a representation, not a route decision) and stays
dissolved despite citing the now-retired `D8c`.

⭐ **The `D8c` hard-stop clause worked as written** — the implementer returned
two forms rather than choosing, built nothing, and left the tree clean at
`e4b4c26c`.

⭐ **`D8a` DISCHARGED at exact `e02ef413`** (`742/2/1`, both failures the
`d86be55d` baseline pair). **The fork resolved to the STRUCTURAL branch and was
measured before it was chosen** — a test-only hook removed reason one (disjoint
walks) exactly and nothing else, and still yielded no second owner, because the
`D5a` availability law refuses first on both plans. Disarmed run is the positive
control.

⚠ **The owner buys NO discrimination on any current population** — correctness
insurance and an earlier transplant catch, not a key that separates anything.
**`D8g` must not attempt an owner-based-separation positive**; there is no
population to demonstrate it on.

**Steward disposition added to `D8b`:** delete the owner-collision refusal in
`composed_worker_view` — measured unreachable and unexercised, and the ruling
authorized encoding the invariant **instead** of a discriminator, not alongside
a dead guard. ⛔ **Not the `D6b` unwitnessed-residual case:** `D6b`'s table was
unreachable because the mechanism was **incomplete**; this is unreachable
because the planner **proves the population impossible**. Delete the collision
guard, **not** the field — the owner's selector role is separately live.

**Why the `D8` series exists.** The `D7a`-`D7e` cut is **SPENT** — Architect
ruling `evt_3dcafs581921e` returned **outcome (c), mis-sized**, on preserved
exact `f3427dae` (`D7a`) and `9f21ff0e` (`D7a2`). Both are **non-candidate
evidence**. The governing cut is `D8a`-`D8g`, and the `D8` series is allocated
so that **label order IS execution order** — the `D7` letters were allocation
order and misled twice.

Order: `D8a` (owner-qualified selector) → `D8b` (composed-call target, planner)
→ `D8c` (the consumption seam) → `D8d` (one environment authority) → `D8e`
(source-machine callee consumer) → `D8f` (checked-marker occupancy) → `D8g`
(non-vacuous closeout) → `D6b` closeout → `D6c`.

**Finding 1 — the selector is FIVE fields, owner-qualified.** Four was
necessary and not sufficient. The planner **deliberately interns distinct
generated contexts for the same raw worker under different continuation
identities**, so two `emission_owner`s can name **different exact callees**
while sharing all four source coordinates. Accepting only when the complete
views agree makes **agreement, not causal identity, the selector**. My analogy
to `continuation_call_binding_for` was the false step — that lookup may fail
closed on duplicate tokens; the composed view **projects an owner-specific
answer**.

**Finding 2 — `9f21ff0e` FALSIFIED `D7a2`'s own premise.** Retaining the
required raw body defines a standalone `Function` whose result is a
`Constructor` containing a raw `Closure`; all 25 newly-red rows stop at the
permanent unit-result closure boundary (`741/2` unarmed, `716/27` armed, one
predicate). ⇒ *"Make the raw body declared-and-defined"* **reopens the exact
boundary the generated-context design exists to avoid.**

**Withdrawn — all three are MY text:** `D7a2`'s raw-body executable-set
equality; `D7e`'s *"prove the raw target is both declared and defined"*; the
four-field selector as final identity.

**The lawful mechanism is a planner-issued COMPOSED-CALL TARGET**, distinct from
both the standalone `RawWorker` `Function` and the IH `GeneratedContext`:
preserve the selected argument's raw argument/capture semantics, consume its
result **in the exact source-machine continuation**, with the closure-valued
result **never crossing a unit boundary**; owner-qualified, and
occurrence-qualified where more than one source call can consume the binding.

⛔ Banned: closure carrier, ABI/runtime lane, boundary exemption, flipping the
test-only retention predicate, substituting the IH `GeneratedContext` (different
semantics and operands), accepting the raw table as a permanently unwitnessed
residual, any carrier callable/helper route.

⚠ **The reconciliation gate now REFUSES IN PRODUCTION by design** at `9f21ff0e`.
Anything that starts calling it before the route is settled reads that as a
**regression** rather than as the checkpoint's own finding.

⭐ **The concurrent review was worth requesting.** `D7a2` was already built on
`f3427dae` when I routed it; the review found the base incomplete, which is
exactly the coupling that mattered — `D7a2`'s exact-set equality is keyed on
`D7a`'s selector, so both sides of that equality would have been derived from
the same wrong key.

### ⛔⛔ I PUBLISHED A FALSE CONCLUSION AND IT IS WITHDRAWN

**This file said "the five reds are the runtime correctly rejecting a malformed
fixture — no Ken defect." THAT IS FALSE.** Its premise — that nothing at the
match level binds `AllocatedBuffer` — counted only the inner
`ComputationalMatch`'s binders and **omitted the enclosing ordinary `Match`**
over `BufferAllocate`, whose `Result::Ok` case has `binders: 1` and **is** the
governed buffer.

The nearest-first environment is
`[InductionHypothesis, ScopeArgument, AllocatedBuffer, ...outer]`, so **`Var(2)`
is well founded** — local under three binders. Confirmed independently by the
production walk `required_surrounding_environment_prefix`, by both lowering
constructions, and by the fixture's own committed structural assertion.

⇒ ⛔ **The five reds are a REAL DEFECT: a functionized-unit environment
transport failure.** The emitted unit keeps the inner IH and constructor
argument and **loses the outer ordinary-`Match` success binder**. ⛔ **Do not
touch the fixture.**

⭐ **The shape to learn from:** three successive dispositions — mine, the
Architect's, and the ring's — each inherited an unprobed premise from the one
before. The anchor was never measured; then the binder count was taken from the
inner match alone. **Each artifact read as grounded because its predecessor
did.**

### `D5` — recut as REAL transport work, split `D5a`/`D5b`

**No new node** (same branch, one candidate — the reasoning that closed
[[RT-UNIT-CLOSURE-CONVERT]]). ⚠ **But `D5` is no longer small.**

**`D5a` planner:** derive the outer ordinary-case binding from the enclosing
`Match` **success-binder provenance**; represent it as an **explicit typed
unit-environment member**, never an implicit caller tail; retain the closed
order **`IHs ++ arguments ++ outer frame`**; **fail closed before emission** on
omission, redirection, wrong provenance, wrong order, or fabricated
availability.

**`D5b` lowering:** consume **that exact planned member**; **positive** that
`Var(2)` is the real `BufferAllocate` success payload **and the `D4a` boundary
is still reached**; **negative** that dropping or redirecting **only** that
member hits the **pre-emission** refusal.

⛔ **Banned:** no capture edit, no source `Var` rewrite, no padding, no
shifting, no synthesized capture, no caller-tail recovery.

⭐ **Still standing:** the `LexicalClosure.captures` **totality** law; the
`Scope`-constructor closure is **innocent** (correct empty capture run); the
`StaticWorker` is **non-causal**. ⭐ **The per-depth compounding worry is
DEAD** — each level builds its own outer match and success binder.

⭐ **[[RT-UNIT-CLOSURE-CONVERT]]'s GAP STATEMENT was right all along** — the
unit environment does not carry what its body needs. Only its **mechanism**
(capture slots) was wrong. It stays `closed`; capture conversion stays banned.

### `D5` — recut TWICE; both earlier edits are WITHDRAWN

`D5` is `RT-CONTSRC-PRODUCER-LOCAL`'s **last deliverable** and clears the
candidate gate. Its authorized edit is now the fixture's **scope tracking**.

| withdrawn edit | why |
|---|---|
| correct `test_objects.rs:176`/`:220` capture literals | ⛔ **wrong anchor.** Those bodies are genuinely closed — zero `Var` nodes — so `captures: Vec::new()` is right and editing them fabricates captures |
| correct the `:12038` capture literal | ⛔ **also innocent.** `closure_scope` is an empty `BinderScope` binding only the closure's own `"buffer"` param, so its `Var(1)` is that param; ambient demand is **zero** |

⛔ **The wrong anchor rode through THREE artifacts** — `D1c` named it from a
grep rather than a probe, then the Architect's ruling and my own `D5` release
each inherited it as measured. ⭐ **A `file:line` in a code fence reads as
measured and is only a claim.**

⭐ **Standing:** `LexicalClosure.captures` must be **total** for ambient lexical
demand (Architect `evt_5g7kaec1xzaf6`) — a **totality** ruling, not a
minimality one; a conservative run may be larger, never shorter. And the
`StaticWorker` at de Bruijn 0 is **non-causal** — it sits innermost, the missing
binding is outermost, so ⛔ `Var` shifting stays banned.

⚠ **Unmeasured, and it belongs to the repair's evidence — NOT another
measurement round:** the construction recurs at every depth and a nested call
builds indices from a fresh `BinderScope::default()` under enclosing binders.
Whether that compounds per level is open.

⛔ **Whatever the ruling, the corrected fixture must still be a `D4a` witness,
proved by a discriminator** — a fixture that stops reaching the boundary is a
deletion wearing a fix's clothes.

### Superseded — the contract question, ANSWERED

Routed at `evt_7p0jwvxm9kwmw`, answered `evt_5g7kaec1xzaf6`:

> **Must `LexicalClosure.captures` cover every free index its `body` reaches?**
> **YES** — total for ambient lexical demand, no lawful undeclared caller tail.

⛔ **INVENTORY IS CLOSED. There will be no `D1d`.** Three passes (`D1`, `D1b`,
`D1c`) each corrected its predecessor's premise; I said `D1c` terminates it and
it does. **Do not authorize a fourth measurement** — the one still owed is
conditional on the ruling (below).

⛔ **`RT-UNIT-CLOSURE-CONVERT`'s PREMISE IS RETIRED.** It was framed to
"activate function-unit closure conversion" for a substrate gap. **There is no
measured production instance of that gap.** My frame defect, the fifth of this
class on this campaign.

### `D1` returned a HARD STOP at `bc754c03`, and I did NOT size from it

Doc-only child of `b3ba2820`; source tree unchanged, all probes reverted.
Record at `docs/program/wp/RT-UNIT-CLOSURE-CONVERT-D1.md` (on the branch, not
`main`). **Accepted.**

- **`CaptureSlot { ordinal: u32 }` CONFIRMED.** Identity exists **only** in
  `captures: Vec<RuntimeExpr>` (`ir.rs:443`), which is consumed for its
  **length** (`semantic_ir.rs:444`) and discarded at the semantic-plane
  boundary. Nothing downstream — `CaptureLayout`, `AbiSlot`, `AbiFrameHeader` —
  has a field that could name a captured value.
- **All five `RT-FNSPLIT-B2R` elements are PRESENT, none a stub**, and live for
  the other **127** closures (90 `LexicalClosure` with one capture, 30 with two,
  7 `Closure` with one). ⭐ **The honest reading is not "the mechanism is
  missing" but "all five are driven off one input that is empty for this
  population."**
- **The five failing units are lexical closure bodies declaring `captures: 0`
  whose bodies reference a free variable.** ⛔ **ABSENT, not inert** — no
  declared-but-unbound slot exists. Binding it would **fabricate a capture**,
  one of the four banned repairs.

⛔ **I REFUSED TO SIZE, and this is the reasoning to keep.** Read alone the
record sizes this as a large node — a new free-variable analysis **plus** a new
identity-bearing representation. **But the record itself says it has not
established why those closures arrive with an empty capture list while 127
others do not**, and those two statements are in tension: *something* populates
`captures` for the 127. **That producer, and the basis on which it decides
membership, is what sizes this node — and it is unmeasured.** Sizing on the
first reading would be creating work on speculation.

**`D1b` is FOLDED, not a new node** (relax, fold, then cut). It asks what
writes `LexicalClosure { captures }` / `Closure { captures }` and on what basis,
then why that list is empty for `defining_origin` 88 and 14. **Three readings
are live and it must distinguish them:** a producer with a gap for this
population (repair is upstream, may be small); a producer copying an
already-known set whose input lacks it (the question moves one plane further
out); or nothing computing captures anywhere (only then is a new analysis the
honest answer).

⚠ **Do not assume the `Var` index is right and the capture list wrong.** The
record's own numbers put the converse in play: `index=2 env_len=2` and
`index=3 env_len=3` against a header of `{parameters: 1, captures: 0}` — **the
lowering environment is already longer than the declared slot run**, so
something other than declared captures contributes to it. If that reading holds,
this population never needed a capture at all.

⛔ **Banned in `D1b`:** any production edit, repair, or representation change —
**specifically, do not add an identity field to `CaptureSlot`.** If `D1b`
confirms a new representation or analysis pass is genuinely required, **that is
an Architect question and I route it**; it grows substrate and is not mine to
cut unilaterally.

### What `D1b` and `D1c` then established — READ BEFORE ACTING ON `D1` ABOVE

⛔ **`D1`'s account above is SUPERSEDED on its central claim.** It said "a free
variable with no declared capture slot", which **assumed the `Var` is valid and
the captures are wrong** — unmeasured. Both later passes are on the branch:
`D1b` `a8b66c5c`, `D1c` `e27d297a`, records at
`docs/program/wp/RT-UNIT-CLOSURE-CONVERT-D1{b,c}.md`.

**`D1b` — the membership basis is POSITIONAL DEPTH, not free variables.** All
three production writers are
`captures: (0..runtime_depth).map(RuntimeExpr::Var).collect()` and **nothing
inspects a body**. ⭐ **The declaration is therefore TOTAL by construction** —
which is the load-bearing fact for the ruling now pending.

⭐ **The `StaticWorker` at de Bruijn 0 is NOT the cause**, and this kills the
tempting repair: both failures are `index == env_len`, short by exactly one at
the **outermost** position, and removing the worker makes the shortfall
**worse**. That is why shifting `Var`s stays banned rather than merely
discouraged — the measurement makes it look plausible.

**`D1c` — the failing units NEVER REACH THAT PATH.** Zero records across the
entire `ken-runtime` lib suite **and** the `ken-elaborator` lib suite. The empty
list is a **literal written by a fixture author**:

```rust
// crates/ken-runtime/src/cranelift_backend/test_objects.rs:176, :220
RuntimeExpr::LexicalClosure { captures: Vec::new(), params: vec!["response"], body: … }
```

among **92** empty-capture construction sites. ⛔ `D1b`'s own hard stop naming
`ken-elaborator` as the required route was an **inference, not a measurement**,
and its author corrected it in place.

### ⛔ THE TRAP — do not clear the five reds by editing the fixtures

**Do not "correct" the five fixtures while the ruling is open.** They came from
`D4a`'s admitted population and exist to exercise something. Greening five reds
by editing the fixtures that produce them is the exact shape of weakening a
proof — and if the contract turns out to be the wider one, it erases the only
witnesses to a real defect. **The reds stay until the Architect rules.**

### What each ruling outcome means, decided in advance

| ruling | consequence |
|---|---|
| `captures` **must be total** | the fixtures are malformed; small correction; `RT-CONTSRC-PRODUCER-LOCAL`'s candidate unblocks cheaply; **I recut this node to that** and it stops sitting large on the critical path |
| a body **may** reach an undeclared enclosing binding | real capability gap; the end-to-end measurement below becomes owed; **a substrate node follows and that expansion is the ARCHITECT's to authorize, not mine to cut** |

⚠ **The measurement still owed, CONDITIONAL on the ruling:** whether
*elaborated* Ken programs ever produce a closure whose body outruns its declared
captures. Neither suite reaches those sites, so that behaviour is **unmeasured —
not shown correct.** It needs an end-to-end `ken-cli` corpus. ⛔ I did **not**
authorize it, because whether it is needed depends on the ruling.

⚠ **OPERATOR-FACING, and it is the campaign-sizing question they already
hold:** only on the second ruling is this node large and sitting ahead of all
seven `RecursiveDescent` retirement nodes, which funnel through
`RT-DECL-CLOSURE-PORT`. That is a critical-path fact, not a Runtime execution
problem. Do not decide it here.

⭐ **Second silent stale anchor this campaign:** the `B2R` table's
`CaptureSlot` at `semantic_ir.rs:438` now lands **six lines from the
`capture_slots` derivation at `:444`** — a reader following the stale table
would plausibly believe they were in the right place. All seven `B2R` anchors
moved again; the re-derived table is in the `D1` record.

⛔ **No merge is owed by me yet.** `RT-CONTSRC-PRODUCER-LOCAL` `D4b` is
**discharged** at exact `b3ba2820` (Architect `evt_gqph7jhjeybx`), and that is
its **last in-node checkpoint** — but the **node candidate stays held**, on
three counts the Architect named: the five `Var: no runtime binding` reds are
the `RT-UNIT-CLOSURE-CONVERT` gate; exact per-instance `V` accounting is still
a candidate `AC` that the coarser diagnostic census does **not** silently
replace; and CI/no-regression plus candidate review have not run.

### The sequencing ruling I made at ~16:1xZ, so it is not re-litigated

The Architect discharged `D4b` and referred the release boundary to me,
reporting that two published artifacts contradict. **They do not — the
contradiction was inside ONE file, six lines apart**, and it was mine.

`docs/program/issues/RT-UNIT-CLOSURE-CONVERT.md` said in one section that the
gate is *"prose, not a `depends_on` edge, because an edge both ways is a
cycle... both `active`, one branch"* — and then in the next section that the
node *"enters the frontier when `RT-CONTSRC-PRODUCER-LOCAL` merges, with no
Steward pass in between."*

⛔ **Read literally the second sentence is a DEADLOCK**: producer-local's
candidate cannot close until unit-convert lands, so waiting for its merge waits
forever. Both **frames** were consistent and correct all along; only the issue
node's status prose was false. It was my standing one-release-ahead boilerplate
applied to the one pair it does not govern.

⭐ **The corpus settled this itself.** The same file names the governing
precedent two paragraphs up — `RT-CONTSRC-PRODUCER-LOCAL` against
`RT-DECL-CLOSURE-PORT`, **both `active`, one branch** — and that pattern is
live right now: producer-local is `status: active` with an unmerged
`depends_on`. I did not need a new rule, only to stop contradicting the one
already there.

**Ruling: the same-branch atomic-set sequence stands.** The two nodes land in
one candidate and **both flip `merged` in one commit**. The `depends_on` edge
is **retained** — it states checkpoint order, which is true, not merge order,
which is not — and `status: active` is the lever that keeps the node off the
releasable frontier (`gen-progress.sh` computes that frontier as `ready` AND
every `depends_on` merged, so a stale `ready` advertises in-flight work as
available to release).

### `D3b` is checkpoint-APPROVED at exact `012a2c88`

A comment/record-only **fidelity child** of `d14eddd6`, tree
`35c72b8fa806b3761d39e2187c166b10a1ef966b`. The Architect verified the delta is
exactly three paths, **every changed Rust line a `//` or `///` comment, zero
non-comment Rust delta** — so QA's evidence on the parent still applies and was
deliberately not re-run.

**QA's evidence on `d14eddd6`:** `728 passed / 7 failed / 1 ignored`, both
`check -p ken-runtime` and `check --profile test` clean, and the seven reds
attributed exactly — 2 standing `D0`, 5 `D4a` at the unit-body `Var: no runtime
binding` boundary.

The child corrected three statements that still asserted the retired mechanism:
the alias index is **not** a post-shift index; `D3b` consumes **planner-issued
alias availability**, not source ABI position; and the record now names the
**nearest-exact-alias lexical index**. Surviving `post-shift` mentions only name
and explain the retired spelling, and the quoted old `source_abi_position`
sentence is explicitly marked previously-true-and-now-false.

⭐ **The frame addition was vindicated by measurement.** The implementer's
mutation of "first coordinate-containing member" reds **only control 3**;
controls 1 and 2 pass under it. `max` for `min` reds 1 and 2 but not 3. Neither
catches the other — which is exactly why the frame says control 3 is not a
variant of control 1.

⭐ **Implementation facts worth keeping:** `nearest_exact_alias` is **one total
rule shared by the planner that issues the claim and the consumer that
revalidates it**, so they cannot drift; `min` is a **fold over the whole
eligible set, not an early break** (an early break reads as first-match and a
later reordering of the scan would silently change the answer). Two-stage
`EntryFrame` is **two types**, so a half-stamped claim is unrepresentable rather
than merely unwritten.

### `D4b` — released 15:07Z, and what it owes

It owns **behavioural activation** of the generated-frame route plus the
post-admission census closeout:

- prove the exact contract partition at the current base: `interned = V`,
  `declined = R`, no extra route modality or special case;
- **re-run the census with program fingerprint identity.** `C`/`V` may move with
  fixtures; ⛔ **the three named `R` causes are the invariant**;
- **exercise the generated-frame consumer behaviourally, not only by
  construction controls** — this is what closes the standing **0/60** boundary
  (no lowered program consumes a generated frame identity today);
- preserve `D3b`'s separately validated views and every fail-closed boundary.

⛔ **The five unit-body `Var` reds are known and out of scope.** No unit-frame
edit, padding, `Var` shifting, caller-tail copying, or fabricated captures.
`RT-UNIT-CLOSURE-CONVERT` is `ready` but dependent. **If that boundary blocks
required `D4b` evidence, the answer is a hard stop naming the concrete missing
route — never a weakened proof.**

### The ruled law: nearest-exact-singleton-alias

The exact-once **lexical position** precondition was FALSE and is retired. It
conflated *does this position certainly hold `S`* with *is it the only position
that holds `S`*; `D3b` needs only the first. `join` deduplicates
`Closed([S])` records, so two positions each holding exactly `Closed([S])` are
**proved aliases of one semantic source** — the measured `let y = x` at indices
0 and 2. Select the **minimum de Bruijn index among eligible positions**, where
eligible means the held authority is exactly `Closed([S])` against the complete
`S` (coordinate, carrier, ownership, storage owner, affinity).

An `If` yielding `S` or `T` joins to `Closed([S, T])` and is not eligible; if
both branches yield `S` the join stays `Closed([S])`, which is the proof. **No
SSA-equality instrument is needed** and adding one would duplicate a fact the
planner authority already states.

**Four operative sites were replaced, not two** — the ruling named `D3b`'s
exact-once walk and the "two positions refuses" clause; the sweep also found
`D2b`'s duplicate-membership fail-closed arm and **both** definitions of the
`CurrentLexical` claim as carrying a "post-shift index", which is what an
implementer builds the claim from.

**The judgment call is SETTLED — CONFIRMED CORRECT, do not reopen.** I kept
`exactly once` for a frame's **ordered capture projection** and for predeclared
frame membership; the Architect verified the landed blob and ruled it right, with
a sharper reason than mine, now recorded in the frame: the lexical case is
licensed by `Closed([S])` from the forward semantic walk **plus de Bruijn
ordering**, and a frame projection has **neither** — it declares ordered ABI
slots. Two frame members carrying one full coordinate make the declared source
slot non-unique, and **selecting one would reintroduce the unkeyed first-member
rule at the ABI boundary.**

⭐ **Two additions beyond the literal ruling**, both load-bearing: the six alias
controls **with the note that control 3 is not a variant of control 1** (3
selects the outer position, 1 the inner — a suite with only 1 and 2 passes under
a positional shortcut too), and a **carve-out in the first-match ban**, which
would otherwise have forbidden the law just ruled. The discriminator is
**eligibility, not ordering**.

### Standing results from that turn — do NOT re-measure

- **Caller-frame multiplicity is NEGATIVE.** Direct owner = emission owner
  40/40; capture indexed frame = enclosing spec's emission owner 20/20 — and
  structurally, since `emission_owner` is a field of
  `ContinuationSpecializationKey`.
- **The capture consumer's source-frame defect is fixed.**
- ⛔ The earlier specialization census suggesting multiplicity was a
  **per-compile-id artifact**; its own author retired it. Not available as
  evidence.
- **Two-stage `EntryFrame` stays in `D3b`**, not moved to `D4b`. The 0/60
  generated-owner consumptions explains the evidence boundary and does not
  authorize a half-stamped accepted plan.
- ⛔ **`41d2b1e5` is not an object on the branch** — quoted from memory in two
  posts, corrected twice.

### The gating multiplicity question is ANSWERED: NO multiplicity

The measurement the previous window flagged as decisive came back **negative**,
so no per-causal-edge claims are needed and no hard stop is owed on it.

- direct emission: `defining_owner == unit.emission_owner()` — **40/40**
- capture: indexed frame = the enclosing spec's emission owner — **20/20**

Structural, not just observed: `emission_owner` is a **field of
`ContinuationSpecializationKey`**, so two emitting frames give two keys and two
interned specializations; a context interned on `(enclosing,
worker_body_origin)` determines the enclosing unit and hence its emission owner.
The source frame is a **function of the context's own key**, and the 60
observations are its positive control.

**The implementer's own first census was an artifact and it flagged it as one** —
keyed on `ContinuationSpecializationId` alone, which is per-compile, so it merged
different fixtures and reported one specialization consumed from three frames.
The collision-immune form asks the question **within one plan**. Do not carry the
old specialization census forward as evidence; the implementer explicitly retired
it.

**The same measurement found and fixed a live defect:** the frame whose
`defining_abi_operands` the capture consumer indexes was a **predeclared**
function in every observation, never the enclosing specialization.

### The NEW hard stop: duplicate full-coordinate lexical claims

Exact `456ec7e6`, `722 passed / 10 failed` — the 7 prior reds unchanged, 3 new
ones sharing one cause. **Exact-once `CurrentLexical` membership fails:** at a
predeclared direct-emission seat, `EntryAbi { owner 0, abi position 1,
Parameter }` occurs at lexical indices **0 and 2**.

This is lawful `let y = x` — a non-`Effect` `Let` pushes the bound expression's
own authority, so one root identity occupies two distinct bindings. An `If` join
may also carry the same coordinate with a **potentially different SSA value**.
No lowering fixture reaches duplicate membership; the three rows are
planner-only.

**Innermost selection makes all 3 pass with no other regression, and the
implementer declined to take it** — it is textually first-match and unmeasured
for value equivalence. That refusal is the frame working; do not read the
passing variant as an available shortcut.

Two-stage finalization remains **deferred**, not built.

**Lineage** (all of `fad24816` and later compile under `--profile test`; the
mid-migration red at `e70ae04c` is cleared):

```
456ec7e6  record the two-stage path's reachability
d81359ae  the deliverable record
8298a321  re-cut the rows the new law changed
fad24816  the capture consumer's source frame, measured
e70ae04c  (the WIP partial the turn resumed from)
```

**`41d2b1e5` is not an object on the branch.** It was quoted from memory in two
posts and corrected twice; ignore it wherever it appears.

### The earlier partial at `e70ae04c` — HISTORY, and its red is CLEARED

*(Superseded: the turn resumed from it and reached `456ec7e6`. `fad24816` and
later compile under `--profile test`. Kept because the description of what
`1f6fc5cf` lands is still the accurate account of the representation.)*

Lineage `f5e4fa9f` (base) → `1f6fc5cf` (production) → `e70ae04c` (WIP), all
contained by `wp/RT-DECL-CLOSURE-PORT-typed-units`. `e70ae04c` did not compile
under `--profile test` because the test migration was mid-flight — that was
where the implementer chose to stop on a working-budget limit, not a
regression, and it was never routed to QA.

**The stop was working budget, not a defect.** The implementer stopped short of
a restructure rather than strand a half-done one, and handed over an exact
state. Read it as the frame working.

**What `1f6fc5cf` lands and is believed correct:** availability as two
consumer-specific planner-issued claims over `CurrentLexical` and `EntryFrame`,
in a closed record keyed by consumer kind — no unkeyed vector, no first-match,
no generic immediate slot, no arm falling back to the other's. At a predeclared
direct-emission seat every input takes `CurrentLexical`, entry root and
producer-local alike. `GeneratedContextCapture` is subsumed. All four clauses
retired by name. Both cross-presentation cases fail closed.

The load-bearing feasibility fact: `continuation_owner_entry_sources` already
seeds the forward walk with `EntryAbi` coordinates, so this needed **no new
walk** — the existing search finds either root arm exactly-once-or-refuses.

One gain worth keeping: the injectivity law at the emission seat could
previously only be scoped to the producer-local domain, because comparing an
ABI-frame slot against a lexical index *was* the conflation. Every claim at one
seat now names a position in the same environment, so the law widens to the
whole emission.

### The ruling that landed MID-TURN and supersedes part of it

Architect `evt_7e04y1zmvrnps` (13:22Z) arrived after `1f6fc5cf` and invalidates
two of the three things built in that same turn. It is binding and requires no
frame recut:

1. **Two-stage construction.** `ContinuationContextId` does not exist while
   `exact_continuation_projection` interns a specialization key, so that phase
   cannot build the final generated-`EntryFrame` claim and later mutation of an
   interned projection is unlawful. `(enclosing_specialization,
   worker_body_origin)` is a **provisional interning key only**; finalization
   resolves it to exactly one `(ContinuationContextId,
   ContinuationSpecializationId)`. Zero or multiple matches refuse. Never expose
   a half-stamped claim. **Not built** — the partial keys on the pair.
2. **The frame ID names the SOURCE/CALLER frame** whose `defining_abi_operands`
   are indexed, not the target `context` argument to `call_declared_context`.
   **Not built** — the partial names the enclosing specialization.
3. **Caller-class ambiguity is fail-closed, not a widened slot.** Never reached.

### The measurement that had to come first — DONE, answered negative

*(Superseded by the LIVE block above: no multiplicity, 40/40 and 20/20 within
plan, and the structural argument from `ContinuationSpecializationKey`. Kept
only as the record of why it was sequenced ahead of the claim shape.)*

**Three planner rows assert refusals the re-cut changed** — the crossed
owner/context refusal (`"different producer owner"`) and the caller
current-lexical proof (`"does not carry a current-"`). Their authority is not
lost: under the re-cut the caller's proof lives in the enclosing
specialization's own projection, built with a predeclared emitter and therefore
carrying the `CurrentLexical` claim. **They must be re-cut onto that, not
deleted.**

### Framing debt: still clear

`docs/program/wp/RT-UNIT-CLOSURE-CONVERT.md` exists and its node is now
`active` — `D1` released 2026-08-05 from exact `b3ba2820`. ⛔ **The sentence
that used to sit here — "it enters the frontier when `RT-CONTSRC-PRODUCER-LOCAL`
merges, with no Steward pass in between" — was FALSE and is retired.** See the
sequencing ruling in the LIVE block: the two nodes are one atomic set on one
branch, so that merge never precedes this node.

### `D3c` RETURNED: the position MOVES. `D3b`'s premise is FALSE.

QA-approved at exact **`f5e4fa9f`** over preserved `bc371f13`; production
unchanged, every added line test-gated. At a real predeclared seat under one
intervening binder, `source_abi_position = 0` but the entry operand `v15` is at
immediate position **1** — production reads `producer_env[0]` and gets `v44`,
the producer-local the binder pushed. In bounds, identical lowering shape, and
**`D3b`'s own consistency law passes**, because both numbers are 0. The seam
emits a well-formed operand of the right contract carrying the wrong value.
Zero-depth rows agree position-for-position in the same window, so the
divergence is attributable to the binder.

### The fidelity recut, and the block it had to clear (history)

The Architect ruled (crossed-post confirmation): **the pairing table does not
survive.** Availability becomes **consumer-specific planner-issued claims** over
`CurrentLexical` and `EntryFrame`, with `GeneratedContextCapture` **subsumed**
into the generated-context `EntryFrame` case. Retired by name: `RootIsImmediate`,
the three-lawful/three-crossed table, `immediate_slot == source_abi_position`,
`ContinuationImmediateResolution::root`.

**A fourth pairing does not fix it** — one unqualified index cannot be authority
for two consumers holding different environments (direct emission reads
`producer_env`; context-capture append reads `defining_abi_operands`).

Recut published at `evt_6wk0fznne72z1`, `origin/main` `b9bd4602`. **Three sites
beyond the checkpoint carried the same premise and were swept**: `D1`'s "Entry
ABI availability remains its existing case, untouched"; Finding 2's rejection
rule, whose basis was emitter-class-for-root-domain; and `AC-2`, which named
three consumers when the count is ten.

⛔ **The first recut was BLOCKED by the Architect and is now corrected.** I
transcribed the `D3b` ruling faithfully and left `D2b`'s operative two-arm law
standing three hundred lines up, with a paragraph under it saying to read `D3b`
instead — annotation around an operative deliverable. Corrected at
`evt_5s943gevnthec`; `origin/main` **`10f492aa`**, frame blob
**`07f49bac4b5a6ce98b359d0efb96928c31fb7a7e`**, object-verified. `D3b` unfrozen.

⭐ **The reusable half:** I treated text as historical because the checkpoint
that wrote it was discharged. **What makes text operative is whether a reader
would build from it, not whether its author is done.** Sweep by that test. Two
hits were then deliberately KEPT with the reason recorded at each — `D3a`'s
refusal sentence (a true record of what it landed; rewriting falsifies history)
and `D1`'s `unchanged` on the Entry ABI **root source** (correct — root identity
is preserved; only *availability* was replaced).

**Three scope calls already made (`evt_3n4k9xx4mvq8b`), do not re-litigate:**

1. The correction **folds in** as the re-cut `D3b`. No new node — same seam,
   same node, and this node's own `D1` already separated the two facts.
2. `D3b`'s landed arms stay **directionally accepted evidence**. The premise is
   what is false, so the re-cut takes the resolution law and the consistency
   check, not the `GeneratedContextCapture` membership machinery or the
   consumer-mutation apparatus.
3. **The masking defect is scheduled work, so this is not latent.** The
   implementer correctly noted the defect is unreachable in green programs
   today because that population already dies at the unit-body boundary.
   `RT-UNIT-CLOSURE-CONVERT` exists to clear that boundary — **landing it is
   what unmasks this.** The correction lands before or with that node.

The one-release-ahead sweep is **done and clean**: both nodes whose
`depends_on` names `RT-CONTSRC-PRODUCER-LOCAL` — `RT-UNIT-CLOSURE-CONVERT` and
`RT-CONTSRC-CALLABLE-CONTRACT` — are now `ready` with written frames, and all
six `RT-DECL-CLOSURE-PORT` successors were already `ready` with frames. **There
is no framing debt on this branch.** (`blocks` and `depends_on` disagree in
this corpus; `gen-progress.sh` reads `depends_on`, so that is the one to grep.)

**Do not schedule `RT-CONTSRC-CALLABLE-CONTRACT` ahead of the campaign** — it
closes 1 of 83 instances and 0 of the 6 failing `D0` rows. That priority is
stated in section 0 of its frame rather than encoded as a withheld status,
which is how it was expressed before and which made it look unstarted.

**`D3b` landed at `bc371f13` and is PRESERVATION-ONLY, not complete**
(Architect `evt_56jh63qntwtfe`). My scope recut: `evt_7he9qv8wbv1yq`. `D4a` was
QA-approved at `ac897a08` (`evt_7yydatq78eqvg`).

⭐ **The two boundaries, and why the split went the way it did.** Item 2 — the
`EntryAbi` premise — **folds in** as `D3c` and **sequences ahead of any further
`D3b` work**, because `D3b`'s pairing law couples root provenance to immediate
availability and its *premise* is what is in doubt (QA proved *fidelity*, which
is a different claim). Item 1 — the unit-body short environment — **cut out** as
[[RT-UNIT-CLOSURE-CONVERT]] rather than folded, because
`RT-CONTSRC-PRODUCER-LOCAL` is already four checkpoints past its own recut, was
itself cut out of `D7`, and is the sole gate on all seven campaign nodes.

⛔⛔ **THE SIZING FACT THAT WILL BE GOT WRONG:** the Architect's mechanism list
reads as an `L` node. It is not. **`RT-FNSPLIT-B2R` is MERGED and landed the
closure-conversion CONTRACT `Inert only`** — typed capture slots, exact
free-variable identities, and the reject-on-missing/extra/mismatched validator
are **production code on `main` today**, with zero executable edge. `B2F`,
`C3-ACTIVATION` and `RT-NATIVE-FNSPLIT` are **also merged**. ⇒ The node
**activates merged substrate** for a population that did not exist until `D4a`
admitted it. ⭐ Its `D1` is an inventory and that inventory is the sizing
instrument. ⛔ Do not size it before that answer exists.

⚠ **`runtime-leader` STALLED AT CAPACITY at ~11:47Z and I re-prompted it.** The
QA approval reached it, the model refused with *"Selected model is at capacity"*,
and ⛔ **the Codex turn ended SILENTLY** — approval unprocessed, ring frozen,
pane looking merely quiet. Re-prompt recovered it (`• Working`). ⭐ **Capacity is
transient and the seat does NOT self-recover; a re-prompt is the whole fix.**

⚠ **Earlier, the implementer was STRANDED, not idle** — composer held unsubmitted
text behind a finished turn. ⛔ **`handoff-gate-compact.sh` leads with a bare
`Enter`, which would have SUBMITTED that strand instead of compacting.** Hand-drove
one pane with a clear-first; `C-u` did not clear and the render stayed stale, so
a probe string proved the buffer empty before staging. ⭐ **The displayed line is
not the buffer.**

### `D4a` ROUND 2 — gated YES, run once, and where it actually stands

**The Architect gated my ruling YES** (`evt_65xkzqppdqdaj`), agreeing the fixture
supplies only the population. ⭐ **It added the half most easily lost: the `D4a`
mutation proves the INSTRUMENT; the `D3b` mutation proves the CONSUMER.** Once
the real `D3b` consumer exists, substituting the locator index for
`post_shift_index` must make the same fixture fail at the consumption boundary.
`D4a` passing does not discharge that. It also pinned condition 5 to my frame
correction: the fixture may add to `V`, **never to `R`**; re-census `C`/`V` at
the new base; any new `R` member or decline cause is a hard stop.

**Released (`evt_6cfjzp9yzvw8g`), run, nothing landed** — tree restored to
`52422da5`, `724/7` unchanged, branch free.

⭐ **THE ORIGINAL BLOCKER IS CLEARED.** Three lanes measured through the real
production path: `ConsoleRead` refused (the old hard stop), `ConsoleIsTerminal`
**visited but plans no seat at all** (it returns `Bool` before seat synthesis),
and **`ConsoleWrite` lowers, reaches the emission seam, and produces a
`CurrentLexical` record**. So a lawful lowerable shifted-population fixture is
reachable and the lane question is answered.

⛔⛔ **THE "NO SHIFT" READING WAS AN INSTRUMENT DEFECT, AND I PUBLISHED IT.** The
mid-round report said no nesting could shift the value, and I wrote its
scrutinee hypothesis into this file as the next bounded act. ⭐ **The first half
was true and the second half was wrong — the probe recorded ONE LINE PER SEAM,
but the seam carries a VECTOR of continuation inputs.** The shifted input was at
**ordinal 1** the whole time, in the shape already built:

| ordinal | binding | locator index | post-shift index |
|---|---|---|---|
| 0 | the enclosing `Match`'s case binder | 0 | 0 |
| 1 | the `Let`-bound host-effect result | 0 | **1** |

⭐ **No nesting search was needed and none should have been scheduled.** This is
exactly the recorded lesson *a short-circuiting probe measures the first cause,
not the set* — it reports one member of a vector and reads as a property of the
population. ⛔ **The `env_len` observations were the tell and I read them as the
obstacle:** growing to 5 while "the index" stayed 0 was the probe holding ordinal
0 fixed, not the program refusing to shift.

**MEASURED at the production planner and lowering path**
(`recursive_port_process_compiles`):

```
post_shift_index = 1        locator.environment_index = 0
producer_env[1] = HostResult(v246, Ok, Err)   <- creation seat recorded v246
producer_env[0] = HostResult(v466, Ok, Err)   <- the decoy
```

⭐ **The decoy is a second `ConsoleWrite`** in the `Match` scrutinee's constructor
argument, matching on carrier, phase, lowering shape and constructor pair — **only
the SSA word differs**, which forces the oracle to be the SSA word rather than any
incidental discriminator.

⭐ **Oracle independence, the part `D3b` relies on:** lowering records the operand
it builds at the binder-creation seat, keyed by its own occurrence id with no
environment index in play; the seam half reads by index; the two join on
`binding_origin`, so **a wrong index breaks the join.** No planner re-walk, no
index arithmetic, no fixture-authored expected index, no direct construction.

**Mutations:** `UseLocatorIndex` and `SwapSlots` committed inside the control,
each asserting its own flip. ⭐ **`SwapSlots` is not redundant** — both indices
stay lawful and in bounds, so it survives a repair that merely bounds-checks.
Three more run by hand and reverted, including *drop the intervening binder*,
which reds loudly if the fixture stops being shifted — **act 1's gap, closed.**

⚠ **One trap a successor must not re-pay:** `ConsoleIsTerminal` looks like a free
win from the consumer list and is not — it returns `Bool` before seat synthesis
and plans no seat at all.

### THE `D4a` BIND — RULED, and round 1's stop. History; do not re-derive.

**The bind was:** the Architect required (`evt_tkzyc61rmd3`) a four-part proof at
one exact predeclared emission — a reaching `CurrentLexical` with
`post_shift_index != locator.environment_index`, the real operand at each index,
an **independent lowering-side** discrimination that does not re-run the planner
walk as its oracle, and a bounded wrong-index mutation. But the only durable
shifted fixture emits **zero seam records**: its `Let`-bound effect is
`HostOpV1::ConsoleRead`, absent from the fixed 13-element
`CRANELIFT_HOST_EFFECT_CONSUMERS_V1`, so lowering refuses it as an unavailable
lane before the seam. ⭐ **The fixture is shifted precisely BY the construct that
makes it unlowerable.** Every route to the seam was already prohibited, so the
required evidence was unobtainable and choosing what gives was a ruling.

**My ruling: a lowerable shifted fixture is AUTHORIZED**, as a second bounded
`D4a` extension round. ⛔ **No new node** — it folds. It lifts exactly one
prohibition (*"do not add a new population member"*) and nothing else. Full text
is now in the frame under checkpoint `D4a`; the three constraints are:

1. ⭐ **The fixture supplies the POPULATION; the MUTATION supplies the
   discrimination.** Building a fixture to exhibit the shift and then observing
   the shift measures nothing. ⛔ **No mutation row ⇒ no fixture**, and the
   outcome is a hard stop, not a green suite.
2. ⛔ **Do not inherit `D2b`'s effect lane.** The shifted value may be a case
   binder; those are already in `V`. Reaching for `ConsoleWrite` is analogy from
   the old fixture's shape, not derivation from the requirement.
3. ⛔ **`contsrc_d2_both_binding_kinds_fixture` is untouched.** Additive only.

**And `D4b`'s counts are now correct in the frame:** `C`=83 / `V`=80 are
**measurements at `e6d4f085`, not invariants** — the discharge condition already
said *post-admission* census. ⛔ **`R`'s three named causes are the invariant.**
A new fixture adding to `R` is a real finding; adding to `V` leaves the partition
intact.

⚠ **The grounding move worth repeating:** the prohibition I lifted was the
Architect's, and their own release named this outcome as *"the new boundary"* and
handed it back. A boundary is a scope call, so it was mine — but the soundness
axis stays theirs, which is why this is one confirming gate and not a
notification.

⛔ **`D4a` measurement is a distinct axis from `D4a` admission.** `52422da5`
already proves admission, a real depth-one predeclared emission, length
agreement and planner-side placement. **Equal indices make the pass-through
defect observationally identical**, and length agreement does not identify the
lowering value — which is why the Architect did not discharge it.

⛔ **THE BINDING ORDER IS FOUR, NOT MY TWO** (Architect `evt_7vc8zh0rvqyps`,
superseding my own `evt_11esqaep9awbs`):

1. **`D3a`** — non-lowering closure; both lowering consumers explicitly refuse;
   seam and pending population stay **visible**. **DONE, QA-approved.**
2. **`D4a`** — bounded admission and measurement. ⭐ **MAY BE DELIBERATELY RED.**
   It exists to produce real reaching producer-local emissions so nonzero-depth
   `CurrentLexical` correspondence can be measured. ⛔ A red here is the
   instrument working, **not** a regression to chase.
3. **`D3b`** — lowering closure, only after that evidence exists; seam deleted
   only when its closed population is empty.
4. **`D4b`** — closeout: `interned = V`, `declined = R`.

⛔ **My recut said `D3b` lands "with or after `D4`" and that was
under-specified where it counts** — `D4` as one unit cannot both *create* the
population and *prove* the partition, so it never named what produces `D3b`'s
evidence. `D4a` is that mechanism. ⭐ Same defect class as the `D1` clause: a
load-bearing sequencing term left ambiguous across two things.

⛔ **Option 2 is INVALID, not merely worse:** `D4` cannot safely admit before
the lowering consumers are explicitly fail-closed — hence `D3a` before `D4a`.

⛔ **The ABI ruling, so it is not re-litigated:** `AbiContinuationInputAuthority`
carries a **closed tagged provenance sum** — `EntryAbi { source_owner }` |
`ProducerLocal { binding_owner }` — keeping ordinal and affinity. A domain-total
bare owner was **rejected as lossy**: it collapses `EntryAbi { source_owner: X }`
and `ProducerLocal { binding_owner: X }` into the same value.

⛔ **"Any mismatch is a hard stop" is CORRECTED as overbroad**
(`evt_6p6vf0aqnjn3g`). Seam 1 must reject `CurrentLexical` at a specialization
emitter before indexing any operand run; a predeclared emitter must reject
`GeneratedContextCapture`. Applying the `CurrentLexical` comparison to a
specialization emitter is **itself a category error**. ⛔ Do not carry the old
phrasing forward from earlier posts in the thread.

⚠ **It woke on the mention, and that contradicts this file's own wake-asymmetry
claim below.** The standing note says a Claude implementer's mention push never
reaches the session. It did here. ⛔ Do not rely on either reading — **read the
pane before rousing**, which is what caught it. I did not establish the
mechanism, so this is an observation, not a retraction.

**`D2b` is QA-APPROVED at exact `7316e13a`** (`evt_3w4s25ta13hc4`), lineage
`e6d4f085` (base) → `2bd724cd` (record) → `7316e13a`. `D3` was released on top
(`evt_7rk80sgaq07fg`), the implementer scoped it and **made no edits**
(`evt_5pqxd21sw5m57`) — branch free, tree clean. ⛔ No merge is owed by me: the
node is mid-flight and the branch accumulates until the WP completes.

⭐ **This ring keeps stopping clean rather than half-applying, twice now.** Both
times it posted the scoping instead of holding it in context, so compaction cost
none of it. Read the stops as the frames working, not as under-delivery.

**`D3b`, `D4b`, candidate, `D6` closure, `#27`/case-emission, the call-result
SCC and downstream `D7` all remain held.** ⛔ `D3a` is DONE (QA-approved) and
`D4a` is ruled-but-ungated, not building — neither is merely held. WIP clock:
derive it from the latest reset event in the thread, never from a stamped
deadline.

### Where the node stands

| deliverable | state |
|---|---|
| `D0` `12d9612a` (zero delta), `D1` `77a24320` | accepted `evt_5zkydewv5kspb` |
| `D2` identity + value contract | accepted preservation at exact `e6d4f085` |
| `D2b` immediate availability | **QA-APPROVED** exact `7316e13a` — `evt_3w4s25ta13hc4` |
| `D3a` | **QA-APPROVED** exact `14b111ae` — `evt_62g4pganvk6f6` |
| `D4a` | **QA-APPROVED** exact `ac897a08` — `evt_7yydatq78eqvg`. `V` admitted `52422da5`; rd 1 hard stop `evt_7xwdw87mgf1q3`; rd 2 ruled `evt_28xx7t69z7j76`, gated `evt_65xkzqppdqdaj`, shifted fixture landed |
| `D3b` / `D4b` | held, in that order |

⛔ **There is no undivided `D4` any more.** The SET EQUALITY definition below
still governs — it is what **`D4b`** discharges; **`D4a`** admits `V` to create
the population `D3b` needs to measure.

**`D2`'s route was blocked once and corrected twice, and both stops were
sound.** `a5a6ce9b` stamped one blanket `ValueWord` contract across a
`ComputationalMatch` binder run that is **not homogeneous** — it is ordered
`[recursive IH binders, constructor argument binders, outer environment]`.
`5377d2ab` fixed the argument half by **reading** the carrier from the
scrutinee's shape instead of choosing one, and hard-stopped on the IH half
rather than defaulting. `e6d4f085` added the census and the fidelity
correction.

### `D4` is now SET EQUALITY, not closure

Unit: **one call to `exact_continuation_source_environment`**, identified by
program fingerprint + consumer owner + continuation origin + producer construct
origin + recursive position + closure origin.

| set | contents |
|---|---|
| `C` | all **83** `(identity, full required vector)` instances |
| `V` | the **80** fully closed under the current value-slot authority |
| `R = C \ V` | exactly **3**: `OPEN[ih-binder]`, `OPEN[let-value:Construct]`, `AMBIG2[let-value:If]` |

`D4` discharges when `interned = V` and `declined = R`. ⭐ **All 17 parity
instances are in `V`** — the population behind the six failing `D0` rows, and
the critical-path fact. ⛔ Call the three **outside-this-contract-domain
residuals**, never "unrepresentable" — the Architect corrected my wording, and
nothing claims a future authority cannot represent them.

⛔ **The program fingerprint is load-bearing.** `StaticOriginId`s are per-compile,
so without it edges from different fixtures collide and the census silently
undercounts: a first pass reported 58 identities of which six were collisions.

### `D2b` — why `D3` hard-stopped, and it is MY frame this time

`D3` reached the real emission seam and measured that **a producer-local value
has no member in the run the seam indexes.** Resolving its arm would need one of
three exits this node bans: widening the emitting function's input ABI run,
giving the seam a second non-ABI environment, or reusing a convention slot.

⛔ **The root cause is a `D1` clause I wrote.** It promised "an exact
emission-time locator into the environment that actually contains it" and
**never said which environment**. `D2` read it as the semantic environment and
populated a scope-relative `(environment_origin, environment_index)`; the seam
indexes a different space. **A load-bearing term left spanning two coordinate
spaces** — a different defect class from this campaign's earlier four, which
were false laws. Nobody could have discharged it as written. `D1`'s wording is
now corrected in place.

⛔ **Do not pin "`producer_env` is always the ABI operand run."** The Architect's
precision correction: the 61 records prove the **currently admitted** population
only, and there are **two** consumers — the retained-frame seat passes the
current `LoweringEnvironmentBinding` run, the detached/generated-context seats
read a function-local ABI operand run.

⭐ **The implementer withheld six of nine consumer sites** rather than land a
partial that would leave the seam no longer naming its own remaining work. That
is now frame law: no partial `D3`. It also **withdrew its own parity remark**
unprompted — it had reverted the probe while the parity run was in flight, so
the empty result could not distinguish "no parity emission reaches this seam"
from "the probe was not compiled in". An absence presented as corroboration,
caught and retracted by its author before anyone read the stop through it.

### The lesson from my own fork, because it will recur

I put a **binary** fork to the ring: zero IH-bearing edges ⇒ no node; nonzero ⇒
substrate first and `D3` waits. The census returned **1 of 83**, and the leader
applied my rule correctly — but the decisive fact had no cell in it: **the IH
edge is one of three non-closed positions from three causes, and a callable
contract closes exactly one.** So "every environment closed" was unreachable by
any node in the graph, with or without the substrate. That is a deadlock, not a
gate — the same thing checkpoint `1f` retired.

⇒ **State a fork by what would DECIDE it, not by the shape of the number you
expect back.** The census was still the right call; the defect was in how I
pre-committed to reading it.

### `RT-CONTSRC-CALLABLE-CONTRACT` — filed, `draft` on purpose

A real capability gap: production continuation inputs have **no callable domain
at all** (`BoundaryUseAvail::Callable` is `#[cfg(test)]`-only), and a recursive
IH is a compiler-only `StaticWorker` with no word, tag, descriptor or carrier.
Grounded in the Architect's ruling plus three source measurements.

⛔ **Held `draft` deliberately — it is NOT framing debt.** It closes 1 of 83
instances and 0 of 6 failing rows, and the one-release-ahead policy is already
satisfied by the six framed successors of `RT-DECL-CLOSURE-PORT`. Promoting it
would put an off-critical-path node in front of a reader looking for the next
kickoff.

### Two things about this node that must not be misread later

**None of `D2`'s stops was a sizing defect.** The heterogeneous-run defect was
caught at the gate; the IH boundary was found by building, which is the only way
it could have been found. Four *earlier* stops on this campaign were my frames
asserting laws the measured plane does not support — do not count these with
them.

⭐ **The implementer retired its own invented `ValueWord` blanket before the gate
ruled on it**, reported the IH half as three measured grounds rather than picking
a default, and declined to choose the graph. That is what these frames are
written to produce.

> ### The mistake I made at 07:36Z, because the shape recurs
>
> My frame-correction post `evt_270c4gk9trrmv` carried the line "`D2`-`D4`
> remain unreleased pending the Architect's gate". **It was already false when
> it landed** — the gate and `D2`'s release had posted 80 and 57 seconds
> earlier, and I had composed against the state I last measured. The Architect
> caught it in under a minute (`evt_1g2ssacct76tq`); corrected at
> `evt_7rbseqb0xnsaq`.
>
> ⇒ **A post whose subject is NOT release state must not assert release state.**
> A status claim carried as background inherits the message's authority and
> escapes its scrutiny. Re-read the channel immediately before posting anything
> that names what is released.

### The three frame corrections, so they are not re-litigated

From `evt_1srfqjmkp5eh8`, all published:

1. **`D3`'s consumer count was 3 in prose, 10 measured.** Frame now sizes `D3`
   from the in-tree seam function `entry_abi_pending_producer_local`, not from
   frame prose. **A frame-side count of a code-side population goes stale the
   moment the code moves.**
2. **`AC-1`'s six red rows are TWO populations** — the `AC-1` row refuses at
   `Match: scrutinee is not a constructor value`, the other five at
   `ComputationalMatch: ...`. Greening the five does not discharge `AC-1`, or
   the reverse. Invisible in `1 passed / 6 failed`.
3. **`AC-5` is pinned to `D4`.** It guards broad admission changing the interned
   population; before `D4` admits anything that condition is unreachable, so an
   earlier "controls green" report is true and meaningless while reading as
   cleared risk.

⚠ **A live-verb grep nearly reported this ring idle.** The implementer's footer
read `✻ Actualizing… (1m 9s)` — a verb absent from the tick's pattern list, so
the sweep printed a blank status. **The busy-check is wrong in both directions
and the verb list is open-ended: a missing verb reads exactly like idle.**
Resolve any blank or `(no-footer)` status by reading the pane, never by
extending the pattern and trusting it.

⭐ **Confirmed open-ended, twice more: `Baked for 5m 51s` and `Grooving…` both
printed `(no-footer)`.** ⛔ Do not chase the list. ⭐ **The cheap independent
instrument is `ctx`, which the tick already captures: a ctx that ROSE between
two reads is work, whatever verb is rendering.** It resolved `Grooving` without
a second pane read.

- Kick (fresh root, its own thread): **`evt_7h92n2tr7pbrm`**.
- `D7` rescope-in-place, posted in `thr_3rx07jfewhjhf`: `evt_14a9cee7fkv2s`.
- Handoff gate ran on all three seats (all 0 ahead, 0 dirty, so the
  `reset --hard` was safe). **Confirmed:** implementer ctx 0% with skills
  restored, both Codex seats show `Context compacted`.

**The wake asymmetry is the thing to watch.** `runtime-leader` and `runtime-qa`
are **Codex** (`gpt-5.6-terra`) and woke on the mention via the tmux backend —
the leader was Working within a minute. **`runtime-implementer` is Claude
(Opus 5) and its mention push never reaches the session.** So the leader's
dispatch to it will not wake it either. If it sits idle at an empty composer,
rouse mechanically: `tmux send-keys -t moot-runtime-implementer -l "<one line:
run get_recent_context, pick up evt_7h92n2tr7pbrm; re-orient per CLAUDE.md>"`
then a **separate** `Enter`. A wake is not task routing and does not breach
Steward-never-to-implementer.

### The branch trap — RESOLVED 07:1xZ, kept because it recurs every release

**Confirmed clear by an independent instrument:** the handoff gate's own
post-compaction worktree read shows `runtime-implementer` at
**`179af863 (wp/RT-DECL-CLOSURE-PORT-typed-units)`**, so it is building on the
proved lineage, not bare `main`. No `preserved/` refs were created — nothing was
ahead. The description below is the standing hazard, not an open item.

**`179af863` is contained by exactly ONE ref —
`wp/RT-DECL-CLOSURE-PORT-typed-units` — and NO worktree has it checked out.**
All three runtime seats sit on their own `*/work` branches at 0 ahead of `main`.
⇒ The implementer must **explicitly check out that branch** before touching the
new node. If it starts on `runtime-implementer/work` it builds on bare `main`
without checkpoints 1/`1b`/`1c`, and a grep for its own `D7` symbols comes back
empty — which reads as missing work rather than as a wrong branch.

### What `1e` got wrong, and the defect is reusable

`1e` ruled the minimal scope was the host-effect-result slot alone. **Falsified**
(`evt_5ngh190h9b1k5`) and the design rejected by the Architect
(`evt_75k8cydbj5127`): every effect-bearing closure needs **two** `Open` inputs,
ordinal 0 an effect result and ordinal 1 a case binder, so the
effect-result-only population is **zero** and closing it moves no row.

**The defect: `1d`'s census recorded the DECLINING ordinal — the first `Open` —
and I read it as a REQUIREMENT census.** "6 effect edges = the 6 failing rows"
was a pair count short-circuited at the first `Open`, compared against a `161`
in a different unit. Corrected closure-edge census: **34 case-binder-only, 4
mixed, 1 `Construct`-only.** A first-failure statistic is silent about every
input after the one that failed, so it cannot support a minimality claim.

**Also settled:** there is no lawful ABI seat for a mid-body value — the
Architect closed all five exits. A producer-local value is a **third
availability class**, which is why this is a representation boundary and not a
missing enum arm.

### The rulings now standing

- **BROAD admission.** All newly representable candidates may intern, not the 4
  `D0` edges alone — the narrow option needs a real edge-selection authority
  with every cheap substitute forbidden. This **dissolves** route modality.
- **~34 edges newly intern**, changing emitted code on green programs. Expected;
  the per-row `D0` and `718/2` baselines are the control.
- **`D7` retired the clause** blocking candidate/QA/`D6`/`AC-4` "while the row
  stands unreached" — it could never be discharged by the node holding it.

### Four stops on this node were MY framing, not Runtime

checkpoint 1 (mislocalized), `1b` (`1/1/1`), `1c` (forward reading), `1e`
(first-`Open` as requirement). **The instruction to measure rather than comply
caught every one.** Keep writing frames that way. This is not a sizing problem
and should not be read as one later.

## SUPERSEDED — 2026-08-05 ~06:4xZ · D7 `1d` answered NEGATIVE; `1e` released

**`origin/main` at last check: `3eeeb5ed`** (the `1e` ruling, PR #1410; D7 frame
blob `b5c240e6`). Verify it; do not trust this line.

### The one thing to do next

**Nothing, until Runtime returns `1e`'s answer.** I ruled at `evt_2tsq017qgvtgh`
(06:41Z); `runtime-leader` released `1e` from exact `179af863` and the Architect
picked up the confirming gate — **both confirmed by pane transition, not just a
posted mention.**

**WIP audit clock: armed from the ~06:43Z leader release, so due ~07:43Z.**

### What `1d` settled, and what `1e` is

`1d` came back **negative over 1110 candidate records** (`evt_5kws532ac99c9`) —
no existing authority both proves the closure-`381` edge mandatory and supplies
an exact edge-local closed environment. Three results outlive the checkpoint:

- **`member=true` on all 612 declines AND all 489 interns** — `1c`'s finding at
  1101-row scale. Closure-level membership is retired as an edge-local
  predicate permanently, not provisionally.
- **`case_emission` is INAPPLICABLE, not insufficient** —
  `build_case_emission_plan` never iterates `ComputationalMatch`. A later node
  reaching for it finds nothing, and now knows why.
- The ring **declined a near-miss discriminator** ("`Open` because of an effect
  result") as confounded with corpus identity: `Effect` occurs in 0 of 1057 lib
  ancestor chains and 60 of 60 parity chains, so it separates two test suites.

**`1e` is folded into `D7` — there is NO new node.** `1d`'s "requires a separate
substrate node" was my prose, not a ruling; the preference order is relax, fold,
then cut, and `179af863` is not on `main`, so a separate node would branch from
an unmerged branch for no independent mergeability.

**Scope is minimal by the inventory's own counts:** the host-effect-result
`ContinuationInputSource` variant plus its ABI position — 6 edges, exactly the 6
failing `D0` rows. **The case-binder slot is OUT** — 161 edges in a corpus at
718/2, no failing row demanding them.

**I refused the second minting.** A route-modality authority's only source is
`1d`'s own distinguish-before-interning requirement, which was a constraint of
the projection-only framing that `1e` retires. Stated to the ring as a question
to measure, not a law — three of this node's stops were exactly that error. **If
they report it IS genuinely required, that one gets a real node.**

**`1d` hard-stops TO ME.** If the inventory finds that satisfying it would mint
a new representation, population, identity, or planner/ABI authority, that needs
a **separate substrate node** which the Architect expressly did not authorize —
and **the graph shape is my call, decided FROM the inventory.** Do not cut that
node before the answer arrives; deciding beforehand is creating a node on
speculation.

### Checkpoint lineage — each an accepted parent of the next

`6a09ed68` (population) → `727b51a1` (per-visit claim group) → `69c68e6e` (body
close) → `f6958b95` (operation-arm claim consumption) → `ae64f687` (lazy
exact-`SiteOperand` + carried exact-`Int`) → `50092c59` (ckpt 1: phase-bearing
capture edges + pre-emission gate) → `ca1c4418` (ckpt `1b`: two arity
coordinates) → `179af863` (ckpt `1c`: the interned-to-member converse).

`4ec5362c` is **preservation-only partial progress**, not an accepted `1b`.

### The standing shape of this node's stops — READ BEFORE RECUTTING AGAIN

**Three stops were MY frame asserting a law the measured plane does not
support**, not Runtime sizing or execution:

1. **checkpoint 1** — I localized the repair to the `#23` producer; the producer
   was correct and the refusal was at the generic `LexicalClosure` value arm.
2. **`1b`** — I demanded a nullary Host-`Vis` be `1/1/1`; the honest relation is
   source-seed `0` / emitted-template `1` / marker-consumer `1`, and forcing the
   seed to `1` moved five non-injecting rows off their `D0` text.
3. **`1c`** — I read generic member status as a planning omission; `Open`
   environment means *do not commit this specialization*, and the forward law
   would falsely reject 23 green programs.

⭐ **The per-row-never-a-count requirement caught #2 and #3.** A pass/fail total
reads `1/7` before and after, identically, and hides both. Keep it as a frame
requirement, never a convention.

**Still held:** checkpoint 2, candidate, QA, `D6` closure, `AC-4`, the
call-result SCC, and the `#27` / case-emission populations.

**WIP audit clock — DERIVE IT, do not read a stamped deadline.** A fixed
timestamp here goes stale on every reset event and then fires a spurious audit;
it needed rewriting twice in the first hour. At tick time, take the **latest**
of these in the WP thread and add 60 minutes:

- a kickoff or re-kickoff (leader release),
- an Architect audit, ruling, or review verdict,
- a candidate or checkpoint handoff,
- a genuine hard stop, or completion.

**A routine progress post is not a reset event.** Counting those makes the
trigger fire never while looking armed.

On this ring the resets have been arriving every 5 to 15 minutes, so **the
60-minute trigger has not come close to firing and probably will not while the
cadence holds.** That is the healthy case, not a broken detector — but it is
also why a stamped deadline was pure noise here.

**Governing base, do not let it drift:** continue only from the `70887529`
lineage. Rebase, merge, or cherry-pick of `fb8fd881`, `430798bf`, `548682c3`,
`42ccd8ec` is banned — they are competing historical implementations, and
importing them reintroduces the role/disposition-derived schema the host-effect
ruling ruled false (Architect `evt_` in `thr_3rx0`, 01:06).

### Frontier: one release ahead is SATISFIED

Every node whose `depends_on` names `RT-DECL-CLOSURE-PORT` is `ready` and has a
frame file in `docs/program/wp/`:

| successor | other unmet deps |
|---|---|
| `RT-SEED-CALL-PORT` | none — this is the immediate next release |
| `PX8-ERRID-ALLOC` | `RT-NATIVE-FNSPLIT` |
| `NATIVE-HANDLE-CARRIER` | `RT-NATIVE-FNSPLIT`, `RT-JOIN-DISPOSITION` |
| `RT-CONTSPEC-LEDGER` | `RT-CONTSPEC-ACTIVATE` |
| `RT-DESCENT-RETIRE` | `RT-SEED-CALL-PORT`, `RT-PRODUCER-MATCH-PORT`, `RT-RECURSOR-TRANSPORT` |

`RT-SEED-CALL-PORT`'s fixed-input blobs are pinned at `origin/main = 14c3c5f7`
(2026-07-29) and are stale by construction — D7 rewrites `core.rs` in front of
it. **The frame says so itself and instructs re-pin at pickup.** That is
shovel-ready, not framing debt; do not re-pin the numbers and call it a
re-measurement.

### Lane state

| ring | state |
|---|---|
| **Runtime** | building — `RT-DECL-CLOSURE-PORT` D7 effect-seat slice 2 |
| **Kernel · Verify · Language · Ergo · Foundation · Spec** | idle, awaiting Steward kickoff — the fleet's single-threaded posture, not a stall |
| **Doc** | stood down after `DOC-PROGRAM-WAVE-RECONCILE` merged |
| **Architect** | serving the Runtime ring; last act was the identity acceptance of `70887529` |

**Tracker statuses reconciled 2026-08-05 ~07:45Z** — four were wrong against the
generator's own legend, where `active` means **a team is building**:

| node | was | now | why |
|---|---|---|---|
| `RT-CONTSRC-PRODUCER-LOCAL` | `ready` | `active` | it IS in flight |
| `KERNEL-NESTED-IND` | `active` | `ready` | deps met, framed inline, no seat |
| `SPEC-MISSION-GROUNDING` | `active` | `ready` | three ACs open, no seat |
| `SURF-SPACE-CELLS` | `active` | `draft` | P1 landed, P2 residual unframed |

**Both of the two nodes that argued for `active` in their own prose used it to
mean "not merged".** `SURF-SPACE-CELLS` said it stays `active` "so a reader
cannot mistake a merged phase for a merged node"; `SPEC-MISSION-GROUNDING` said
the AC reconciliation "is the reason it is `active` rather than `merged`." The
anti-merged signal those blocks wanted is carried by the blocks themselves. Both
operative sentences are rewritten, not appended to.

⇒ The releasable-frontier list now shows `KERNEL-NESTED-IND` and
`SPEC-MISSION-GROUNDING`. **That is accurate and is not a kick order** — the
single-threaded hold is a release decision, not a dependency, and the tracker is
generated so it cannot carry the hold. `SPEC-MISSION-GROUNDING` in particular is
**not** releasable by me: `AC-M3` names a pass `COORDINATION §10⁻a` forbids the
Steward to request.

### Unlanded finished work — research, 4 days old

`wp/research-kernel-extension-assessment` @ `0c450267` (2026-08-01) carries
`research/kernel-extension-assessment.md`, 746 lines, absent from `main`. No
`git_request` for it reached me. **Its path is neither `library/` nor a Steward
route, so the fail-closed predicate sends it to the Architect** — who is
currently the Runtime ring's reviewer. Do not spend that seat on it mid-slice;
bundle the routing question with the next Architect contact, or ask research
whether anything is owed.

### My own transport — fixed by the 01:48 restart

The seat now runs the original flagged process
(`--dangerously-load-development-channels server:convo-channel`), not the
unflagged `bg-pty-host --fork-session --resume` fork that silently dropped every
mention. Channel subscription confirmed on `spc_4q7g0se87rgje`. **The
generalization I posted at 01:35 — that `runtime-implementer` shared the
defect — was wrong and is retracted at `evt_` 01:45; route to it normally.**

### ⛔ OWED TO THE OPERATOR — four items, none self-resolvable

1. **`MAP-TRANSPORT-CODEC` candidate 3** — a wire format for a **non-Ken peer**.
   Candidates 1 and 2 were measured and answered *no*; this one is a **roadmap
   call and is not answerable from the repository.** The node is closed
   `not-needed`; if this comes back *yes* it reopens with a fresh frame.
2. ⛔ **`SPEC-MISSION-GROUNDING` `AC-M3` names a pass I am forbidden to request.**
   The AC says the adversary refutation pass is owed; `COORDINATION §10⁻a`
   forbids the Steward from asking the adversary to hunt anything. **Two
   operator-authored artifacts conflict.** Needs the operator to dispatch the
   adversary directly or re-route the AC. **Raised, unanswered.**
3. **T3 / `Property`** — there is **no `ken test` subcommand** (`ken-cli`
   dispatches `repl|run|check|native-build|fmt|version|help`) and **no spec
   chapter for the CLI at all** (task `#143`). `Tooling/Testing/Property.ken.md`
   exists but is deterministic finite checks — no randomness, shrinking, or
   seeds, deliberately. ⇒ T3 is blocked on a **design input**, not on code.
4. **Linux ABI** — `ABI-S3`'s three ops landed `RepresentedUnavailable` by
   design and **no Track-A node promotes them**, so `§6`'s exit condition is
   unreachable through `ABI-A1/A2/A3`.

### ⛔ Still operator-HELD

**`DOC-ATTEST-LIVING`** — ⛔ **do not release, do not re-ask.** Node:
`docs/program/issues/DOC-ATTEST-LIVING.md`.

### ⛔⛔ THE `integrator` GHOST — do NOT chase it again

**The seat was RETIRED by PR #1052** (50 files). It has **no tmux session, no
entry in `.moot/actors.json`, and no playbook** — `agent/playbooks/federation/`
holds only adversary, architect, librarian, research, steward.

⚠ **But `orientation()` and `list_participants` still show it**, carrying a
**stale stored status** — *"PR #365 green on head `befc2dc4`, awaiting Steward
routing."* ⭐ **That reads exactly like a live seat blocked on you, and it is
not.** I treated it as a real open query, investigated it, and posted a routing
reply to a seat that cannot read. No one was waiting; nothing was owed.

⇒ ⛔ **A retired seat's last status is indistinguishable from a live seat's
current one.** Before acting on any participant status, check for a tmux session
**and** an `actors.json` entry. Convo has no tool to remove a participant, so
this ghost persists — it is operator/convo-admin item, already raised.

For the record on its content: `befc2dc4` is **on no ref at all**, and
`scripts/scripted-pr-automerge.sh` **is** on `main` (blob `76afaf31`) — the
capability landed and I run it every publish. ⛔ Do not re-propose that commit;
its `COORDINATION.md` / `04-git-and-integration.md` / `steward.md` versions are
from 2026-07-08 and re-landing them would revert weeks of work.

### ⭐ Traps measured this window — positional, so they will recur

1. ⛔ **The decisions read path field is `decision_id`, NOT `id`.** `d.get('id')`
   returns `None` for **every** record, so a lookup reports **NOT FOUND for a
   decision that exists**. I was one step from blocking a merge on this. ⇒ Always
   run a positive control against a decision you know exists.
2. ⛔ **Any non-doc-only publish MUST be `run_in_background: true`.** Full-CI
   polling exceeds the Bash tool's 600000ms cap and the tool kills the publisher
   (exit 143). Doc-only finishes in ~2 min and is safe in the foreground.
3. ⭐⭐ **"Awaiting merge" may already BE merged** — twice this window (Ergo
   `a85c0dc5`, Kernel `5396f9a7`). The publisher **squashes**, so every ancestry
   check says unmerged. Only a blob diff of the candidate's **own** paths against
   `origin/main` discriminates. ⚠ And a path-drift check *actively misleads*: both
   Kernel paths read as "changed on main since the approval base," which normally
   means a stale base — here the thing that changed them was the candidate's own
   already-landed work.
4. ⛔ **Require the exact BRANCH as well as the exact SHA.** Kernel had
   `wp/KERNEL-NESTED-IND` (D1a, `e685570c`) and `wp/KERNEL-NESTED-IND-D3` (the
   approved D3a) live at once, and the approved SHA was **not** an ancestor of the
   branch matching the node name.
5. ⚠ **A node annotation written at merge time can gate another team's frontier.**
   I wrote *"`RT-VALUE-TOTALITY` stays `active` for its remaining scope"* — it had
   none, and that stale `active` was the last unmet `depends_on` of
   `RT-FNSPLIT-C1`. It would have idled Runtime behind a complete node. It
   surfaced only because `runtime-leader` refused to infer a branch and asked.

## ⛔ CORRECTIONS — two claims the old file made that were FALSE

⭐ Both were **time-varying state wearing a permanent-looking hat** — the exact
failure the heartbeat prompt bans. Recorded so the *shape* is recognisable, not
just the instances.

### 1. ⛔ "ARMED COUNTERS — the SOLE count of record" was stale AND retired

It read `RT-NATIVE-FNSPLIT: hard-stop 10 · next research pull #11` and `Architect
production blocks: 6 · next check #9`. **Both numbers were behind**, and the chain
they counted **is retired** — the operator stopped the FNSPLIT effort on
2026-07-26 and `SPEC-STORE-SPLIT` replaces it.

⛔ **A counter calling itself "the SOLE count of record" is the worst thing to
leave stale**: it invites a reader to trust it *instead of* measuring. ⇒ **There
are no armed counters now.** When the re-cut program exists, its node owns its
counts.

### 2. ⛔ "TRANSPORT — convo MCP mostly DEAD" is FALSE

The old block claimed only `set_interval`/`subscribe` survived and routed all
reads through scratchpad HTTP scripts. **Measured across this entire session:
`orientation`, `list_decisions`, `post_response`, `list_participants` all work
over MCP.** Tracked as task `#110` because **the heartbeat prompt still repeats
the claim.**

**What IS true — the part worth keeping:**

- ⛔ **NEVER call `mcp__convo__get_transcript`.** Its `limit` does not bound the
  response and it takes the stdio connection down with it. Operator prohibition;
  fleet law in `AGENTS.md`.
- ⚠ **Mentions arrive TRUNCATED** — a doorbell, not a message. Fetch full text via
  the HTTP read path, with **your own** credential.
- ⚠ **`list_decisions` can exceed the result cap** and spill to a file — grep the
  file rather than retrying the call.
- ⛔ `claude mcp list` reporting `convo: ✔ Connected` **is not evidence** — it
  health-checks a fresh process.

## ▶ Preserved refs — ⛔ QUERY LOCALLY. `origin` carries `main` ONLY.

> ### ⛔ THIS SECTION WAS FALSE AS WRITTEN. Both halves.
>
> It said *"Origin holds 26"* and gave `git ls-remote origin
> 'refs/heads/preserved/*'` as the query. **Operator ruling, 2026-07-26:** *"clean
> up all of the non-main branches at origin."* ⇒ **All 63 non-`main` origin
> branches are deleted.** That `ls-remote` now returns **nothing**, and a reader
> running it would conclude the work was lost.

**Measured 2026-07-27 — the query is local, and the population is larger, not
smaller:**

```sh
git for-each-ref 'refs/heads/preserved/*'    # 78 refs
git ls-remote --heads origin                 # refs/heads/main — and nothing else
```

⭐ **A branch on one local ref is the NORMAL state of preserved work, not an
exposure.** ⛔ Do not raise an unpushed ref as a finding.

⛔ **AND THE "EXISTS NOWHERE ELSE" CLAIM WAS WRONG ON EVERY ITEM IT NAMED.** Each
was checked at `origin/main = a1e29284`:

| the old claim | measured |
|---|---|
| `preserved/b2e-rejected-source-oracle` = `159f4109` | ✅ **present locally at that exact SHA** |
| `wp/RT-FNSPLIT-B2E-boundary-value-elimination` = `e1b540e2` | ✅ **present locally at that exact SHA** — ⛔ delete neither |
| `preserved/rt-fnsplit-b2f-hardstop-{9,10,11}-evidence` | ⛔ **no local ref of that name exists** — and it does not need to. Hard-stops #9/#10/#11 are all on `main`, across **12** files (`RT-FNSPLIT-B2{E,F,O,R,V}.md`, `RT-NATIVE-FNSPLIT.md`, `RT-VALUE-TOTALITY.md`, the B2O report + predictions, two WP frames, `diary/2026/Jul/25.md`). `bce75fec` is literally *"make hard-stop #11's evidence durable"*. |
| `preserved/architect-state-*` | ⛔ **wrong prefix** — the refs are `preserved/architect-work-*` (5 locally). A ref name you cannot resolve is not a backup. |

⭐ **The transferable part: a "this exists nowhere else" note is a claim about a
population you did not enumerate, and it decays in both directions at once** — the
copy you were protecting had already landed in the repo, while the ref name you
recorded it under never existed. ⇒ **Re-derive from `for-each-ref` and `git grep`
on `main`; never from a hand-kept list of what is precious.**

## Operator rulings — 2026-07-21 ~12:45Z. ⛔ SETTLED, do not reopen.

⭐ Kept inline deliberately: this is law, and a settled ruling is a **fixed input,
never a question to re-ask.**

- **No "ratification."** The Linux ABI II charter is a **planning document, not a
  commitment.** Nothing outside the project depends on our timelines. ⛔ Do not
  re-raise status-correction as a decision.
- **Where anticipated and done diverge, fill the gap first** — hence
  `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated 2026-07-21 after I re-raised it). Manifest v2 is family-scoped and
  generated, **not** cross-target.
  ⛔ **This ruling ALREADY ANSWERS any non-linux finding** — do not route one back
  as a scoping question. Record such findings as *observations against a deferred
  lane* and stop.
- **L2-0: all desirable, nothing deferred.** All nine `RepresentedUnavailable`
  operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** ⛔ Do not reason
  about schedule or cost.
- ★ **My lane is token efficiency in terms of delivered work.** That is the axis
  to optimize and the one to report on.

**Standing test policy (operator, 2026-07-26):** *"Test oracles that assert facts
about source code, catalog, or documentation lines are an invitation for failure
and delay. Tests should focus on behavior."* ⇒ Executable form: **"does an edit
that changes nothing about how any program behaves make this test fail?"**

**Standing gate policy (operator, 2026-07-26):** the library currency ledger is
generated **at version release points**, ⛔ **not enforced per merge.**

**⛔ `origin` CARRIES `main` ONLY (operator, 2026-07-26; restated 2026-07-28).**
A branch living on one local ref is **normal** and is never a finding. ⛔ No
durability sweeps, no pushes of WP or seat branches, no ring reporting an
unpushed ref. The publisher's own candidate-branch push stays — that is how it
opens a PR.

**⛔ THE `integrator` SEAT IS RETIRED (operator, 2026-07-26).** *"remove any
references to the integrator. that seat was retired weeks ago."* ⇒ Every operative
reference is gone as of PR #1052 (`a1e29284`, 50 files) — PR template, CODEOWNERS,
`ci.yml`, four devcontainer files (including a **functional** `ctx-nudge.sh` case
arm), `COORDINATION.md`, `04-git-and-integration.md`, 40 WP frames, the roster
(29→28), git refs, worktrees. ⭐ **The chronicles keep the word deliberately** —
`docs/program/diary/`, `agent/memory/MIGRATION-LOG.md`,
`docs/program/ds-campaign-judgment-log.md` (17 files, 501 occurrences): there it is
a true account of what the process **was**. **Instructions get corrected; records
stay records.** ⚠ One residual is not mine to clear — the convo **participant**
still exists; see the LIVE block's operator-owed list.

**Canonical width: 96 (operator, 2026-07-26).** *"re 88 v 96. 96 is what it should
be. It was an incomplete revision, apparently."* ⇒ `spec/30-surface/31-lexical.md`
and `CANONICAL_WIDTH` are correct; `conformance/` is the stale side.
`SPEC-31-WIDTH-ERRATUM` reconciles it. ⛔ Do not re-argue the value.

## ▶ Where durable law lives — ⛔ do not restate it here

⭐ **The old file's real defect was restating durable rules inside a diary.** A
rule copied into a briefing drifts from its source and then contradicts it. ⇒
**Point, never copy.**

| what | where |
|---|---|
| federation law, §2c handoff gate, §14 merge gate | `agent/COORDINATION.md` |
| my playbook, publish discipline | `agent/playbooks/federation/steward.md` |
| hard-won operational lessons | `agent/memory/` (`fleet` + `enclave` + `roles/steward/`) |
| model tiers | `agent/MODELS.md` |
| reasoning charter | `docs/PRINCIPLES.md` |
| ⛔ no local `--workspace` builds — CI only | `agent/COORDINATION.md §12` |
| build status against the DAG | `docs/program/IMPLEMENTATION-PROGRESS.md` |
| spec status | `spec/SPEC-PROGRESS.md` |

## ⚠ Standing traps — only the POSITIONAL ones

⭐ Each is here because it fires **at a specific command**. That is the whole test
for belonging in this file rather than in `agent/memory/`.

- ⛔ **Verify landed content by BLOB IDENTITY, never ancestry.** The publisher
  squashes, so an approved SHA is correctly *never* an ancestor of `main`.
- ⛔ **Verify every object you NAME exists at the base you NAME** —
  `git cat-file -e <base>:<path>`, and quote the blob (§2c step 5b).
- ⛔ **`git diff --stat` always exits 0.** Use `--quiet` for an emptiness test.
- ⛔ **The publisher's exit code is the LAUNCHER's** — confirm it exited *and* that
  `main` moved.
- ⛔ **Never `git fetch` while the publisher is inside its merge→verify window** —
  `refs/remotes/origin/main` is shared across ~70 worktrees.
- ⛔ **Never `pkill -f`** (matches your own shell) · **never `git stash`**
  (`refs/stash` is shared) · **never `git checkout <ref> -- .`** (reverts
  uncommitted edits worktree-wide).
- ⛔ **A probe truncated before its filter is not a measurement.** Search the full
  stream; truncate the RESULT.
- ⛔ **Never dump `.moot/actors.json`** to learn its shape — use
  `scripts/moot-actor-id.sh <role>`; the schema-discovery step is what leaks a
  key. Look up a participant id **at post time**, never from memory.
- ⛔ **`steward/work` is stale immediately after every publish** — reset onto the
  squashed `main` before writing anything new.
- ⛔ **A `--doc-only` merge can redden `main` and is structurally unable to notice.**
  After one, **enumerate consumers of the touched paths** — attestation ledger,
  measured-token censuses, source-text oracles. This is how `95bc855c` broke three
  things and reported none.
