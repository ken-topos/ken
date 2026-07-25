# `RT-FNSPLIT-B2A-S` — defunctionalize retained body selection

**WP frame (Steward). Owning team: Runtime. Size: M. One branch, one merge
Decision.** Parent: `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md`.
Predecessor: `RT-FNSPLIT-B1R` (**landed `7151ae58`**).
Authority: Architect ruling `evt_6h5gw5c503n5z` + amendment `evt_25ynt8615r9sk`,
gated behind research advisory `evt_4w1rf45d4fkv3`.

> ## ⛔ WHAT THIS WP IS NOT. Read this before the objective.
>
> The frame this replaces (`RT-NATIVE-FNSPLIT-recut-B2a-emission-port.md`) is
> **RETIRED** — its `Retain`/`Replace` lists were inherited from `b077eb7a`, a
> branch that **never landed**, so it called a construction a port. ⛔ **Do not
> read it, and do not carry its vocabulary forward.** That vocabulary *is* the
> defect.
>
> **This WP is a deliberately narrow intermediate representation.** It:
>
> | DOES | does **NOT** |
> |---|---|
> | close retained-body **selection identity** | make B1R's semantic plane the emission source of truth |
> | retire symptom-inventory **entry 1** | create target functions |
> | install one closed consumer | remove whole-configuration specialization |
> | — | constitute scaling / O(n) evidence |
>
> ⛔ **Do not name or describe this as a semantic-plane emission port.** The
> milestone is *"defunctionalize retained body selection."* Bundling selection
> identity together with semantic authority under one name is the **same
> overclaim shape that cost B1R a round trip** — and the ring itself accepted
> that correction at `evt_2t9jdvmrtmhmf`.
>
> ★ **Symptom-inventory entry 2 — `lower_expr` re-lowering each retained body per
> call site in that call site's whole configuration — STAYS OPEN, deliberately.**
> Say so in the retro. It is the *same already-named predicate*, not a new
> defect, and it belongs to `RT-FNSPLIT-B2F`.

## Objective

Retained closure / work-item body selection currently keys on a **cloned
`RuntimeExpr`** — a dynamic property naming static code, which is the chain's
predicate. Replace that with a **static tag plus one closed consumer**: identity
is the preallocated `StaticOriginId`; the source occurrence is a **payload that
the tag selects**, never the thing that establishes identity.

## Fixed inputs — RULED, do not reopen

⚠ Every current-state claim below was verified on **`origin/main` = `70bd2c74`**
and is **perishable**. Re-verify at pickup; **if one is false, say so and
escalate — do not build around it.** (That clause is what produced this WP.)

1. **`StaticOriginId` is `pub(super)`** at `semantic_ir.rs:18`, re-exported
   nowhere; zero references outside `planning/`.
2. **`StaticTransitionPlan`** is `pub(in crate::cranelift_backend)` at
   `static_transition.rs:158`; built by `plan_static_transition_graph`
   (`:1396`), called once at `lowering/core.rs:35`, and **`drop`ped at
   `core.rs:204`**.
3. **`compile_expr_into_module`** (`lowering/core.rs:14`) is the compile entry
   and the **non-escape boundary** — the plan is consumed entirely inside it.
4. **`CompiledModule`** is at `compiled.rs:19`. ⛔ Nothing from the reference
   table may reach it or any runtime state.
5. **`Lowered::Closure { body, … }`** carries the cloned body; construction at
   `core.rs:532` / `:3551`, destructuring at `:265` / `:638`. Siblings
   `Lowered::DeclarationClosure` (`:517`, `:3575`) and
   `Lowered::ComputationalRecursorClosure` (`:441`, `:583`) are in the same
   family — **decide explicitly whether each is in the covered population, and
   say which.**
6. **The origin population is already preallocated** — `StaticOriginId` values
   are derived from planned node ordinals (`semantic_ir.rs:194`, `:231`).
   ⇒ You are **not** minting a new identity space; you are giving the existing
   one a resolvable payload.

⚠ **`static_transition.rs:1924` contains
`StaticOriginId(((index + 1) % plan.nodes.len()) as u32)`** — a rotate-by-one that
looks like an existing cross-wire control. **Determine what it is before writing
D6**; if it is already the swap discriminator, extend it rather than adding a
second one.

## Deliverables

**D1 — the dense table, allocated in the same walk.** Allocate the occurrence
reference in the **same planner source walk, at the same moment** as its
already-preallocated `StaticOriginId`. Store an exact **dense, compile-local**
table keyed by that ID. It must be **total and one-to-one** with the planned
occurrence population. **Lookup is range-checked and verifies the entry's stored
origin equals the index.**

**D2 — the lifetime, and the non-escape property that licenses it.** Keep the
table on a compile-local **`StaticTransitionPlan<'src>`** (or an equivalent
private companion owned by that plan). References **may** borrow the
root/declaration source trees **because the plan cannot escape
`compile_expr_into_module`**. ⭐ **Make that non-escape checkable, not asserted** —
it is the entire justification for introducing a lifetime here.

**D3 — visibility and naming.** Widen `StaticOriginId` **only** to
`pub(in crate::cranelift_backend)`. Spell the carrier field **`static_origin`**.
⛔ Never bare `origin` — `RecursorProducerOriginId` already spells that word on
these same records.

**D4 — ⭐ REMOVE THE OLD BODY CARRIER IN THE SAME DIFF.** Raw/cloned
`RuntimeExpr` bodies leave **every** retained closure/work-item representation the
slice covers. In particular **`Lowered::Closure { body: RuntimeExpr, … }` cannot
survive for that population beside the new tag.**

⛔ **Replacing identity while retaining the old body carrier preserves two
authorities and FAILS the slice.** This is the requirement most likely to be
softened under pressure, and it is the one that must not be. If D4 cannot be
completed for some member of the population, that is a **hard-stop**, not a
partial landing.

**D5 — one closed consumer.** Install `lower_static_origin(static_origin,
dynamic_env)` (name it as you like; the shape is ruled). It range-checks the
table, selects the borrowed occurrence **solely by the tag**, then invokes the
existing recursive lowering as **temporary payload semantics**. **Every** retained
closure application / resume in scope routes through it; **none** selects a body
directly.

★ **The recognition criterion is that the consumer is the SOLE point of
consumption.** A tag with a second consumption path is not a defunctionalization;
it is a tag with a leak.

**D6 — controls, each red at a named artifact.** Carry D0's two carrier controls
(they were defined *on* the carrier and so were not constructible in a plane-only
slice), plus the discriminators that matter at *this* boundary:

- **swap two equal-shaped origin entries** → wrong-body/reject control **reddens**;
- **duplicate / missing / out-of-range origin** → **loud planner failure**
  (`PlannerInvariant`, per `RT-PLANNER-ATTRIB-K` — an invariant violation is a
  compiler bug, not a capacity limit);
- **perturb the borrowed address without changing the ordinal mapping** → **no
  identity change.** ⭐ This is the predicate as an executable test: if identity
  moves when only the address moved, the tag is not authoritative;
- **replace a tag lookup with a pointer/content lookup** → **fails loudly.**

Each mutation applied at its **natural production site**, restored
**byte-identically**, verified with `git diff --quiet`.

**D7 — re-enumerate the carrier sites.** ⛔ The inherited **29/28** census
(`core.rs` 13 `E0063` + 16 `E0027`; `mod.rs` 14 + 14) is **evidence, not
authority** — the consumer boundary has changed. Re-derive it with a compiler
probe against the new consumer population and **state your window** (which files;
whether it includes `cfg(test)` and definitions).

## Acceptance criteria

- **AC-1 — identity is the ordinal, provably.** D6's address-perturbation control
  passes and the pointer/content-lookup control **reddens**. ⛔ A review that only
  reads the code does not discharge this; it needs the executed controls.
- **AC-2 — the table is total, injective, and range-checked.** Show the totality
  and one-to-one properties as **assertions in the tree**, each independently
  falsifiable. ⛔ A single composite check discharges none of them.
- **AC-3 — ⭐ ZERO cloned `RuntimeExpr` bodies remain in the covered
  population.** Grep-checkable, and **state the population explicitly** —
  including which of `Closure` / `DeclarationClosure` /
  `ComputationalRecursorClosure` are in scope and why. *A boundary you can grep
  is worth more than one you can argue.*
- **AC-4 — the consumer is the sole consumption path.** Enumerate every retained
  closure application/resume site and show each routes through D5. ⚠ **Name your
  window** — two of my own counts on `ATTRIB-K` were wrong because the window
  silently included a `fn` definition and a doc file.
- **AC-5 — the plan does not escape.** Mechanically show no reference reaches
  `CompiledModule` (`compiled.rs:19`) or runtime state. A committed structural
  test is worth more than prose here.
- **AC-6 — `fixed_k` unchanged:** still `8,8,8,8,8` against cap `8`, the
  pairwise-equal census row survives, `MAX_HELPERS_PER_STATIC_SOURCE` unchanged.
  *(carried from the retired frame's AC-4)*
- **AC-7 — no new opcode, no wildcard arm.** `semantic_ir.rs`'s six-opcode
  grammar unchanged and still exhaustive; show it. *(carried, retired AC-5)*
- **AC-8 — D6's controls each reddened at a named artifact**, each restored
  byte-identically.
- **AC-9 — no regression.** **Full** `scripts/ken-cargo test -p ken-runtime`, **no
  filter** — a reifier/minted-shape change ripples to sibling observation tests a
  targeted run cannot see. ⛔ Workspace, `--locked`, and conformance are **CI's**,
  never local (COORDINATION §12).
- **AC-10 — the claim in the retro is the narrow one.** The retro states that
  selection identity is closed **and that entry 2 / whole-configuration
  specialization remains open.** ⛔ An accurate diff with an overclaiming
  summary does not pass.

## Do-not-reopen guardrails

- ⛔ **No target functions, no calling convention, no persistent-store runtime.**
  All of that is `RT-FNSPLIT-B2F`, and it is **atomic** with switch-over and
  old-path removal. If you find yourself declaring a Cranelift function, you have
  left this WP.
- ⛔ **Do not report growth metrics as an acceptance argument** and do not tune for
  them. The scaling verdict belongs to B2F.
- ⛔ **Do not extend the six-opcode grammar** to make selection easier. A new
  opcode is a hard-stop.
- ⛔ **Do not re-litigate** the `pub(in crate::cranelift_backend)` visibility or
  the `static_origin` spelling. Ruled.

## Cadence — the chain is ONE STOP PAST a research pull

**Hard-stop count of record = 6** (the Steward holds it;
`docs/program/issues/RT-NATIVE-FNSPLIT.md` is the authoritative line and **wins
any disagreement**). The #6 pull **fired and is consumed**. **Next pull = #9.**
A review fold is not a hard-stop.

**Symptom inventory = 2 entries**, and ⛔ **the predicate question is already
answered** — entries 1 and 2 reduce to *a dynamic property must not name static
code*, so a third entry that also reduces to it is **evidence this chain's
emitter work is incomplete, not a new defect.** Say that rather than ruling it.

## Contention

**Scope is `crates/ken-runtime/**` only.** Verified at framing: the ledger
(`library/SOURCE-ATTESTATIONS`) attests
`crates/ken-runtime/src/cranelift_backend.rs` — **not** the `planning/` or
`lowering/` files this WP touches. ⚠ **If this slice edits
`cranelift_backend.rs` itself, tell me before you land** — that crosses the
ledger axis and I re-derive the consumer population first.

## Escalation — hard-stop rather than improvise

**Stop and tell me** if: **D4 cannot be completed** for any member of the covered
population · the table cannot be made total/one-to-one without inferring an ID
from pointer, content, clone order, or activation · a **fixed input above is
false** against the landed code · the non-escape property cannot be shown
mechanically · you need a **new opcode** or a **9th outer helper**.

⛔ **Do not resolve any of those by widening the tag with dynamic content.** That
is the predicate this whole chain exists to remove.

⚠ **My framing is the thing most likely to be wrong here** — it was on the WP
this one replaces. If the slice boundary does not survive contact with the code,
that is a finding, not a failure; splitting again is cheap.
