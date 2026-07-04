# mutual-recursion-surface — a surface construct for mutual recursion (VAL2 #3)

**Steward frame → spec enclave (small surface-syntax OQ) → Team Language
(build).** VAL2 finding #3: no surface way to write mutually-recursive
definitions, though the **kernel mechanism already exists**
(`declare_recursive_group` + SCT termination). Owner: spec-leader resolves the
one surface-syntax OQ from existing grammar conventions; **Language** builds the
parser/elaborator wiring. Gate: **Architect approach-review** (the group must
route through SCT, per [[sct-unapplied-self-reference-over-accepts]]) + Language
QA + CI. Findings → **Steward**.

## Settled inputs — DO NOT REOPEN

- **The kernel mechanism exists** — `declare_recursive_group` + SCT already admit
  mutually-recursive groups with termination checking. This WP is **surface +
  elaborator wiring to that existing mechanism**, not a kernel change.
- **Termination is non-negotiable** — the group must go through **SCT** exactly as
  single recursion does; mutual recursion must **not** open an unchecked path
  (the whole group is one SCT problem). Kernel/`trusted_base` untouched.
- **SCT is now lexicographic-capable — the termination dependency is MET (do not
  treat it as a blocker).** When this gap was first recorded, its `KNOWN-GAP.md`
  flagged that a mutual group's termination "presumably" needs cross-function
  lexicographic descent, "similar to the Ackermann gap's unmet need." That
  capability has since **landed** — SCT (b) multi-measure / lexicographic descent
  merged (`e889284`, PR #256). So a mutually-recursive group whose termination
  measure spans the cycle lexicographically is now **checkable by the existing
  SCT** — this WP wires the surface/elaborator to it; it does **not** need any
  new termination-checker capability. (`isEven`/`isOdd` is a trivial structural
  case and does not even exercise the lexicographic path.)

## Release bundling (Steward)

Released **bundled with `surface-arrow-and-infix.md`** as one Language-lane
surface-syntax wave — both touch the same parser/elaborator surface, so one
branch + one gate avoids self-contention. The `#5` sequencing dependency both
frames cite ("after #5") is **cleared** — `L-match-ih-fix` merged (`07d167f`,
PR #236). Findings from both → **Steward**.

## The one OQ (spec-leader resolves from grammar conventions)

**Surface form:** a `mutual … and …` block vs. forward declarations vs. auto-
grouping of adjacent same-scope definitions. Resolve from Ken's existing grammar
conventions (how `data`/`where`/blocks already read); pick the one that composes
cleanly with the current definition syntax. This is a small surface-syntax
decision, not a semantics fork.

## Deliverable outline

1. **Surface syntax** (per the resolved OQ) for a mutually-recursive group.
2. **Elaboration** — the group lowers to `declare_recursive_group`; **the whole
   group is submitted to SCT as one termination problem** (no member escapes the
   check).
3. **Conformance** — a canonical mutually-recursive pair (e.g. `isEven`/`isOdd`)
   elaborates, passes SCT, and evaluates correctly; a **non-terminating** group is
   correctly **rejected** by SCT (the discriminating pair — accept the terminating
   group, reject the divergent one).

## Acceptance criteria

- **AC1 — Kernel untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
  reuses `declare_recursive_group` + SCT; no new kernel variant.
- **AC2 — Mutual recursion works + is termination-checked.** `isEven`/`isOdd`
  elaborates → SCT-accepted → correct values; a divergent group is **rejected**
  by SCT (discriminating pair — the termination check is load-bearing, not
  bypassed).
- **AC3 — No regression.** `cargo test --workspace` green; single-recursion + SCT
  behave identically pre/post.

## Sequencing

- **Gate:** spec enclave resolves the surface OQ (light) + elaborates if `/spec`
  grammar is touched → Architect approach-review (SCT routing) + Language QA + CI.
- **Lane:** Language. Branch off `origin/main`. After #5 / arrow-infix (shared
  parser/elaborator surface).
