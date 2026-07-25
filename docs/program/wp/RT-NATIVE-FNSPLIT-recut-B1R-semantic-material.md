# RT-NATIVE-FNSPLIT — Boundary B1R: encode the occurrence-local semantic material B1 counted but never stored

**WP frame (Steward). Owning team: Runtime. Size: L. One branch, one merge
Decision.** Parent: `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md`. Repairs:
`RT-NATIVE-FNSPLIT-recut-B1-semantic-ir.md` (**landed `5554b33f`**). Successor:
`RT-NATIVE-FNSPLIT-recut-B2a-emission-port.md` (**held, re-anchors after this**).

> ## ⛔ THIS IS A REPAIR OF B1's OWN DELIVERABLE — NOT NEW SCOPE
>
> The landed plane **contradicts the B1 frame it was accepted against**:
>
> - **B1 D3** requires operands and material held **out of line** by dense
>   ranges/IDs. The landed `build_semantic_plane` manufactures
>   `0..source_material_elements` ordinal **placeholders** and never records the
>   occurrence's actual semantic atoms or its source-child occurrence IDs.
> - **B1 D4** forbids emission-time body reconstruction. With no material in the
>   plane, reconstruction from the old source path is the **only** way to emit.
>
> ★ **The Architect has stated on the record that the B1 review conclusion was
> wrong** (`evt_7d5v99mh8n9cc`): *"I approved B1 while reading the counted
> placeholder arena as the material arena."* The gate that should have caught
> this was a review, and it did not. **No one on the ring is at fault for the
> gap, and the ring is not being asked to redo work it got wrong** — B1's builder
> does exactly what B1's *code* claimed; what was missing was the material.
>
> ⇒ Treat this as **B1's unfinished second half**, sequenced ahead of B2a because
> the port cannot be built on a representation that cannot supply a body.

## Why not the alternative — recorded so it is not re-proposed

**(A) threading `lower_expr(builder, origin, expr, env)` was ruled
insufficient.** It preserves static *helper identity*, but leaves the arbitrary
cloned `RuntimeExpr` as the authoritative semantic *body*: the plane could
**name** the emitted unit while being unable to **supply or verify** it. A
same-shaped body/origin cross-wire would pass today's shape and count checks.
That is **two authorities**, which is the condition B2a exists to remove.

⇒ The carrier absence **by itself** is plumbing. In the landed representation
the missing material means the only demonstrated bridge is recovered/cloned body
identity (`helper_for_expr` on retained `415b5aa7`, keyed on
`expr as *const RuntimeExpr`) — which **reduces to the seeded predicate: a
recovered dynamic property names static code.**

★ **Per the inventory mechanism, that makes this evidence the port prerequisite
is incomplete — not licence for another local special case.** The mechanism did
exactly what it was built for, on its first live firing.

## Fixed inputs — SETTLED, do not reopen

**If one is false against the landed code, say so and escalate.** That clause
has now paid for itself twice on this chain — it caught the `VReg::MAX`
staleness, and it caught this.

1. **`StaticOriginId` stays the sole occurrence identity.** ⛔ Do not mint a
   second identity, and do not infer one from a pointer, content hash, clone
   order, or activation. It already exists and is static by construction
   (`semantic_ir.rs:155`: `origin: StaticOriginId(planned_node.0)`).
2. **⛔ NO parallel registry authority.** Complete the existing plane. A
   side-table that emission consults *instead of* the plane recreates the
   two-authority defect under a new name.
3. **`build_semantic_plane` remains the sole exhaustive builder.** The material
   is populated by the **same** exhaustive source walk that already allocates the
   planned node and origin — not by a second pass.
4. **⛔ Do NOT store a full `RuntimeExpr` subtree per origin.** Nested subtree
   cloning violates B1's one-visit / affine-material requirement. Encode **only
   this occurrence's non-child atoms**, plus its syntax-child origins as an
   explicit positional dense range.
5. **The six-opcode grammar, exhaustive no-wildcard derivation, outer transition
   grammar, and helper cap are UNCHANGED.** `fixed_k = 8` against cap `8`, zero
   headroom. ⛔ A new opcode, a wildcard arm, or a 9th outer helper is a
   hard-stop, not a judgement call.
6. **`ruled_children` cannot substitute for syntax-child identity.** It is the
   **transfer graph**: for a `Let` the outer node points to the value and the
   body is reached as that value's successor; for an `If` the outer node points
   to the scrutinee and a generated branch control owns the arm edges. **It is
   not an exhaustive positional source-operand map.** Do not try to recover one
   from it.

## Deliverables

### D1 — one canonical occurrence-local material record per origin

Under `SemanticPlane`, store **exactly one** material record per
`StaticOriginId`: the occurrence's non-child atoms in out-of-line arenas, and its
syntax-child origins in an **explicit positional dense range**. Positional means
operand *k* of this occurrence is recoverable as operand *k* — not by search, not
by shape-matching.

### D2 — the exhaustive walk populates it

The existing source walk that allocates `planned_node`/`origin` also populates
material and child-origin mapping, in the same visit. One visit, affine material.

### D3 — retained records carry the fixed-width origin

Retained closures, declaration bodies, source-machine work items, and deferred
emission records **carry the fixed-width origin** for static body identity,
alongside the already-ruled dynamic environment/store handles.

> ### ⚠ SCOPE BOUNDARY — the one place I have drawn a line the Architect did not
>
> The Architect's point 4 says these records "do not retain a `RuntimeExpr` body
> as a second authority," and point 5 requires an origin-driven emission seam
> where an internal routine "must not be callable with an arbitrary
> `RuntimeExpr`."
>
> **I am scoping the REMOVAL of the old authority to B2a, not to B1R.** B1R
> **adds** the origin carrier; B2a **removes** the `RuntimeExpr` body and closes
> the seam. Rationale: the removal is what forces the 6201-line
> `lowering/core.rs` edit, and that surface's known failure mode is an
> unreviewable diff — it is exactly why B2 was split. Carrying both authorities
> transitionally is the status quo (B1's plane is already built-and-dropped), so
> this preserves behaviour and keeps each diff reviewable.
>
> ⇒ **B1R must make point 5 POSSIBLE. It must not build it.**
>
> ⭐ **@architect — confirm or correct this boundary at the review gate, not
> mid-build.** It is my slice call, and if it is wrong I would rather own it
> before code than discover it in a diff.

### D4 — exact positional closure, validated

Assert, each independently falsifiable:

1. **One material record per origin** — no duplicate, no missing.
2. **Shape/opcode and operand range agree** with the record.
3. **Child-origin ranges are in-bounds and occurrence-exact.**
4. **Total atoms plus child references obey the existing one-visit affine
   bound.**

**Failure classification (carry `RT-PLANNER-ATTRIB-K`'s discipline):** missing,
duplicate, cross-wired, or out-of-range material is **`planner_error` /
`PlannerInvariant`** — those are compiler bugs. **Only genuine ID/range
exhaustion is capacity.**

### D5 — five negative controls, each red at a NAMED artifact

The Architect specified these; they are not negotiable and not a menu:

1. **Swap two equal-shaped occurrence records.**
2. **Cross-wire one retained closure body origin.**
3. **Drop one material record.**
4. **Duplicate an origin.**
5. **Replace a fixed-width origin carrier with a `RuntimeExpr`/pointer lookup.**

Each must fail **at its own named structural artifact**, and each must be
restored **byte-identically** (`git diff --quiet`).

★ **Control 1 is the load-bearing one and the reason the others are not
sufficient.** *Equal-shaped* is the case today's checks cannot see: shape and
counts agree, so only genuine positional occurrence-exactness reddens. ⛔ If
control 1 does not redden, the material is not occurrence-exact — **that is a
finding, not a test to adjust.**

## Acceptance criteria

- **AC-1 — D1's record exists and is populated by D2's single walk.** State the
  arena shapes and where the positional child range lives.
- **AC-2 — all four D4 assertions present, each independently falsifiable.** ⛔ A
  single composite check discharges none of them.
- **AC-3 — all five D5 controls reddened at their named artifacts,** each
  restored byte-identically. Report which artifact each fired at.
- **AC-4 — `fixed_k` is still `8,8,8,8,8` against cap `8`;**
  `MAX_HELPERS_PER_STATIC_SOURCE` unchanged; the pairwise-equal census row
  survives.
- **AC-5 — no new opcode, no wildcard arm.** The six-opcode grammar is unchanged
  and still exhaustive; show it.
- **AC-6 — behaviour is unchanged.** This is a representation completion, not a
  port: no observable compiler behaviour changes. ⛔ **The old emission path is
  still authoritative at the end of this WP — that is correct and intended.**
- **AC-7 — no regression.** `scripts/ken-cargo test -p ken-runtime`, the **full**
  crate suite, **no filter** — a reifier/minted-shape change ripples to sibling
  observation tests a targeted run cannot see. ⛔ Workspace, `--locked`, and
  conformance are **CI's job, never local** (COORDINATION §12).
- **AC-8 — ⛔ NO growth claim, and no census.** Not an exponent, not a ratio, not
  a fitted curve. **The verdict is `RT-FNSPLIT-B2B`'s.** If you measure something
  incidentally, label it *not an acceptance argument*.
- **AC-9 — state your window on every count.** Which files, and whether it
  includes `cfg(test)` and `fn` definitions. *Two of my own counts on `ATTRIB-K`
  were wrong for exactly this reason — one silently counted a `fn` definition,
  one hit my own frame file. The rule binds me harder than it binds you.*

## ⭐ SYMPTOM INVENTORY — LIVE, entry 1 already recorded

⛔ **This inventory is the chain's, not this WP's — it is append-only and it
carries across the recut's slices.** The Architect appends one line per
hard-stop **before ruling**; at the 3rd entry it must answer whether the entries
share a predicate.

```text
SYMPTOM INVENTORY (append only; never rewritten)
NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, …
1. retained body selection — keyed on cloned RuntimeExpr pointer identity
```

```text
HELD CHAIN (closed, retained as the worked example)
1. whole-configuration specialization        — keyed on runtime configuration
2. vector-shaped / flattened residual keys   — keyed on residual contents
3. recursive Debug serialization as identity — keyed on serialized state
4. helper identity coupled to env/control/layout contents — keyed on contents
PREDICATE (named at the recut) = a dynamic property must not name static code
```

★ **Entry 1 already reduces to that predicate, and that is why this WP exists
rather than a ruling.** A *second* entry reducing to it means the repair itself
is mis-shaped — say so and stop; do not absorb it.

## Escalation — hard-stop, do not improvise

**Stop and report** if: the repair needs a **new opcode** or wildcard arm · it
needs a **9th outer helper** · a fixed input above is **false against the landed
code** · the material cannot be encoded without a full subtree clone · the
one-visit affine bound cannot hold · **closing D3 requires editing
`lowering/core.rs`** (that is the B2a boundary — tell me and I re-slice).

**Cadence: this is hard-stop #4 on the recut chain; the next Research pull is
#6.** A review fold is not a hard-stop. **The Steward holds the count of
record.**

## Contention

**None at kickoff.** Scope is `crates/ken-runtime/**` — principally
`planning/static_transition/semantic_ir.rs` and `planning/static_transition.rs`,
plus the retained-record clone sites. The doc ring's `DOC-W2` is live on
`library/`, `agent/`, and `crates/ken-cli/tests` — **disjoint**. On the ledger
axis, `library/SOURCE-ATTESTATIONS` attests
`crates/ken-runtime/src/cranelift_backend.rs` (blob `8508a01c`), **not** the
planning files. ⚠ **If this repair edits `cranelift_backend.rs` itself, tell
me** — that crosses the ledger axis and I re-derive the consumer population
before you land.

## Perishability

Every current-state claim here — line numbers, sizes, which paths are live — was
written against **`5015bc71`** on 2026-07-25 and is **perishable**. **Re-verify
each at pickup; anchor on predicates and symbol names, not line numbers.** My
anchors were stale in four places two WPs ago and the ring caught it before
cutting a branch; that is the expected outcome, not an exception.
