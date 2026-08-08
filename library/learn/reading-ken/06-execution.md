# Execution

Chapters [01](01-anatomy.md)–[05](05-packages-and-provenance.md) taught you to
read a declaration, its contract, its assurance class, its authority, and its
provenance — all before anything runs. This chapter asks the question those
readings deliberately deferred: once the kernel has admitted a term, what
happens to it, and what does the runtime guarantee about
that? As with chapters 04 and 05, this is the chapter most able to turn a
gap in what this corpus exercises into an imagined gap in what the language
can do — so every claim below states which of the two it is.

## Execution Paths

A kernel-admitted core term can go two ways: the **reference interpreter**
(`X1`) walks it to a value directly; the **native backend** (`X3`) lowers it
to machine code first. Neither types, elaborates, or decides anything about
soundness — both consume a term the kernel has *already* checked, so a bug in
either produces a **wrong result or behavior**, never a false `proved`
([§1](../../../spec/40-runtime/45-native-backend.md#1-why-a-native-backend-and-where-it-sits),
[§2](../../../spec/40-runtime/45-native-backend.md#2-the-trust-posture--the-backend-is-not-in-the-tcb)).
The interpreter is the **reference**: it defines the meaning of a Ken program,
and everything else is judged correct by agreement with it
([§1](../../../spec/40-runtime/42-evaluation.md#1-relationship-to-the-kernels-reduction),
[§5](../../../spec/40-runtime/42-evaluation.md#5-the-interpreter-as-oracle-and-the-repl)).
The backend earns its trust the same way: not by inspection, but by matching
the interpreter over a differential corpus. Closure-free ground observations
are compared directly. A result containing a callable is observed only through
selected, well-typed projections or applications that produce closure-free
ground observations; closure identity and representation are never compared.
On any disagreement, the interpreter is right by definition, and the backend
is the defect
([§4](../../../spec/40-runtime/45-native-backend.md#4-the-differential-equivalence-discipline--the-interpreter-is-the-oracle)).
Neither is in the type-soundness TCB — the kernel already settled that; what
both earn is **`tested`**, the same assurance word chapter
[03](03-assurance-and-trust.md) taught you to read precisely, not `proved`.

The `ken` binary has distinct subcommands for these distinct jobs, and they are
not interchangeable: `ken check <file>` calls only the elaborator — it builds
an `ElabEnv`, elaborates every declaration, and stops; `ken run <file>`
elaborates, then also drives the result through the reference interpreter
against a host or mock host; `ken native-build <file> <dir>` elaborates and
lowers through the native backend to an executable
([CLI paths](../../../crates/ken-cli/src/main.rs), `check_file`/
`elaborate_cli_file`, `run_file`, `native_build_file`). A pure-library entry
(no `proc main`, the ordinary shape
for a `catalog/packages/` component) is validated with `ken check` precisely
because `ken run` rejects a pure library with no `main` entrypoint. That
rejection is not evidence against the entry; it only shows that the file is
not a runnable program
([§3](../../../docs/program/07-catalog-style-guide.md#3-code-block-roles-the-fence-taxonomy)).

Read that against what `ken check` calls: `check_file`
constructs an `ElabEnv` and elaborates the file — it never constructs a
`ken_interp` host, never calls `run_program`, and never invokes the native
backend. **None of the seven fragments this curriculum is built from declares
a `proc main`**. Each of the seven registered files has zero such
declarations, so **every "still checks" claim `fragments.md` makes rests on
elaboration alone** — the
kernel's own conversion checking, not a single step of the reference
interpreter or the native backend running on any of them. This is exactly
chapter [05](05-packages-and-provenance.md)'s shape, one layer down: a
corpus-usage gap (no registered fragment is ever run or native-built here), not
a claim that `ken run`/`ken native-build`/the engines behind them don't work —
the later runtime sections ground them from their own tests, precisely
because no fragment in this set can ground it for you.

The [System IO fragment](../../../catalog/packages/Capability/System/IO.ken.md)
states, about its checked
proof terms: "Exactly-once settlement and liveness remain runtime-enforced,
delegated boundary properties." Chapter [03](03-assurance-and-trust.md)
first showed you this sentence to teach `delegated`; read it for what it
says about *execution* specifically. The five lemmas above that sentence are
kernel-checked proofs about the shape of `writeAll`'s recursion and its error
handling — `ken check`-passing code. They do **not** claim that a single
write settles exactly once against a file descriptor, or that the loop makes
progress against a slow or failing device. That guarantee, if this entry were
driven by `ken run` against a host, would be the **effect driver**
performing the entry's `FS`-effect `Vis` nodes one at a time — perform,
observe, resume, in exactly the order they appear on the tree's spine
([§6](../../../spec/40-runtime/42-evaluation.md#6-effect-evaluation-running-the-interaction-tree)).
The live event and its durable trace are distinct. If an operation argument,
response, or terminal result contains an ordinary closure, the event still
occurs, but durable export refuses the whole trace before producing bytes or a
content hash. It does not redact the closure, substitute an identity, or drop
the event
([§6.4](../../../spec/40-runtime/42-evaluation.md#64-effect-sequencing-and-ordering)).
For other closure-free values, durable behavior is separate from in-process
storage. Values in the durably canonicalizable domain have deterministic
bytes. Proved `Map` and `Set` package trees instead preserve extensional
equality, ordered `to_list`, and durable round-trip; their internal bytes are
not observable. The runtime may copy, share, intern, or directly embed either
kind without changing those guarantees
([capacity §1](../../../spec/40-runtime/44-capacity.md#1-logical-values-are-separate-from-physical-storage)).
Nothing in this corpus exercises that path for this entry: it is a
pure-library component, checked, never run. The entry's own word,
"delegated," is therefore not a hedge — it is naming precisely the boundary
between what the kernel checked and what only a driven run could show.

## Marked Partiality

Ken's transparent checked core is **total** — definitions admitted there are
structurally recursive or SCT-certified and terminate on all inputs
([§1](../../../spec/40-runtime/43-termination.md#1-the-total-core)). But
partiality still enters, always
at a **marked** point, never silently. Five such points, each with its own
runtime behavior
([§2](../../../spec/40-runtime/43-termination.md#2-where-partiality-can-appear-and-is-marked)):

1. An **open verification hole** evaluates to `unknown` — the operational face
   of an unproven postulate, propagating strictly through everything except
   an eliminator branch it was never selected into
   ([§4](../../../spec/40-runtime/42-evaluation.md#4-unknown-propagation)).
   Read against chapter [03](03-assurance-and-trust.md): `unknown` is what
   an `unknown`-labelled claim does at runtime, not just a word on
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
   ([§3.3](../../../spec/40-runtime/42-evaluation.md#33-per-form-reduction-reconciled-with-17-1),
   "δ").
5. **Resource-limit exhaustion** — a runtime or deployment may declare a
   finite resource profile. When that limit is exhausted, the operation
   **MUST** report a typed `CapacityExhausted`-class failure before it could
   drop a value, alias unequal values, corrupt live state, or substitute a
   sentinel. The specification does not require the measured resource to be
   slots, values, bytes, pages, or table buckets. This case is distinct from
   the other four because the program stays logically total, so Ken generates
   no static "never exhausts" obligation — the stance is detect-and-fail-loud,
   not prevent-by-proof
   ([§2](../../../spec/40-runtime/44-capacity.md#2-capacity-and-loud-failure-oq-5-decided)).
   The [interpreter store](../../../crates/ken-interp/src/eval.rs) records its
   configured-limit error in `EvalStore::capacity_error`, and
   `capacity_tests::interp_loud_capacity_error_not_silent` drives that store to
   its limit and asserts that the error is recorded. This is evidence about
   one private store implementation. It does not make interning, slot counts,
   or deduplication part of the language contract, and it does not establish
   that every higher layer exposes the recorded error as a catchable fault.

None of the seven registered fragments contains an open hole, an opaque
non-total definition, or an unguarded partial primitive — this is a
statement about what this specific, small, deliberately-conservative
teaching set contains, not a claim that these traps are rare in general Ken
code; they are ordinary, named, and marked wherever they occur.

## Native Backend

Two checkable facts sit side by side here, and this page does not
resolve the tension between them. The native-backend specification
[§5](../../../spec/40-runtime/45-native-backend.md#5-the-backend-target--oq-backend-target-open-operator-ratifiable)
states plainly that the native backend's build effort **"does not start
until"** the target/toolchain decision (`OQ-backend-target`) is
operator-ratified, and the spec's own open-decisions register
([open decisions](../../../spec/90-open-decisions.md)) still records that
decision as **OPEN**, not ratified. At the same time, a Cranelift-lowering
backend is present in the tree. `ken native-build` calls it, and the
[native production tests](../../../crates/ken-cli/tests/px4b_native_production.rs)
drive programs through it and assert on their exit codes and output.

Read this precisely, without smoothing it over: the cited spec section's own
words gate the *start* of this work on a ratification the cited
open-decisions register says has not happened, and tested code
inconsistent with that gate exists anyway. Neither source authorizes
treating this as resolved — this page states both observed facts and leaves
the inconsistency exactly where it is, rather than supplying a reading that
would explain it away. A specification
record and an implementation can diverge, and noticing that divergence by
checking both directly is itself part of the reading discipline this
curriculum teaches — chapter [05](05-packages-and-provenance.md)'s citation
chain lesson applies here too, one layer up.

The differential discipline chapter
[45](../../../spec/40-runtime/45-native-backend.md) prescribes — the same term
through the interpreter and native backend, compared at closure-free ground
observations — has a test for that closure-free case. The
[native parity test](../../../crates/ken-cli/tests/rt_parity_native.rs)
contains six `assert_narrowed_alike` cases that run the same fixture through
both executors and assert on the exact result variant, not merely `is_err`.
All six are currently ignored. The five rows formerly waiting on
the byte-span repair did not re-arm: byte-span observation now succeeds, but
each path seat is also read as `SiteOperand(0)`, which still requires a
compile-time `Lowered` template. The sixth remains quarantined because a
runtime-local closure has no durable boundary lane. The file's only live test
calls `elaborates()`, a source-scope rejection check that does not run either
executor and is not a differential.

The binary remains excluded from the **sharded** test run, and a separate
`native-rt-parity` job still runs it. The required `build + test` job depends
on that job and checks its result. Read the resulting green precisely: it
shows that the dedicated job ran and that the live source-scope check passed.
It carries no current evidence that the interpreter and native backend agree
on the six narrowing cases. The differential is therefore **unavailable**
while all six remain ignored. Re-arming one cause would make it partial; only
re-arming every row makes it live.

Chapter [04](04-effects-capabilities-and-authority.md) now shows a checked
filesystem authority exemplar: an explicit `Cap a` parameter beside `[FS]`,
with elaboration controls for missing capability and authority-index
separation. Authority-as-signature is therefore **available** in checked
catalog form. The exemplar and its controls establish elaboration, not a
driven run. A program's `main` is resolved by an ABI-shaped name and supplied
capabilities by the host at the moment `ken run` drives it
([runtime entry](../../../crates/ken-cli/src/lib.rs), `run_program`), while
none of the seven registered teaching fragments declares a `proc main` with a
capability parameter. Nothing in this curriculum therefore exercises that
host-supply step. Label the execution evidence precisely: **partial** — the
checked corpus shows authority-bearing signatures, but not a driven program
whose execution is authority-gated. That is a corpus gap, not evidence that
authority-gating is unsupported. Chapter 04 keeps the checked surface and the
trusted host/runner complement distinct.

You can now separate elaboration from execution, identify the marked places
where partial behavior may enter, and treat the interpreter as the semantic
oracle for native differential tests. The backend implementation and its
required CI job are present even though the target decision remains open in
the specification, but the job currently carries no live interpreter/native
differential. The curriculum records both divergences instead of reconciling
them.

---

**Sources:**
[evaluation §§1, 3.3–6](../../../spec/40-runtime/42-evaluation.md#1-relationship-to-the-kernels-reduction);
[termination §§1–2](../../../spec/40-runtime/43-termination.md#1-the-total-core);
[capacity §§1–2](../../../spec/40-runtime/44-capacity.md#1-logical-values-are-separate-from-physical-storage);
[native backend §§1–5](../../../spec/40-runtime/45-native-backend.md#1-why-a-native-backend-and-where-it-sits);
[open decisions](../../../spec/90-open-decisions.md);
[CLI paths](../../../crates/ken-cli/src/main.rs);
[runtime entry](../../../crates/ken-cli/src/lib.rs);
[capacity producer](../../../crates/ken-interp/src/eval.rs);
[native differential test](../../../crates/ken-cli/tests/rt_parity_native.rs);
[native production tests](../../../crates/ken-cli/tests/px4b_native_production.rs);
[CI routing](../../../.github/workflows/ci.yml);
[registered fragments](fragments.md).
This explanatory chapter keeps specification, implementation, tests, and
corpus coverage distinct. It does not resolve their recorded backend
disagreement or extend the store-level capacity result to higher layers.
