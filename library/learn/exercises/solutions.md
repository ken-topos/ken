# Solutions

Matches the numbering in [`exercises.md`](exercises.md). Each answer states
the citation that decides it — open the source if you want to verify it
yourself rather than take the answer on this page's authority.

## 01 — Anatomy

1. `print (text : String) : IO (Result IOError Unit) visits [Console]` — the
   row `[Console]` says the *only* thing `print` may do to the world is act
   on the `Console` effect; it rules out any other effect (`FS`, `Net`,
   `Clock`, `Rand`, …) regardless of what its body happens to contain
   (`library/learn/reading-ken/04-effects-capabilities-and-authority.md`
   §1; `spec/30-surface/36-effects.md`
   [§1](../../../spec/30-surface/36-effects.md#1-effects-as-a-static-row)).
2. `decide` is declared `fn` — a pure, computational definition. That keyword
   alone tells you it carries the empty effect row: it can compute, and
   nothing else — no `visits` clause is even possible on a `fn`
   (`library/learn/reading-ken/01-anatomy.md` §3;
   `library/learn/reading-ken/04-effects-capabilities-and-authority.md` §1).

## 02 — Types, contracts, and proofs

1. No. A passing `ken check` only confirms the `Proved` certificate checks
   *and* the kernel's own `trusted_base()` carries no postulate for that
   goal — that conjunction, not the exit code alone, is what licenses the
   `proved` label
   (`library/learn/reading-ken/03-assurance-and-trust.md` §6;
   `spec/20-verification/21-spec-syntax.md`
   [§5.4](../../../spec/20-verification/21-spec-syntax.md#54-the-honesty-guard-unknowntesteddelegated-never-read-proved)).

## 03 — Assurance and trust

1. No, not the same claim. Combinators' zero is literal and unqualified —
   its producer test confirms `trusted_base()` is set-equal before and
   after loading the package. EmptyDec's "Zero new trust category" is
   narrower: its own Design notes name a possible instantiation
   (`dec_eq_decides Int`) whose `DecEq Int.sound` rides an axiom, so
   EmptyDec's zero covers only the *admission* of its own two inductives,
   not every instantiation a caller might choose
   (`library/learn/reading-ken/03-assurance-and-trust.md` §§2, 5, 6;
   `library/learn/reading-ken/05-packages-and-provenance.md` §2).
2. Something else: it is real, current, checked prose that shows *why*
   testing and proving are different activities — useful groundwork for
   understanding the concept `tested` names — but it is not an instance of
   the spec's formal, tagged `tested` construct (an `assume`/`test`-tagged
   clause), which remains proposal-level and unexhibited by any registered
   fragment
   (`library/learn/reading-ken/03-assurance-and-trust.md` §4;
   `spec/20-verification/21-spec-syntax.md` §5.2, §5.5).

## 04 — Effects, capabilities, and authority

1. Per the fragment's own second paragraph: "the current authority check is
   coarse and is **not** path-confined." `AFull` permits writes and deletes
   anywhere the host process can access — it does not yet narrow to
   particular paths, the kind of scoping `attenuate` is built to express
   (`library/learn/reading-ken/04-effects-capabilities-and-authority.md`
   §4).
2. True. This was measured across the **whole** `catalog/packages/` tree,
   not just this curriculum's seven registered entries, and is recorded as
   a standing fact, not re-derived per chapter
   (`docs/program/issues/CAT-CAPEX.md`;
   `library/learn/reading-ken/04-effects-capabilities-and-authority.md`
   §3; `library/learn/reading-ken/06-execution.md` §6).

## 05 — Packages and provenance

1. `List` genuinely is prelude (the taxonomy's closed prelude set is
   exactly `{Bool, Char, List}`). `Bytes` is actually **built-in**, not
   prelude — one of the five primitively-provided types the taxonomy names
   in its built-in set. `Result`, `Unit`, `Nat`, and `UInt8` are not named
   in either closed list the cited taxonomy sections state, so this
   curriculum does not classify them — that is an honest gap in what the
   cited source pins, not a fact derivable by guessing
   (`library/learn/reading-ken/05-packages-and-provenance.md` §3;
   `spec/30-surface/30-taxonomy.md`
   [§3](../../../spec/30-surface/30-taxonomy.md#3-the-built-in-set--the-surface-tcb-irreducible),
   [§4](../../../spec/30-surface/30-taxonomy.md#4-the-prelude-tier--ken-defined-always-present-closed)).
2. Not because `import` doesn't work — cross-file `import` is real and
   tested (a separate acceptance test resolves one file's reference to
   another file's declaration through the roots-based loader). The
   fragment's own word for what it is doing is "self-containment" — a
   style choice, the same idiom a proof-techniques guide entry uses
   elsewhere — not a claim that `import` failed or could not deliver
   `Transport`'s lemmas
   (`library/learn/reading-ken/05-packages-and-provenance.md` §§4, 5).

## 06 — Execution

1. No — none of the seven declares `proc main`. It follows that every
   "still checks" claim this curriculum makes rests on **elaboration
   alone** (`ken check`, which never constructs a `ken_interp` host or
   invokes the native backend): neither the reference interpreter nor the
   native backend has ever run any of these seven fragments within this
   corpus. That is a corpus-usage gap, not evidence against either engine
   — both are grounded as real and tested from their own sources elsewhere
   (`library/learn/reading-ken/06-execution.md` §§2, 3, 5).
2. No. The five `lemma`s are proofs about the *shape* of `writeAll`'s
   recursion and its error handling — real, kernel-checked claims about
   the call-bound, the exact-prefix property, and first-error preservation.
   They are not proofs that a real write actually lands exactly once or
   that the loop actually makes progress against a real device — the
   fragment's own next sentence names that boundary explicitly as
   `delegated`, runtime-enforced, not proved here
   (`catalog/packages/Capability/System/IO.ken.md`;
   `library/learn/reading-ken/03-assurance-and-trust.md` §3;
   `library/learn/reading-ken/06-execution.md` §3).
3. Yes, without contradiction, because they are two independently-checked
   facts about two different things: `spec/90-open-decisions.md` records
   that the backend's target/toolchain has not been formally ratified as a
   locked spec decision; `crates/ken-runtime/src/cranelift_backend/` (plus
   the working `ken native-build` CLI subcommand and its integration
   tests) is real code that exists and runs today. An open design-register
   entry means the choice isn't locked in normative prose — it does not
   mean no engineering has proceeded on the design ring's stated lean
   (`library/learn/reading-ken/06-execution.md` §5).
