# Solutions

Matches the numbering in [`exercises.md`](exercises.md). Each answer states
the primary source that decides it — open it directly if you want to verify
the answer yourself rather than take it on this page's authority.

## 01 — Anatomy

1. `print (text : String) : IO (Result IOError Unit) visits [Console]` — the
   row `[Console]` says the *only* thing `print` may do to the world is act
   on the `Console` effect; it rules out any other effect (`FS`, `Net`,
   `Clock`, `Rand`, …) regardless of what its body happens to contain
   (`catalog/packages/Capability/Console/Text.ken.md`;
   `spec/30-surface/36-effects.md`
   [§1](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row)).
2. `decide` is declared `fn` — a pure, computational definition. That keyword
   alone tells you it carries the empty effect row: it can compute, and
   nothing else — no `visits` clause is even possible on a `fn`
   (`catalog/packages/Core/Logic/EmptyDec.ken.md`;
   `spec/30-surface/36-effects.md`
   [§1](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row)).

## 02 — Types, contracts, and proofs

1. No. A passing `ken check` only confirms the `Proved` certificate itself
   validates — it says nothing, on its own, about whether the kernel's
   `trusted_base()` also carries no postulate for that same goal.
   `proved` requires **both** conjuncts, and the second one is not
   something `ken check`'s exit code establishes by itself; it requires a
   separate, explicit inspection of `trusted_base()`
   (`spec/20-verification/21-spec-syntax.md`
   [§5.4](../../../spec/20-verification/21-spec-syntax.md#54-the-honesty-guard-unknowntesteddelegated-never-read-proved)).

## 03 — Assurance and trust

1. No, not the same claim. Combinators' zero is literal and unqualified —
   its own producer test confirms `trusted_base()` is set-equal before and
   after loading the package
   (`catalog/packages/Data/Sums/Combinators.ken.md`;
   `crates/ken-elaborator/tests/surface_transport_acceptance.rs`,
   `transport_package_adds_zero_trusted_base_delta`). EmptyDec's "Zero new
   trust category" is narrower: its own Design notes name a possible
   instantiation (`dec_eq_decides Int`) whose `DecEq Int.sound` rides an
   axiom, so EmptyDec's zero covers only the *admission* of its own two
   inductives, not every instantiation a caller might choose
   (`catalog/packages/Core/Logic/EmptyDec.ken.md`;
   `crates/ken-elaborator/tests/ds1_empty_dec_acceptance.rs`,
   `ac3_trusted_base_delta_is_ordinary_inductive_admission_only`).
2. Something else: it is real, current, checked prose that shows *why*
   testing and proving are different activities — useful groundwork for
   understanding the concept `tested` names — but it is not an instance of
   the spec's formal, tagged `tested` construct (an `assume`/`test`-tagged
   clause), which remains proposal-level and unexhibited by any registered
   fragment
   (`catalog/packages/Tooling/Testing/Property.ken.md`;
   `spec/20-verification/21-spec-syntax.md` §5.2, §5.5).

## 04 — Effects, capabilities, and authority

1. Per the fragment's own second paragraph: "the current authority check is
   coarse and is **not** path-confined." `AFull` permits writes and deletes
   anywhere the host process can access — it does not yet narrow to
   particular paths, the kind of scoping `attenuate` is built to express
   (`catalog/packages/Capability/Filesystem/Errors.ken.md`).
2. True. A grep of every file under `catalog/packages/` for an explicit
   capability-typed signature, an `attenuate` call, or an authority
   comparison used as live code returns none — this was checked across the
   **whole** tree, not just this curriculum's seven registered entries
   (`library/learn/reading-ken/04-effects-capabilities-and-authority.md`
   §3).

## 05 — Packages and provenance

1. `List` genuinely is prelude (the taxonomy's closed prelude set is
   exactly `{Bool, Char, List}`). `Bytes` is actually **built-in**, not
   prelude — one of the five primitively-provided types the taxonomy names
   in its built-in set. `Result`, `Unit`, `Nat`, and `UInt8` are not named
   in either closed list the cited taxonomy sections state, so this
   curriculum does not classify them — that is an honest gap in what the
   cited source pins, not a fact derivable by guessing
   (`catalog/packages/Tooling/Testing/Property.ken.md`;
   `spec/30-surface/30-taxonomy.md`
   [§3](../../../spec/30-surface/30-taxonomy.md#3-the-built-in-set--the-surface-tcb-irreducible),
   [§4](../../../spec/30-surface/30-taxonomy.md#4-the-prelude-tier--ken-defined-always-present-closed)).
2. Not because `import` doesn't work — cross-file `import` is real and
   tested: a real acceptance test writes two separate files and resolves
   one's reference to the other's declaration through the roots-based
   loader
   (`crates/ken-elaborator/tests/n2_in_repo_loader.rs`,
   `cross_file_import_resolves_lazily_through_plural_root_api_and_caches`).
   The fragment's own word for what it is doing is "self-containment" — a
   style choice, the same idiom a proof-techniques guide entry uses
   elsewhere — not a claim that `import` failed or could not deliver
   `Transport`'s lemmas
   (`catalog/packages/Core/Logic/EmptyDec.ken.md`;
   `catalog/packages/Core/Logic/Transport.ken.md`).

## 06 — Execution

1. No — none of the seven declares `proc main`, checked directly in each
   file. It follows that every "still checks" claim this curriculum makes
   rests on **elaboration alone** (`ken check`, which never constructs a
   `ken_interp` host or invokes the native backend): neither the reference
   interpreter nor the native backend has ever run any of these seven
   fragments within this corpus. That is a corpus-usage gap, not evidence
   against either engine — both are grounded as real and tested from their
   own sources elsewhere
   (`crates/ken-cli/src/main.rs`, `check_file`/`run_file`/
   `native_build_file`; `library/learn/reading-ken/06-execution.md`
   §§2, 3, 5).
2. No. The five `lemma`s are proofs about the *shape* of `writeAll`'s
   recursion and its error handling — real, kernel-checked claims about
   the call-bound, the exact-prefix property, and first-error preservation.
   They are not proofs that a real write actually lands exactly once or
   that the loop actually makes progress against a real device — the
   fragment's own next sentence names that boundary explicitly as
   `delegated`, runtime-enforced, not proved here
   (`catalog/packages/Capability/System/IO.ken.md`).
3. No — and that is the honest answer, not a resolution. `spec/40-runtime/
   45-native-backend.md` §5 states the backend's build effort does not
   start until its target/toolchain decision is operator-ratified, and
   `spec/90-open-decisions.md` still records that decision as open, not
   ratified. At the same time, `crates/ken-runtime/src/
   cranelift_backend/` is real code in the tree, `ken native-build` is a
   working CLI subcommand that calls it, and `crates/ken-cli/tests/
   px4b_native_production.rs` drives real programs through it today. Both
   facts are independently checkable and both are true; this page states
   the inconsistency rather than reading past it — nothing in either
   source resolves it.
