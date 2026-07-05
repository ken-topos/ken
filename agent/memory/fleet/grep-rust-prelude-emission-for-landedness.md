---
scope: fleet
audience: all agents
source: merges former private memories
  `grep-ken-sources-misses-rust-emitted-prelude`
  and `negative-landed-claim-grep-the-rust-prelude-emission` (same CAT-4 `Perm`
  finding, independently written up twice)
related: check-main-via-git-object-store-not-find
---

# A landedness grep must check the Rust-emitted prelude too, not just `.ken`

Ken registers kernel globals **two ways**: surface `.ken` `view`/`data`
declarations, and **Rust-prelude registrations** in
`crates/ken-elaborator/src/prelude.rs` (`declare_def`/`declare_inductive` +
`elab.globals.insert("X", id)`). A `git grep '^view X'` / `.ken`-only grep, or
even a `spec/`-only grep, **misses the second class entirely** — it looks
exactly like "unimplemented" when the symbol is in fact landed and load-bearing.

**The concrete miss (CAT-4, the `Perm` cross-chapter reconcile, 2026-07-04).**
Both spec-leader and the Architect independently grepped `packages/**.ken` +
`spec/` for a landed `Perm` declaration, found nothing, and concluded a `37 §6`
vs `57 §3.1` `Perm` disagreement was pure spec-vs-spec prose contradiction (no
landed code affected) — an errata/doc-only fix. **Both grounding passes were
wrong.** `crates/ken-elaborator/src/prelude.rs:760-778` builds `Perm` directly
in Rust (`Term::Trunc` over `perm_rel_id`, `declare_def`,
`elab.globals.insert("Perm", perm_id)`) — a real, kernel-checked global, never
authored as `.ken` source text. It is also **test-consumed**, not dead: six
acceptance-test sites (`es2_acceptance.rs`, `l3a_acceptance.rs`,
`l3b_acceptance.rs`) assert `env.globals["Perm"]` directly. A believed doc-only
errata was actually a genuine build-affecting symbol-identity collision.

**How to apply.**
1. A **negative** landed-existence claim ("there's no landed prelude `Perm`", "X
   doesn't exist on main") is the **highest-risk** kind to accept on someone
   else's grep — it passes vacuously if their pattern couldn't have matched the
   real emission. Before building on "X isn't landed," also grep
   `crates/ken-elaborator/src/prelude.rs` for `globals.insert("X"` /
   `declare_def`/`declare_inductive`, and check for acceptance-test references
   (`env.globals["X"]`).
2. This binds a **co-reviewer's** plausible conclusion too, not just your own
   draft — re-derive from the producer, don't inherit someone else's grep result
   just because two people independently reached it (two independent greps with
   the same blind spot still corroborate nothing).
3. When you refute a negative-existence claim, lead with file:line ground truth
   and state the widened consequence plainly (here: doc-only → landed-symbol
   disposition) — being right on a load-bearing fact matters more than avoiding
   the correction.
4. Extends check-main-via-git-object-store-not-find with a second axis: it's not
   just about grepping the right *store*, but the right *layer* —
   Rust-prelude-emitted globals are a distinct category from `.ken`-authored
   defs, and both must be checked before a landedness claim is safe to build a
   scope decision on.
