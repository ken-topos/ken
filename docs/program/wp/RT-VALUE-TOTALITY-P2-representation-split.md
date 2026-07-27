# `RT-VALUE-TOTALITY-P2` — the representation split

**Node:** `docs/program/issues/RT-VALUE-TOTALITY.md` · **owner:** runtime ·
**size:** L · **base:** `origin/main = 7e9cfc96`

Covers `AC-V4`, `AC-V5`, `AC-V6`, `AC-V8`, `AC-V9`, `AC-V10`, `AC-V12`.
⛔ Not `AC-V11` — that is `P3`, it does **not** depend on this phase, and it is
releasable independently.

> ## ⛔⛔ CORRECTED — THE THREE ROWS DO NOT EXECUTE. STAGE 1 IS **AUTHOR**.
>
> **The heading below said your acceptance oracle "already exists and is RED."
> That was wrong, and the instruction it produced — *"do not build a new
> oracle"* — would have blocked necessary work.** Measured by
> `runtime-implementer` on the branch tip: the realization convention here is a
> marker comment `// --- conformance: runtime/values/<row> ---`, and all three
> rows have **ZERO** realizations in `crates/`.
>
> | row | executable realizations |
> |---|---|
> | `closure-containing-aggregate-has-no-deceq` | **0** |
> | `closure-publication-rejected-transitively` | **0** |
> | `empty-capture-closure-is-not-static-reference` | **0** |
>
> ⭐ **With a positive control, because three zeroes pass for any reason:** the
> identical probe over the identical tree **does** find realized rows —
> `canonical.rs:900/931/960/982/1004`, `store.rs:571/592/602/617/638`, and
> `store.rs:875`. So the probe works and the absence is real.
>
> ⇒ **The three rows are behavior CONTRACTS, not runnable tests.** They exist as
> **rows**, not as **code**. ✅ **Corrected instruction:** author faithful
> realizations **under their exact conformance names**, covering **only** their
> stated behavior. ⭐ That is *completing* the existing oracle — ⛔ not
> substituting for it and ⛔ not front-running it. Keep the realization-census
> positive control, and keep `AC-V15`'s positive control independently.
>
> ⚠ **The general defect, and it is the fourth of this shape on my frames today:
> I asserted a thing EXISTS without checking it is EXECUTABLE.** A conformance
> row, a status marker, and a runnable test are three different objects. ⛔ Before
> a frame tells a ring *"the oracle already exists"*, grep for its **realization**
> and pair the count with a positive control.
>
> ---
>
> **Superseded heading, kept so the correction is legible:** *"your acceptance
> oracle already exists and is RED."*
>
> This is **not** a WP whose tests you invent. `SPEC-CLOSURE-BOUNDARY` and
> `SPEC-STORE-SPLIT` both landed, and the conformance corpus already names this
> phase's deliverable and marks it **`RED-UNTIL`**:
>
> | row | status text, verbatim |
> |---|---|
> | `runtime/values/closure-containing-aggregate-has-no-deceq` | **RED-UNTIL runtime value implementation is reconciled to `41 §2.1`** |
> | `runtime/values/closure-publication-rejected-transitively` | **RED-UNTIL runtime publication is reconciled to `41 §2.1`** |
> | `runtime/values/empty-capture-closure-is-not-static-reference` | **RED-UNTIL empty-capture publication follows `41 §2.1`** |
>
> ⇒ **`P2` is the work that turns those three rows green** — which, per the
> correction above, means **authoring their realizations and then making them
> pass**, not finding existing failures. All three are in
> `conformance/runtime/values/README.md`, and the same file's *Realization
> status* block names the exact sites (§2 below reproduces them measured).
>
> ⭐ Frame your evidence against **those rows' behavior**, not against a
> structural restatement of them. Operator test policy, 2026-07-26: *"Test
> oracles that assert facts about source code, catalog, or documentation lines
> are an invitation for failure and delay. Tests should focus on behavior."*

## ⚠ 0. RE-BIND AT POINT OF USE — a blob id in a message has a shelf life

⭐ **Second `runtime-implementer` finding, 2026-07-27, and it sharpens the rule
rather than contradicting it.** The release mention handed over node blob
`a7e7e47a`. By the time the ring read it, `origin/main` had advanced to
`dae0a7b3` and the node was **`5d062c38`** — because the Steward's *own*
bookkeeping commit edited exactly that file, **between the bind and the post**.

⚠ Direction: **immaterial here.** The advance was 6 insertions replacing the
node's *"has no frame"* sentence with a pointer to this frame — zero AC,
deliverable, or substrate change — and the **frame** blob was unchanged at
`b5e79055` across both SHAs.

⇒ **Binding by blob is necessary and not sufficient.** A blob id quoted in a
message is a **snapshot**, and the coordinator can invalidate it seconds later.
⛔ Do not treat a handed-over blob as the governing value; **re-derive it at the
point of use** (`git show origin/main:<path>`) and again whenever you resume.
⭐ The implementer's phrasing is the durable form: *"the habit is right; the
re-bind has to be at point of use, not once at kickoff."*

## 1. Fixed inputs — settled, ⛔ do not reopen

Each was ruled or landed before this frame. Verify them at the base if you
like — ⛔ do not re-litigate them.

| input | where |
|---|---|
| ordinary closures are opaque: **no** Ken-visible structural equality, `DecEq`, ordering, canonical hash, slot identity, or provenance | `spec/40-runtime/41-values.md §2.1` |
| closures are **transitively non-persistable** — publication rejects **before** bytes/digest/slot exist, and ⛔ MUST NOT substitute a pointer, ordinal, digest, or handle | `41 §2.1` |
| ⛔ empty-capture optimization **MUST NOT** silently promote an ordinary closure to `StaticCallableRef` | `41 §2.1` |
| `StaticCallableRef` and any future `FrozenClosure` are **separate explicit types** | `41 §2.1` |
| the fork *"does `Closure` still belong as a variant of `Value`?"* is ruled **(b)** — it does not | `dec_1dckq8c0f9xjv`, node §3 |
| the outer carrier split **already exists** — ⛔ do not invent one | node §3a; `ir.rs:487` / `:514` |
| in-process sharing, hashing, allocation and identity are **private runtime choices** | `spec/40-runtime/41-values.md §3, §3b` (landed by `SPEC-STORE-SPLIT`) |
| P1's iterative worklist traversal is the totality mechanism; ⛔ **no second traversal beside it**, ⛔ **no semantic `MAX_DEPTH`** | `wp/RT-VALUE-TOTALITY-P1-…md` D1/D2 |

### 1a. ✅ Two things that WERE open when the node was written are now CLOSED

⭐ **Both are measured at the base, and both would otherwise cost you an
escalation.** The node's redirect block instructed the Steward to surface them
*"to the Architect before P2 picks an arm."* `SPEC-STORE-SPLIT` landed in
between and settled them.

**1. The `AC-V8` / slot-id collision is GONE.** The node warned that a live
conformance row asserted *equality **is** slot id* while `AC-V8` requires
agreement with **canonical** identity — *"a real possible collision, not a
wording problem."* **Measured at `7e9cfc96`: all four rows are absent from
`conformance/` and `spec/`** —

```
runtime/values/equality-is-slot-id
runtime/values/dedup-shares-slot
surface/collections/structurally-equal-collections-o1-comparable
runtime/evaluation/det-sharing-dedups-by-slot
```

⚠ **Positive control, because a negative grep passes for any reason:** the same
probe over the same trees **does** find `runtime/values/closure-publication-…`
and `runtime/addressing/no-lattice-on-hot-path`, so it was not silently
returning empty. ⇒ **No Architect escalation is owed on `AC-V8`.**

**2. The two closure store rows are RETIRED.** `runtime/values/closure-content-
addressed` and `runtime/values/closure-distinct-env-no-collision` are absent
from `conformance/` (same probe, same control). ⭐ The tests bearing those names
still exist in `store.rs` — see §2d. **They now assert a retired contract**, and
the values README says so in its own words: *"Those controls assert the retired
contract and are not retained anchors."*

## 2. Measured substrate — at `origin/main = 7e9cfc96`

⚠ **This is a starting map, not a bound.** Two prior nodes on this arc shipped
against a census that was complete against its own unwritten notion of the
surface. Treat every table below as a **checklist you extend**, and report what
you add. ⭐ The `ABI-S3` ring did exactly that on 2026-07-27 and it was the right
call.

### 2a. The canonical carrier still carries `Closure`, and derives identity over it

`crates/ken-runtime/src/values.rs`:

| line | text |
|---|---|
| `:22` | `#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]` on `pub enum Value` |
| `:87` | `Closure { code_id: u64, captured: Vec<Value> }` |
| `:143` | `Value::Closure { captured: kids, .. } => out.append(kids)` — `detach_children` |
| `:194` | `Value::Closure { code_id, .. } => …` — `rebuild` |
| `:313` | `Value::Closure { captured: kids, .. }` — the hand-written iterative `Clone` |

⛔ `Clone` is **not** derived here (P1 replaced it). `Debug`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash` **are** — which is §2c of the node: three of the
exact capabilities `41 §2.1` forbids, granted to closures by the enum.

### 2b. The closure canonical encoding is live at three positions

`crates/ken-runtime/src/canonical.rs`:

| line | what |
|---|---|
| `:25` | `pub const CLOSURE: u8 = 0x09;` — the kind tag `41 §2.1` says Ken *assigns no more* |
| `:269` | production encode arm, with the comment *"Full canonical encoding of captured values … memcmp-exact, NOT a hash digest"* |
| `:456` | the `#[cfg(test)]` recursive **reference** encoder's twin arm |
| `:612` | `tag::CLOSURE =>` in `decode_canonical` |
| `:547`–`:554` | the decoder doc: *"`Closure` is covered because store adoption and independent recovery need it, and for no wider reason"* |

⭐ **`:456` is not optional cleanup.** `AC-V1b`'s whole value is that the two
encoders are compared; leaving a closure arm on one side of a differential whose
other side no longer has one is a silent asymmetry.

### 2c. The two disagreeing pairs `AC-V8` needs are at named lines

| site | normalization | the pair that disagrees |
|---|---|---|
| `canonical.rs:62` `minimal_limbs`, used at `:173` / `:188` | strips trailing zero limbs | `BigInt{limbs=[5]}` vs `BigInt{limbs=[5,0]}` — encode **identically**, compare **unequal** under the derives |
| `canonical.rs:218` (`:215` comment) | NFC at encoding time | two NFC-distinct spellings of one `String` — encode **identically**, compare **unequal** |

⛔ **These are unsound today, with no closure involved.** Removing `Closure`
from `Value` does **not** fix them. §3c of the node is explicit; ⛔ do not read
the ruling headline and skip it.

⚠ `:167`–`:190` and the corresponding reference-encoder lines are under **live
`SPEC-ALIGN-A1` stops** (`STOP-4` bignum tag `0x01`; `STOP-4/1` sign-magnitude
minimal-limb). ⛔ **`P2` may not alter those encodings.** This phase is a refusal
at one arm and an identity mechanism above it — **not an encoder redesign.**

### 2d. The store still content-addresses closures

`crates/ken-runtime/src/store.rs`: `:507`, `:557` (production adoption paths);
`:678` `closure_content_addressed`, `:697` `closure_distinct_env_no_collision`
(tests bearing the two **retired** row names, §1a).

### 2e. The operational carrier derives equality across `ClosureRef`

`crates/ken-runtime/src/ir.rs`:

```
:486  #[derive(Clone, Debug, PartialEq, Eq)]
:487  pub enum RuntimeValue {          … :499  ClosureRef { symbol, captured }
:513  #[derive(Clone, Debug, PartialEq, Eq)]
:514  pub enum RuntimeGroundValue {    … closure-free
```

⇒ **Pin 2 of the ruling is failing right now**, on the operational carrier.
⭐ Measured: **4** explicit `assert_eq!`/`==` sites name `RuntimeValue` directly
(out of 366 mentions) — the derive is nearly unexercised, which is *not* the
same as sound. §3c's lesson applies verbatim: **zero consumers is an unexercised
contradiction, not soundness.**

### 2f. `ken-foundation` carries an obsolete twin

`crates/ken-foundation/src/values.rs:10` — `#[derive(Debug, Clone, PartialEq,
Eq, PartialOrd, Ord, Hash)]`; `:64` the `Closure` variant, whose doc comment
still asserts *"Encoded inline (memcmp-exact) … so the 'equal slot ⇒
structurally equal' invariant is total."*
`crates/ken-foundation/src/canonical.rs:26` `CLOSURE = 0x09`, `:163` the encode
arm. `src/testing.rs:146` builds one.

⚠ Non-production — and a **shipped public validation model**. Leaving it is a
second, contradictory answer to the question this phase settles.

### 2g. ⛔ B2V's landed boundary lane has a `PersistentClosure` class — READ §5

`boundary_value.rs` (`:122` `PersistentClosure = 6`, `:367` `BoundaryClass::
Closure = 7`, `:655` the tag↔class row, `:2400`–`:2413` the adopt arm returning
`Value::Closure`) and `boundary_value_clif.rs` (~15 sites, `:6296` onward).
`boundary_value.rs:2277` reasons explicitly: *"`Closure` deliberately does NOT
go through … would be a second value taxonomy; `Value::Closure` already **is**
the …"* — **a premise this phase removes.**

⇒ **This is scoped OUT of `P2` and it is a named residual, not an omission.**
§5 states the boundary and the AC that keeps it honest.

## 3. Deliverables

### D1 — the canonical carrier loses `Closure`

Remove the ordinary-closure variant from `ken-runtime::values::Value` (an
explicit rename to `CanonicalValue` / `CanonicalGraph`, or an equivalently
sealed type, is preferred but ⛔ **the spelling is yours**). **This carrier alone**
may enter canonical encoding, hashing, interning, persistence, or slot identity.

Consequences you own, all measured in §2: the `tag::CLOSURE` constant and all
three `canonical.rs` arms (`:269`, `:456`, `:612`), the `values.rs` child-position
arms (`:143`, `:194`, `:313`), and the `store.rs` adoption paths (`:507`, `:557`).

### D2 — ordinary closures live only in the operational carrier

`RuntimeValue::ClosureRef` (or its replacement) is where an ordinary closure
lives, recursively permitting closure-containing runtime-local aggregates.

⛔ **It must not expose Ken-semantic `Eq`, `Ord`, `Hash`, `Canonical`,
persistence, or slot identity merely because it is one Rust enum.** Concretely:
`ir.rs:486`'s blanket `PartialEq, Eq` must go, or become reachable **only**
through an explicitly-named compiler-private route.

⭐ **The property, not the spelling:** generic code that requires `PartialEq`
on `RuntimeValue` must **fail to compile**.

> ### ⛔ CORRECTED 2026-07-27 — "4 explicit sites" was WRONG, and wrong in kind
>
> §2e said *"the migration is small — 4 explicit sites … if you find more, that is
> a report, not a stop."* **Measured to fixpoint with the compiler: 6 direct
> consumers and 25 transitive types.** It was also not "a report, not a stop" — it
> was a genuine fork needing a ruling, and stopping was correct.
>
> ⛔ **The error is one of kind, not arithmetic: a count of explicit sites cannot
> measure a derive removal.** A derive's cost is carried by **reachability**, not
> by call sites — so the number to state was the transitive closure, which is 25.
>
> ✅ **RULED (Steward, scope axis): route A — remove the derives.** The 25 include
> leaf metadata enums with no closure relationship (`RuntimeObligationStatus`,
> `RuntimeRecursionAdmission`, `RuntimeEffectBoundary`,
> `RuntimeLowerabilityStatus`), and that cost is accepted because:
>
> 1. ⭐ **Its direction is OVER-STRICT, not unsound.** The defect `D2` fixes
>    *grants* a closure equality `41 §2.1` forbids; route A merely *withdraws* an
>    equality those enums were permitted to have. ⛔ Not the same class — one
>    admits a forbidden operation, the other loses a convenience.
> 2. It is **contained entirely inside `ken-runtime`**.
> 3. ⛔ Route B enters `crates/ken-elaborator/src/erasure.rs`, which **collides
>    with kernel's `D5`** — more expensive *and* contended.
>
> ⛔ **A hand-written ~40-arm `PartialEq for RuntimeExpr` stays rejected**: one
> wrong arm is a **silent** miscomparison in a spec-governed carrier, with no
> compiler check.
>
> ⚠ **Still open and NOT mine:** whether `AC-V4`'s failure-to-compile control
> admits an *explicitly named* escape hatch (route B) or requires that structural
> closure equality not compile at all. **Architect's call.** If it admits one,
> route B revives and the contention question returns to the Steward.

### D3 — ⭐ THE `AC-V8` ARM IS CHOSEN FOR YOU: the **sealed canonical witness**

The ruling permits two structural answers. ⛔ **This frame selects the second,
and the choice is not yours to revisit** without a hard stop:

> Equality, order, and hash are exposed **only on a sealed canonical witness,
> defined FROM the canonical contract** — i.e. from the canonical bytes P1's
> iterative encoder already produces. ⛔ Not `#[derive(Eq, Ord, Hash)]` on the
> carrier; ⛔ not a canonical-by-construction carrier.

**Why, stated so you do not have to re-derive it:**

1. **It is the only arm that discharges `AC-V8` and `AC-V12` together.** The
   node's own table: canonical-by-construction buys *agreement* by constraining
   the carrier, and leaves the comparison walking structurally — so identity
   comparison stays process-aborting at depth, invisibly, because the AC it
   would ride is already green.
2. **Canonical-by-construction would require the public enum to enforce
   `minimal_limbs` and NFC at construction time** — which is an edit to
   `canonical.rs:167`–`:190` under live `A1` `STOP-4` rows (§2c). ⛔ Out of scope
   by the node's own §7 item 3.
3. Agreement becomes **definitional** rather than asserted: the witness *is* the
   bytes, so there is no second definition of identity to keep in step.

⛔ **A frame that left this implicit would have chosen arm 1 by default.** It is
stated here for that reason.

### D4 — the checked projection, sharing P1's mechanism

The **only** route operational → canonical is a **transitive, iterative,
fail-closed** projection that proves the whole graph closure-free and canonical
**before** any byte, hash, slot, or publication exists.

⛔ **No recursive adapter, and no second traversal mechanism beside P1's.** A
private recursive projection reintroduces the exact overflow P1 removed, one
layer out.

### D5 — the false doc text is EDITED

Replace, don't annotate:

- `canonical.rs:271`–`:272` — *"Full canonical encoding of captured values …
  memcmp-exact, NOT a hash digest"*
- `canonical.rs:551`–`:554` — *"`Closure` is covered because store adoption and
  independent recovery need it"*
- `ken-foundation/src/values.rs:64`'s variant doc — *"Encoded inline
  (memcmp-exact) … the 'equal slot ⇒ structurally equal' invariant is total"*
- `values.rs:1`–`:4` module doc — *"Compounds are content-addressed"* is now
  narrower than the carrier's contract; state the closure exclusion.

⛔ **An appended "see the new boundary" note leaves the false text operative,
and it is the text positioned to be believed by the next reader.**

### D6 — `ken-foundation`'s twin is retired in the SAME sweep

Remove its closure arm, closure encoder, and closure-content-addressing tests
(§2f) — **or** explicitly retire the crate's stale model, saying so in the
crate doc. ⛔ Not "it is only a bench, leave it": it is a shipped public
validation model, and the next reader has no way to tell which of the two
answers binds.

## 4. Acceptance criteria

⛔ **Each face gets its own isolated control.** Bundling means one control's
green is read as covering mechanisms it never exercised.

### `AC-V4` — the forbidden capabilities are UNREACHABLE, on BOTH carriers

No consumer may obtain structural equality, ordering, or a canonical hash of an
ordinary closure.

- ⛔ **A grep showing no current caller does not discharge this.** The claim is
  **reachability**. The positive control is that the forbidden operation **fails
  to compile**, or is statically absent from the type.
- ⚠ **Discharge it on the operational carrier too.** Removing `Closure` from
  `Value` and leaving `ir.rs:486`'s derive in place discharges **half an AC that
  reads as whole** (§2e).
- **Control:** a `compile_fail` doc-test (or equivalent) per carrier, **plus** a
  sibling that **does** compile for a closure-free value — otherwise "rejected
  because closures have no `Eq`" and "rejected because the test is malformed"
  are the same green.

### `AC-V5` — closure publication is REFUSED at the position the spec names

Green on `runtime/values/closure-publication-rejected-transitively`. Refusal
happens **before** canonical bytes, hash, slot, provenance, or publication
exist. ⛔ Not redaction, not substitution by a digest/pointer/handle, not partial
emission.

- ⛔ **Per-position arms are required.** A single value carrying closures in
  every position cannot prove the check is per-position. The row names them:
  directly, and nested as **record field**, **data constructor argument**,
  **array element**, and **map value**.
- **Positive control, from the row itself:** the closure-free canonical record
  **succeeds**. That is what proves the boundary — rather than the carrier shape
  — causes the refusal.

#### ⚖️ `AC-V5` CARRIER-DOMAIN AMENDMENT — Architect Decision `dec_1dccecns4c2fr`

**Resolved `resolved` by the Architect 2026-07-27T13:23:47Z; transcribed here
from the Decision object** (`RT-VALUE-TOTALITY-P2` exact `0d431840`, whose merge
Decision `dec_7bxah5q76mw6v` cites it). This **amends the five-position list
above**; it does not amend the normative transitive-refusal contract, which is
unchanged.

| position | disposition under this amendment |
|---|---|
| **direct** | **behavioral control required** — constructible today |
| **record field** | **behavioral control required** — constructible today |
| **constructor argument** | **behavioral control required** — constructible today |
| **array element** | **discharged STRUCTURALLY for this phase** |
| **primitive map value** | **discharged STRUCTURALLY for this phase** |

⇒ Direct, record-field, and constructor-argument are **the complete constructible
closure-bearing publication positions today**, and they require the isolated
behavioral controls now present.

**Why the last two discharge structurally** — three independent legs, all named
in the ruling:

1. the only closure-bearing carriers (`RuntimeValue`, `BoundaryClass`) have **no
   Array/Map child positions**;
2. canonical `Value::Array` / `Value::Map` **cannot contain an ordinary closure,
   because `D1` removes the closure variant**;
3. the **proved package `Map` is constructor data**, so its current operational
   child edge is the **already-covered constructor-argument path** — not a
   separate runtime `Map` producer.

⭐ Leg 3 is the one that closes the route a carrier-shape argument alone does not
reach: it answers *"what about the `Map` that demonstrably exists?"* by showing
that its child edge is a position already under control, rather than by asserting
no `Map` exists.

> ### ⛔ THIS IS NOT A PERMANENT WAIVER — the reopening trigger is normative
>
> **Adding any closure-capable operational Array/Map position REOPENS `AC-V5`**
> and requires **its own refusal arm plus a closure-free positive control**
> before that carrier may publish.

⚠ **The trigger does not live next to the code it governs.** A reader adding a
`Map` arm to `RuntimeValue` is standing in `ken-interp/src/lib.rs`, not in this
frame — so the obligation above is, today, reachable only from the frame and the
Decision. ⇒ Carrying it into the source is folded into the next Runtime WP that
touches that file (recorded by `runtime-implementer`, `evt_3hc3gxbt8jbdr`); it is
**not** a defect in `0d431840` and did not gate its publication.

⚠ Also stale-on-transcription, by construction: the candidate's in-source
realization note reads *"reported to the frame as an amendment request, not
settled unilaterally"* — true when written, and **this transcription is what
makes it stale.** The ruling explicitly sanctions the note as-is meanwhile
(*"the exact candidate's explicit realization note is acceptable meanwhile"*), so
⛔ no branch change was required and none was made.

### `AC-V5b` — empty capture does not promote

Green on `runtime/values/empty-capture-closure-is-not-static-reference`. An
ordinary empty-capture closure `\x. x + 1` is still refused by publication.
⛔ **Empty captures never change an ordinary closure's class** (ruling pin 5).

⚠ This is its own AC and not a reading of `AC-V5`, because it fails through a
different mechanism: `AC-V5` fails if the refusal is missing, this fails if an
*optimization* routes around a refusal that is present.

### `AC-V6` — the aggregate has no `DecEq`, and the doc text is edited

Two independent halves, both required:

1. Green on `runtime/values/closure-containing-aggregate-has-no-deceq`: the
   all-`Int` record compares structurally; the record with one `Int -> Int`
   field is **rejected**, and ⛔ MUST NOT compare a pointer, slot, code id, or
   captured environment.
2. Every doc string in **D5** is **replaced**. ⚠ Cheap check with a real failure
   mode: `git grep -n 'memcmp-exact'` returns **nothing** in `crates/`, and the
   same probe **does** find a string you know is present.

### `AC-V8` — equality/order/hash agree with CANONICAL identity

Delivered on the **sealed-witness arm** (D3). ⛔ Freezing `#[derive(Eq, Ord,
Hash)]` does not discharge this.

⭐ **The controls are already known and cheap, and BOTH pairs are required:**

| pair | assert |
|---|---|
| `BigInt{limbs=[5]}` vs `BigInt{limbs=[5,0]}` | encodings **identical** **and** the equality/order/hash verdict **agrees with that** |
| two NFC-distinct spellings of one `String` | same |

⛔ **One pair does not discharge it.** They fail through different mechanisms —
limb truncation vs character normalization — so a passing arm on one says
nothing about the other.

### `AC-V9` — the projection is transitive, iterative and fail-closed

It proves the whole graph closure-free and canonical **before** any byte, hash,
slot, or publication exists, sharing P1's mechanism.

- **Control:** run the projection at **the same depth `D` `AC-V1` exercises**,
  out of process, and ⚠ **state `D` as a number before running.** A control that
  projects nothing reports the same green as one that projects a deep value.
- ⛔ **And a negative arm:** a graph with a closure at depth `D-1` refuses, and
  refuses **without** having produced bytes. A refusal at depth 1 does not
  establish transitivity.

### `AC-V10` — `ken-foundation`'s model is retired in the SAME sweep

D6, delivered. ⚠ If you choose the "explicitly retire the crate's stale model"
route, that statement must be **in the crate**, where a reader of the crate
reaches it — not only in this WP's report.

### `AC-V12` — the chosen `AC-V8` mechanism is DEPTH-TOTAL

⛔ **This is NOT a clarification of `AC-V8` and must not be folded into its
text.** `AC-V8` pins *agreement*. This pins *totality of the comparison*.

- **Control per comparison operation** — `==`, `<`, and `hash` each get their
  own arm at `AC-V1`'s `D`, out of process. ⛔ **One arm does not stand in for
  the other three** — that is the `AC-V8` two-pair lesson on a different axis.
- ⭐ **Pin the mechanism, not a depth.** *"It survives `D = 131072`"* is green
  against one depth on one platform and re-derives nothing if the traversal
  changes. The claim is: **the witness is the canonical bytes, which P1's
  iterative encoder produces, therefore comparison is heap-bounded.** Cite the
  measurement beside it as corroboration.

## 5. ⛔ WHAT `P2` DOES NOT DISCHARGE — every residual gets a cell

⭐ **This section exists because three nodes on this arc in a row shipped a
representation whose eliminator was the next node's problem, and each residual
was found downstream.** Standing lesson:
`a-representation-node-must-name-who-eliminates-it`. It is named here, in the
frame, where you are standing.

| residual | who owns it |
|---|---|
| **B2V's `PersistentClosure` / `BoundaryClass::Closure` lane** (§2g) — a store-adoption path for closures, which `41 §2.1` forbids | the **FNSPLIT re-cut** (`SPEC-STORE-SPLIT` §7 item 1). ⛔ The re-cut must not re-land a persistent-closure lane |
| `RECUT 2`'s phase-closure artifact | unchanged hard gate; this phase does not relieve it |
| `AC-V11` — derived `Debug` depth-totality | **`P3`**, releasable independently |
| the two adversary findings on P1 (`AC-V1b`'s frozen `25`; `Step::Val` constructible in-module) | node §7; separate repairs, ⛔ they do **not** reopen P1 |
| ⛔ whether landing this dissolves FNSPLIT hard-stop `#11` | **nobody, yet.** ⛔ Do **not** write *"P2 unblocks B2F"* anywhere. `#11` is re-put to the Architect against the new representation **after** this lands; that re-ask is a deliverable, not a premise |

### 5b. ✅ `AC-V15` DISCHARGED — not production-reachable; ONE compile-forced edit

**Measured by `runtime-implementer`, ruled by `runtime-leader`.** ⛔ No stop-report
trigger: §5a's trigger is *"if it turns out reachable"*, and by the strongest
available measurement it is not.

⭐ **The verdict is per-half, deliberately — a single verdict would have hidden that
the three halves differ:**

| half | status | how measured |
|---|---|---|
| **producers** — CLIF emitters building `BoundaryClass::Closure` (`emit_closure_node`), and every `store.adopt(…)` caller in the repo | **test-only** | all inside `boundary_value_clif.rs`'s `#[cfg(test)]` mod, brace-matched span `2681–8117`; all 6 `emit_closure_node` callers at `:6504`–`:6921` |
| **classifier** — `boundary_disposition` mapping `LoweredVariant::Closure` → `RepresentedHandle{PersistentClosure, Closure}` | **production code, but DEAD** | ⭐ **rustc, not grep**: `ken-cargo check -p ken-runtime --lib` reports *"method `boundary_disposition` is never used"*, with the whole phase model alongside it |
| **consumer** — `BoundaryValueStore::adopt` (`pub fn`) → `canonicalize` → `canonical_image`'s `Closure` arm | **live production code, zero in-repo production callers** | `boundary_value.rs` has no `cfg(test)` mod at all; every `.adopt(` caller is in the test mod above |

⭐ **Positive control, because "never used" and "nothing reached it" pass for any
reason:** the same `--lib` build emits **17** warnings naming specific dead items
while leaving the rest of the crate's items un-warned. ⇒ It is not a build that
warns about everything.

⚠ **A dead classifier is not evidence of a live path** — and that is why the three
rows are reported separately rather than aggregated.

> ### ⛔ TRANSCRIBED RULING — `D1` owns ONE construction inside the scoped-out lane
>
> ⚠ **My §2g said "scoped out" without noting what crosses that boundary by
> necessity.** `D1` removes `Closure` from the canonical carrier, so
> `canonical_image`'s `BoundaryClass::Closure` construction **cannot compile** and
> `D1` owns it.
>
> ✅ **Replace only that construction** with the existing fail-closed escape/error
> shape. ⛔ **Do NOT alter** the tag/class/storage taxonomy, the disposition model,
> or the CLIF emitters. ⭐ *"This is a compilation consequence of the
> canonical-carrier split, not a re-cut of the B2V lane."*
> **Record the exact error path and the residual handoff.**

### 5a. ⛔ The `AC-V15` you owe on the B2V lane, because scoping it out is a claim

Scoping §2g out is only honest if the lane is **unreachable from a production
publication path**. So prove it:

**`AC-V15`** — no production publication path reaches
`BoundaryClass::Closure`. ⛔ **A negative claim, so it needs a positive
control:** the same probe must show a boundary class that **is** reached. If
the probe cannot distinguish the two, it has established nothing, and the lane
comes back into scope.

⚠ If it turns out reachable, **stop and report** — do not widen this WP on your
own authority. That is a Steward sizing call.

## 6. Validation — ⛔ TARGETED ONLY
> ### ⛔⛔ ADDED 2026-07-27 — `-p <crate>` IS NOT A BLAST-RADIUS ORACLE
>
> ⚠ **This section conflated two different questions under one instrument, and a
> ring paid for it.** Measured by `runtime-implementer` on `P2`:
>
> `scripts/ken-cargo check -p <crate> --all-targets` builds that crate and **its
> own** targets. ⛔ **It does not rebuild dependent crates** — so a downstream
> derive that can no longer be satisfied is **invisible** to it. On `P2` a
> `-p ken-runtime` check reported clean while `ken-elaborator` was left red; the
> failure surfaced only under `-p ken-interp` / `-p ken-verify`, which pull
> `ken-elaborator` in. The measured claim *"entirely inside `ken-runtime`"* was
> produced by an instrument that could not see outside `ken-runtime`.
>
> ⇒ **Two questions, and `-p` answers only the first:**
>
> | question | instrument |
> |---|---|
> | *does this crate still build?* | ✅ `-p <crate>` |
> | *does anything that DEPENDS on it still build?* | ⛔ **not `-p <crate>`** — build a **dependent** crate, or state that CI answers it |
>
> ⭐ **Trigger:** any change to a **public type**, or to a **derive on one**. A
> derive is part of a type's public contract, so removing one is a breaking change
> to every dependent crate — and its cost is carried by **reachability**, not by
> call sites.
>
> ⛔ **This does NOT relax the resource rule.** `--workspace` remains forbidden
> (operator, `COORDINATION §12`). The correct move is **one more targeted build of
> a chosen dependent crate**, not a wider one. ⚠ And if you leave the question to
> CI, **say so** — an unmeasured blast radius reported as "targeted run green" is
> the defect this note exists to prevent.
>
> ### ⛔ THIRD INSTRUMENT FINDING — a cargo re-run is NOT idempotent for REPORTING
>
> Measured by `runtime-implementer` mid-census, 2026-07-27: a per-crate loop
> reported **2** sites for `ken-elaborator`; the raw uncached run showed **4**.
> **Cargo had cached the failed `lib test` target and did not re-emit its
> diagnostics on the second invocation.**
>
> ⇒ ⛔ **A second `cargo`/`ken-cargo` invocation can report FEWER failures than
> the first while nothing changed**, because diagnostics are not replayed from
> cache. **A census must read raw first-run output — never a re-filtered second
> pass.**
>
> ⚠ This defeats the ordinary habit of *re-run and grep*: the grep is correct, the
> input is silently truncated, and the result looks like a smaller problem rather
> than a broken measurement. ⭐ If you must filter, `tee` the first run and grep
> the file.
>
> ---
>
> ⭐⭐ **Three instruments, three different blind spots, all found by measuring the
> measurement rather than trusting it:**
>
> | instrument | cannot see |
> |---|---|
> | `-p <crate>` | dependent crates |
> | `--lib` | test targets |
> | a re-run | diagnostics the first run already consumed |
>
> ⇒ **State what each named build ANSWERS and what it CANNOT SEE.** A validation
> list that names commands without naming their blind spots is a list of
> instruments, not a measurement plan.


```
scripts/ken-cargo test -p ken-runtime
scripts/ken-cargo test -p ken-interp        # store.rs / reifier change ⇒ FULL suite
scripts/ken-cargo test -p ken-foundation
```

⛔ **NEVER `--workspace`** on this box — it OOMs and stalls the whole fleet
(`agent/COORDINATION.md §12`). The full-workspace build, the `--locked` gate, and
the conformance suite run **in CI on GitHub**. Any "no regression" criterion here
means **green in CI**, never a local `--workspace` run.

⚠ **`-p ken-interp` in full is not optional.** This touches `store.rs` and the
interpreter's closure slots (`eval.rs:211`, `:215`) — a reifier/store change needs
the whole suite, not a targeted `--test`.

## 7. Contention

This rewrites `crates/ken-runtime/src/{values,canonical,store,ir}.rs` and
`crates/ken-foundation/src/{values,canonical,testing}.rs`.

⚠ **Check that file set against every WP in flight, not just the frontier.**
At the time of writing, Runtime is building **`ABI-S3`** in `ken-host` /
`ken-interp` / `ken-elaborator`. **`ken-interp` is a shared crate** — `ABI-S3`
adds host ops there while this phase changes closure slots. ⛔ Confirm the exact
in-flight file set with the Steward before you cut the branch.

## 8. Reporting discipline

- Cut `wp/RT-VALUE-TOTALITY-P2` **fresh from `origin/main`**. ⛔ P1 was
  squash-merged and its branch deleted on origin — there is nothing to continue.
- Report the branch cut and your slice plan before building.
- Report an unpushed ref and keep going; **the Steward pushes.** Agents never
  touch GitHub.
- Wrap markdown at 80 columns.
- ⛔ **Verify every fixed input at the base rather than taking it from this
  file.** If your measurement disagrees with §2, **your measurement wins** —
  report the disagreement and proceed; §2 is a checklist, not a bound.
