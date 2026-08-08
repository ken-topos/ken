# RT-RECURSOR-TRANSPORT — carry an active recursor across a unit boundary

**An active computational recursor's result carries invocation-local
scope/return-hole state. Two residual classes exist solely because that state
cannot cross a functionized unit boundary. This node builds that transport — or
proves the state need not cross — and retires both classes.**

**Owner:** Team Runtime. **Branch:** assembled on the **D7 lineage** — see §0.
**Size:** L. **Risk:** ⭐ **highest in the campaign.** This is the mechanism the
B2F migration stopped at.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** It carries
the campaign's binding traps and the schedule. This frame does not repeat
them.

---

## 0. ⭐⭐ RECUT 2026-07-29 — `D1` ANSWERED, ORDER CHANGED, ASSEMBLY ATOMIC

**Architect ruling `evt_5zr53v2dp86md`, on exact `820d3e53`.** This node was
sixth in a seven-node campaign and reached by nobody. It is now **the reached
successor of `D7`**, and three things about it changed at once. ⛔ Read all three
before touching the deliverables below — each one invalidates a reading of this
frame that was correct yesterday.

### (1) ⭐ It is THIS node — not a new one, and not `RT-PRODUCER-MATCH-PORT`'s

The terminal refusal measured at the D7 seam is **this node's predicate
verbatim**:

```text
ObjectEmission / checked_process_object /
ComputationalMatch: a computational recursor closure names an in-flight
activation, not a transferable value
```

⇒ a computational recursor reached a separately emitted unit edge while its
invocation-local **activation / resume / return-hole** control was still embodied
in `ComputationalRecursorClosure`. ⛔ **Not a new node. Not another `D7`
disposition. Not the syntactic-residual retirement owned by
[[RT-PRODUCER-MATCH-PORT]].**

⭐ `ComputationalRecursorClosure` is correctly **`EscapeForbidden`** — admitting
it would publish compiler control. **`D7` did its ruled job**: it preserves the
phase-bearing known-constructor operands, performs the known-constructor
elimination in the source machine, and advances the edge to the next owner's
fail-closed admission check. That is a **lawful successor seam**, not a defect.

### (2) ⛔ THE `RT-PRODUCER-MATCH-PORT` DEPENDENCY IS RETIRED FOR THIS POPULATION

**Ruled:** *"The existing schedule dependency on `RT-PRODUCER-MATCH-PORT` is no
longer a mechanism prerequisite for this reached population: `D7` already
supplies enough producer-`Match` path to expose the recursor boundary."*

⇒ ⛔ **Do not wait for [[RT-PRODUCER-MATCH-PORT]].** Its `depends_on` edge to
this node is removed. ⚠ **What is NOT retired:** that node's own retirement of
the `ProducerMatchCall` syntactic residual. It remains separate, it remains
owed, and ⛔ **this node must not absorb it** (see `AC-16`).

### (3) ⛔⛔ ATOMIC ASSEMBLY ON THE `D7` LINEAGE — ONE CANDIDATE, TWO NODES

**Ruled:** this mechanism *"must be assembled **atomically on this D7 lineage**
because `D7` cannot satisfy its named parity gate without the reached successor,
and the successor has no reaching production witness on the pre-`D7` tree."*

| fact | value |
|---|---|
| ⭐ **resume base** (Architect `evt_5c9ys1my7hr51` §5) | `c45a59a9f7bd6a911441e58ebb5e9e303e1bc7ac` |
| its tree | `1e3cfe58037f28902d92951f9cbb358b033e468e` |
| `D7` preservation point (its parent) | `820d3e53014899da50e7d8fab0584b8c267c5874` |
| that tree | `5faee6ef816ce35369a2eadee5f4de305834ad85` |
| `D7`-only adjustment | ⛔ **NOT AUTHORIZED** |

> ⛔⛔ **RESUME FROM `c45a59a9`, NOT FROM `820d3e53`.** The first recursor pass is
> preserved at `c45a59a9` (parent exactly `820d3e53`; 2 files, `lowering/core.rs`
> and `lowering/mod.rs`, +183/−38; verified by the Steward against the report, not
> taken from it). It is the tip of branch `wp/RT-DECL-CLOSURE-PORT`.
>
> ⭐ **It is preservation-only and it is also the next repair base** — those are
> not in tension. ⛔ **Do not reset it, do not discard it, and do not route it to
> QA.** The required adjustment is a continuation **on top of** it.
> ⚠ Cutting from `820d3e53` instead silently discards a ruled-correct advance:
> `c45a59a9` is where the recursor refusal was *made to advance*.

⇒ **[[RT-DECL-CLOSURE-PORT]] and this node land as ONE candidate, in ONE PR, and
both tracker nodes flip `merged` together.** Neither can go green alone: `D7`'s
parity gate needs the reached successor, and this node has no production witness
without `D7`.

⭐⭐ **This is the resolution of a real circularity, not a convenience.** Before
the ruling, `D7`'s frame held it *"until its consumers are complete"* while the
tracker ran `RT-DECL-CLOSURE-PORT → RT-SEED-CALL-PORT →
RT-PRODUCER-MATCH-PORT → RT-RECURSOR-TRANSPORT`. That is a cycle, and it was not
merely a policy one — `rt_parity_native` is its own CI job, so the publisher gate
is **mechanical**. Atomic assembly breaks it.

⛔ **Atomic assembly does NOT relabel the mechanism as `D7`.** The recursor work
is reviewed, owned, and accepted as **this node**. ⛔ Do not fold its ACs into
`D7`'s, and ⛔ do not describe recursor code as a `D7` deliverable in the
candidate description.

## 1. Fixed inputs

| path | blob at the resume base `c45a59a9` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | ⚠ **re-pin at pickup** |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | ⚠ **re-pin at pickup** |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | ⚠ **re-pin at pickup** |
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | ⚠ **re-pin at pickup** |

⛔ **The former pins at `origin/main = 14c3c5f7` are RETIRED — do not use them.**
`D7` rewrote `core.rs`, `units.rs` and `static_transition.rs` on this lineage
(2658 insertions at `79029d4c`, plus the seam repair at `820d3e53`), and
`c45a59a9` then rewrote `core.rs` and `mod.rs` again, so every line anchor
derived from `14c3c5f7` — **or from `820d3e53`** — is stale. ⭐ **Re-pin against
`c45a59a9` before deriving anything**, and ⛔ do not re-pin the numbers and call
that a re-measurement.

⚠ `lowering/mod.rs` joined this list at `c45a59a9`; it was not in the `820d3e53`
pin set. ⭐ **The pin set is a derivation, not a fixed list** — re-derive it with
`git diff --stat <your base> <the ruled base>` rather than trusting this table's
membership.

## 2. The two classes are one mechanism

**`MatchScrutineeRecursor`** — a `Match` whose scrutinee is a
`ComputationalMatch` with a non-empty `recursive_positions`.

**`LexicalCallArgumentRecursor`** — a `Call` whose callee is a `LexicalClosure`
and whose **argument** matches that same shape.

⭐ **The code states the shared mechanism itself**, in
`LexicalCallArgumentRecursor`'s doc comment:

> *"The recursive result still carries invocation-local scope/return-hole state.
> Passing it through a separately declared lexical unit is not one of the
> completed functionized ports, so the established recursive descent lane
> retains the whole call."*

⇒ Same predicate, same carried state, two syntactic positions: match scrutinee
and lexical call argument. ⛔ **Retiring one without the other builds the
transport twice.**

⭐ **The ruling confirms the fold:** *"This choice covers both existing
positions… The two reached `fs_*` rows are population for that shared mechanism,
**not permission to repair only two cells**."*

## 3. ⭐⭐ `D1` IS ANSWERED — OUTCOME **(b)**, THE STATE NEED NOT CROSS

⛔ **`D1` is no longer a probe to run. Do not re-run it to "confirm" the
answer.** It was pulled forward exactly as this frame's former §3 pre-authorized,
and the Architect answered it on the measured seam (`evt_5zr53v2dp86md` §2).

> **Outcome (b): prove the invocation-local state need not cross the runtime
> ABI.**

**Planning must split the recursor edge before emission**, in these five steps:

1. **decompose** the capsule into its **runtime residual operand** and its
   **checked compiler-only invocation segment**;
2. **consume/validate** the exact **oriented affine control obligation** in the
   caller-owned plan;
3. **embody the static continuation** in **graph-derived, out-of-line
   worker/continuation unit targets**;
4. **carry only ordinary** runtime residual / environment / result operands
   through **typed ABI slots**; and
5. use the **statically connected call/return edge as the return hole**.

**Per position:**

- **Match scrutinee** — the worker result returns to a **caller-owned `Match`
  continuation unit**.
- **Lexical call argument** — planning **splits/specializes the transparent
  call** around that recursor use, so **prefix, worker, and suffix units are
  statically connected**. ⛔ The recursor capsule is **not a parameter**.
- **Recursion** reuses an **interned static continuation definition** and a
  **direct unit target**.

**What may carry static identity:** the checked call/frame template, producer
origin, sibling/parameter position, recursive-body origin, and source call site.

⛔ **What stays compiler-local and is consumed affinely** — dynamic activation
IDs, resume cursors, unwind vectors, open obligations, splice handles. ⭐ **They
are neither ABI key material nor runtime data.** Putting any of them in a
descriptor is the failure this outcome exists to forbid.

### ⭐⭐ RECUT 2026-07-30 — THE DYNAMIC CASE IS **CAUSAL CONTINUATION SPECIALIZATION**

**Architect `evt_7dhwrk26ks9m0`**, ruled on preservation object `93746ada`
(tree `8b3d06f9`, parent rejected `d55bceb5`) and Research advisory
`evt_2q9mnxs8w5bke`. ⛔ **This is not a new node, disposition, carrier lane, or
atomic participant.** It is the **exact dynamic case of outcome (b)** above, and
the atomic scope remains exactly `RT-DECL-CLOSURE-PORT` `D7` + this node.

#### The false premise this replaces

The scalar `active_static_recursor_result`, and the post-join
`selected_static_recursor_worker_residual_token(… predecessor_body=761)`, encode
that **the predecessor body selects exactly one worker.** The measured trace
falsifies it: body `761` returns **three** live continuation alternatives —
bodies `442`, `641`, `723` — and the consumer sees only an **environment word**.

⛔⛔ **`442` and `723` have the same seven-capture shape.** ⇒ **Equal tag, arity,
field count, or capture layout is NOT a discriminator.** The witness is a `723`
environment bound to a `442` body, reaching malformed borrowed process input —
⭐ the payload was **structurally well-sized and semantically wrong**, which is
exactly the failure a shape check cannot see.

⭐ **The information is destroyed at the join.** Nothing downstream can
reconstruct callable identity from the surviving environment word ⇒ **code
identity must be discharged BEFORE the identity-erasing join.**

#### Ownership

| owns | what |
|---|---|
| **producer branch** | the fact *"this alternative produced worker 442 / 641 / 723"* |
| **caller / consumer** | the continuation / suffix into which that worker result returns |
| **planner** | the **finite relation** connecting those two facts |
| **runtime** | execution of the **already-selected** unit with ordinary operands — nothing else |

#### The lawful mechanism — closed, though names may vary

1. During the generated-unit / specialization / continuation fixed point,
   **enumerate the exact producer-result alternatives BEFORE their same-tag join
   erases callable identity.**
2. For each exact alternative, derive one **continuation-specialization key**
   binding at least: producer unit/owner and exact **causal producer-result
   occurrence** · consumer unit/owner and exact checked continuation/frame or
   suffix identity · recursor parent and recursive/sibling position · exact
   worker closure/body identity, declared arity, and **ordered capture
   provenance** · the ordinary input/result ABI convention and the required
   lifetime/provenance contract.
3. **Intern that key BEFORE recursive discovery.** ⛔⛔ **Capture/environment
   VALUES are operands, never key material.** ⛔ Equal tag, arity, field count,
   layout, or constructor identity is **never** a substitute for body or causal
   occurrence identity.
4. Add **one explicit closed planner-derived unit-definition arm** for this
   semantic class — conceptually **`ContinuationSpecialization`**. ⛔ It may
   **not** be smuggled through `SchedulingEntry`, `ClosureBody`,
   `TransparentDeclarationClosure`, or **`StaticCallableSpecialization`**. ⭐ The
   distinction is exact: `StaticCallableSpecialization` specializes a
   **transparent callable parameter**; this class embodies a **caller-owned
   recursor continuation / return hole**. See [[RT-DECL-CLOSURE-PORT]] `D7`.
5. Each exact producer alternative emits a **statically connected direct
   call/return edge to its exact interned continuation specialization, BEFORE the
   identity-erasing join.** The unit binds **one** compiler-known worker and
   **one** caller-owned continuation/suffix. A join, if still required, occurs
   **only after** that continuation has produced an ordinary result.
6. The unit ABI carries **only** ordinary residual/result operands, the exact
   worker environment fields, and any ordinary caller-continuation environment
   fields already justified by their owner/lifetime records. ⛔ **Absent:**
   worker/body/continuation identity, selector/tag, cursor, activation, frame,
   splice, capsule, function pointer, trampoline, side-table handle.
7. **Close and validate BEFORE descriptor, environment, function, or object
   allocation:** discovered specialization keys **biject** defined units ·
   planned causal call edges **biject** emitted direct call edges · every call
   edge names **exactly one** specialization and **exactly one** worker · every
   synthesized/source boundary identity and lifetime/provenance obligation is
   **consumed exactly once** · recursive cycles **fold to already-interned
   identities**.

> ⛔⛔ **THIS IS BRANCH-LOCAL EDGE FORMATION, NOT A POST-JOIN SPECIALIZATION
> LOOKUP.** ⭐ **Merely cloning `442`/`641`/`723` while leaving one joined
> consumer call is UNLAWFUL** — it reproduces the identical information loss and
> fails the same test as the current source-derived target. ⚠ This is the single
> easiest way to build something that looks like the ruling and is not it.

#### ⛔ The rejected fork — and it is not a fallback either

**Producer-side continuation fusion is NOT the governing mechanism and is NOT an
authorized fallback.** Moving or duplicating the caller-owned eliminator/suffix
body into producer branches is this frame's prohibited **"inlining as
transport"**, and it invites duplicated/reordered traps, effects, broken linear
consumption, and code growth. ⭐ **The producer branch may SELECT and CALL its
exact out-of-line unit; it may not ABSORB that unit's semantic body.**

⇒ **If planning cannot discharge this, it must REJECT before allocation/emission
— ⛔ there is no runtime-carrier fallback.** Reject when: a possible worker is
**open/unknown** · the continuation **escapes** or is observed as a first-class
value · an exact producer alternative **cannot be tied to one direct
specialization edge** · effect order, trap order, or affinity **cannot be
preserved**.

## 4. Deliverables

- **⭐ `D0` — the delta-free regression baseline, PER ROW.** ⛔ Not an aggregate.
  ⭐ **It already exists for the seven parity rows and it is binding** — measured
  on detached `483ef7ab` (`evt_3tc0zm7smx9h2`), exactly **1/7**:

  | row | base result |
  |---|---|
  | `buffer_freeze_malformed_span_is_unconstructible_at_the_landed_surface` | **PASS** |
  | `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` | FAIL — `BoundaryCarrier: a closure capture is a specialized-only surface…` |
  | `fs_read_at_malformed_offset_narrows_to_invalid_offset` | FAIL — `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
  | `fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset` | FAIL — same |
  | `fs_read_at_malformed_window_narrows_to_invalid_bounds` | FAIL — same |
  | `fs_write_at_malformed_offset_narrows_to_invalid_offset` | FAIL — same |
  | `fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset` | FAIL — same |

  ⚠ **Why the row map and not the count.** `AC-1b` is a **per-row** property. A
  `1/7`-then vs `1/7`-now agreement is **count** agreement, and one of six
  contributors can defect underneath an aggregate that holds. ⭐ Extend this map
  to any suite you newly touch; ⛔ never replace it with a total.
- **`D2` — Full-residual enumeration** across the measured programs, including
  the population unmasked by [[RT-PRODUCER-MATCH-PORT]]'s `D4` **if that node has
  landed by pickup**. ⚠ It no longer gates this one — scope against what is
  measured on **`c45a59a9`** (⚠ **not** `820d3e53`, and not the pre-campaign
  number): the resume base already advanced the recursor edge, so enumerating
  against the parent would re-count a residual that is no longer there.
- **`D3` — The transport, per outcome (b)'s five planning steps** (§3), covering
  **both** syntactic positions. ⛔ Not a runtime carrier; ⛔ not an ABI slot.
  ⚠ **Two different "five"s — do not conflate them.** §3's five are the
  **pre-emission planning split**; §8c's five are the **per-edge validation
  obligations** that must all discharge **before any allocation**. `D3` owes
  **both sets**, and §8c is the one `c45a59a9` has not closed.
- **`D4` — Remove both `MatchScrutineeRecursor` and
  `LexicalCallArgumentRecursor`**, and only then re-run `AC-1a` and `AC-1b`.
- **`D5` — The atomic candidate.** One branch on the `820d3e53` lineage carrying
  `D7` **and** this mechanism, one PR, both nodes flipping `merged` together.

## 5. Acceptance criteria

- **`AC-1a` — the ceiling moved.** The selector reports
  `authority=FunctionizedUnits` / `residuals=none` on every program `D2` named
  as firing either class.
- **`AC-1b` — the objects still build.** Those programs **compile and pass**
  their existing suites, **and every row PASS in `D0`'s row map is still PASS**.
  ⛔ Not "the residuals are gone" — the objects build. ⚠ `AC-1a` does **not**
  discharge this: it quantifies over the firing set, and the regression
  population is its complement (campaign doc, Trap 2).
- **`AC-2`.** Outcome **(b)**'s proof obligation is discharged **by a control**,
  ⛔ not by prose.
- **`AC-3` — both positions, not one.** A control exercises the recursor in
  **match-scrutinee** position and another in **lexical-call-argument**
  position, and a mutation defeating the transport **reds** each. ⭐ One control
  covering one position would let the fold silently become a half-fold.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property in the residual
  classifier is preserved. ⛔ No wildcard arm.
- **`AC-6`.** Emitted function count and per-function code-size distribution
  recorded for the affected programs. ⛔ Report; do not tune, do not pin a
  threshold.

### ⭐⭐ The eight required discriminator ACs (Architect `evt_5zr53v2dp86md` §3)

⛔ **All eight. They are the ruling's price for outcome (b)**, and each one names
the mutation that must **red** it.

- **`AC-9` — two-position closure.** One **real** Match-scrutinee witness and one
  **real** lexical-call-argument witness traverse the static split. ⛔ **Omitting
  either position fails the closed edge census BEFORE emission.**
- **`AC-10` — continuation identity.** The same runtime residual under **two
  distinct** checked continuation/frame identities yields **two distinct**
  expected results. ⛔ Collapsing or swapping their static continuation keys must
  **red** the oracle.
- **`AC-11` — zero-control ABI.** Descriptor slots contain **exactly** ordinary
  residual / environment / result convention slots. ⛔ Adding activation, cursor,
  frame, splice, capsule, or selector material must **fail the ABI census**.
- **`AC-12` — owner-exact return hole.** Returning into the **callee**, the
  **wrong caller suffix**, or the **wrong sibling/parameter position** must
  **red** the result/control oracle.
- **`AC-13` — affine control.** Duplicate, omit, reorder, transplant, or reuse a
  dynamic splice / open-obligation token, and planning must **reject before
  CFG/object emission**.
- **`AC-14` — finite recursion.** Repeated dynamic invocation of one static
  recursor site reuses **one interned continuation graph**. ⛔
  Clone-on-invocation must fail the unit census / fixed-point check.
- **`AC-15` — fail-closed escape.** Direct **whole-capsule** transfer remains
  rejected **before** descriptor / function / object allocation. ⭐ The outer
  capsule must continue to fail transfer before allocation or publication.
- **`AC-16` — population and non-absorption.** Preserve the exact `D0` row map
  and `D7`'s controls, and **prove the reached rows advance through this
  mechanism without absorbing** the separate syntactic-residual retirement owned
  by [[RT-PRODUCER-MATCH-PORT]].

### ⭐⭐ SHARPENED 2026-07-29 (Architect `evt_5c9ys1my7hr51` §4)

⛔⛔ **These are AMENDMENTS to `AC-9`–`AC-16`, not new ACs.** Ruled verbatim:
*"These sharpen the existing `AC-9`–`AC-16`; they do not create a separate test
node."* ⇒ ⛔ **Do not open `AC-17`+, and do not file a test node.** Each row below
adds a required discriminator **inside** the AC named in its first column.

| sharpens | required discriminator |
|---|---|
| **`AC-9`** | Exercise a **captured static worker** through **both** ruled recursor positions **where present** — not only the position that first exposed the edge |
| **`AC-10`** | **Worker identity.** Two recursive workers with the **same arity, capture shape, and capture values** but **different body origins** produce observably **different** results / direct targets. ⛔ Collapsing or swapping body origins must **red** the oracle |
| **`AC-11`** | **Exact envelope + ABI census.** The environment has **exactly `capture_count`** ordinary fields **in declaration order**; the worker call is ordinary arguments **followed by** those captures. ⛔ Wrong carrier class, count, order, owner, phase, lifetime, or declared arity **rejects before the call**. Census **both** the environment **and** the worker ABI and prove **absence** of body/callable identity and of all activation / cursor / frame / splice / return-hole material |
| **`AC-13`** | Duplicate or **replay** the projection **or** the affine segment ⇒ **reject before CFG/object emission** |
| **`AC-14`** | **Captures are inputs, never keys.** The **same** worker with **different capture values** reuses **one** static worker definition and produces **different** results. ⛔ Capture-value **keying**, permutation, omission, or duplication must **red** validation/result evidence |
| **`AC-15`** ⭐ | **Pre-allocation negative pair — the control that catches §8d.** On body mismatch, arity mismatch, capture-count mismatch, **or one non-ordinary capture**, the environment / parent / descriptor / function / object **allocation and publication counters all stay at ZERO.** ⭐ This is what distinguishes validate-then-allocate from **allocate-then-validate**; an outcome-only assertion cannot |
| **`AC-15`** ⭐⭐ | **Whole-`Closure` negative control.** The **same closure**, taken **outside** the exact planner-proved recursor residual edge, **still fails** through `CallableCapsuleEscape` **before allocation.** ⭐ Without this the repair is indistinguishable from a **global admission weakening** — it is the control that proves the new member is an exception, not a hole |
| **`AC-16`** | **Matrix omission.** Omit or reclassify the real static-worker-residual member and **planning fails before function/object emission**. ⛔ It **may not fall through** to the late generic `Closure` refusal — a late refusal is the *wrong* failure and would pass a naive "it still rejects" check |

⚠ **Two of these are load-bearing in a way the other six are not**, so do not
let them average out in review: `AC-15`'s **counter pair** is the only control
that can see the ordering defect §8d names, and `AC-15`'s **negative control** is
the only one that distinguishes a proved per-edge exception from a weakened
default. ⭐ **The rest assert that the right thing happens; these two assert that
the wrong thing still cannot.**

### ⭐⭐ RECUT 2026-07-30 — the TEN required-evidence rows (`evt_7dhwrk26ks9m0` §4)

⛔⛔ **These are REQUIRED EVIDENCE on one exact candidate, in addition to every
existing `D7`/`RT-RECURSOR` AC. They do NOT open `AC-17`+ and they do NOT create
a test node** — the same rule as the 2026-07-29 sharpening above. ⛔ **No
approval transfers from any earlier SHA.**

| # | what fresh QA must establish | the mutation that must **red** it |
|---|---|---|
| **`E-1`** | The **real `761` witness** reaches the **correct distinct** continuation/worker for **all three** alternatives `442`, `641`, `723`; the measured **`723`-environment / `442`-body malformed-borrow trap disappears** | — (this is the positive witness) |
| **`E-2`** | The same-shape **`442`/`723` pair stays DISTINCT** | Collapsing, swapping, omitting body identity, or keying **only** by tag/arity/layout **reds before emission** |
| **`E-3`** | Same worker + same static continuation + **different capture values** reuse **one** interned unit and produce **value-dependent** results | **Capture-value keying reds the census** |
| **`E-4`** | The same worker under **two** checked continuation/frame or suffix identities produces **two exact** return-hole results | Collapse or swap **reds** |
| **`E-5`** | ⭐ **Omit, duplicate, transplant, or retarget a causal producer→specialization call-edge token** ⇒ validation **rejects before definition/allocation** | ⭐⭐ **A post-join single-call mutation MUST FAIL** — this is the row that discriminates branch-local edge formation from a post-join lookup |
| **`E-6`** | **ABI census proves ZERO callable/control identity** and exactly the ordinary ordered fields with unchanged class, owner, provenance, aliasing, and invocation lifetime | any control/identity word present |
| **`E-7`** | **Untaken alternatives perform ZERO captures, traps, effects, or publication**; selected effects/traps retain **order and affine consumption** | Lifetime / owner / field-order mutations fail with **all allocation and publication counters at ZERO** |
| **`E-8`** | Recursive **A→B→A** discovery terminates by **intern-before-descent**; repeated dynamic invocation reuses the **finite static population** | an occurrence/owner **collision** mutation **reds** |
| **`E-9`** | **Unknown producer**, non-tail/escaping observation where relevant, and whole-`Closure` use **outside** the exact proved edge remain **fail-closed before allocation** | — |
| **`E-10`** | Re-run the **complete 19-row CI-regression population** that rejected `d55bceb5`, **all** named `D7`/`RT-RECURSOR` controls, and obtain **literal all-check CI green** | ⛔ partial green is not green; ⛔ a cancelled shard is not a run |

⚠ **`E-5` and `E-7` are the load-bearing pair.** `E-5` is the only row that can
fail while every other row passes on a **post-join clone** — it tests *where* the
edge is formed, not *whether* three units exist. `E-7` is the only row that sees
an **untaken branch doing work**, which no outcome assertion can reach because
the outcome is still correct. ⭐ **The rest assert the right thing happens; these
two assert the wrong thing still cannot.**

## 6. ⛔ Banned scope

- ⛔ **Retiring only one of the two classes.** They are folded for a stated
  mechanism reason; half is a worse state than neither, because it hides that
  the transport is incomplete.
- ⛔⛔ **Every item on the ruling's forbidden list, at every depth:** a
  capsule / activation / cursor / frame-vector **ABI slot**; a **tag, selector,
  function pointer, trampoline, side table, or codec**; a **value-selected
  continuation**; **reconstruction of `Lowered::ComputationalRecursorClosure`**;
  **inlining as transport**; **validator or admission weakening**.
- ⛔⛔ **EXTENDED 2026-07-30 (`evt_7dhwrk26ks9m0` §3) — still forbidden at every
  depth:** choose-first · source-residual fallback · disposition state ·
  environment shape · runtime body word · runtime selector · **tag dispatch for
  callable identity** · function pointer · trampoline · side table · capsule
  reconstruction · **capture-value keying** · clone-on-invocation · validator
  weakening · lifetime promotion.
- ⛔⛔ **Producer-side continuation fusion — as the mechanism OR as a fallback.**
  ⭐ It was the *rejected* fork, not the runner-up: absorbing the caller-owned
  eliminator/suffix into producer branches is the prohibited **inlining as
  transport**. The producer branch **selects and calls**; it does not **absorb**.
- ⛔ **A post-join specialization lookup dressed as this mechanism.** Cloning
  `442`/`641`/`723` while one joined consumer call can still reach all three is
  **unlawful** — see §3's recut.
- ⛔ **Absorbing [[RT-PRODUCER-MATCH-PORT]]'s `ProducerMatchCall` retirement.**
  The dependency is retired; the sibling work is not.
- ⛔ **Repairing `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`.**
  It remains **unruled** at `Match: scrutinee is not a constructor value`, and
  ⛔ **nothing in the ruling attributes or authorizes repair of it.**
- ⚠ **REVERSED 2026-07-29 — this line formerly banned counting or repairing the
  ordinary-`Closure` refusal.** It is now **attributed to this node** and
  repairing it **is the work** (§8). ⛔ What remains banned is repairing a
  `Closure` refusal **outside** the exact planner-proved recursor residual edge,
  and ⛔ weakening the generic `CallableCapsuleEscape` arm to reach it.
- ⛔ **A `D7`-only adjustment to `820d3e53`.** Not authorized. ⚠ Nor is a
  `D7`-only adjustment to `c45a59a9` — the continuation is **this node's**.
- ⛔ **Deleting the selector or the `RecursiveDescent` lane** — that is
  [[RT-DESCENT-RETIRE]].
- ⛔ **Weakening the `recursive_positions` predicate** to shrink the population.

## 7. Hard stop

Stop and report if outcome **(b)** cannot be discharged as specified — ⛔ **do
not fall back to a runtime carrier**, and ⛔ do not soften an infeasibility into
a described-but-unbuilt mechanism. ⭐ If the two positions turn out to need
different transports, that **falsifies this node's fold** and is the Steward's
re-cut, not the ring's to absorb. ⚠ Per Trap 2, this node exposes the largest
newly reachable population in the campaign; **expect a fail-closed invariant to
fire and route it as its own node.**

### ⭐ 2026-07-30 — planning must REJECT, not degrade

Per §3's recut, planning **rejects before allocation/emission** — ⛔ never
degrades to a runtime carrier — when any of these holds:

- a **possible worker is open or unknown**;
- the **continuation escapes** or is observed as a first-class value;
- an exact producer alternative **cannot be tied to one direct specialization
  edge**;
- **effect order, trap order, or affinity cannot be preserved**.

⭐ **A rejection here is a correct outcome, not a failure to find a mechanism.**
⛔ Do not treat "planning refuses" as a reason to widen the lane — with no users
on `main`, a fail-closed refusal costs a developer some confusion, not an
outage, and the refusal is what proves the exception is per-edge rather than a
weakened default.

## 8. ⭐⭐ ATTRIBUTED 2026-07-29 — the ordinary-`Closure` refusal IS this node's

⛔ **This section formerly read "UNATTRIBUTED, do not count it" and banned
repairing it. That text is RETIRED, not qualified.** The §8 one-shot bounded
protocol **fired, and it worked**: the implementer took exactly one
diagnostic-only witness on one named row, removed the probe, and preserved the
exact tree (`evt_6stmz1wsg17pd`). The Architect then closed attribution on that
evidence (`evt_5c9ys1my7hr51`). ⭐ **The protocol is now SPENT for this edge** —
⛔ do not re-run it here; it was a one-shot and it has been consumed.

### 8a. What the bounded witness proved

```text
ComputationalRecursorClosure / static-worker residual / Closure / Captures[Carried x7]
```

with producer `RecursorProducerOriginId(0)`, sibling `1`, generated-unit owner,
worker `StaticOriginId(723)`, arity `1`, `dynamic_splices=1`,
`open_obligations=0`, and pre-allocation parent/descriptor counters `0 / 0`.

⇒ The ordinary `Closure` is **the callable residual inside the recursor split** —
⛔ not an unrelated `D7` cell and ⛔ not [[RT-PRODUCER-MATCH-PORT]]'s.

> ⚠⚠ **THE ATTRIBUTION IS POPULATION-SCOPED. Ruled verbatim: it *"does not
> globally attribute every future `Closure` refusal."*** ⛔ Do not cite this
> section to explain, absorb, or repair a `Closure` refusal arising anywhere
> outside the exact planner-proved recursor residual edge. A future refusal with
> the same *text* is a fresh attribution question. ⛔ **The bounded protocol is NO
> LONGER available for one** — it was unspent when this paragraph was written, and
> the block immediately below records it being consumed on stop `#27`'s edge the
> same day. **Both charges are gone.**
>
> ### ⭐⭐ THAT FUTURE ARRIVED THE SAME DAY — and the population scope is what
> ### made the ring stop instead of absorbing it
>
> **Stop `#27`, 2026-07-29 (`evt_4tvysmzr6mfpb`).** On the authorized row
> `fs_read_at_malformed_offset_narrows_to_invalid_offset`, a `Closure` refusal
> with **the same text** was localized to `transfer_constructor_operands` on a
> carried computational-match path:
>
> | | this fresh edge | §8a's ruled member |
> |---|---|---|
> | constructor parent | `StaticOriginId(655)` | — |
> | closure child | `StaticOriginId(650)` | — |
> | closure body | `StaticOriginId(641)` | worker `StaticOriginId(723)` |
> | arity | `1` | `1` |
> | captures | **`8`** | **`Carried x7`** |
> | planner-proved token | **none exists** | exact static-recursor residual |
>
> ⭐ **Same refusal text, different body and capture population, no planner-proved
> member ⇒ a different edge.** The ring did not admit it, classify it as recursor
> transport, repair `ProducerMatchCall`, weaken generic admission, relabel
> `CallableCapsuleEscape -> EscapeForbidden`, or touch buffer-allocate. ⭐ **The
> population scope above is the clause that produced that restraint** — without
> it, "same text as the thing we just got ruled" absorbs cleanly and wrongly.
>
> ⛔⛔ **THE BOUNDED PROTOCOL IS NOW SPENT ON THIS EDGE TOO.** It fired once here,
> diagnostic removed before commit, preservation at `07ce6ef1`. ⇒ **It is spent on
> both known edges and there is no third charge.** A further refusal outside both
> populations gets **no** new diagnostic under this section — it is an Architect
> attribution question first, and the section that authorizes a witness would have
> to be re-opened deliberately.
>
> ⚠ Routed to the Architect as a **derivation** question, not a cell request:
> `evt_3tx7ndxp5pm4j`. See `RT-DECL-CLOSURE-PORT` §5a for the five-instance
> record that framing rests on.

> ### ⛔⛔ RULED AT STOP #27 (`evt_4p9ne0vcds5hb`) — THIS NODE'S SCOPE CHANGED, AND
> ### THE "ONE NEW MEMBER" BELOW IS NOW *LOCAL CLASSIFICATION ONLY*
>
> The Architect **withdrew part of its own `#26` ruling**: *"my prior statement that
> #26 was 'one new member' remains valid only as a **local classification of the
> measured 723 residual**, not as evidence that the global matrix is now closed."*
> ⇒ ⛔ Read §8b below as a fact about the `723` residual, ⛔ never as authority that
> `D7`'s population is closed.
>
> ## STEWARD FLAG 2026-08-08 — RE-DERIVE THIS OBLIGATION BEFORE PICKUP
>
> **The paragraph below was written while `RT-DECL-CLOSURE-PORT` `D7` was
> unlanded. `D7` has since merged, and the Architect has ruled on what it did
> and did not owe (`evt_1v9m7t4m9dmj7`).** Two things changed under it:
>
> - **`D7` landed the narrower `PlannedEffectSeat` population**, derived from
>   actual host-effect consumers, with a deliberately separate vocabulary. Some
>   or all of "replace `D7`'s population authority in place" may already be
>   discharged.
> - **The four continuation-side `BoundaryUse*` axes are being deleted** by
>   `RT-CONTSPEC-LEDGER`, which this node sits behind. They are an unowned
>   schema fragment with no consumer. **The `BoundaryUse` record named below is
>   the host-effect population, not those four axes** — same word, different
>   domain, and conflating them is the exact confusion `D7` was built to
>   prevent.
>
> ⇒ **Do not start from this paragraph.** Re-derive what remains owed against
> the merged `D7` and route the delta to the Steward. This flag is a caution
> that the text is stale, **not** a ruling that the obligation is discharged —
> nobody has measured that yet.
>
> **WHAT THIS NODE NOW OWES, and it is not a cell repair:** replace `D7`'s
> **population authority in place** — re-derive from the actual owner/phase
> transition graph, one exact `BoundaryUse` record per static lowering event, one
> choke-point API with unforgeable planned-edge tokens, and a planned-set-vs-
> emitted-ledger comparison **before function definition**. The full requirement and
> its ten controls live in `RT-DECL-CLOSURE-PORT`'s `D7` deliverable — ⛔ read it
> there, do not re-derive it from this summary.
>
> ⛔ **Ordering is fixed: population authority FIRST, cell-level repair after.** Do
> **not** admit or classify the origin-650 `Closure`, add a fifth cell / seventh
> disposition / node, weaken the generic `Closure` guard, relabel
> `CallableCapsuleEscape -> EscapeForbidden`, touch `ProducerMatchCall` or
> buffer-allocate, or route QA until #27 **fails early when omitted** and every real
> transition is ledger-accounted.
>
> ✅ **`07ce6ef1` SURVIVES AND IS THE REPAIR BASE — do not reset it, do not cut a
> subset commit.** Continue atop it, replacing the false authority in place.
> **Retained as locally sound mechanism:** the compiler-only `StaticRecursorWorker`
> and direct out-of-line target · complete capture/body/arity/provenance preflight
> before environment allocation · the ordinary positional `Record` envelope with no
> callable/control identity · class/count/order checks and
> ordinary-arguments-then-captures ABI · the unchanged whole-capsule
> `CallableCapsuleEscape -> EscapeForbidden` · the additional governed recursor
> routes and their local worker controls.
> **Superseded as population/closure authority:** the separate lowering-only enum as
> proof of real-edge extent · lowering-time `.token()` minting · the global
> `static_recursor_worker_residual_disposition` flag · on-demand residual-edge
> synthesis by source-occurrence search · omission/reclassification of that flag as
> proof the population is closed.

### 8b. Ownership and disposition — one new MEMBER, not a new lane

| axis | ruling |
|---|---|
| mechanism owner | **this node**, outcome **(b)** |
| matrix authority | `D7`'s closed boundary-operand matrix ([[RT-DECL-CLOSURE-PORT]]) enumerates one **lowering-only** edge, e.g. `StaticRecursorWorkerResidual` |
| disposition | **existing `CallableCapture`** |
| new node? | ⛔ **NO** |
| seventh disposition? | ⛔ **NO** |

⛔ **It is NOT `StaticCallableElimination`** — there is no callable declaration
parameter / use-closure here; the recursor plan **already selects the direct
worker**. ⛔ It is not `Forwarding`, and ⛔ **it does not make the `Closure` a
value.** It retains body/parameter identity in compiler plan material while the
ordered captures retain their own phase.

⛔⛔ **The generic whole-capsule rule is UNCHANGED.**
`LoweringOnlyOperandEdge::CallableCapsuleEscape -> EscapeForbidden` remains
correct for **every unproved or value-position whole `Closure`**. ⛔ Do not
relabel or weaken that admission arm — the new member is an *exception proved
per-edge*, never a softening of the default.

### 8c. Lawful transport — validate FIRST, allocate SECOND

⛔ **Before any carrier / frame / descriptor / object allocation**, the
recursor-owned split must consume a **planner-issued token for that exact
residual edge** and prove **all five**:

1. the invocation segment is the **exact checked affine segment** for the parent
   producer / sibling, and is **consumed once**;
2. `Closure.body` **equals** the planned worker body origin;
3. closure **parameter count** equals the planned declared arity, **and** the
   **complete ordered capture contract** equals the planned
   `StaticRecursorCaptureProvenance` — ⛔ **not merely `capture_count`**;
4. capture **order, phase, owner, and lifetime** equal the plan;
5. **every runtime capture has an ordinary transferable lane.** ⛔ A nested
   callable / control capsule is **not ordinary data** and **must fail before
   allocation** unless an already-ruled static binding eliminates it.

> #### ⛔⛔ SHARPENED 2026-07-30 — `capture_count` IS NOT THE CONTRACT
>
> **Architect `evt_21gpwrsewyxax`, on CI-red exact `4dc120c5`.** The unified
> identity did store the ordered provenance — but the **move-only token exported
> only `capture_count`**, `validate_static_recursor_worker_residual_identity`
> revalidated **only that count**, and `prepare_planned_static_recursor_worker`
> then rejected every capture not **already** `LoweringOperand::Carried`.
>
> ⭐ **Two distinct failures in one projection.** It is **weaker** than §8c items
> 3–4 (the per-capture phase / owner / provenance is dropped, so items 3 and 4
> cannot actually be checked at consumption), **and** it is **narrower** than
> item 5 (⛔ *"already carried or reject"* is not *"has an ordinary transferable
> lane"* — it refuses a **specialized ordinary** capture that item 5 admits via
> the one-way producer). ⇒ ⭐ **The binding rule is both stricter AND more
> capable than what was built.** It is not a matter of tightening one direction.
>
> **What consumption must therefore revalidate, per capture:** ordinal · source
> provenance · owner · expected phase / lane · lifetime · exact-once producer
> authority where needed. Carried ordinary captures pass unchanged; specialized
> ordinary captures cross the one-way producer **exactly once**; a nested
> callable / control capsule stays **fail-closed with every allocation and
> publication counter at zero**.
>
> ⚠ **The control that catches the regression, and why an outcome assertion will
> not:** at least one environment must contain **both** a carried **and** a
> specialized ordinary capture. ⭐ A **single-phase** environment cannot
> distinguish *"validates the whole ordered contract"* from *"counts captures and
> requires them pre-carried"* — both are green on it. Add phase / owner / order /
> omission mutations, the nested-capsule zero-allocation negative, and the
> exact-member versus same-closure-outside-edge pair.
>
> ⛔ Hard stop **#23** is now attributed **here** (with `D7`), not to a buffer
> operation and not to either later syntactic residual node — see
> [[RT-DECL-CLOSURE-PORT]]. That authorizes transporting an ordinary specialized
> capture; ⛔ it does **not** authorize carrying a nested capsule.

**Only after that complete validation** may lowering erase the `Closure` wrapper
and create the ordinary ordered environment payload.

**For the measured `Carried x7` case**, an existing carrier `Record` envelope is
**lawful**: exactly the seven capture words **in declaration order**, and ⛔ **no**
body origin, code pointer, callable-selecting tag, activation, cursor, splice,
return hole, or other callable/control identity. At invocation: validate the
envelope **class** and **exact field count**, project the seven fields
**positionally**, append them to the ordinary invocation arguments, and call the
**statically selected out-of-line worker directly**.

⭐ **The worker identity stays compiler-only, and capture VALUES are not
worker-key material.** ⛔ Unlawful at any depth: `Carried -> Lowered`, a
reconstructed `Closure`, a runtime-selected worker, a side table, a codec, a
trampoline, a function pointer, declaration inlining, or admission weakening.

### 8d. ⛔⛔ THE DEFECT IN `c45a59a9` — it allocates before it has proved

**Ruled:** `c45a59a9` is *"directionally correct in introducing
`StaticRecursorWorker` and direct out-of-line capture inputs, but it is not closed
yet. In particular, the current terminal split **allocates the environment before
it has proved every capture is carried**; that ordering is not acceptable."*

⇒ ⭐ **This is the repair, and it is an ORDERING repair, not a new mechanism.**
Validate the whole residual / environment **first**, then allocate and store.
⚠ Note what this means for the controls: an allocate-then-validate implementation
can satisfy every *outcome* assertion and still be wrong, because the wrong state
is created and then found acceptable. **`AC-15`'s sharpened counter pair is the
only control that sees it** — see §5.

⛔ **And it must be reached through EVERY governed recursor position** — not only
the source-machine terminal-constructor route that first exposed it.
