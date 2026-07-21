# SEAL-2 — carrier producer closure, over a *derived* enumeration

**Status:** DRAFT (not yet framed to a branch, not yet kicked)
**Owner:** Team Foundation · **Size:** M · **Base:** to be re-anchored on
`origin/main` at kick time (RT-PARITY is in flight and touches adjacent prose)
**Origin:** adversary hunt `evt_74mjc4txd9y1e` on SPAN-SEAL `cd4184b8` (S1 + S2)
**Blocks:** nothing. **Blocked by:** RT-PARITY closing (fleet is single-threaded)

---

## Why this exists

SPAN-SEAL landed a producer-closure oracle for `BufferSpan` and it **holds** —
the adversary attacked the real property from outside the oracle and every
forgery route rejects. This WP does not repair a live defect. It repairs the
**gate**, which guarantees materially less than the sentence it is carrying.

Two independent gaps, both reproduced against `cd4184b8`:

- **The oracle iterates one namespace at one position.** It asks whether the
  **head** of a result type, after Pi-walk and WHNF, is the carrier — over
  `env.globals` **only**. A declaration returning `Result E BufferSpan` is
  skipped (carrier present, not at the head). A **class field**
  (`class_env.classes[C].field_types`, `crates/ken-elaborator/src/classes.rs`)
  is skipped entirely — it never enters `env.globals`, yet is source-reachable
  by projection.
- **`TransferCount` has no gate at all.** Its closure is *true* (independently
  verified: head `{}`, deep `{}`, both forgery routes `UnresolvedCon`) but
  nothing fails if a future prelude decl breaks it — and RT-PARITY rests a
  soundness argument on it.

Neither gap is exploitable from source today: both need a seed producer to build
the wrapped value or the instance body, and there is none. **These are
prelude-leak vectors** — precisely how `write_all_advance_span` leaked in the
first place.

### The mistake this frame must not repeat

SPAN-SEAL's AC-3 was written three times and **each version named a
mechanism**: a Pi-codomain walk, then `GlobalEnv::lookup`, then "declarations
**and** constructors." Each implementation was *correct against my words* and
inherited that mechanism's blind spot. The adversary's findings are instances
**#4 and #5** of the same defect.

**So the enumeration is the deliverable, and the axes below are stated as
properties.** Note especially what the SPAN-SEAL loud-fail axis did *not* buy:
it panics on an id in `globals` that classifies as neither declaration nor
constructor. It closed the **classification**; it never closed the
**enumeration**. Making a total classifier over a partial domain does nothing.
**A loud-fail guard is only as closed as the domain it iterates.**

---

## Acceptance criteria

**AC-1 — The oracle is parameterized by carrier, and instantiated at every
sealed carrier.** One derivation, applied to `BufferSpan` and `TransferCount`.
Adding a third sealed carrier must be a one-line instantiation. No copy of the
walker per carrier.

**AC-2 — ★ The enumeration is DERIVED FROM THE ELABORATOR'S OWN STRUCTURE, not
from a hand-maintained list of namespaces.**

This is the whole WP; the rest is consequence. The derivation must range over
**every namespace in which a source-reachable producer can live**, and it must
be **structurally impossible to add a namespace to the elaborator without this
test noticing.** Bind the walk to the environment's own type structure — a
`match` on the environment/class-env fields that the compiler forces to be
exhaustive, or an equivalent construction — so that **a new namespace is a build
break, not a silent pass.**

If you find yourself writing a list of namespaces, you have built the thing that
already failed five times. A hand list is not closed; it is an enumeration
waiting for its next omission. The known-today members (`env.globals`,
constructors, class field types) are **fixtures to test the walker against**,
never the walker's definition.

**AC-3 — Closure over POSITION, not just head.** The carrier must be detected
**anywhere it occurs in a result type**, modulo definitional equality — under
`Result`, `Sigma`, pairs, records, and any other former — not only at the head.
The existing WHNF discipline (reduce before the Pi decision and after every
codomain step, carrying a `Context`) is retained and extended to the nested
positions. Retain SPAN-SEAL's two alias discriminators unchanged; they proved
each WHNF point independently load-bearing.

**AC-4 — Closure over SOURCE ROOT.** The oracle currently sees `{prelude +
Buffer.ken.md + IO.ken.md}`. A producer in any other catalog package is outside
its environment entirely. Range over every catalog package. If the set of roots
is discovered by a glob, the glob must **fail loudly on a root it cannot
classify** rather than silently covering fewer roots than it claims. *(Same
missing-glob-walker class as A3 — if A3 lands a shared walker first, consume it
rather than duplicating it.)*

**AC-5 — Loud failure on anything unhandled, on EVERY axis above.** Not only an
unclassifiable id: an unhandled **namespace**, an unhandled **type former** in a
result position, an unclassifiable **source root**. Each must fail with the
offending name, never be `?`-filtered, skipped, or defaulted. The failure text is
pinned by test.

**AC-6 — Every discriminator states what the PREVIOUS oracle would have done.**
SPAN-SEAL's ring did this unprompted and it is the difference between a test and
evidence. Each new arm carries its contrast — e.g. *"the head-only oracle derives
`{}` here; this test fails against it."* Both adversary escape families (wrapped
result, class field) and both `TransferCount` probes become permanent
discriminators, each demonstrably failing against the `cd4184b8` oracle.

**AC-7 — Production derives `{}` for both carriers**, on the full enumeration.
If it does not, that is a **live finding** — stop and route it to the Steward
before changing anything.

---

## Starting evidence — `adversary/SEAL2-repros @ 70a603da`

One file, `crates/ken-elaborator/tests/adversary_seal2_repros.rs`. Branch is
**local-only** (cut from `cd4184b8`, not on origin) — but every seat is a
worktree of the same repo, so the ref is already visible from your worktree; no
fetch from origin is needed. Run targeted: `scripts/ken-cargo -p ken-elaborator
--test adversary_seal2_repros`. Consume these; do not rediscover them.

**★ All six PASS today. The split is not pass/fail — it is property vs gap,**
and the distinction is load-bearing (adversary correction, `evt_26m2zd0zd1enn`;
my original "expected-fail today" framing was wrong):

- **4 PROPERTY tests** — deep wrapped-inclusive sweep, both forgery batteries
  (the `BufferSpan` one carries the Q3 transparency rejection), and
  `transfer_count_has_no_public_producer`. These assert something **true that
  must keep holding**. If one breaks, you have a live finding — route it.
- **2 GAP tests** — `gap_wrapped_return_*` and `gap_class_field_producer_*`.
  These pass **because the landed oracle is blind**. They pin the defect, not a
  property, and each carries an inline `SEAL-2 INVERTS THIS` note in its failure
  message so a green cannot be misread as success.

**AC-8 (from this) — SEAL-2 MUST INVERT THE TWO GAP TESTS.** If
`gap_wrapped_return_*` and `gap_class_field_producer_*` still pass **unchanged**
after your work, the producer enumeration was **not** closed. They are a
**completion check on SEAL-2**, not a regression check on SPAN-SEAL. This is the
cheapest honest signal that AC-2 actually landed.

### ⛔ Do NOT adopt the adversary's `mentions()` walker as the closure

Its `producers()` is already parameterized by carrier — that *is* the shape AC-1
wants — but the `mentions()` walker is **deliberately conservative and is marked
in-file as explicitly not a proposed fix**: it does not unfold `Const`s in
non-head positions, so `Option BufferSpanAlias` slips past it. It is strong
enough to *demonstrate* the gap and nothing more. **Adopting it wholesale
inherits a fourth blind spot and produces instance #6** of the exact defect this
WP exists to end. Use it as a fixture and a demonstration; derive the closure per
AC-2.

### Exploitability — do not overstate it in your prose

Both S1 families are recorded in-file as **prelude-leak vectors, not source-only
forgeries**. Each still needs a seed producer for the wrapped value or the
instance body, and there is none today. **Source cannot forge a span at
`cd4184b8`** — the four property tests are the evidence. Gate them because that
is how `write_all_advance_span` leaked, not because a live hole exists.

## Out of scope

- Any change to the sealed surface, the catalog, or the `writeAll` recursion.
  This WP touches the **gate**, not the product. A required product change is a
  finding to route, not a fix to make.
- `TransferCount`'s runtime bound. RT-PARITY's concern, separately tracked.
- The `write_all_exact_prefix_prop` catalog-legibility regression (adversary S3)
  — **folded into PX8-F-PROOF (F4)**, not here.

## Verification

Targeted only, per `agent/COORDINATION.md §12`: `scripts/ken-cargo -p
ken-elaborator --test px8f_buffer_io_surface` and the sibling px8 suites.
**Never `--workspace`** — workspace-green means green in CI.
