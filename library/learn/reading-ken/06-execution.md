# 06 — Execution: what actually runs, and what the runtime honestly promises

Chapters [01](01-anatomy.md)–[05](05-packages-and-provenance.md) taught you to
read a declaration, its contract, its assurance class, its authority, and its
provenance — all before anything runs. This chapter asks the question those
readings deliberately deferred: once the kernel has admitted a term, what
actually happens to it, and what does the runtime honestly guarantee about
that? As with chapters 04 and 05, this is the chapter most able to turn a real
gap in what this corpus exercises into an imagined gap in what the language
can do — so every claim below states which of the two it is.

## 1. Two paths from one checked core

A kernel-admitted core term can go two ways: the **reference interpreter**
(`X1`) walks it to a value directly; the **native backend** (`X3`) lowers it
to machine code first. Neither types, elaborates, or decides anything about
soundness — both consume a term the kernel has *already* checked, so a bug in
either produces a **wrong value**, never a false `proved`
(`spec/40-runtime/45-native-backend.md`
[§1](../../../spec/40-runtime/45-native-backend.md#1-why-a-native-backend-and-where-it-sits),
[§2](../../../spec/40-runtime/45-native-backend.md#2-the-trust-posture--the-backend-is-not-in-the-tcb)).
The interpreter is the **reference**: it defines the meaning of a Ken program,
and everything else is judged correct by agreement with it
(`spec/40-runtime/42-evaluation.md`
[§1](../../../spec/40-runtime/42-evaluation.md#1-relationship-to-the-kernels-reduction),
[§5](../../../spec/40-runtime/42-evaluation.md#5-the-interpreter-as-oracle-and-the-repl)).
The backend earns its trust the same way: not by inspection, but by producing
**identical values** to the interpreter over a differential corpus — on any
disagreement, the interpreter is right by definition, and the backend is the
defect
(`spec/40-runtime/45-native-backend.md`
[§4](../../../spec/40-runtime/45-native-backend.md#4-the-differential-equivalence-discipline--the-interpreter-is-the-oracle)).
Neither is in the type-soundness TCB — the kernel already settled that; what
both earn is **`tested`**, the same assurance word chapter
[03](03-assurance-and-trust.md) taught you to read precisely, not `proved`.

## 2. What actually validates a fragment in this corpus — elaboration, not execution

The `ken` binary has distinct subcommands for these distinct jobs, and they are
not interchangeable: `ken check <file>` calls only the elaborator — it builds
an `ElabEnv`, elaborates every declaration, and stops; `ken run <file>`
elaborates, then also drives the result through the reference interpreter
against a real or mocked host; `ken native-build <file> <dir>` elaborates and
lowers through the native backend to a real executable
(`crates/ken-cli/src/main.rs`, `check_file`/`elaborate_cli_file`, `run_file`,
`native_build_file`). A pure-library entry (no `proc main`, the ordinary shape
for a `catalog/packages/` component) is validated with `ken check` precisely
because `ken run` on such an entry always fails — "last definition is not an
IO tree" — which is not evidence against the entry, only evidence that it
isn't a program
(`docs/program/07-catalog-style-guide.md`
[§3](../../../docs/program/07-catalog-style-guide.md#3-code-block-roles-the-fence-taxonomy)).

Read that precisely against what `ken check` actually calls: `check_file`
constructs an `ElabEnv` and elaborates the file — it never constructs a
`ken_interp` host, never calls `run_program`, and never invokes the native
backend. **None of the seven fragments this curriculum is built from declares
a `proc main`** (`grep -c "proc main"` on each returns zero), so **every
"still checks" claim `fragments.md` makes rests on elaboration alone** — the
kernel's own conversion checking, not a single step of the reference
interpreter or the native backend running on any of them. This is exactly
chapter [05](05-packages-and-provenance.md) §4's shape, one layer down: a real
corpus-usage gap (no registered fragment is ever run or native-built here), not
a claim that `ken run`/`ken native-build`/the engines behind them don't work —
§3 and §5 below ground that they do, from their own real tests, precisely
because no fragment in this set can ground it for you.

## 3. What running would show — grounded in a fragment's own words

`catalog/packages/Capability/System/IO.ken.md` states, about its own checked
proof terms: "Exactly-once settlement and liveness remain runtime-enforced,
delegated boundary properties." Chapter [03](03-assurance-and-trust.md) §7
first showed you this sentence to teach `delegated`; read it now for what it
says about *execution* specifically. The five lemmas above that sentence are
kernel-checked proofs about the shape of `writeAll`'s recursion and its error
handling — real, `ken check`-passing code. What they do **not** claim, in
their own words, is that a single write actually lands exactly once against a
real file descriptor, or that the loop actually makes progress against a real,
possibly slow, possibly failing device. That guarantee, if this entry were
ever driven by `ken run` against a real host, would be the **effect driver**
performing the entry's `FS`-effect `Vis` nodes one at a time — perform,
observe, resume, in exactly the order they appear on the tree's spine
(`spec/40-runtime/42-evaluation.md`
[§6](../../../spec/40-runtime/42-evaluation.md#6-effect-evaluation-running-the-interaction-tree)).
Nothing in this corpus exercises that path for this entry: it is a
pure-library component, checked, never run. The entry's own word,
"delegated," is therefore not a hedge — it is naming precisely the boundary
between what the kernel checked and what only a real, driven run could show.

## 4. Traps: where totality gives way, on purpose and in the open

Ken's checked core is **total** — the kernel admits only structurally
recursive or SCT-certified definitions, so an admitted, hole-free program does
not diverge
(`spec/40-runtime/43-termination.md`
[§1](../../../spec/40-runtime/43-termination.md#1-the-total-core)). But
"total all the time" is not the honest claim; partiality still enters, always
at a **marked** point, never silently. Five such points, each with its own
runtime behavior
(`spec/40-runtime/43-termination.md`
[§2](../../../spec/40-runtime/43-termination.md#2-where-partiality-can-appear-and-is-marked)):

1. An **open verification hole** evaluates to `unknown` — the operational face
   of an unproven postulate, propagating strictly through everything except
   an eliminator branch it was never selected into
   (`spec/40-runtime/42-evaluation.md`
   [§4](../../../spec/40-runtime/42-evaluation.md#4-unknown-propagation)).
   Read against chapter [03](03-assurance-and-trust.md): `unknown` is what
   an `unknown`-labelled claim actually *does* at runtime, not just a word on
   a page.
2. A **partial primitive** — division by zero, a non-wrapping overflow, an
   out-of-bounds index — either carries a refinement precondition that makes
   it total, returns `Option`/`Result`, or, unguarded, faults or yields
   `unknown`; the obligation to avoid it is generated statically, so this is
   a visible, provable concern, never a silent trap.
3. The **FFI/effect boundary** — a `foreign` call may diverge or fault outside
   Ken's control; it is a listed, trusted postulate, not a default.
4. An **opaque, SCT-rejected definition** never δ-reduces in the kernel's
   conversion (so it cannot corrupt type-checking), but the interpreter still
   unfolds it to run the program — the one place a pure, admitted program may
   **diverge at runtime**, an explicit, surfaced choice, never a default
   (`spec/40-runtime/42-evaluation.md` §3.3, "δ").
5. **Resource-limit exhaustion** — the content-addressed store hitting its
   capacity bound raises a loud, catchable `CapacityExhausted` at the `space`
   boundary. This is distinct from the other four: the program stays
   logically total, so Ken generates no static "never exhausts" obligation —
   the stance is detect-and-fail-loud, not prevent-by-proof
   (`spec/40-runtime/44-capacity.md`
   [§2](../../../spec/40-runtime/44-capacity.md#2-capacity-is-an-engineering-choice-not-numerology-oq-5)).

None of the seven registered fragments contains an open hole, an opaque
non-total definition, or an unguarded partial primitive — this is a
statement about what this specific, small, deliberately-conservative
teaching set contains, not a claim that these traps are rare in general Ken
code; they are ordinary, named, and marked wherever they occur.

## 5. The native backend: real, landed code — and a spec decision still open

Two facts are both true at once here, and neither cancels the other. The
**target/toolchain decision** for the native backend, `OQ-backend-target`, is
recorded **OPEN, operator-ratifiable** in the spec's own open-decisions
register (`spec/90-open-decisions.md`) — no target is written into normative
prose as decided. At the same time, a Cranelift-lowering native backend is
**real, landed, and under active development**:
`crates/ken-runtime/src/cranelift_backend/` exists, `ken native-build` is a
working CLI subcommand that calls it (`crates/ken-cli/src/main.rs`,
`native_build_file`), and a substantial `ken-cli` integration-test population
(for example `crates/ken-cli/tests/px4b_native_production.rs`) drives real
programs through it and asserts on their exit codes and output today. Reading
this precisely: an *open design-register entry* records that the toolchain
choice has not been formally ratified as a locked spec decision; it does not
mean no code exists — engineering work can, and here does, proceed on the
design ring's stated on-principle lean (Cranelift) while the operator
ratification remains open in the register. State the fact you can check
(the code exists and runs) separately from the fact the register states
(the decision is open) — neither one lets you infer the other.

The differential discipline chapter [45](../../../spec/40-runtime/45-native-backend.md)
prescribes — same term, interpreter and native backend, identical value —
has a real test for it:
`crates/ken-cli/tests/rt_parity_native.rs`, which runs the same fixture
through both executors and asserts on the exact result variant, not merely
`is_err`. **As of this chapter's `REVISION`, that specific test binary is
excluded from the sharded CI test run** — `.github/workflows/ci.yml` names it
explicitly in an `-E 'not (binary(rt_parity_native) or …))'` filter — because
one of its seven cases costs over three minutes of wall time on its own, not
because any case fails; this is recorded, in progress, and owned outside this
chapter (`docs/program/issues/CI-SKIPPED-NATIVE-TESTS.md`). Read this
precisely too: the differential discipline is real, and this specific test
file asserts it faithfully — but a green CI run today does not currently
execute this file, so it is not, right now, part of what a green CI run
tells you. That is a fact about **which suite currently has a running home**,
not about whether the native backend agrees with the interpreter — a claim
this chapter cannot make or refute from the suite's exclusion alone, and does
not attempt to fix or re-derive here.

## 6. Authority at execution time — the same unavailable gap, now at the boundary

Chapter [04](04-effects-capabilities-and-authority.md) §3 already showed you
that no checked fragment in `catalog/packages/` carries an explicit capability
token, an attenuation call, or an authority comparison. That gap has an
execution-time consequence worth stating plainly, not re-deriving: a real
program's `main` is resolved by an ABI-shaped name and supplied capabilities
by the host at the moment `ken run` drives it (`crates/ken-cli/src/lib.rs`,
`run_program`) — real, working machinery — but since no registered fragment
is a program with a capability parameter, nothing in this corpus ever
exercises that supply step for you to read. The catalog-wide measurement
behind chapter 04's claim was taken across the whole `catalog/packages/`
tree, not just this curriculum's seven entries, so this is the same
recorded gap, not a new one: **zero capability-typed signatures, `attenuate`
calls, or authority-lattice code exist anywhere in the catalog today**
(`docs/program/issues/CAT-CAPEX.md`). Label it, once more, precisely:
**unavailable** in checked-fragment form — the corpus does not yet show a
program whose execution is authority-gated, not that authority-gating is
unreal or unimplemented; §3's Filesystem/Errors.ken.md limitation and §1's
authority discipline (chapter 04) are both real, checked, or normative
sources that already establish the mechanism exists.

## Reader can now answer

- Given a `catalog/packages/` entry, what does its `ken check`-passing status
  actually tell you it went through — and what does it *not* tell you about
  whether the reference interpreter or the native backend ever touched it?
- What does a fragment's own word "delegated" name, precisely, when the
  fragment is never itself run — and what would running it actually exercise?
- Name the five marked points where a total, kernel-admitted program can still
  behave partially at runtime, and which one is the only one that can produce
  an outright non-terminating run.
- Why can both of these be true without contradiction: the native backend's
  target/toolchain decision is recorded open in the spec, and real, tested
  native-backend code already exists and runs programs today?
- What does it mean that a real differential test file can assert a true
  property faithfully while currently being excluded from the CI run that
  gates every merge — and why doesn't that exclusion, by itself, cast doubt
  on the property the file asserts?

---

**Grounds this page:**
`spec/40-runtime/42-evaluation.md` §§1, 4, 5, 6, and §3.3's `δ`/opaque-
definition rule;
`spec/40-runtime/43-termination.md` §§1, 2;
`spec/40-runtime/44-capacity.md` §2;
`spec/40-runtime/45-native-backend.md` §§1, 2, 4;
`spec/90-open-decisions.md` (the `OQ-backend-target` entry, cited whole-file,
no anchor — its own heading carries no stable slug worth pinning here);
`docs/program/07-catalog-style-guide.md` §3;
`docs/program/issues/DOC-W1.md`; `docs/program/issues/CAT-CAPEX.md`;
`docs/program/issues/CI-SKIPPED-NATIVE-TESTS.md`.
Authority class: `explanatory` — this page orders and interprets those
sections and the cited fragment/code's own text; it does not assert a rule
they do not already state. Every citation rests on the **content-currency**
predicate (`DOC-CURRENCY-ANCHOR`): the cited byte ranges are re-verified
unchanged between `library/REVISION` and `HEAD` by
`scripts/gen-doc-status.sh`. Content currency is necessary but not
sufficient — the same discipline chapter 05's footer states: a citation can
be byte-unchanged and still not carry the semantic claim made from it, which
is exactly the trap the ch05 review caught and this chapter was written
against from the start.

Section 2's "no registered fragment declares `proc main`" claim is grounded
in a direct grep of the seven registered files, not an assertion, and in
`crates/ken-cli/src/main.rs`'s own `check_file`/`run_file`/`native_build_file`
bodies, read to confirm which of `ken_elaborator`/`ken_interp`/the native
backend each subcommand actually calls. Section 5's landed-code claim is
grounded directly in `crates/ken-runtime/src/cranelift_backend/`,
`crates/ken-cli/src/main.rs`'s `native_build_file`, and
`crates/ken-cli/tests/px4b_native_production.rs` existing and passing at the
candidate SHA; its open-decision claim is grounded directly in
`spec/90-open-decisions.md`'s own recorded `OQ-backend-target` status — the
two are cited from two different, independently-checked sources precisely
because neither implies the other. Section 5's CI-exclusion claim is
grounded directly in `.github/workflows/ci.yml`'s own exclusion filter and
the linked, already-filed, in-progress issue — this page does not
re-derive, re-litigate, or attempt to close that issue; it reports the
state as of this page's `REVISION` only. Fragments cited are drawn from the
already-selected, registered set in [`fragments.md`](fragments.md); this
chapter does not introduce a fresh selection.
