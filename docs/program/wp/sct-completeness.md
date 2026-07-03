# sct-completeness — make the kernel's termination checker accept more valid terminating programs (VAL2 #12 + Ackermann)

**Steward frame → Team Kernel (build). KERNEL / TCB LANE — this WP MODIFIES the
trusted root** (`crates/ken-kernel/src/sct.rs`), unlike the outer-ring gap WPs.
Its whole risk is soundness, so **Architect's approach-review is the central,
mandatory gate — not a light touch.** Goal: fix the **completeness** of the
size-change termination (SCT) checker so it *accepts* valid terminating programs
it currently *over-rejects*, **while preserving the termination guarantee** (it
must still reject **every** non-terminating program). Owner: **Kernel**. Gate:
**Architect approach-review + soundness** (preserves termination) + Kernel QA +
CI + **spec-leader confirms** whether the SCT completeness-class needs a
`spec/10-kernel` note. Findings → **Steward**.

Base: `origin/main`. Branch (pre-staged by Steward): **`wp/sct-completeness`**.

## The bug class — completeness, not soundness (why this is safe to attempt)

SCT today is **fail-CLOSED**: it rejects some **valid, terminating** programs
with `KernelRejected(NotTerminating …)`. That is an *over-rejection*
(completeness gap) — the safe direction. No unsound program is admitted; some
sound ones are wrongly refused. This WP moves the accepting set **outward**
toward the true set of terminating programs — and the load-bearing obligation is
that it **stays within** that set (never admits a divergent program). See
[[kernel-rejects-is-completeness-fix-is-where-soundness-converts]]: the rejection
is positive evidence the checker is live; the soundness risk lives entirely in
the **fix**.

## Two over-rejection shapes to subsume (one WP) — GROUND against landed `sct.rs`

**⚠ The root-cause descriptions below are *perishable* — verify each against the
landed `crates/ken-kernel/src/sct.rs` before coding; do not trust this frame's
mechanism claims over the code.**

- **(a) VAL2 #12 — nested sub-pattern split + flat-sibling-field recursion.** A
  `match` arm that combines a **nested** sub-pattern split with a recursive call
  on a **flat sibling** field trips `NotTerminating("idempotent self-loop")`.
  Suspected mechanism (verify): `enter_method` (`sct.rs:139`) peels exactly `n`
  leading `Lam` binders and assigns **positional provenances**; when a nested
  `Elim`/`match` is compiled into the method body, the nested pattern binders
  shift the indices `size_rel` (`sct.rs:116`) reads, so the sibling call's
  **strict size-decrease `↓` is mis-attributed** and never lands on the
  idempotent self-loop's diagonal → false rejection (`has_strict_diagonal`
  returns false, `sct.rs:92`). **Newly *reachable* after the #5 match-compiler
  fix** (`07d167f`) — pre-existing kernel limit, freshly hit. Also unblocks the
  parked `tree-traversal/KNOWN-GAP.md` VAL3 example (re-pinned to #12).
- **(b) Ackermann-style / lexicographic-descent nested recursion.** The known
  SCT gap: a genuinely terminating nested/lexicographic recursion (outer arg
  strictly decreases across the call, or the outer stays equal while an inner
  strictly decreases) whose descent SCT's current matrix construction doesn't
  capture → false rejection. **Ground the exact current-rejection shape against
  the landed code** — do not assume it is identical to (a)'s mechanism.

**If (a) and (b) require fundamentally different analysis extensions and one is
substantially larger, propose a decomposition back to Steward** (e.g. ship #12
now, Ackermann as a tracked follow-on) — do **not** silently drop one.

## The fix is to the ANALYSIS, never to the acceptance criterion

The accepting rule — *"every idempotent self-loop matrix has ≥1 strict `↓` on
its diagonal"* (`sct.rs:6-7`, `has_strict_diagonal`) — is **correct and stays
verbatim.** The completeness gap is upstream of it: the **call-edge / provenance
/ size-change-matrix construction** (`enter_method`/`collect_calls`/`size_rel`)
is *incomplete*, so a real strict decrease is missing from the matrix. **Fix the
construction so the true size-change edges are present; do NOT weaken the
acceptance test.** Any change that makes acceptance *easier* (a new early-accept
path, an `edges.is_empty() ⇒ accept` shortcut, dropping the strict-diagonal
requirement) is the exact anti-pattern [[sct-unapplied-self-reference-over-accepts]]
warns against — it would admit `c := c` / `loop := id loop` /
recursion-through-`map` and inhabit Bottom. **The criterion is sacred; only the
matrix feeding it becomes more complete.**

## Acceptance criteria

- **AC1 — SOUNDNESS PRESERVED (load-bearing, the whole gate).** A curated
  **adversarial divergent set STILL trips `KernelRejected(NotTerminating)`**:
  (i) the classic non-terminators `c := c`, `loop := id loop`,
  recursion-through-`map` ([[sct-unapplied-self-reference-over-accepts]]); **plus
  (ii) a genuinely-divergent near-miss for EACH newly-accepted class** — a
  nested-split program whose "sibling" call does **not** decrease, and a
  non-well-founded (equal-outer, equal-inner) lexicographic shape. These are the
  **discriminating negatives**: they share the *syntactic shape* of the
  now-accepted programs but diverge, so a fix that accepts them is the soundness
  hole. **The acceptance criterion is unchanged** — grep the diff: no new
  early-accept/short-circuit path, `has_strict_diagonal`'s strict-`↓` requirement
  intact.
- **AC2 — Completeness (#12).** The nested-split + sibling-recursion program
  (ground the exact shape from the #12 repro) **passes SCT** and **evaluates to
  the correct value** through the interpreter (accept *and* run, not just accept).
- **AC3 — Completeness (Ackermann).** A terminating Ackermann / lexicographic
  program **passes SCT** and **evaluates correctly**.
- **AC4 — Monotone, no regression.** `cargo test --workspace` green. Completeness
  is **monotone**: every program SCT accepted before is **still** accepted (the
  accepting set only grows). Together with AC1 (the divergent set still rejected),
  this pins the accepting set's new boundary: `{old accepted} ⊊ {new accepted} ⊆
  {terminating}`. A kernel change's blast radius is workspace-wide — validate the
  **full workspace**, not just `-p ken-kernel`
  ([[kernel-reduction-change-full-workspace-green]]).
- **AC5 — Trust surface: no GROWTH.** `trusted_base()` **unchanged** — SCT is
  *already* in the TCB; this makes its analysis more complete, it does **not**
  add a `declare_primitive`/`declare_postulate` or a new `Term`/`Decl` variant.
  The TCB's *surface* is identical; only the termination checker's *precision*
  improves. Confirm by grep (no new trusted declaration; no new variant).

## Guardrails (do-not-reopen)

- **SOUNDNESS FIRST — if you cannot clearly argue the fix admits only terminating
  programs, STOP and escalate.** Do not ship a termination checker whose new
  acceptances you can't justify. A single admitted non-terminator is a Bottom
  inhabitant, not a completeness win.
- **Fix the matrix construction, not the acceptance criterion.** No new
  early-accept path; `has_strict_diagonal`'s strict-`↓` requirement stays.
- **Ground the current-rejection mechanism against the landed `sct.rs`** — the
  root-causes above are perishable.
- **Subsume both shapes, or propose a decomposition** — never silently narrow to
  one.
- **This IS a kernel change** — there is no "kernel-untouched" AC here; the
  obligation is *guarantee preserved*, verified by the AC1 adversarial net + the
  Architect soundness gate, not by an empty diff.

## Sequencing

- **Gate:** **Architect approach-review** (vet that the analysis extension
  preserves the termination guarantee — the central, up-front gate for a
  trust-root change) → Kernel builds → **Architect soundness** on the candidate +
  **Kernel QA** + **CI**. **spec-leader confirms** whether the completeness-class
  needs a `spec/10-kernel` note (likely an implementation-completeness fix that
  the spec's "terminating recursion is accepted via size-change" already covers —
  à la #5 — but spec-leader makes the call; if `/spec` is touched, CV reviews).
- **Lane:** Kernel. Branch `wp/sct-completeness` off `origin/main`, pre-staged by
  Steward. **Independent of the in-flight `[State]` build (ken-elaborator/interp)
  and Map (packages)** — disjoint crate (`ken-kernel`), no contention; Kernel is
  idle and can start now.
- **Downstream benefit:** unblocks the parked `tree-traversal/KNOWN-GAP.md` VAL3
  example (currently re-pinned to #12).
