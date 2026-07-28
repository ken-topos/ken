---
id: RT-VALUE-TOTALITY
title: "Make every total traversal of Value non-recursive in the host stack, and remove the closure capabilities the landed closure boundary forbids"
status: merged
owner: runtime
size: L
gate: none
depends_on: [SPEC-CLOSURE-BOUNDARY]
blocks: [RT-FNSPLIT-B2V]
github: null
origin: Architect cycle-contract ruling evt_5pzxf6sm4z08 ("host recursion may not be the totality mechanism -- a deep acyclic chain must adopt without host-stack growth and must not be reclassified as malformed") plus closure-identity ruling dec_3b1r19v59v20y / SPEC-CLOSURE-BOUNDARY. Steward-filed 2026-07-26 per COORDINATION §2 as move 2 of three from the closure-identity ruling: the repair is a BLOCKING DEPENDENCY for RT-FNSPLIT-B2V acceptance but a SEPARATE implementation slice, and must not be built as a B2V-local adapter. Scope was re-derived against the landed code rather than taken from the ruling's prose, which surfaced three mechanisms the ruling did not name.
---

> ## ✅ P3 MERGED 2026-07-27 — PR #1116, `origin/main = b55d292c`
>
> `Value`'s `Debug` is depth-total: an encoder-shaped iterative state machine
> over one heap `Vec<DebugStep>`, one host frame deep, no depth cap. Blob-verified
> — `values.rs` = `042642ca`, `value_depth_totality.rs` = `cde48b42` on main and
> on the approved candidate `c630c66d`. Derived `Debug` removed.
>
> ### ⚠ TWO NAMED RESIDUALS — recorded so they are not thread-only asides
>
> 1. ⛔ **The alternate `{:#?}` pretty format is GONE.** The Steward ruled this
>    acceptable on a **zero-consumer census**: the pre-change base has zero
>    alternate-format callers and zero crate `Formatter::alternate` consumers.
>    ⭐ **This is an unconsumed capability LOSS, not an impossibility claim.**
>    Nothing in the iterative encoder forecloses pretty-printing — it can carry an
>    indent level. If a consumer ever wants it, that is a build, not a redesign.
>    ⛔ Do not cite this line as evidence that `Value` "cannot" pretty-print.
> 2. **The current-population inventory is REVIEW-ENFORCED, not mechanically
>    closed** (Architect + QA, and stated as such by both). The accepted guarantee
>    is **today's** `Value` population. A future child-bearing variant omitted from
>    *both* the mixed fixture and the inventory would not be caught. The audit is
>    independent of the list it checks — trip count from `MIXED_CHAIN_CYCLE`,
>    observations from outer fixture nodes — so omission and duplication mutations
>    do redden it; what it cannot see is a variant absent from both.
>
> ### ✅ ALL THREE PHASES MERGED — THIS NODE IS COMPLETE
>
> `P1` (PR #996), `P2`, and `P3` (PR #1116) are all merged. §5's phase table
> names those three as the whole plan and calls `P3` *"the node's residual"*.
> ⇒ **There is no remaining scope; status is `merged`.**
>
> ⭐ **This discharges the last unmet `depends_on` of
> [`RT-FNSPLIT-C1`](RT-FNSPLIT-C1.md)** — its other three (`B2O`, `B2R`, `B2V`)
> were already merged, so C1's dependencies are now fully met.

> ## ⭐⭐ OPERATOR RULING 2026-07-26 — THIS NODE IS NOW THE RUNTIME TEAM'S DIRECTION
>
> **Operator, verbatim:** *"the full linux abi campaign is stalled by a
> particular spec relaxation that triggered the general spec relaxation effort.
> The particular one should be carried through the enclave, and redirect the
> runtime team so they can complete their work."*
>
> **The particular relaxation is `SPEC-CLOSURE-BOUNDARY`** — and that it is *the*
> trigger is recorded, not inferred:
> `14-spec-mission-alignment-campaign.md` opens *"`SPEC-CLOSURE-BOUNDARY` removed
> persistent content-addressed closure identity from observable semantics… **This
> campaign is that WP's generalization.**"* Its own `origin:` names the cause —
> **six consecutive Architect production blocks on `RT-FNSPLIT-B2V`.**
>
> ### ⛔ AND IT WAS NEVER CARRIED THROUGH. That is this node.
>
> `SPEC-CLOSURE-BOUNDARY`'s closing line, in its own words: *"it settles the
> **contract**, not the implementation. `crates/` still contradicts it."*
> Measured, still true: `canonical.rs:182` encodes closures **memcmp-exact**, and
> `Value`'s derive list still grants `Closure` the structural equality, ordering
> and hashing **the landed boundary forbids** (§2c, §2d — these are *spec
> violations* now, not robustness gaps).
>
> ⇒ **P1 landed the totality half. The representation half — the half the
> relaxation was actually about — is `P2`, and it IS now framed:**
> [`RT-VALUE-TOTALITY-P2-representation-split.md`](../wp/RT-VALUE-TOTALITY-P2-representation-split.md)
> (444 lines: fixed inputs, measured substrate §2a–§2g, `D1`–`D6`, ACs).
> ⚠ This sentence previously read *"has no frame"* — true when written, false now;
> edited rather than annotated. **`P3` is now framed too**
> ([`RT-VALUE-TOTALITY-P3-debug-depth-totality.md`](../wp/RT-VALUE-TOTALITY-P3-debug-depth-totality.md),
> 2026-07-27) — this line previously read *"`P3` still has none"*, edited
> rather than annotated.
> **`P2` MERGED 2026-07-27** — verified on `main = 5df415c1`: `Value` derives
> `Debug` only, and `CanonicalWitness` carries the `Eq`/`Ord`/`Hash` the enum
> used to grant `Closure`. ⇒ **`P3` is the node's residual.**
>
> ### ▶ THE REDIRECT, CONCRETELY
>
> | | |
> |---|---|
> | **Runtime goes to** | **`P2` + `P3` of this node** — the carry-through |
> | **`RT-FNSPLIT-B2F` stays** | ⛔ **HELD at hard-stop #11.** Do not resume it |
> | **`RT-FNSPLIT-B2E`** | ⛔ **RETIRED 2026-07-26 — superseded by `SPEC-STORE-SPLIT` §7 item 1.** This row used to read *"land it — do not discard a nearly-complete unit"*; it is edited, not annotated, because it is the text a reader acts on. `B2E`/`B2F` are built around the constraint the store split removes. **Retire them and write fresh.** ⛔ Do not delete `wp/RT-FNSPLIT-B2E-boundary-value-elimination = e1b540e2` or `preserved/b2e-rejected-source-oracle = 159f4109`; the salvage decision is §7 item 4 there |
>
> ⭐ **Why the redirect is right and not merely a re-prioritisation.** `B2F`'s
> wall is that every eliminator needs a compile-time template, and all three
> escapes are closed by settled authority. **Runtime has been building
> increasingly elaborate machinery to satisfy a representation the spec has
> already relaxed** — `B2E` exists solely to bridge it. ⛔ **That is designing
> compliance around a constraint instead of asking whether it should still
> bind** — the same error the Steward made twice on 2026-07-26 (the source-text
> oracle, the currency gate), and the operator has now called it a third time on
> a far more expensive lane.
>
> ### ⛔⛔ THREE OTHER CAMPAIGN ITEMS LAND ON `P2`. The frame must carry them as inputs.
>
> **Answering the operator's question of 2026-07-26 — *"do any of the other spec
> relaxation items impact the work in front of the runtime team?"* Measured
> against `spec-align-a1-census.md` and the Track C dispositions: yes, three.**
>
> **1. ✅ CLOSED 2026-07-27 — the slot-id collision is GONE, no escalation owed.**
> This item used to read *"a live conformance row says equality IS SLOT ID …
> surface it to the Architect before P2 picks an arm"*; it is **edited, not
> annotated**, because the instruction it carried is now wrong.
> **`SPEC-STORE-SPLIT` retired all four rows.** Measured at
> `origin/main = 7e9cfc96`: `runtime/values/equality-is-slot-id`,
> `runtime/values/dedup-shares-slot`,
> `surface/collections/structurally-equal-collections-o1-comparable`, and
> `runtime/evaluation/det-sharing-dedups-by-slot` are **absent** from
> `conformance/` and `spec/`. ⚠ Positive control — the same probe **does** find
> `runtime/values/closure-publication-rejected-transitively` and
> `runtime/addressing/no-lattice-on-hot-path`, so it was not silently returning
> empty. ⇒ `AC-V8` has no live row asserting the other side of it, and fork C7
> was decided inside `SPEC-STORE-SPLIT` §5.
>
> **2. ⚠ C2 already ruled the key interface `AC-V8` is choosing inside.** The
> Architect ruled C2 as option (b), **`KeyEq` derived from the order**
> (task `#106`). `AC-V8`/`AC-V12` pick an equality/order/hash arm for `Value`.
> ⇒ **P2 must not pick an arm that contradicts C2.** ⛔ Do not let P2 re-decide
> a ruled question by arriving at it from the other side.
>
> **3. ⚠ `canonical.rs` is under live A1 stops.** Bignum tag `0x01`
> (`STOP-4`) and sign-magnitude minimal-limb normalization (`STOP-4/1`) both have
> live conformance rows over the exact bytes. P2 edits that file. ⇒ **P2 may not
> alter those encodings as a side effect of the closure-arm work**; it is a
> refusal at one arm, not an encoder redesign.
>
> ### ⭐⭐ AND THE REDIRECT MOVES A PREMISE THAT A1 RELIED ON
>
> **A1's cleared set is EMPTY — every candidate stopped.** Three of those stops
> name **`B2E` itself** as part of the reason:
> - open addressing / bucket layout — *"the store family is C7-coupled and **live
>   `B2E` infrastructure consumes it**"*
> - 4 MiB pages / bump allocation — *"**C7/`B2E` entanglement**"*
> - FNV-1a / `memcmp` / monotonic slots — *"live rows and **C7/`B2E`
>   entanglement**"*
> - and the summary: *"clearance because C7 and **live `B2E` work own the physical
>   store boundary**"*
>
> ⇒ ⛔ **A load-bearing premise of A1's stop list is that `B2E`/`B2F` is actively
> building against the store family. The redirect changes that.**
>
> ⚠ **It does NOT clear them, and I am not treating it as though it does.** Each
> of those stops rests on **two or three** independent reasons — live conformance
> rows and the open **C7** fork — and only the `B2E` one moved. ⭐ **But a stop
> whose stated justification has partially expired must be re-read rather than
> inherited**, and A1's census is the artifact a later relaxation will cite. ⇒
> Recorded here as a **premise to re-examine when the redirect settles**, owned
> by the Steward, not by P2's author.
>
> ⚠ **Honest residual: it is NOT established that landing `P2` dissolves `#11`.**
> The wall is stated in terms of the `Lowered` lattice, not `Value`'s derives.
> ⇒ **Do not write "P2 unblocks B2F" into any frame.** When `P2` lands, `#11` is
> **re-put to the Architect** against the new representation. That re-ask is the
> deliverable; a dissolved wall would be a welcome outcome, not a premise.

> ## ⛔ THIS IS NOT A B2V-LOCAL ADAPTER
>
> `RT-FNSPLIT-B2V` cannot discharge its acceptance by wrapping a deep-value
> workaround inside its own layer. The recursion is in the **shared** `Value`
> traversals that every consumer reaches, so a B2V-local fix leaves every other
> caller overflowing. That is why this is its own node.

> ## ✅ PHASE 1 IS MERGED. ⛔ PHASE 2 and PHASE 3 have no frame yet.
>
> Split into three phases. **P1 merged 2026-07-26 as PR #996, squash, landing
> `origin/main = 8f677ebc`** and carrying exact
> `2d12a10abd4d12ba0b9350268842f9b9c8ae3c82` — the SHA `dec_10qxwx9s8wscn`
> resolved, unchanged. Verified by **blob identity** on all five files, by
> `landed tree == merge-tree(53dc0360, 2d12a10a) == e26cd9cc`, and by the
> currency checker on the landed `origin/main`. ⚠ The publisher printed
> `Publication is now FROZEN` — its post-merge `git fetch` lost a
> compare-and-swap on the **shared** `refs/remotes/origin/main`, so verification
> never ran and the freeze was accurate when written; both skipped clauses were
> then discharged by hand. ⛔ `wp/RT-VALUE-TOTALITY-P1` is deleted on origin and
> was **squash**-merged, so it cannot be continued — cut P2/P3 fresh from
> `origin/main`.
>
> P1 was kicked to `runtime-leader` 2026-07-26 at `evt_64xwmxt5v3qk`, base
> `origin/main = 63ad112c`, handoff gate run in full (B2R retros confirmed
> `evt_v3gb9yyne1m8`/`evt_3q5d2qdnj0vsb`/`evt_5n9kybev0x9q2`; all three seats
> compact-verified; leader observed `Working`).
>
> | phase | frame | covers |
> |---|---|---|
> | **P1 — totality** | ✅ **MERGED** `docs/program/wp/RT-VALUE-TOTALITY-P1-iterative-canonical-traversal.md` | `AC-V1` iterative encoder · `AC-V2` structural pin · `AC-V3` clone+drop |
> | **P2 — representation** | ✅ **MERGED** `docs/program/wp/RT-VALUE-TOTALITY-P2-representation-split.md` | `AC-V4`–`AC-V6`, `AC-V8`–`AC-V10`: carrier split, derives, closure arm, `ken-foundation` twin, checked projection · **plus `AC-V12`**, which rides `AC-V8` and is not a reading of it |
> | **P3 — residual totality** | ✅ **WRITTEN** [`RT-VALUE-TOTALITY-P3-debug-depth-totality.md`](../wp/RT-VALUE-TOTALITY-P3-debug-depth-totality.md) | `AC-V11`: derived `Debug` is depth-total. ⚠ **Does NOT depend on P2** — releasable any time after P1 |
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

> ⚖️ **AMENDED at the carrier boundary — Architect Decision `dec_1dccecns4c2fr`
> (`resolved` 2026-07-27T13:23:47Z), transcribed in
> `docs/program/wp/RT-VALUE-TOTALITY-P2-representation-split.md` under `AC-V5`.**
> **Direct, record-field, and constructor-argument** are the complete
> constructible closure-bearing positions today and carry behavioral controls.
> **Array-element and primitive-map-value are discharged STRUCTURALLY** for this
> phase: the closure-bearing carriers have no Array/Map child positions, `D1`
> removes the closure variant from canonical `Value`, and the proved package
> `Map` is constructor data routing through the already-covered
> constructor-argument path.
> ⛔ **Not a permanent waiver** — any future closure-capable operational
> Array/Map position **reopens `AC-V5`** and requires its own refusal arm plus a
> closure-free positive control before that carrier may publish. The normative
> transitive-refusal contract is unchanged.

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

**`AC-V11` — derived `Debug` is depth-total (P3).** After P1, `Debug` is the
**only** remaining `Value` traversal that is both host-recursive *and* reachable
from code written for an unrelated purpose: a `{:?}` in a panic handler, a log
line, or an `assert_eq!` failure message. ⇒ The abort fires **while a maintainer
is diagnosing something else**, which is what separates it from the identity
comparisons behind deliberate call sites.

⛔ **`Debug` is named by no other `AC` in this node** — it appears exactly once,
inside a quoted derive line — so unlike the identity comparisons it has **no
P2 edit to ride on**. That is why it is its own item rather than folded into P2:
P2's subject is representation, and depth is not representation.

- Discharge: hand-write `Debug` over the same iterative worklist P1's encoder and
  `Clone`/`Drop` use. ⛔ **Not** a second traversal mechanism beside it (the
  `AC-V9` prohibition applies here for the same reason).
- Control: assert a `{:?}` at the **same `D` `AC-V1` exercises**, out-of-process,
  and ⚠ **state the depth as a number before running** — a control that renders
  nothing reports the same green as one that renders a deep value.
- ⚠ Do **not** accept "`Debug` output is unspecified so depth does not matter."
  The claim under test is *does it return*, not *what does it print*.
- ⛔ **A MEASURED ABORT DEPTH IS CORROBORATION, NOT THE PIN.** The
  `runtime-implementer` measured landed `Debug` dying of stack overflow at
  `D = 131072`, out of process (`evt_2119bqa3tnz0a`). ⚠ That number is a **single
  finite probe**, and the Architect's standing correction on `B2V` applies to it
  unchanged: *a finite probe supports a structural claim and does not constitute
  one*. ⇒ Whoever takes `AC-V11` must pin the **mechanism** — `Debug` traverses the
  same iterative worklist, therefore its depth is heap-bounded — and cite the
  measurement as evidence beside it. ⛔ A discharge whose whole content is "it
  survives `D = 131072` now" is **not** this AC: it is green against one depth on
  one platform, and it re-derives nothing if the traversal changes.

**`AC-V12` — whichever `AC-V8` arm P2 picks, the resulting equality/order/hash
is depth-total (P2).** ⛔ **This is NOT a clarification of `AC-V8` and must not
be folded into its text.** `AC-V8` pins *agreement*; it pins it correctly; and
**exactly one of its two permitted arms also delivers totality**:

| `AC-V8` discharge | agreement | totality |
|---|---|---|
| **canonical-by-construction carrier** | ✅ | ⛔ agreement is bought by constraining the *carrier* — the comparison still walks structurally |
| **sealed witness defined FROM the canonical contract** | ✅ | ✅ inherits P1's iterative encoder |

⇒ A P2 author can discharge `AC-V8` **completely, on the arm listed first**, and
leave identity comparison process-aborting — and it is invisible because the AC
it would ride is already green. ★ **A frame that leaves this implicit has chosen
the first arm by default.** So `AC-V12` must name the requirement independently:
either mandate the witness arm, or require iterativeness explicitly on whichever
arm is chosen, with a control at `AC-V1`'s `D` per comparison operation
(`==`, `<`, `hash`) — ⛔ not one arm standing in for the other three, which is
the `AC-V8` two-pair lesson applied to a different axis.

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

- ✅ **`RT-FNSPLIT-B2V` acceptance was blocked on this node, and that edge is
  SATISFIED as of P1's merge (`8f677ebc`).** ⚠ **Read the granularity:** the
  blocking part was only **P1** (see the phase table — *"P1 is also the only part
  on `RT-FNSPLIT-B2V`'s critical path"*). ⛔ Do **not** read `B2V`'s
  `depends_on: [… RT-VALUE-TOTALITY]` as requiring **P2 or P3** — this node stays
  `active` until they land, and a mechanical reading of that edge would idle the
  Runtime ring behind two phases that are not on B2V's path. Neither is the
  converse true: P2/P3 are not thereby optional, only not prerequisite.
- ✅ **P1's merge was also a RE-ANCHOR EVENT for `B2V`, and the re-anchor is
  DONE** (Steward, 2026-07-26, against `origin/main` = `a7d3e2b0`). P1 rewrote
  `canonical.rs` and `values.rs`; the held `B2V` branch
  (`wp/RT-FNSPLIT-B2V-executable-value-abi`, origin `a7aa60eb`, merge-base
  `aecdb001`) predates it. ⚠ **Measured, correcting an earlier claim here that it
  touched the same *two* files: it touches `canonical.rs` only** — its `D2` decode
  inverse adds a region at ~`:259`; it does **not** touch `values.rs`. A read-only
  `git merge-tree` probe against `a7d3e2b0` is **clean** (merged tree `f26ba8d9`),
  verified with a synthetic same-line control that does report a conflict. ⛔ That
  is textual only and is **not** "it still builds" — the ring's first act on
  resume is `-p ken-runtime`. Frame anchors re-derived in the frame's
  re-anchor block; the `abi.rs` path there was wrong from birth and is fixed.
- ⛔ `RECUT 2`'s phase-closure artifact must be **re-derived** against the settled
  three-lifecycle partition regardless — that remains a hard gate and this node
  does not relieve it.
- ⚠ **Contention:** this rewrites `crates/ken-runtime/src/canonical.rs` and
  `values.rs`. Check the file set against every WP **in flight**, not just the
  frontier candidates, before release. A `store.rs`/reifier change needs the
  **full** `-p ken-runtime` **and** `-p ken-interp` suites.
- ⛔ Targeted builds only — never `--workspace`; the full gate runs in CI.
- Commit, report the exact SHA, and keep going; the Steward publishes. Wrap
  markdown at 80 columns.

## 7. ⛔ POST-MERGE ADVERSARY FINDINGS ON P1 — two, both open

**Source:** adversary `evt_wv5fng3kt2yx`, 2026-07-26 07:10Z, against
`origin/main` = `8f677ebc`. Method: rebased, mutated, measured, restored
byte-identically (`git diff --quiet` exit 0), re-ran `-p ken-runtime --lib` →
371/371. Scope `crates/` only. Tree clean at `ed74117e`, no mutations left.

⛔ **P1 IS CLOSED — merged with all three retros in — and these do NOT reopen it.**
They are repairs to land separately, on the `KW-ORACLE-CLOSURE` precedent for
post-merge adversary findings. ⛔ **Neither is a QA miss; see the position note
at the end.** ⛔ §10⁻a: the adversary channel is report-only — these are routed,
never answered.

### 7a. `AC-V1b`'s coverage guard is a frozen literal — its doc says otherwise

`canonical.rs:750` `ac_v1b_corpus_covers_every_value_variant`. The doc at
`:746`–`:749` claims the count is *"counted from the corpus itself **against the
enum's own arm count**, so adding a variant without extending the corpus reddens
rather than silently narrowing coverage."*

The body is `assert_eq!(kinds.len(), 25)` where `kinds` is a `BTreeSet` of
`encode(value)[0]` over `differential_corpus()` **alone**. ⇒ **The test body
contains no reference to `Value`'s cardinality, so `kinds.len()` is invariant
under adding a variant by construction.** (Steward re-confirmed by reading; no
mutation needed.)

**Measured:** a 26th variant `Value::AdvProbe(bool)` plus **only** the five arms
the compiler demanded, with `differential_corpus()` **untouched**:

> #### ⛔ THE ADVERSARY'S LINES ARE COORDINATES ON ITS **MUTANT**, NOT ON `main`
>
> Its report locates the five arms at `canonical.rs:168` / `:~530`,
> `values.rs:141` / `:182` / `:311`. **Not one of those resolves against
> `f87adc3f`**, because they were read off the tree *with the 26th variant
> already inserted* — and the insertion shifted each file by a different amount:
>
> | site | as reported (mutant) | TRUE at `f87adc3f` |
> |---|---|---|
> | `canonical.rs` `encode_header` | `:168` | **`:167`** |
> | `canonical.rs` `encode_canonical_recursive_reference` (`#[cfg(test)]`) | `:~530` | **`:362`** |
> | `values.rs` `detach_children` | `:141` | **`:138`** |
> | `values.rs` `rebuild` | `:182` | **`:179`** |
> | `values.rs` `Clone`'s `Visit` arm | `:311` | **`:309`** (`Job` enum at `:299`) |
>
> ⛔ **A mutation proof's locators are measurements of the mutant. They are
> evidence about the finding and are NOT locators against `main`** — transcribing
> them as such imports the mutation's line shift as a silent error.
>
> ⚠ **I did exactly that, and it reached a live kickoff.** I copied
> `values.rs:141`/`:182` into `RT-FNSPLIT-B2V`'s new-machinery table and re-quoted
> them in the resume kick; the `runtime-implementer` caught both at `f87adc3f`
> (uniform **`+3`**, *"one derivation, not two typos"*). ⭐ **And it names the shape
> better than I did: both wrong lines land INSIDE the correct function body** —
> `:141` is a `Record` arm within `detach_children`, `:182` a `Constructor` arm
> within `rebuild` — *"so anyone who opens the file sees plausible code and reads
> the locator as good."* Path right, line wrong, and **nothing looks off**.
> ⇒ Sibling of the `abi.rs` defect in the same frame, inverted: there the path was
> wrong and every line exact; here the path is right and the line is wrong. **Both
> coordinates must be checked, and neither one vouches for the other.**

| check | result |
|---|---|
| `ac_v1b_corpus_covers_every_value_variant` | **PASS** |
| `ac_v1b_iterative_encoding_is_byte_identical_to_the_recursive_reference` | **PASS** |
| `ac_v1b_corpus_is_non_vacuous_and_discriminating` | **PASS** |
| full `-p ken-runtime --lib` | **371 passed, 0 failed** |

⇒ The new variant's encoding was **written twice and compared zero times**, and
nothing in the crate noticed.

⭐ **Why this outweighs its size.** The module doc at `:350`–`:355` already states,
honestly, that the differential is *"not an independent oracle for the byte
values — a mutation to a `tag` constant moves both sides and this differential
stays green."* **Coverage is therefore its entire value**, and the frozen `25` is
exactly what fails to bind it. `AC-V1b` is the pin establishing that the
restructuring changed no bytes; if a variant can enter uncompared, that
establishment is silently partial.

⚠ **Honest scope — the compiler does real work here.** This is **not** "a variant
slips in unhandled": exhaustiveness is genuine and the five error sites were
precise. A variant slips in **unverified**. For a hand-written twin-encoder
differential that is the one risk it exists to cover.

**Fix shape (adversary's, and the machinery is already present):** bind the number
to the enum instead of restating it beside — e.g. `fn kind_tag(&Value) -> u8`
whose exhaustive match the compiler forces, and count **its** distinct outputs;
roughly the shape `AC-V1b` already uses for the tag byte. ⛔ **Do not "fix" this
by editing the doc down to match the weaker mechanism** — that keeps a coverage
claim the code cannot make. ⭐ `RT-FNSPLIT-B2V`'s `D4` is the correct pattern
already in the corpus: *"a new variant is a compile error, not a silent
`ValueWord`."*

### 7b. The `"will not compile"` bound is scoped to five positions

`values.rs:14`–`:20` says giving a recursive child position indirection *"will not
compile"*. True and verified **for the five named positions**:
`child_positions::push` is bounded on a sealed trait implemented only for
`Vec<Value>` and `BTreeMap<Vec<u8>, Value>`, so retyping `args` to
`Vec<Rc<Value>>` fails at the call site.

⛔ **But `Step::Val` is constructible directly in the parent module** —
`canonical.rs:149` does exactly that for the root — so a future variant's arm can
push children without routing through `push`, and the bound never applies to it.
Closing move: make `Step::Val` constructible only inside `child_positions`.
Preventive, one line. Flagged because the cycle-unconstructibility argument leans
on that bound being the **sole** path.

### 7c. ⭐ THE POSITION LESSON — third instance in ONE WP, pattern now proven

Three defects of the same class in this WP — *stating what the author believed the
code did rather than what it does* — with outcomes governed by **position, not
diligence**:

| instance | position | outcome |
|---|---|---|
| `assert_eq!(compound_subvalues, 8)`, subject has **7** | executable assertion | **died on first run, under a minute, unassisted** |
| the `breadth-first` `Drop` comment | `///` doc comment | survived full QA; **Architect block** caught it |
| `AC-V1b`'s *"against the enum's own arm count"* (7a) | `///` doc comment | survived QA **and** close; **adversary** caught it post-merge |

⛔ **Do not file any of these as a QA miss.** A doc comment on a trusted source is
structurally exempt from every instrument the project owns — not under-tested,
**untestable in place**. The implementer established this in its own retro
(`evt_2119bqa3tnz0a`) by refuting the Steward's weaker "someone should read it
more carefully" candidate with the counter-example from inside the same WP;
7a is the **third** data point and it confirms the position reading.

### 7d. ⭐ WHAT SURVIVED ATTACK — preserve this; it is evidence, not praise

The adversary calls P1 *"the most rigorous WP I have hunted"* and its residual
disclosure *"the best I have seen in this fleet."* It opened the hunt expecting the
disclosed-deferral failure — artifacts around a deferred half quietly claiming the
whole — **and found the opposite**:

- The evidence doc measures `Debug`, `PartialEq`, `Ord` **and** `Hash` all dying
  at `D` on the **landed post-change** code, states in its own voice that
  *"`Value` traversals are total" is **false***, and identifies that `Debug` has
  **no cell** in the frame's §7 residual table while the cell that does exist is
  scoped to *disagreement with canonical identity* rather than totality. It then
  found the independent `ken-foundation` twin carrying the same defect and
  explained why it was correctly out of scope. **No claim overreaches its
  evidence.**
- **Reachability of those four:** no `BTreeSet<Value>` / `HashMap<Value, _>`
  anywhere in production. The `BTreeMap<Value, Value>` hits in `lowering/mod.rs`
  are `cranelift_codegen::ir::Value`, an unrelated type. **No live overflow path
  today.**
- **`D = 131072` is not budget-fitted.** Bisected out of process against the
  *pre-change* mechanisms at two stack sizes, both thresholds published in the
  module header — 16× the 1 MiB drop threshold, 2.0× the 8 MiB one, all six
  (mechanism, stack) pairs confirmed to abort at exactly that `D`. Per-scenario
  pinned `STACK_BYTES` removes ambient `ulimit -s` from the claim. ⇒ The
  "green by 3% of a hidden budget" check **passes cleanly**.
- **The `Drop` comment the Architect blocked is now right in the way that
  matters.** It does not merely say "depth-first": it states that a LIFO worklist
  holds the unvisited siblings along the current root-to-node path while a FIFO
  frontier holds an entire level, and that **neither dominates for every shape**.
  That is the memory-bound contract the block was about, not a wording repair.
  Verified: `detach_children` + `Vec::pop` is LIFO.
- **`Drop` allocates nothing per node** — `Vec::new()` does not allocate until
  first push, and an already-detached child pushes nothing, so teardown adds no
  allocation failure path.
- **`Clone`'s `Map` rebuild zip is sound** — children pushed from
  `entries.values().rev()`, popped in key order; `rebuild` zips against
  `entries.keys()`, and `BTreeMap`'s `keys()`/`values()` share that order. A `zip`
  mismatch would truncate **silently**, so it was checked rather than assumed.
- **`done.len() - children` cannot underflow** — `Finish` is pushed before its
  children so it pops after all of them, and each contributes exactly one entry.
