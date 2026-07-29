# RT-EFFECT-DIFF — the reusable interpreter-vs-native observation boundary

**Owner:** Runtime · **Size:** L
**Node:** `docs/program/issues/RT-EFFECT-DIFF.md`

**Authority:** Architect `dec_3tawbngh6k761` (2026-07-29), the *separate row-3
obligation* clause. Research advisory `evt_6980s92jgvf4h`, row 3 and the
prior-art survey. Registration requested by the Runtime Leader,
`evt_3dxjc38x8w1sa`.

---

## 1. Fixed inputs, measured

All measured at `04cdce4eca585be8a3e9bcf42987bab5d5ba44be` (the rejected
RECUR-PORT candidate, preserved on `origin/wp/RT-FNSPLIT-RECUR-PORT`). ⚠ These
are **anchors for reading**, not a base to build on — build from whatever `main`
holds when this node is released.

| what | where |
|---|---|
| the narrow observation | `crates/ken-runtime/src/ir.rs:865-870` — `RuntimeObservation` limited to returned ground values or traps |
| the narrow decoder | `crates/ken-runtime/src/native_execution_differential.rs:2460-2497` — scalar `Int`/`Bool` only; trap decoding unavailable at `:505-527` |
| the existing differential | `crates/ken-runtime/src/native_execution_differential.rs:1-11, 2500-2555` — native vs runtime-IR vs interpreter lanes |
| the rich surface that already exists | `crates/ken-host/src/effect_v1.rs:2403-2412` — stdout, stderr, filesystem delta, terminal error, canonical effect trace, terminal class, exit status |
| a consumer of the rich surface | `crates/ken-cli/tests/rt_parity_native.rs:365-445` — identical source through native and interpreter; distinguishes exact public error selection **and dispatch skip** via canonical events |
| a second consumer | `crates/ken-cli/tests/rt_escape_second_resource_native.rs:100-133` — compares termination and effect-operation sequence |

⭐ **Both differential ingredients already exist. They do not meet at one
observation boundary.** That sentence is the whole node.

## 2. The design judgments, front-loaded

These are settled by the ruling and the advisory. ⛔ Do not re-open them; if one
is wrong, that is a hard stop to the Architect, not a local choice.

**J1 — one comparator, not two corpora.** Factor a single reusable comparator
plus fixture schema over `EffectObservation`. ⛔ Copying CLI assertions into a
runtime-local corpus is explicitly forbidden — two corpora drift, and the drift
is invisible until exactly the moment you need them to agree.

**J2 — a normalized projection is acceptable, but its omissions must be
declared.** You may compare a deliberately normalized projection of
`EffectObservation` rather than the raw surface. ⚠ If you do, **every omission
and every nondeterministic field must be explicitly classified**, in the artifact,
as omitted-and-why. ⛔ An undeclared omission is the failure this node exists to
end: a boundary that silently cannot see a class of divergence reports agreement
it never checked.

**J3 — inputs must be identical, and identity is enumerated.** Interpreter and
native adapters receive the same **source, arguments, environment, cwd, host
fixtures, and plan identity**. ⚠ Enumerated because a differential is only as
sound as its input equality, and `plan identity` in particular is the one a
representation change perturbs.

**J4 — report the FIRST divergence.** Not a diff of end states. The #18
diagnostic's entire value came from naming the first divergent observable at each
site; a comparator that reports only "outputs differ" would have produced none of
it.

**J5 — the interpreter stays the reference oracle.** Metamorphic relations
(same program under `RecursiveDescent` vs `FunctionizedUnits`, helper factoring
vs inline, harmless wrappers, semantically irrelevant plan renaming) are
**amplifiers**, not replacements. ⛔ They never adjudicate against the
interpreter.

**J6 — the CLI suites survive untouched** as an independent
packaging/integration backstop. ⛔ Not weakened, not filtered, not re-baselined,
not deleted once this lands.

## 3. Deliverables

**D1 — the comparator.** One reusable component: given two adapters and one
fixture, run both, compare the applicable terminal result/error, canonical effect
trace and order, stdout, stderr, filesystem delta, terminal class, and exit
status, and report the **first** divergence with enough locus to act on.

**D2 — the two adapters.** Interpreter and native, both driven from the identical
input tuple of **J3**.

**D3 — the fixture schema**, feedable from **both** runtime-local and CLI fixture
sources, so one fixture can be exercised from either side without duplication.

**D4 — the classification table.** Every field of `EffectObservation` marked
compared · normalized-then-compared · omitted, each omission carrying its reason.
⭐ This is the artifact that makes **J2** auditable, and it is a deliverable in
its own right — not a comment in the code.

**D5 — the seed corpus:** the RECUR-PORT divergence populations. All six measured
sites from `evt_3d41hdqe49pga`, plus the sibling populations the diagnostic closed
by mechanism:

```
rt_span_prov_native                     6/6
px8ta_oriented_subcontinuation          3/3
rt_parity_native                        7/7
px8f_buffer_native                      1/1
ken-verify::px8f_write_partition        1/1
rt_escape_second_resource_native        6/6
```

**D6 — generated combinations** across recursive matches, helper/declaration
calls, joins, traps, nested resources, provenance-sensitive spans, and effect
order. ⚠ Fixed suites are the thing that failed here; the advisory's Csmith
citation is about exactly this.

## 4. Acceptance criteria, each with its control

**AC-1 — the boundary catches what it was built for.** Reintroduce the row-2
defect (raw walk-order indexing in the `DeclarationCall` validator) and the
**backend-local** differential must red, naming a first divergence. ⭐ This is
the node's whole claim: it currently takes a dependent-crate CI job to see this.
⛔ A control that only demonstrates the comparator runs is not this AC.

**AC-2 — positive control on each compared field.** For every field marked
*compared* in **D4**, a fixture that diverges in **that field alone** reds.
⚠ A negative check passes for any reason, including never looking.

**AC-3 — the omissions are honest.** For every field marked *omitted* or
*normalized* in **D4**, a fixture that diverges only in that field is
demonstrated to **pass** the comparator, and that pass is recorded next to its
declared reason. ⭐ This converts **J2** from a promise into a measurement, and
it is the AC most likely to be skipped — it is the one that proves the boundary
knows what it cannot see.

**AC-4 — input equality is enforced, not assumed.** Perturbing any one element of
the **J3** tuple between the two adapters is detected and fails the run rather
than being silently compared.

**AC-5 — the seed corpus passes on a green `main`,** and each of the six D5
populations is exercised through the new boundary at its full file total.

**AC-6 — no CLI assertion changed.** `git diff` against the merge-base touches no
existing `ken-cli` or `ken-verify` test assertion. ⛔ Mechanical and binding.

**AC-7 — no second comparator, no second corpus.** One comparator; the CLI suites
still exist and still run.

## 5. Contention check

- **Files:** new runtime-local test infrastructure plus fixtures. ⚠ Touches
  `native_execution_differential.rs` and may touch `ir.rs` if `RuntimeObservation`
  is widened rather than bypassed.
- **In flight:** the RECUR-PORT row-2 successor is live in
  `crates/ken-runtime/src/planning/static_transition/semantic_ir.rs` and
  `static_transition.rs`. ⚠ **Not the same files**, but Runtime is
  single-threaded on one shared build turn — this node cannot start while the
  row-2 repair holds it, regardless of path disjointness.
- ⛔ **No spec or conformance scope.**

## 6. What this node does NOT own

- ⛔ **The row-2 repair.** Separate, ruled, and already in flight.
- ⛔ **Widening `RuntimeObservation` as an end in itself.** If the boundary can be
  built over a projection without touching it, that is fine. The deliverable is
  the comparison, not the type.
- ⛔ **Replacing the CLI suites.** They stay (**J6**).
- ⛔ **A verdict on the `FunctionizedUnits` representation.** `dec_3tawbngh6k761`
  ruled the #18 evidence is row 2 and that no representation reframe is
  authorized by it. ⚠ This node makes future divergence *observable earlier*; it
  does not re-open that question.

## 7. Release note for the Steward

⭐ **Registration alone discharges the `dec_3tawbngh6k761` gate** — RECUR-PORT may
close and RT-SCALE-B may run without this landing. ⇒ ⛔ Do not release this node
into the Runtime ring ahead of the ABI critical path on the theory that something
is blocked on it. Nothing is.

⚠ **The one live caveat:** `RT-SCALE-B` may not return a verdict asserting the
representation is *complete or verified* while this node is open. It may measure
and it may run.
