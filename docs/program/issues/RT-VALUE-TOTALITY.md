---
id: RT-VALUE-TOTALITY
title: "Make every total traversal of Value non-recursive in the host stack, and remove the closure capabilities the landed closure boundary forbids"
status: ready
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

> ## ✅ PHASE 1 IS FRAMED AND READY. ⛔ PHASE 2 IS NOT — its frame does not exist.
>
> The work is **split into two phases**, and only the first is releasable:
>
> | phase | frame | covers |
> |---|---|---|
> | **P1 — totality** | ✅ `docs/program/wp/RT-VALUE-TOTALITY-P1-iterative-canonical-traversal.md` | `AC-V1` iterative encoder · `AC-V2` structural pin · `AC-V3` clone+drop |
> | **P2 — representation** | ⛔ **NOT WRITTEN** | `AC-V4`–`AC-V6`, `AC-V8`–`AC-V10`: carrier split, derives, closure arm, `ken-foundation` twin, checked projection |
>
> **P1 is first because P2's checked projection must SHARE P1's mechanism**
> (§3b pin 3 — *"no recursive adapter"*). If P2 ran first it would grow its own
> recursive traversal, which is the same defect one layer out. **P1 is also the
> only part on `RT-FNSPLIT-B2V`'s critical path.**
>
> ⛔ **Do not release P2 on the strength of the §3 ruling alone** — §3c corrects a
> premise a reader working from §2 plus the ruling headline would still get wrong,
> and the Steward owns that frame.

## 1. What was ruled

Two rulings converge on the same shared type:

- **Cycle contract** (`evt_5pzxf6sm4z08`): ⛔ **Host recursion may not be the
  totality mechanism.** A deep **acyclic** chain must adopt **without host-stack
  growth**, and must **not** be reclassified as malformed to avoid the problem.
  **That half binds here and is this node's whole job.**

  ⛔ **The cycle half does NOT bind on this carrier — corrected
  `evt_45x5dn9jcrhhq`, 2026-07-26.** This bullet used to read *"cycles are
  malformed and fail closed, via iterative tri-colour / worklist traversal with
  postorder canonicalization."* ⚠ **Every clause of that is wrong for `Value`,**
  and it is edited rather than annotated because it is the text a reader reaches
  first:
  - a cycle in `Value` is **unconstructible**, not malformed — so there is nothing
    to fail closed on, and the obligation is **retargeted to B2V's
    `BoundaryPersistentImage`** (see `AC-V2`, and `RT-FNSPLIT-B2V`);
  - **no tri-colour marking** — it would be a vacuous defence for an input the
    type cannot carry;
  - **no postorder** — measured, `encode_canonical` is a **streaming pre-order
    append** whose parent bytes never depend on child bytes. (`Clone` *is*
    postorder; they are different traversals.)
  - ⛔ **and no semantic `MAX_DEPTH`** — depth is not a validity predicate.
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

## 3. ✅ RESOLVED — Architect ruling, Decision `dec_1dckq8c0f9xjv`

**Ruled 2026-07-26 against exact `origin/main = fc63ca65`, resolved by the
Architect.** The fork was *"does `Closure` still belong as a variant of
`Value`?"*

> **Choose (b) — but NOT the naïve "move one enum arm and keep every derive"
> reading.** Ordinary `Closure` does **not** remain in the
> canonical/content-addressable `Value` carrier. It remains a runtime value in
> the separate runtime-local **operational** carrier.

⚠ **Not a duplicate of the prior closure-identity ruling.** That one settled the
*semantic partition* and deliberately pinned **no Rust enum layout** — but it
already rules out placing an ordinary closure inside a carrier whose component
contract is *"all members are canonical/store values."*

### 3a. ⭐ The outer carrier split ALREADY EXISTS — do not invent one

Verified at `fc63ca65`:

| type | site | shape |
|---|---|---|
| `RuntimeValue` | `ir.rs:487-505` | recursive, **includes** `ClosureRef { symbol, captured }` and closure-containing aggregates |
| `RuntimeGroundValue` | `ir.rs:512-526` | **closure-free**, the comparison-observation carrier |

⇒ **The natural component boundary is present.** The canonical store type should
model the **canonical subset**, not every operational result.

### 3b. The required representation boundary — five pins, all ruled

1. **Canonical/store carrier** — remove ordinary `Closure` from
   `ken-runtime::values::Value` (prefer an explicit `CanonicalValue` /
   `CanonicalGraph` name, or an equivalently sealed type). **This carrier alone**
   may enter canonical encoding, hashing, interning, persistence, or slot
   identity.
2. **Operational carrier** — ordinary closures live only in `RuntimeValue` or its
   replacement, recursively permitting closure-containing runtime-local
   aggregates. ⛔ It **must not expose** Ken-semantic `Eq`, `Ord`, `Hash`,
   `Canonical`, persistence, or slot identity **merely because it is one Rust
   enum**.
3. **Checked projection** — the *only* route operational → canonical is a
   **transitive, iterative, fail-closed** sealing/projection that proves the
   whole graph closure-free and canonical **before** producing any bytes, hash,
   slot, or publication. ⛔ It must share this node's non-host-recursive totality
   mechanism; **no recursive adapter.**
4. **Comparison** — closure-free observations continue through
   `RuntimeGroundValue` or an equivalent closure-free witness. ⚠ If compiler
   tests need representation comparison of operational IR, that must be
   **explicitly named, compiler-private, and observably irrelevant** — never a
   public semantic capability on runtime values.
5. **Other callable kinds** — `StaticCallableRef` and any future `FrozenClosure`
   stay separate explicit types with separately specified contracts. ⛔ **Empty
   captures never change an ordinary closure's class.**

### 3c. ⛔⛔ THE CORRECTION TO OPTION (b)'s PREMISE — read before sizing

**Moving `Closure` out does NOT by itself make the blanket derives sound.**
`Value` is **not canonical-by-construction today**, and the Architect grounded it
on two encoder-time normalizations the public enum does not enforce:

| site | what encoding does | consequence |
|---|---|---|
| `canonical.rs:62-70` + `:75-98` | `minimal_limbs` **strips trailing zero limbs** for `BigInt` / `BigDecimal` | `limbs=[5]` and `limbs=[5,0]` encode **identically**, compare **unequal** under derived `Eq`/`Ord`/`Hash` |
| `canonical.rs:123-131` | `Value::String` is **NFC-normalized at encoding time** | two normalization forms encode **identically**, compare **unequal** |

⇒ **Raw derived equality/hash/order already disagrees with canonical identity —
today, with no closure involved.**

⚠ **This corrected a measurement I handed the Architect.** I reported that no
consumer anywhere keys a `BTreeMap`/`HashMap` on `Value` and nothing calls `.cmp`
on one, so *"the axis §3 names as (b)'s cost may be close to empty."* **The
measurement was true and it did not entail what it looked like it entailed** —
the defect is not *who consumes the derives*, it is that **the derives do not
mean what the carrier's contract says they mean.** Zero consumers today is not
soundness; it is an unexercised contradiction.

⇒ **The frame must require one of two STRUCTURAL answers** — ⛔ not a consumer
inventory, and ⛔ **not freezing `#[derive(Eq, Ord, Hash)]` as the desired
mechanism**:

- make the store carrier **canonical-by-construction**, or
- expose equality/hash/order **only on a sealed canonical witness**, defined
  **from the canonical contract**.

### 3d. Sweep and retained obligations, per the ruling

- ⛔ **`ken-foundation` carries the same obsolete closure-inclusive validation
  carrier and encoder.** Non-production, but a **shipped public validation
  model** — remove/retire its closure arm, closure encoder, and
  closure-content-addressing tests **in the same semantic sweep**, or explicitly
  retire the crate's stale model.
- ⚠ **`ir::RuntimeValue` derives `PartialEq`/`Eq` across `ClosureRef`**
  (`ir.rs:487`). Audit separately: internal artifact-shape comparison may remain
  **only** behind an explicit non-semantic/private boundary; it must not become
  ordinary closure equality. **This is pin 2 failing on the operational carrier
  right now.**
- ⛔ **The ruling does NOT discharge deep-acyclic encoding, clone, or destruction
  totality.** Removing `Closure` closes the *capability* contradiction only; the
  shared iterative representation/traversal repair remains required **for the
  closure-free canonical graph**.

**Frame and build scope are the Steward's from here.**

## 4. Acceptance criteria — draft, and deliberately per-mechanism

⛔ **Each face below gets its own isolated control.** Bundling them means one
control's green is read as covering mechanisms it never exercised.

**`AC-V1` — deep ACYCLIC adoption completes with no host-stack growth.** A chain
deep enough to overflow the current recursive encoder must canonicalize and adopt
**successfully**. ⛔ It must **not** be reclassified as malformed, and ⛔ a
depth-limit rejection does **not** discharge this — the ruling requires success,
not a clean failure.

**`AC-V2` — ⛔ SUPERSEDED. THE CYCLE CLAUSE DOES NOT BIND ON THIS CARRIER, AND AN
AC DEMANDING A CYCLE WITNESS HERE IS UNSATISFIABLE.** Ruled `evt_45x5dn9jcrhhq`
(2026-07-26) against `7415dbd8`. Recorded rather than deleted so it cannot be
re-read as still owed.

The original AC asked for cycles to fail closed with an isolated-process control.
⚠ **The question was wrong, not the control shape.** `Value`'s recursive positions
are `Vec<Value>` and `BTreeMap<Vec<u8>, Value>` with **no** identity-bearing
indirection, interior mutation, slot/index edge, or shared ownership, and
`Store::intern` canonicalizes the whole tree to **one flat byte image interned as
one slot**. ⇒ A back-edge is **unconstructible**, not a malformed inhabitant.
Tri-colour state here is *"a vacuous defence for an input the type cannot
carry."*

⛔ **The obligation was RETARGETED, not dropped** — to B2V's sealed, emitted
`BoundaryPersistentImage(BoundaryRegion)` at `BoundaryValueStore::adopt`, whose
node-indexed region graph is mutable before sealing, whose child words can name
other persistent-region nodes, and where **the parked evidence demonstrates
emitted code constructing a cycle.** The grey/black distinction, image-local
node-index key, deterministic refusal before publication, and shared-DAG positive
control belong **there**. ⚠ They bind on **neither** current `Value` **nor**
current recursively-owned `RuntimeValue`.

⇒ **Replaced, for this node, by a structural pin** (Phase 1 `AC-V2a/b/c`): the
canonical carrier stays an owned finite tree, its recursive child positions may
not acquire reference/handle/arena/slot/index indirection or interior mutation,
and interning stays whole-value-bytes-to-one-slot. ⚠ **And it travels:** if the
representation later makes cycles expressible, the cycle contract **moves with the
new carrier** and must be discharged **before it publishes values.**

**Second-order, ruled in the same turn:** deep acyclic canonicalization/interning
must be **iterative** with ⛔ **no semantic `MAX_DEPTH`** — finite memory is an
ordinary resource boundary and **depth is not a validity predicate.** ⚠ This does
**not** discharge deep `Clone`/`Drop`, which remain separately required *even
though cycles are impossible*.

**`AC-V3` — `Clone` and DROP are total at the same depth as `AC-V1`.** ⛔ A value
that constructs and encodes must also **clone and drop** without overflow. Drop
cannot signal failure, so this face needs its own control at the `AC-V1` depth,
exercising deallocation specifically.

**`AC-V4` — the forbidden closure capabilities are UNREACHABLE, not merely
unused — on BOTH carriers.** No consumer may obtain structural equality,
ordering, or a canonical hash of an ordinary closure. ⛔ A grep showing no current
caller does **not** discharge this — the AC is about **reachability**, and the
positive control is that the forbidden operation **fails to compile** (or is
statically absent from the type), not that nobody calls it today.

⚠ **Discharge it on the operational carrier too, not only the canonical one.**
Per ruling pin 2, `RuntimeValue` must not expose Ken-semantic `Eq`/`Ord`/`Hash`
*merely because it is one Rust enum* — and `ir.rs:487` derives `PartialEq`/`Eq`
across `ClosureRef` today. ⛔ Removing `Closure` from `Value` and leaving that
derive in place discharges **half** an AC that reads as whole.

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

**`AC-V7` — ⛔ SUPERSEDED BY THE RULING. A consumer inventory does not discharge
this, and the ruling says so explicitly.** The original AC asked for an
enumeration of what depends on `Value: Ord`/`Hash`. ⚠ **That question has been
measured and it was the wrong question** — see §3c: there are no such consumers
*and the derives are still unsound*, because `minimal_limbs` and NFC
normalization happen **at encoding time** while the public enum admits the raw
forms. ⇒ **Replaced by `AC-V8`.** Recorded rather than deleted so the superseded
requirement cannot be re-read as still owed.

**`AC-V8` — equality/order/hash agree with CANONICAL identity, by construction or
by a sealed witness.** Deliver **one** of the two structural answers the ruling
names: the store carrier is **canonical-by-construction**, or
equality/order/hash are exposed **only on a sealed canonical witness** and
**defined from the canonical contract**. ⛔ Freezing `#[derive(Eq, Ord, Hash)]`
as the desired mechanism does not discharge this.

⭐ **The controls are already known and they are cheap, because §3c names two
concrete disagreeing pairs.** Both must be settled *and* each must have an arm:
`BigInt{limbs=[5]}` vs `BigInt{limbs=[5,0]}`, and two NFC-distinct spellings of
one `String`. Per pair, assert the encodings are **identical** and the
equality/order/hash verdict **agrees with that**. ⛔ A single pair does not
discharge it — they fail through different mechanisms (limb truncation vs
character normalization), so one passing arm says nothing about the other.

**`AC-V9` — the operational → canonical projection is transitive, iterative and
fail-closed, sharing the `AC-V1` mechanism.** It proves the whole graph
closure-free and canonical **before** any byte, hash, slot, or publication
exists. ⛔ **No recursive adapter**, and ⛔ not a second traversal mechanism
living beside the one `AC-V1` delivers — a private recursive projection
reintroduces exactly the overflow `AC-V1` exists to remove, one layer out.

**`AC-V10` — the `ken-foundation` closure-inclusive validation model is retired
in the SAME semantic sweep.** Its closure arm, closure encoder, and
closure-content-addressing tests go, or the crate's stale model is **explicitly
retired** with that stated. ⚠ Non-production, but a **shipped public validation
model**: leaving it is a second, contradictory answer to the question this node
settles, and the next reader has no way to tell which one binds.

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
