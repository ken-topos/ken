---
id: RT-VALUE-TOTALITY
title: "Make every total traversal of Value non-recursive in the host stack, and remove the closure capabilities the landed closure boundary forbids"
status: draft
owner: runtime
size: L
gate: none
depends_on: [SPEC-CLOSURE-BOUNDARY]
blocks: [RT-FNSPLIT-B2V]
github: null
origin: Architect cycle-contract ruling evt_5pzxf6sm4z08 ("host recursion may not be the totality mechanism -- a deep acyclic chain must adopt without host-stack growth and must not be reclassified as malformed") plus closure-identity ruling dec_3b1r19v59v20y / SPEC-CLOSURE-BOUNDARY. Steward-filed 2026-07-26 per COORDINATION §2 as move 2 of three from the closure-identity ruling: the repair is a BLOCKING DEPENDENCY for RT-FNSPLIT-B2V acceptance but a SEPARATE implementation slice, and must not be built as a B2V-local adapter. Scope was re-derived against the landed code rather than taken from the ruling's prose, which surfaced three mechanisms the ruling did not name.
---

> ## ⛔ THIS IS NOT A B2V-LOCAL ADAPTER
>
> `RT-FNSPLIT-B2V` cannot discharge its acceptance by wrapping a deep-value
> workaround inside its own layer. The recursion is in the **shared** `Value`
> traversals that every consumer reaches, so a B2V-local fix leaves every other
> caller overflowing. That is why this is its own node.

> ## ⛔ THE FRAME IS NOT WRITTEN, AND ONE FORK NEEDS THE ARCHITECT FIRST
>
> `status: draft`. §3 is a **design fork the Steward may not rule**. Do not
> release this to a ring until the Architect has answered it — the wrong answer
> gets implemented six times over, once per mechanism in §2.

## 1. What was ruled

Two rulings converge on the same shared type:

- **Cycle contract** (`evt_5pzxf6sm4z08`): cycles are malformed and fail closed,
  via **iterative tri-colour / worklist** traversal with postorder
  canonicalization. ⛔ **Host recursion may not be the totality mechanism.** A
  deep **acyclic** chain must adopt **without host-stack growth**, and must
  **not** be reclassified as malformed to avoid the problem.
- **Closure boundary** (`dec_3b1r19v59v20y`, landed as `SPEC-CLOSURE-BOUNDARY`):
  ordinary closures are runtime-local and opaque, with **no** structural
  equality, `DecEq`, ordering, canonical hash, slot identity or provenance, and
  are **transitively non-persistable** — durable export refuses the whole
  envelope before any bytes or content hash exist.

## 2. Measured state — six recursive mechanisms, not one

Measured at `origin/main` `dd9f4e76`. ⚠ The ruling named the canonical encoder.
**The type grants five more, and three of them are now spec violations rather
than robustness gaps.**

### 2a. `encode_canonical` is host-recursive at five sites, with no guard

`crates/ken-runtime/src/canonical.rs`:

| line | variant | recurses on |
|---|---|---|
| 109 | `Value::Constructor` | each argument |
| 119 | `Value::Record` | each field |
| 147 | `Value::Array` | each element |
| 164 | `Value::Map` | each entry value |
| 190 | `Value::Closure` | each captured value |

⛔ **A search for `worklist` / `tri-colour` / `iterative` / `MAX_DEPTH` /
`depth_limit` in `canonical.rs` and `values.rs` returns nothing.** There is no
depth guard, so a deep acyclic `Value` does not fail closed — **it overflows the
host stack**, and a Rust worker stack overflow may **abort the process** rather
than return an error. ⚠ That failure mode is why an in-process `join` alone does
not discharge a totality claim.

### 2b. `Clone` and drop glue are recursive over the same structure

`crates/ken-runtime/src/values.rs:10`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
```

Derived `Clone` recurses structurally; automatic **drop glue** recurses through
the nested `Vec<Value>` / `BTreeMap<_, Value>` owners. ⛔ **Drop cannot return an
error**, so a depth guard on the encoder does not make deallocation total — a
value shallow enough to *construct* can overflow while being *dropped*.

### 2c. ⛔⛔ The derive list itself now contradicts the landed spec

`PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` are derived on the **whole enum**,
which includes `Value::Closure`. That **grants ordinary closures structural
equality, total ordering, and hashing** — three of the exact capabilities the
landed boundary says they must not have.

⛔ **This is not fixed by "do not call it."** The capability is *reachable by any
consumer*, including generic code that requires `Ord` or `Hash` on `Value` and
never mentions closures. A prohibition the type does not enforce is not a
prohibition — it is a convention with a hole in it.

### 2d. ⛔ The closure canonical-encoding arm is now a spec violation

`canonical.rs:182-192` encodes `Value::Closure` as `code_id` + arity + the **full
inline canonical encoding of every captured value**, with the comment:

```
// Full canonical encoding of captured values (design doc §1.9):
// memcmp-exact, NOT a hash digest.
```

⛔ **That is a faithful implementation of the constraint the spec just removed.**
Under the landed boundary a closure must be **refused before bytes exist**, not
encoded. The doc comment on `values.rs` — *"code pointer + full canonical
captured environment … encoded inline (memcmp-exact)"* — is now **false text**,
not merely stale.

⚠ **This gap was created by landing the spec.** It did not exist yesterday, and
no seat has been told: the enclave's review scope was `spec/` and `conformance/`,
so nothing in that review could have seen `crates/`.

## 3. ⛔ THE FORK THE ARCHITECT MUST ANSWER FIRST

**Does `Closure` still belong as a variant of `Value`?**

- **(a) Keep it in `Value`.** Then the derives must go, and every capability
  (`Eq`, `Ord`, `Hash`, canonical encoding) must be re-provided **per variant**
  or behind a closure-free witness, so the type cannot hand a consumer a
  forbidden operation. Cost: every consumer that relies on `Value: Ord`/`Hash`
  must be inventoried and re-typed.
- **(b) Move it out.** Ordinary closures become a separate runtime-local carrier
  that is not a `Value` variant, so `Value` stays a closure-free content-
  addressed type and the derives remain sound **by construction**. Cost: the
  `Value::Closure { code_id, captured }` sites and the evaluator seam change
  shape.

⛔ **The Steward is not ruling this** — it is a component-design call, and
choosing wrong means implementing the wrong answer across all six mechanisms in
§2. ⚠ Note that **(b) is the option that makes §2c true structurally rather than
by enumeration**, and a fix covering a category needs a structural closure rather
than hand enumeration — but that observation is an input to the Architect, not a
decision.

## 4. Acceptance criteria — draft, and deliberately per-mechanism

⛔ **Each face below gets its own isolated control.** Bundling them means one
control's green is read as covering mechanisms it never exercised.

**`AC-V1` — deep ACYCLIC adoption completes with no host-stack growth.** A chain
deep enough to overflow the current recursive encoder must canonicalize and adopt
**successfully**. ⛔ It must **not** be reclassified as malformed, and ⛔ a
depth-limit rejection does **not** discharge this — the ruling requires success,
not a clean failure.

**`AC-V2` — cycles fail closed, and the control proves the guard is load-bearing.**
⭐ The strongest available control shape on this mechanism is known: **removing
the cycle guard must not merely redden a test — the uninstrumented failure is a
stack overflow that aborts the test binary.** So the control must run the
traversal **in an isolated process** and assert on the process outcome; an
in-process assertion cannot distinguish "guard fired" from "binary died".

**`AC-V3` — `Clone` and DROP are total at the same depth as `AC-V1`.** ⛔ A value
that constructs and encodes must also **clone and drop** without overflow. Drop
cannot signal failure, so this face needs its own control at the `AC-V1` depth,
exercising deallocation specifically.

**`AC-V4` — the forbidden closure capabilities are UNREACHABLE, not merely
unused.** After the fork is settled, no consumer may obtain structural equality,
ordering, or a canonical hash of an ordinary closure. ⛔ A grep showing no current
caller does **not** discharge this — the AC is about **reachability**, and the
positive control is that the forbidden operation **fails to compile** (or is
statically absent from the type), not that nobody calls it today.

**`AC-V5` — closure canonical encoding is REFUSED, at the position the spec
names.** Export refuses the whole envelope **before bytes or a content hash
exist**. ⛔ Not redaction, not substitution by a digest/pointer/handle, not
partial emission. The refusal arms must isolate **each independent position**
that can carry a closure, because a single value with closures in every position
cannot prove the check is per-position.

**`AC-V6` — the false doc text is EDITED, not annotated.** The `values.rs` and
`canonical.rs` comments asserting memcmp-exact inline capture encoding must be
**replaced**. ⛔ An appended "see the new boundary" note leaves the false text
operative and it is the text positioned to be believed by the next reader.

**`AC-V7` — the `Value: Ord`/`Hash` consumer inventory is stated BEFORE the type
changes.** Whichever fork branch is taken, enumerate what depends on those
bounds. ⛔ Not a post-hoc discovery during the build.

## 5. Armed triggers — ⛔ these are LINES TO RE-READ, not a tally to reconstruct

⚠ An unarmed count is not a trigger. On `RT-NATIVE-FNSPLIT` the chain reached
**10** hard-stops with **zero** research pulls, because the count lived only as
prose. Both lines below are re-read on **every** hard-stop.

```text
HARD-STOP COUNT (this node)  = 0
NEXT RESEARCH PULL           = 3rd hard-stop, then 6th, 9th, …
```

```text
SYMPTOM INVENTORY (Architect appends one line per hard-stop; NEVER rewritten)
NEXT PREDICATE CHECK = 3rd entry, then 6th, 9th, …
(empty)
```

⛔ **This node opening a fresh chain at 0 is a statement about a new
implementation surface, NOT a reset of the arc it came from.** The
`RT-NATIVE-FNSPLIT` chain stands at **10** with its catch-up pull armed at
**#11**, and that count is unaffected by anything here. ⚠ Filing a descendant
node must never be usable to launder a deep chain into a shallow one — if a
hard-stop here is *the same wall* the FNSPLIT chain kept hitting, it counts on
**both**.

## 6. Standing

- ⛔ **`RT-FNSPLIT-B2V` acceptance is blocked on this**, and `RECUT 2`'s
  phase-closure artifact must be **re-derived** against the settled
  three-lifecycle partition regardless — that remains a hard gate and this node
  does not relieve it.
- ⚠ **Contention:** this rewrites `crates/ken-runtime/src/canonical.rs` and
  `values.rs`. Check the file set against every WP **in flight**, not just the
  frontier candidates, before release. A `store.rs`/reifier change needs the
  **full** `-p ken-runtime` **and** `-p ken-interp` suites.
- ⛔ Targeted builds only — never `--workspace`; the full gate runs in CI.
- Report an unpushed ref and keep going; the Steward pushes. Wrap markdown at 80
  columns.
