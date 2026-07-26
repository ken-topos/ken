# `RT-FNSPLIT-B2V` — per-pin evasion attempts

One row per pin, as the frame's evasion AC requires. **Branch:**
`wp/RT-FNSPLIT-B2V-executable-value-abi`, base `origin/main` = `aecdb001`.

> ⛔ **Failing to find a witness is evidence about the witnesses you could
> think of, never about the property.** Every "not found" row below says which
> surface was searched, so the limit is inherited rather than hidden.

## ⛔ TWO MORE PINS WERE FALSE GREENS

Found by QA — and by generalizing QA's finding rather than fixing its instance.

**Runtime QA blocked `c3f6da02`** with a compile-preserving mutation I had
not attempted: `LAST_TAG` from `8` to `8 + 1`. **Every boundary test stayed
green (13/13).** The emitted-code negative probed the single byte `0xFF`, so
tag `9` became accepted by `define_resolve` and nothing asked about it.

★ **The Rust-side twin already swept all 256 bytes.** The discipline was applied
to one side of the same property and not the other — which is precisely how a
per-candidate reminder gets satisfied by the control you were thinking hardest
about. *Sampling one witness is not a sweep, and it was my own table that called
the tag set "closed".*

⇒ **Generalizing the defect class rather than fixing the reported instance found
a second one.** `AC-9`'s inventory pin asserted properties of
`BOUNDARY_LOCAL_HELPERS` — its length, its uniqueness — and **never asked the
emitter anything**. Renaming a helper at its `declare` site left it green.
**The row below that claimed a name swap reddens was FALSE, and I wrote it.**

| pin | mutation | before | after |
|---|---|---|---|
| tag closure at the emitted interface | `LAST_TAG + 1` | 🔴 green (13/13) | 🟢 reddens: *"emitted `class` admitted tag byte 9"* |
| `AC-9` inventory | rename one helper at its `declare` site | 🔴 green | 🟢 reddens, naming `ken_boundary_owner_RENAMED` |

**Both repairs interrogate the artifact instead of the declaration of intent:**
the tag pin sweeps all 256 bytes through the emitted helpers and derives its
expectations from `from_bits`/`referent_owner` — a *different* expression of the
rule than the CLIF's threshold comparisons, so the two must agree rather than
one restating the other; the inventory pin reads the module's actual declared
`ken_boundary_*` symbols. ⚠ Both repairs are **test-only** — every diff hunk
sits inside `mod tests`, and the production bytes QA reviewed are unchanged.

## The one that succeeded — and what it cost

| | |
|---|---|
| **pin** | `b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm` |
| **first form** | `!region.contains("_ =>")` |
| **evasion** | `unhandled => ProtocolOnly { .. }` — a **binding** catch-all |
| **compiles?** | ✅ yes (unreachable-pattern warning only) |
| **pin verdict** | 🔴 **GREEN with the catch-all in place — the evasion won** |

★ A binding catch-all silences exhaustiveness exactly like `_` while matching
no `_ =>` substring. **The pin was a claim about one spelling where the
property is about the shape of every arm** — a *granularity* error, which is
the diagnosis that says to enumerate rather than to add the second spelling to
a list that is open at the top.

**Repair:** every line carrying `=>` in the region must begin with `Lowered::`
or a `|` continuation. Verified as a non-degenerate pair — with the evasion
applied the repaired pin reddens **and names the offending arm**; with it
removed the pin greens.

## Every pin

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-1` tag set closed (**Rust**) | mint a word with tag byte `9`..`255` | ✅ | 🟢 refused — `from_bits` is a closed `match` with `_ => None`, and the test sweeps **all 256 bytes** rather than a sample |
| `AC-1` tag set closed (**emitted**) | `LAST_TAG + 1` | ✅ | 🔴 **DEFEATED — QA's finding.** The emitted negative sampled one byte. Repaired with a 256-byte sweep through the emitted helpers; verified as a non-degenerate pair |
| `AC-1` list/enum drift | add a `BoundaryTag` variant, leave `ALL` alone | ✅ | 🟢 `ALL`'s declared length `9` reddens; the sweep's `byte < ALL.len()` boundary reddens too |
| `AC-2` no value-specialization | pass a seed value into the construction site | ❌ **does not compile** | 🟢 **strongest row — the surface is closed, not merely unwatched.** `boundary_value` imports no `NativeSeedEnvironment` and no environment vector, so there is nothing in scope to specialize from |
| `AC-2` magnitude boundary | claim the immediate range one value wider | ✅ | 🟢 reddens — the case list tests `MAX`, `MAX+1`, `MIN`, `MIN-1`, not typical magnitudes |
| `AC-3` no wildcard | `_ =>` / `unhandled =>` | ✅ | 🔴 **defeated in first form; repaired — see above** |
| `AC-3` demote a live arm | move `Lowered::Constructor` into `FailClosedForbidden` | ✅ | 🟢 reddens — the fail-closed block is checked **positionally**, not by counting arms |
| `AC-3` second dispatch | add a second `boundary_disposition` with a wildcard | ✅ | 🟢 reddens on the single-dispatch assertion |
| `AC-4`/`AC-5` template read | `field` returns a baked-in word | ✅ | 🟢 **M1** — reddens `left: 7, right: -3`. Discriminating: the head=`7` case still **passed**, so the catch came only from running one probe against three values |
| `AC-4`/`AC-5` constant table | `host_payload` selects a fixed arm | ✅ | 🟢 **M2** — reddens `left: 11, right: 22`, one test, no collateral |
| `AC-6` wrong referent owner | collapse `referent_owner()` to one constant | ✅ | 🟢 **M3** — reddens `left: 1, right: 2` |
| `AC-6` degenerate pair | — | — | 🟢 guarded: the test asserts the two owners **differ**, so an oracle that collapsed them fails instead of agreeing with itself |
| `AC-7` escape | return `OK` for invocation-owned words | ✅ | 🟢 reddens; and a malformed word must report `ERR_TAG`, **not** `ERR_ESCAPE` — a corrupt word and a lifetime violation are different answers |
| `AC-7` vacuity | — | — | 🟢 positive control on the permitted side: persistent and immediate words must return `OK`, so "everything refused" cannot pass |
| `AC-9` inventory | add a 14th helper | ✅ | 🟢 reddens — the count is derived from `BOUNDARY_LOCAL_HELPERS`, so the list and the emission must move together |
| `AC-9` name swap | rename a helper at its `declare` site | ✅ | 🔴 **DEFEATED — the row here previously claimed this reddened, and it did not.** The pin only read the list, never the module. Repaired to compare the module's declared `ken_boundary_*` symbols against the permitted set |
| `AC-1` unreachable tag | keep a tag no disposition produces | ✅ | 🔴 **found by inspection, not by a pin** — `ImmediateCapability` / `ImmediateResource` were in the closed set and unreachable. Fixed by removing them (`f934d233`); ⚠ **recorded as a residual: no mechanical check enforces tag reachability**, so a future unreachable tag is review-caught, not CI-caught |

## Mutation provenance

⚠ Provenance was established by **content delta** (`sha256` + `git diff
--numstat`), never by re-counting an anchor. That mattered: the single-line
anchor `let child = b.ins().load(types::I64, MemFlags::trusted(), address, 0);`
occurs **twice** in `boundary_value_clif.rs` — in `define_field` and in
`define_host_payload`. A naive anchor count would have been ambiguous about
which site was mutated; each mutation was therefore applied through a
uniqueness-asserted multi-line block.

All three restored byte-identically, each verified with `git diff --quiet`
(exit 0). ⚠ `git diff --stat` always exits 0 and is not an emptiness test.

## `AC-8` — the census, predicted before measuring

| pin | predicted | measured |
|---|---|---|
| `correspondence_adds_no_emitted_unit_to_the_production_census` | unchanged | ✅ unchanged |
| `the_backend_production_surface_inventory_is_closed` | unchanged, 12 modules / list 13 | ✅ unchanged |
| `px8i_…identical_local_helper_clif` (`LOCAL_HELPER_COUNT = 6`) | unchanged | ✅ unchanged |
| `lowering/mod.rs` census row | stays `0`/`0`/`0` | ✅ stays `0`/`0`/`0` |

⇒ The frame's original *"the pin you will trip"* prediction was **false for
this placement**, reported before building rather than discovered as a green.
Every landed census is scoped to `cranelift_backend/**`; `native_int_clif.rs`
already declares 8 functions and appears in none of them. **A pin's silence is
scoped to the question it asks**, so their silence about a sibling file is not
evidence — which is why `AC-9`'s boundary-helper census exists at all.

---

# ⛔ ARCHITECT BLOCK of `78a57d90` — two defects in the PRODUCTION mechanism

Neither was repairable at the pin layer, and that is the point worth recording:
**every pin above was green, sound, and asking the wrong question.** The
controls verified that the representation behaved as specified. They could not
observe that the specification itself made a persistent word name storage that
dies, nor that the interface had no way to build one.

## 1. A persistent tag on an ephemeral locator

Every handle payload was an **invocation-arena node index**, while
`check_escape` permitted `Persistent*` words to leave the invocation from the
tag alone. After the arena died, a word cleared to escape named freed storage;
the real `SlotId` was reachable only by first resolving the ephemeral node.

★ **This is the failure mode of a pin that measures conformance to a design.**
Every control asked *"does the code do what the representation says?"* — yes.
None asked *"does the representation's permission agree with its lifetime?"*
That question has no natural home in a per-mechanism control, which is why it
took a reader with the whole design in view.

## 2. A construct interface with one constructor

`make_immediate` was the only producer. Every live handle was minted Rust-side
before publication, so the `D5` controls demonstrated a separately compiled
consumer walking a **Rust-built fixture** — the half `#10` already had. A
producer with dynamic children had no executable way to mint the word.

⚠ **`AC-4` says "construct, discriminate, and project."** The candidate
discharged two of three, and the AC read as satisfied because the two it did
discharge were the two the controls were built around. *A control set assembled
around the mechanism you built cannot notice the clause you did not build.*

## The mutations on the new mechanism

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M4b** | `resolve` jumps with `arena` instead of the loaded persistent region — thresholds untouched | survival, both constructor controls, record, nested aggregate, escape, frozen, malformed | 9 tests |
| **M5** | `store_field` drops the persistent-parent escape guard (`dangling := 0`) | `b2v_a_persistent_node_refuses_an_invocation_owned_child` | **none — 1 test** |
| **M6** | the frozen-prefix comparison becomes `index >= 0` | `b2v_the_frozen_prefix_refuses_emitted_mutation` | **none — 1 test** |
| **M7** | the capacity ceiling returns `ERR_BOUNDS` instead of `ERR_CAPACITY` | `b2v_construction_fails_closed_at_each_ceiling` | **none — 1 test** |
| **M8** | `make_immediate` bakes a constant payload | the non-constant constructor control, and survival | 2 tests |

⭐ **M5/M6/M7 each redden exactly one control and nothing else.** That is the
property worth reporting, not the count: a mutation that reddens nine tests
proves the suite noticed *something*, but not which detector fired. A
single-test redden names its detector.

⚠ **M4b is deliberately the surgical form.** The blunt version — setting
`LAST_PERSISTENT_TAG` to `FIRST_HANDLE_TAG - 1` — reddens 11 tests including
the threshold pin itself, which fires by construction and therefore tells you
nothing about the dispatch. M4b leaves the thresholds intact and breaks only
the region lookup, so the 9 reddened tests are all reacting to the *dispatch*.

All five restored byte-identically, each verified with `git diff --quiet`
(exit 0).

⚠ **One anchor missed, and the green it produced was not evidence.** M6's first
form did not match — `rustfmt` had wrapped the expression across three lines
after I wrote the anchor. The run reported **22 passed**, which reads exactly
like "the mutation failed to redden" and is in fact *the unmutated build*. The
uniqueness assertion caught it; without it, a formatting-induced no-op would
have been recorded as a defeated pin. **The harness now asserts the mutation
text is present before restoring, so a silent no-op cannot pass as a result.**

## Evasions attempted against the new pins

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-6` persistent survival | resolve persistent words against the arena | ✅ | 🟢 reddens — the control **drops** the first arena and resolves through a second invocation that shares only the store |
| `AC-6` survival vacuity | make both regions the same table, so "survival" is trivially true | ✅ | 🟢 guarded by the orphan-arena positive control: an invocation bound to **no** persistent region must return `ERR_BOUNDS`. If resolution silently used the arena, that assertion fails |
| `AC-6` frozen prefix | let emitted code rewrite a materialized node's `SlotId` | ✅ | 🟢 **M6**, one test. Positive control: a node emitted code *allocated* must remain writable, so "refuses everything" cannot pass |
| `AC-7` one layer down | persistent parent embeds an invocation-owned child | ✅ | 🟢 **M5**, one test. **Two** positive controls: a persistent child is admitted, and an invocation parent may hold an invocation child — so the refusal is about the child's owner, not about `store_field` refusing |
| `AC-4` construction is real | producer returns a word for a node Rust built | ❌ **does not compile in the control's shape** | 🟢 the producer is a **separate JIT module** with its own helper graph, compiled once and called three times; it receives the head at run time and holds no image of it |
| `AC-4` constant payload | `make_immediate` ignores its argument | ✅ | 🟢 **M8** — two of the three heads redden. The single-head form would have passed |
| `AC-4` capacity | admit one allocation past the reservation | ✅ | 🟢 the ceiling controls assert the **exact** status, so an off-by-one that still refuses is not enough; **M7** shows an inexact status reddens |
| `AC-4` overflow | caller-supplied `field_count = u64::MAX` wraps the sum into "fits" | ✅ | 🟢 refused — the addend is bounded **before** the sum, and the control passes `u64::MAX` explicitly |
| `AC-1` region bands | reorder `BoundaryTag` so the owner bands are no longer contiguous | ✅ | 🟢 `b2v_the_region_thresholds_agree_with_referent_owner` reddens; it also asserts both bands are **non-empty**, so agreement cannot hold vacuously |

### ⚠ Residuals — stated, not detected

| residual | why no mechanism | who catches it |
|---|---|---|
| tag **reachability** — a tag no disposition produces | needs a whole-program reachability argument over the disposition; the closed-set pin asks a different question | review (unchanged from the prior candidate) |
| emitted-constructed persistent nodes are **not content-addressed** (`NULL_SLOT`) | interning is a content-addressing pass over a whole value; it is not Θ(1) at a construction site | pinned as an *assertion of the limit* in the survival control, and documented at `BoundaryPersistentImage`. Closing it is a `B2F` lifecycle decision |
| the escape check does not **walk** a structure | O(size) at every crossing, re-answering a question settled at construction | the construction-time invariant is enforced at both paths (`materialize` debug-asserts; `store_field` returns `ERR_ESCAPE`) and **M5** pins the emitted half |

⛔ **The middle row is the one to read twice.** "Survives the invocation" and
"is content-addressed" are different properties, and a persistent region could
easily be described as giving both. It gives the first. The control asserts
`NULL_SLOT` explicitly so the boundary is inherited rather than assumed.

---

# ⛔ SECOND ARCHITECT BLOCK of `657f60a0` — three more production defects

## 1. The admitted representation was lossy **to emitted code**

A spilled `Int` wrote `NODE_PAYLOAD = 0`; `Bytes`/`String` kept only a length.
A separately compiled consumer saw every wide integer as **zero** and could not
tell two equal-length strings apart. The paths that *could* — the typed
residency map, the canonical decoder — are **Rust**.

⭐ **That is hard-stop `#10` reproduced one layer along, inside the node built to
close it.** `#10` said: the aggregate path works only because the consumer is
Rust. The candidate said: the *content* path works only because the consumer is
Rust. Same sentence, smaller noun — and every control was green, because they
all asked about structure and none asked about **content**.

## 2. The producer interface could forge store identity

`ken_boundary_store_slot_local` took a **caller-supplied** `SlotId`, and the
frozen guard expressly permits writes to a *newly allocated* node. So emitted
code could replace the allocator's `NULL_SLOT` with any slot it liked.

⛔ **I wrote the residual that this contradicts, and I pinned it.** The control
constructed a node and read back `NULL_SLOT` — **without ever calling
`store_slot`.** It asserted a property of a field nothing had written to. A
green there was compatible with the field being writable by anyone.

⚠ **The general form: a pin that never exercises the mechanism which would
violate it is not evidence about that mechanism.** It is not a spelling problem
and not a granularity problem — the assertion was true, about a path that did
not include the risk. *Ask which call sequence would break the property, and put
that sequence in the control.*

Closed by **removal**, not by a guard: assigning store identity is not an
emitted-code operation at all.

## 3. `alloc` admitted the Cartesian product of tag × class

Both sets were closed. Their **product** contained pairs no disposition can
produce — `PersistentClosure + HostResult`, `InvocationHostResult +
Constructor` — which minted successfully and failed much later at an unrelated
projection, reporting the wrong defect in the wrong place.

⚠ **Two closed sets do not make a closed relation.** Every previous control
checked membership of each set independently, which is exactly what made the
gap invisible: both halves passed.

## The mutations on the new mechanisms

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M9** | `alloc` treats the relation mask as always-admitting | the whole-product relation sweep | **none — 1 test** |
| **M10b** | `int_limb` perturbs the limb the native decoder returned | the spilled-`Int` content control | **none — 1 test** |
| **M11** | byte access ignores the index within the node's span | both `Bytes` and `String` content controls | 2 tests |
| **M12** | `store_int_tag` admits a persistent `Big` | the `Big`-refusal control, and the content producer | 2 tests |
| **M13b** | a `NODE_SLOT` setter is re-wired at its emission site | **every test that emits the graph**, at emission: *"emitted code may not set node offset 16"* | by design |

### ⚠ M13 was a no-op, and its green is reported as one

The first form of M13 widened the guard's *predicate*
(`… || offset == NODE_SLOT`) and **all 28 tests stayed green** — with the anchor
verified as matched, so this was not the formatting no-op from the last round.

⭐ **It is green because the assertion is a latch, not a detector.** Nothing
calls `define_store_node_word` with `NODE_SLOT` any more, so widening what the
guard *would* admit changes nothing observable. The latch fires on the event it
exists to prevent — **re-wiring the setter** — which is what M13b does, and it
panics at emission in every test that builds the graph.

⛔ **Reporting this distinction matters more than the redden.** "The guard is
mechanically enforced" is true of M13b and false of M13; a table that showed
only M13b would imply the predicate itself is defended, and it is not. What
defends the *inventory* is the allowed-writer list in
`b2v_emitted_code_cannot_assign_store_identity`.

### ⚠ And one bad pin of my own, caught by its own control

That control first asserted `!BOUNDARY_LOCAL_HELPERS.iter().any(|n|
n.contains("slot"))`. It **failed immediately** — on `ken_boundary_slot_local`,
the *reader*, which is meant to exist because reading a node's slot is how
`AC-6` is observable at all. A forbidden-substring needle colliding with
unrelated surface, and it would also have missed a writer that spelled the field
differently. Replaced with the **allowed writer inventory**, so any new writer
reddens including one nobody imagined.

### ⚠ `rustfmt <file>` reformatted a module tree, and the first commit carried it

Formatting `lowering/core.rs` also reformatted `lowering/core/tests/control.rs`
and ~68 unrelated lines of `core.rs` itself — `rustfmt` follows `mod`
declarations, and this repo's committed formatting was not produced by a bare
`rustfmt --edition 2021`. My real change to that file is **one line**. Caught by
a `git diff --stat` audit against the base before handoff, restored, and the
commit amended. The narrower sibling of the crate-wide-`fmt` churn rule: *the
unit `rustfmt` formats is the module tree, not the file you named.*

### Evasions attempted against the new pins

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-4` `Int` content | return the node's payload directly instead of decoding | ✅ | 🟢 reddens — the control reconstructs the magnitude from sign+limb and the payload alone is not it |
| `AC-4` `Int` content vacuity | pass values that fit the immediate range | — | 🟢 guarded: each case asserts `!int_fits_immediate(value)` first, so the control cannot silently test the wrong arm |
| `AC-4` `Bytes`/`String` | discriminate by length | ✅ | 🟢 every case is the **same length**; the control also asserts the results are mutually distinct |
| `AC-4` `String` construction | break emitted `String` writes only, leaving reads intact | ✅ | 🔴 **DEFEATED on `ea8d9824`** — `M14`. Closed by making the class a run-time argument to one emitted producer; `M14` now reddens |
| `AC-4` class axis | build the wrong class and let the content carry the test | ✅ | 🟢 **M16** — the sweep reads the class back per case, and the cross-class agreement control means content alone cannot stand in for it |
| `AC-4` content by identity | return the store slot or a hash | ✅ | 🟢 the `String` cases include two that differ by one interior byte; identity would collapse them only if the store interned them together, and the control compares **bytes** |
| `AC-1` relation | admit a superset | ✅ | 🟢 **M9** — the sweep covers the whole 81-pair product with **both** counts asserted, so neither arm can pass vacuously |
| `AC-1` relation drift | change the CLIF mask without the table | ❌ **cannot** | 🟢 the mask is *computed* by `boundary_class_mask` from the one table; there is no second place to edit |
| `AC-6` identity | re-add a slot writer under another name | ✅ | 🟢 the allowed-**writer** inventory reddens on any new `ken_boundary_store_*` |
| `AC-6` identity | re-wire the generic setter to `NODE_SLOT` | ✅ | 🟢 **M13b** — panics at emission, in every test |

### ⚠ One more residual — `NativeIntV1::Big` is out of reach at a persistent boundary

Not a scoping choice: it is the **same rule** that made persistent words safe to
escape. A `Big`'s limbs live in an entry `ken_native_int_intern_local` mallocs
into the *invocation's* native arena, so a persistent node naming one is a
surviving parent pointing at storage that dies first — `store_field`'s refusal,
one representation down. `store_int_tag` returns `ERR_ESCAPE` for it, and
`materialize` returns `None` rather than the previous silent zero.

⭐ **Failing closed is strictly better than what shipped**, which admitted the
`Big` and rendered it as `0`. Making `Big` cross persistently needs store-backed
limb storage — a representation decision for the store/spec, not for `B2V`.

---

# ⛔ QA BLOCK of `ea8d9824` — the `String` producer path was never walked

QA defeated the `AC-4` content pin with a compile-preserving mutation confined
to `define_store_bytes_len`:

```rust
class_guard(&mut b, node, &[BoundaryClass::Bytes, BoundaryClass::String]);
→ class_guard(&mut b, node, &[BoundaryClass::Bytes]);
```

That makes emitted `String` **construction** impossible while leaving `Bytes`
construction and `String` reading intact. **Every test stayed green**, including
`b2v_a_separately_compiled_consumer_distinguishes_equal_length_strings` — because
that control materialized its handles in Rust and only had emitted code *read*
them. It proved emitted `String` **projection**, never the producer path.

## ⭐ This is the third instance of one failure class in this candidate

Round 2 closed the `store_slot` defect and I wrote the rule for it: *a pin that
never exercises the mechanism which would violate it is not evidence about that
mechanism.* **I then shipped another instance of it in the same file**, and my
own doc comment states the false premise out loud:

> *"The producer arm is covered by the `Bytes` control above, which shares every
> code path but the class."*

⛔ **The class is exactly the axis `store_bytes_len` and `store_byte` guard on**,
so it is the one code path that is *not* shared. "Shares every code path but X"
is never an argument that X is covered — X is the difference, and the difference
is what needs the test. A special-cased branch does not inherit the invariants
of the generic path, and neither does a *guarded* one.

## The repair is reachability, not a stronger assertion

`emit_bytes_producer` becomes `emit_span_producer` and takes the class as a
**run-time argument**, so one separately compiled body drives
`alloc(PersistentGround, class)` → `store_bytes_len(len)` →
`store_byte(i, seed + i)` for **both** classes at run-time bounds. Both arms of
every span-writing guard are now reached by emitted code, and the class cannot
be baked in at compile time even in principle.

| control | what it adds |
|---|---|
| `b2v_emitted_code_constructs_equal_length_bytes_and_strings_by_content` | both classes × three equal-length seeds, built and read entirely by emitted code |
| `b2v_the_two_string_producers_agree_byte_for_byte` | a store-materialized `String` and an emitted-constructed `String` read **identically** through the same consumer — retains the coverage the removed control carried |

⚠ The cross-class **positive control** is that the same seed yields the same
bytes in either class. Without it the sweep could pass by inferring the class
*from* the content, which would mean neither producer was really being varied.

## The mutations

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M14** | **QA's exact mutation** — `define_store_bytes_len`'s `class_guard` narrowed to `Bytes` | **both new controls** | **none — 2 tests, 25 pass** |
| **M15** | `define_byte_access`'s `class_guard` narrowed to `Bytes` | both new controls | **none — 2 tests** |
| **M16** | *(test-side vacuity control)* `emit_span_producer` ignores the class argument and hard-codes `Bytes` | both new controls, on the class assertion | none — 2 tests |

**M14 is the record.** It is the mutation that was green on `ea8d9824` and is
red here, on the same production bytes — so the delta is the control, which is
what a reachability repair has to demonstrate.

**M15 answers the regression question** the replacement raises: the removed
`String` control covered the *read* side, and M15 confirms the two new controls
still redden when `String` reading breaks. Coverage retained, not traded.

**M16 is labelled as what it is** — a mutation of the *test*, not of production,
so it is evidence about the control's non-vacuity and nothing else. It exists
because the sweep's class assertion would otherwise be the one thing in the
repair with nothing behind it: without M16, "the producer built a `String`"
rests on a `class_code` read that no mutation had ever falsified.

Each restored byte-identically, verified with `git diff --quiet`. `-p
ken-runtime` **395/0** at the fold; all three censuses re-run **unmoved** (no
production bytes changed).

---

# ⛔ THIRD ARCHITECT BLOCK of `ddff2fae` — two production defects

The test-only fold was sound on its own axis and production bytes carried
unchanged. Two defects remained, and the Steward's armed line fired on this
block: three consecutive production blocks on one node, so the Architect was
asked whether they share a predicate. **They do** — *the admitted disposition is
not closed under emitted producer → boundary word → separately compiled consumer
round trip.* Both defects below are faces of it.

## 1. The disposition promised a spill that did not exist

`lowering/mod.rs` classifies **every** `Lowered::Int` as an immediate with a
`PersistentGround`/`Int` spill. Materialization was `int.as_small()?` and
`store_int_tag` refused a persistent `Big` — so the promised spill did not exist
for **exactly the values a bignum language exists to carry**, and I had recorded
that as a `NO CONTROL — open residual` rather than the missing deliverable it
was.

⚠ **The residual was honestly written and still wrong, and the reason is worth
keeping.** My argument was: a `Big`'s limbs live in an entry the *invocation's*
native arena mallocs, so a persistent node naming one is the ephemeral-locator
defect. **That argument is correct.** What it establishes is that
`NATIVE_INT_BIG_TAG_V1` cannot be persistent — **not** that a persistent wide
`Int` cannot exist. I generalized from "this representation cannot" to "the
value cannot," and the second does not follow.

### The mechanism

A wide `Int`'s magnitude goes in **the region's own limb table** — the same
region-selection rule every other class already obeys. A `Bytes`'s content is in
its region's data table; a `Constructor`'s children are in its region's word
table; a persistent value's magnitude belongs to the persistent region, beside
the node that names it.

| marker | magnitude lives | admitted for |
|---|---|---|
| `NATIVE_INT_SMALL_TAG_V1` | the node's own payload word | any owner — it names no storage |
| `NATIVE_INT_BIG_TAG_V1` | the invocation's `NativeIntArenaV1` | `InvocationArena` |
| `BOUNDARY_INT_REGION_LIMBS` | the region's limb table | `PersistentStore` |

`BOUNDARY_INT_MARKER_OWNER` is that table, and `boundary_int_marker_mask`
compiles it to the bitmask the CLIF tests in Θ(1) — the third instance of the
pattern, after the tag × class relation and the immediate domains.

⭐ **A consequence, derived rather than decided:** `Int` appears under
`PersistentGround` and nowhere else in the tag × class relation, so **every
allocatable `Int` node is persistent** and the invocation `Big` marker is
refused on every node the ABI can build. That is not a second rule — it falls
out of the first table, and the sweep asserts the `Int`-admitting tag set is
exactly `[PersistentGround]` so that admitting an invocation-owned `Int` later
reddens here and forces the marker question to be re-answered rather than
inherited.

⚠ **A dedicated table and dedicated node fields, not a reuse of the word
table.** `ken_boundary_field_local` and `ken_boundary_field_count_local` are
**not class-guarded**, so limbs parked in the word table would be readable as
child *words* — a raw magnitude limb returned where a tagged `BoundaryWord` is
expected. Node stride 64 → 80, region header 112 → 136.

**Emitted construction ships with the read path** (`store_int_limbs`,
`store_int_limb`), because QA's block established that a working read path is
not evidence the producer exists. Helpers 25 → 27.

## 2. The emitted immediate mint truncated instead of checking

`ken_boundary_make_immediate_local` built its word with a left shift and checked
only that the tag was below the first handle tag. **A shift is total.** A payload
wider than the 56-bit field silently became a *different value*; a `Bool` payload
of `2` became a third boolean — while `boundary_value.rs:268` said emitted code
performed the identical range test.

⚠ **The only magnitude control exercised `materialize_ground`**, which is a
different producer entirely. Same shape as the `String` defect QA found one
round earlier: a property asserted about a path the control never walked.

`BOUNDARY_IMMEDIATE_DOMAIN` is now the one authoritative payload domain per
immediate tag. `BoundaryWord::immediate` asserts it — the shift is total in Rust
too — and the CLIF evaluates all three domain predicates and selects by a mask
computed from the same table. A `Bool` that is not a bit is `ERR_SHAPE`; a
magnitude past the field is `ERR_BOUNDS`, so a control can tell which rule
refused without reading the payload back.

## The mutations

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M17b** | the signed-payload predicate always holds | the immediate sweep | **none — 1 test** |
| **M18** | the CLIF's `Bit` domain lookup drifts to `UnsignedPayload` | the immediate sweep | **none — 1** |
| **M19** | the region magnitude reads its length from `NODE_FIELD_COUNT` | both wide-`Int` controls | **none — 2** |
| **M20** | claiming a limb span does not advance the region's live count | the emitted wide-`Int` producer | **none — 1** |
| **M21** | a persistent `Int` is checked against the *invocation's* marker mask | the marker sweep **and** the producer | 2 |
| **M22** | a materialized wide `Int` is marked as if its magnitude were a `Small` | the materialization control | **none — 1** |

### ⚠ M17's first form reddened 142 tests and is reported as uninformative

`let admitted = b.ins().bor(some, signed_ok);` → `bor(some, one)` looked like
"stop range-checking." It reddened **142** tests across the whole backend —
`object_linker_packaging`, `native_execution_differential`, every lowering
suite. That breadth is the tell, and the captured message is the proof:

```
boundary-value local helper verification:
  - inst26 (v27 = bor.i8 v26, v5): arg 1 (v5) has type i64, expected i8
```

⛔ **It never tested the range check.** `one` is an `i64` and the domain flags are
`i8`, so the mutated graph is type-invalid and fails the CLIF verifier at
**emission** — every consumer of the boundary graph dies before any value is
minted. A wide redden reads like a strong result and here it establishes only
that emission is verified. **M17b** mutates one typed-correct predicate and
reddens exactly the sweep.

⭐ Recorded rather than dropped, for the same reason M13's no-op was: a table
showing only M17b would imply I had found the defect on the first try, and the
first try measured something else. This is *"a mutation that reddens does not
confirm which detector caught it"* with the detector being the **verifier**.

## What this closes against the RECUT, and what it does not

The recut promotes three `NO CONTROL — open residual` rows into `AC-10`'s scope.
Stated exactly, so *discharged* and *never asked* cannot read alike:

| promoted residual | status after this fold |
|---|---|
| `AC-4` `Big` at the persistent boundary | **CLOSED** — a real persistent representation, materialized, emitted-constructed, and read back across an arena drop |
| `AC-1` tag *reachability* | **STILL OPEN** — the marker sweep asserts the `Int`-admitting tag set, which is one axis of it, not the sweep over the admitted domain `AC-10` asks for |
| `AC-6` persistent *content-addressing* | **STILL OPEN** — an emitted-constructed node still carries `NULL_SLOT` |

⛔ **`AC-10` itself — one control total over the admitted disposition — is NOT in
this fold.** The recut is a review ref that has not bound, and the Architect
stated the classification adds no new constraint to the fold in flight. The two
defects here are faces of the predicate and are closed as such; the structural
closure that would make further faces unreachable is the next fold's
deliverable, and claiming otherwise would be the overclaim the recut exists to
stop.

---

# ⛔ FOURTH ARCHITECT BLOCK of `fd4e7f08` — three production defects

All three were real and all three are closed. ⭐ **What is different about this
block is that every finding is a property the previous fold *asserted in prose*
and did not enforce** — a size constant with no consumer, a "fails closed before
publication" comment above a check that could not see the thing it claimed to
check, and a "before any address is formed" comment above an address formed from
an unchecked operand.

## ⛔ REPAIR #1 WAS REDONE — the bound `AC-1` layout-closure clause excludes it

The recut bound mid-fold (`e4fa5ec5`) with a clause my first repair does **not**
discharge:

> The node/header field inventory is the **sole layout authority**. Any
> declared/exported extent is **mechanically derived** from that inventory **and
> is consumed** by allocation/publication, **or it does not exist**. ⛔ Checking
> a hand-maintained constant against another hand-maintained constant does not
> discharge this.

⚠ **My first repair was exactly the excluded shape** — I kept
`BOUNDARY_REGION_HEADER_BYTES` as a hand-written `136` and added a hand-written
`BOUNDARY_REGION_HEADER_FIELDS` list to check it against. Two authorities cannot
check each other; whichever one a future editor updates, the other silently
becomes the wrong one, which is precisely how 136-vs-144 happened.

**Mechanism chosen: derivation, backed by exhaustiveness.**

| what | before | now |
|---|---|---|
| inventory | a hand-written `&[(&str, i32)]` list | `NodeField` / `RegionHeaderField` enums |
| offsets | hand-written literals `0, 8, 16, …` | `position × 8`, from the enum |
| extents | hand-written `88` / `136` | `ALL.len() × 8` |
| publication | positional `vec![…]` | sized from `ALL`, each word placed at its own field's offset |
| node construction | positional array literal | placed by field |
| a new field | a list entry someone must remember | a **compile error** in `publish` / `push_node` |

⭐ **Both consumers place every word through a `match` with no `_` arm**, so the
inventory is not merely *declared* to be the authority — a field that exists and
has no value does not build. That is the exhaustive-by-construction rule the
federation already holds, applied to layout.

⛔ Two axes therefore **cannot drift**, and I would rather say so than
manufacture a mutation for them: the declared extent is `ALL.len() × 8` and the
offsets are `position × 8`, so neither has an independent value to drift *to*.
What remains falsifiable is what publication actually emitted and whether an
emitted-side constant still names its own field — and those are M29 and M30.

## 1. The declared layout was not the published layout — in both directions

`BOUNDARY_REGION_HEADER_BYTES = 136`; `publish` emitted **18** words = 144 bytes;
the constant had **no consumer anywhere in the tree**. So the reviewed
"112 → 136" claim was false *and* unenforced, and neither half could detect the
other.

⚠ **I introduced the discrepancy by copying a pattern I had not read.** The
previous header ended in a trailing `0` and I kept it, reasoning "mirror the
existing spare." The old form was 14 named fields for a 112-byte constant plus
one pad; mine was 17 named for 136 plus one pad. **The pad was never load-bearing
— it was the residue of a positional literal nobody derived**, and preserving it
carried the defect forward one layout at a time.

`publish` now sizes its vector *from* the constant and writes *through* the
offset constants, so a stale constant is an out-of-bounds panic.
`BOUNDARY_REGION_HEADER_FIELDS` and `BOUNDARY_NODE_FIELDS` close the other
direction: the offsets must be exactly the 8-byte slots of the declared size. A
**new field without a size bump**, a **size bump without a field**, and a **field
nobody listed** all redden.

## 2. Emitted wide-`Int` construction could publish a word denoting no integer

`store_int_limbs` checked `sign <= 1` and capacity. It admitted `len = 0`, a
leading zero limb, and negative zero — a leading zero gives one value two
encodings and negative zero gives zero a second one, both against
`RuntimeIntV1::canonical_sign_and_limbs`.

⚠ **And my control could not have caught it**: it used an arbitrary nonzero seed
and a fixed length, so it never went near any of the three boundaries. This is
[the boundary-testing rule] and the *shape* of QA's earlier finding — a property
asserted about inputs the control never supplied.

⭐ **The clauses are not checkable where the span is claimed**, because no limb
exists yet. The Architect said so explicitly (*"cannot be placed before the limbs
exist unless the interface changes"*), and the interface change is a completion
step: `ken_boundary_seal_int_local` checks the finished magnitude and **every
reader requires the seal**. An unsealed node denotes nothing — which is the only
operative meaning of "before publication" once `alloc` has handed a word back.
Claiming a span **unseals**, so a stale canonicity proof cannot survive a
reclaim.

⚠ A one-limb `[0]` **is** canonical — it is the value zero. Rejecting it would be
an over-strengthening the contract does not entail, and the case list includes it
on the admitted side.

## 3. The span check wrapped

`end = at + len` with CLIF's wrapping `iadd`, accepted on `end <= live`, then the
address formed from the **unchecked** `at`. A start near `u64::MAX` wraps to a
small sum and passes.

★ **The Rust oracle three hundred lines away used `checked_add` and was
correct.** Two halves of one property, written to different standards, in one
candidate — and the CLIF half carried a comment asserting the property the Rust
half actually had. `region_limb_base` is now the one non-wrapping form
(`at <= live && len <= live - at`), shared by the reader and the seal.

## And the writer inventory was a prefix scan

Adding `seal_int` reddened `b2v_emitted_code_cannot_assign_store_identity` —
because the pin discovered writers by `name.starts_with("ken_boundary_store_")`,
and `seal_int` writes `NODE_INT_SEALED` under a name that does not match.

⛔ **That is the forbidden-needle defect wearing an allowed-inventory costume.**
The *comparison* was against a permitted list, which is right; the *discovery*
was a spelling rule, which is not, and a discovery rule keyed on spelling cannot
enumerate an inventory. It is now a **total partition** — every helper is a
declared reader or a declared writer and the union must equal
`BOUNDARY_LOCAL_HELPERS` — so a new helper of any name reddens until someone
classifies it.

⭐ My own change surfaced this, which is the pin working; but it had been true
since the writer list was written.

## The mutations

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M23** | the shipped **wrapping** span check, restored verbatim | the wraparound control | **none — 1 test** |
| **M24** | the declared header size drifts from the named fields | the layout inventory | **none — 1** |
| **M25** | the seal accepts every magnitude | the canonicity control | **none — 1** |
| **M26** | readers stop requiring the seal | the canonicity control | **none — 1** |
| **M27** | the length floor is dropped where the span is claimed | the canonicity control | **none — 1** |
| **M28c** | the inventory grows without a distinct field | the layout control | **none — 1** |
| **M29** | publication emits one word more than the derived extent | the layout control | **none — 1** |
| **M30** | an emitted offset constant names another field | the layout control **and** the wide-`Int` read | 2 |

### ⚠ Two more mutations that did not measure what they aimed at

**M28** (first form) added an eighteenth entry to a `[…; 17]` array. It is a
compile error — but the mechanism that fired was the **array length**, not the
`match` exhaustiveness I was aiming at, and saying "compile error" without
saying *which* one would credit the wrong closure. **M28c** grows the inventory
in a form that compiles and reddens the control.

**M31** halved the derived offset stride. The harness **SIGSEGV**s: every emitted
read lands in the wrong place and the process dies before an assertion runs. That
demonstrates the offsets are genuinely consumed rather than decorative, which is
worth something — but it is **not** evidence that a control catches the drift,
and it is the same class as M17's 142-test redden: the mutation destroyed the
artifact instead of exercising the detector.

M23 is the record for defect 3: it is the **exact prior code**, so its redden is
the discriminator rather than an argument about one.

### ⚠ The restore check was wrong for this fold, and I am saying so

Every run printed `!! NOT RESTORED`. The harness tests `git diff --quiet`, which
answers *"is the tree dirty"* — true throughout, because **the fold was
uncommitted while the mutations ran.** The restore itself was byte-exact (the
harness writes back the captured original), and I confirmed it two ways: each
mutation site greps back to its production form exactly once, and the suite
returns 401/0. The mutations were then **re-run against the committed baseline**,
where `git diff --quiet` is meaningful and reports `BYTE-IDENTICAL`.

⛔ Two lessons, both mine. **`git diff --quiet` is a claim about the whole
worktree, not about the file you mutated** — its scope silently stopped matching
its question the moment the baseline was dirty. And I ran mutations before
committing the fix, which is the ordering my own notes forbid *precisely because
it destroys the baseline the check depends on*.

---

# ⛔ `AC-3` — the five static encoding policies, routed after I flagged the gap

I reported in the `9b4e6684` handoff that the `AC-3` control proved
wildcard-freedom only and that policy assignment was unstarted. The leader held
the handoff and routed it; this closes it.

## The gap was that the TYPE could not express the claim

`AC-3` requires each of the 21 variants to carry **exactly one of five** static
policies, and names the misassignment it cares about: *a variant with a declared
spill must not be assigned immediate-only.* But *immediate-only* and
*immediate-with-declared-handle-spill* were **the same constructor**
distinguished by an `Option` field, so "exactly one of five" was a **reading** of
`BoundaryDisposition` rather than a fact about it — and a claim the type cannot
express is one a control has to restate, which is how the misassignment survives.

⛔ **And exhaustiveness does not help.** "No `_` arm" says every variant has *a*
disposition; it says nothing about *which*. The existing control could not have
caught a spill arm assigned immediate-only, and neither could any strengthening
of it.

## Two structural changes, one of which was the real problem

**`StaticEncodingPolicy`** is now a five-variant enum derived from a disposition
by an exhaustive `match`, so the five are five in the type.

**`LoweredVariant`** is the variant *tag*, and `boundary_disposition` takes
**that** rather than `&Lowered`. ⭐ **This is the one that matters.** The frame
says a policy is a claim about a whole variant, never about a sampled value — but
a disposition taking `&Lowered` can only be swept by constructing 21 values, so
any sweep would have been asserting a variant-level claim from value-level
evidence. Taking the tag makes the sweep **total by construction**: there is no
value to sample, and the function cannot come to depend on a payload without
someone changing its signature. Both `Lowered::variant` and
`LoweredVariant::boundary_disposition` are `match`es with no `_` arm.

## The control and its discriminators

`b2v_ac3_every_variant_carries_exactly_one_of_the_five_static_policies` sweeps
all 21 tags, asserts each policy is in the closed five, and checks the spill
correspondence in both directions. Its non-degenerate pair is `Int` (declares a
spill) against `Bool` (does not) on the same assertion — a checker ignoring
`spill` puts both in one policy and passes everything else. Every policy must be
**inhabited and not universal**, so an unreachable policy and a degenerate
assignment both redden.

| # | mutation | reddens | collateral |
|---|---|---|---|
| **M32** | `policy()` maps a declared spill to *immediate-only* | the policy control | **none — 1 test** |
| **M33** | `Lowered::Int` drops its declared spill | the policy control | **none — 1** |

⚠ **The old source scan's positive control fired during the refactor** — its
extractor anchors went stale and it failed with *"the extracted region does not
contain a token that is certainly in it, so its silence about `_ =>` means
nothing."* That is the control working: it refused to report a green it could not
justify. Retargeted onto the tag dispatch and kept, because it guards something
the new sweep does not — that nobody **silences** exhaustiveness with a binding
catch-all.

⚠ **`rustfmt` reformatted the module tree again.** Formatting `lowering/mod.rs`
reflowed 65 lines of `lowering/core.rs`, which this fold does not touch. Caught
by the diff-stat audit and reverted; `control.rs`'s delta is content-only under
`git diff -w`.

---

# ⛔ `AC-10` — total classified-domain closure

Routed after I reported it as `NO CONTROL — open residual`; the landed frame
promotes the three former residuals into its scope, so that spelling is no longer
merge-permitted.

## The claim is unenumerable, so the closure is structural — and says so

⛔ **"One control total over every value" is not an executable oracle.** The
admitted domains include unbounded integers, arbitrary byte contents, ownership
states and recursive parent → child reachability. A finite sweep wearing a
universal name is worse than an honest one, because it reads as total. The
closure is therefore two layers:

1. the sealed wildcard-free disposition closes the **variant** layer;
2. every **value-dependent discriminator** is a closed finite partition —
   *magnitude/shape*, *lifetime/owner*, *parent → child reachability*, and *which
   producer minted the referent* — reached from a value by a **total**
   projection: `int_fits_immediate`, `referent_owner`, and "does this aggregate
   hold an invocation-owned child".

⭐ **The infinite domain is covered by construction; only the finitely many CELLS
need controls.** `BoundaryInput::outcome` is a total function from a cell to
exactly one `BoundaryOutcome`, with no `_` arm anywhere, and the sweep runs the
whole 21 × 2 × 3 × 2 product.

## Classification first, behaviour entailed — not "either/or"

⛔ The failure arm belongs to the **unrepresentable** class, never inside the
admitted one. A predicate reading *"for every admitted value, either round-trip
or fail closed"* is satisfied vacuously by an implementation that rejects
everything, which is why the frame was rewritten and why the control asserts
`outcome.permitted_by(policy)` **and** that all four outcomes are inhabited.

## What each partition buys, and how it is falsified

| discriminator | boundary | witness pair | causal mutation |
|---|---|---|---|
| **magnitude/shape** | `int_fits_immediate` | `MAX` vs `MAX + 1`, **adjacent**, through one emitted body making a run-time decision | **M34** (spill arm stops being a handle), **M37** (the *emitted* test drifts off the field width) |
| **parent → child reachability** | a child that dies before its parent | the same variant with sound children is admitted | **M35** |
| **lifetime/owner** | persistent vs invocation | the tag's `referent_owner` must equal the declared owner | folded into the sweep's handle check |
| **producer / identity** | store-materialized vs emitted-constructed | `StoreMinted` vs `NoStoreIdentity` on one variant | **M36** |

⭐ **`b2v_ac10_the_magnitude_boundary_is_a_real_emitted_partition` is the one
that was missing.** The classifier's magnitude claim is a claim about *emitted*
behaviour, and every prior magnitude control ran **Rust** materialization — the
same defect QA found on the `String` arm. One compiled body now performs the
run-time test and takes both arms; a separately compiled consumer reads back the
spill arm's class, owner, identity and content. ⚠ On the immediate arm the
`owner` probe **refuses** (`ERR_SHAPE`) rather than answering `NoReferent` — an
immediate has no node to project from — and that refusal *is* the nondegenerate
half: the same probe answers `PersistentStore` one value later.

## ⚠ One classification I chose, and the fork behind it

`AC-6`'s promoted residual is that an emitted-constructed node stays
`NULL_SLOT`. I closed it by making identity an **outcome**: `HandleIdentity` is
part of the classification, so a consumer recovers exactly the identity the
classifier predicted rather than the question going unasked.

⛔ **This is a classification, not a narrowing** — no value leaves the admitted
domain. But it takes one reading of *"identity intact"*, and the alternative —
the store **adopting** emitted-constructed nodes so they carry a real `SlotId` —
is a lifecycle decision above this node and would change the answer. I am
flagging it rather than presenting the reading as the only one.

---

# ⛔ ARCHITECT RULING — `NoStoreIdentity` does not satisfy `AC-10`/`AC-6`

`81a68435` is preserved as a checkpoint, not a candidate. I had flagged the
identity reading as a fork and asked for a ruling; the ruling went against my
reading, and the reasoning is decisive.

## Why my closure was wrong

⛔ **Explicitness is not preservation.** A separately compiled consumer can
recover the *fact that no store identity exists*. It cannot thereby recover
**the same identity intact**, which is what `AC-10` requires of a handle
outcome. Renaming a residual as an outcome does not discharge the residual.

⛔ **And it contradicted its own layout.** The candidate declared the referent
owner `PersistentStore` while `NODE_SLOT` stayed null — and this ABI's node
contract says a null slot **denotes invocation-arena ownership**. The word was
internally inconsistent, and I did not notice because I was reading the identity
question as a lifecycle *preference* rather than as a fact the layout already
fixed. Reserving persistent-region storage is **storage governance, not
adoption**.

## The mechanism

`BoundaryValueStore::adopt` is the trusted store-owned boundary:

1. **Bottom-up over the reachable graph**, so no parent is adopted while a
   reachable child is still pending; an invocation-owned child is `ERR_ESCAPE`.
2. **Canonicalize and intern** through the landed `persist` path — so equal
   values independently emitted converge on one `SlotId` and unequal values
   cannot alias, *because `Store::intern` already guarantees exactly that*. No
   second identity mechanism.
3. **Mint or reuse.** An already-placed slot returns the existing store-owned
   word and abandons the pending node; otherwise the slot is installed here.

⭐ **Mint authority stays the store's.** `set_node_slot` is module-private with
one caller; emitted code still has no `NODE_SLOT` setter and the emission latch
keeps it that way. The control asserts both halves together, so "adoption works"
and "forgery is impossible" cannot be confused for one another.

⛔ **The emitted escape gate now requires adoption**, which is what makes it
non-optional rather than advisory: a pending persistent word cannot cross a
generated-function boundary. `AdoptionPartition` joins the closed partitions, and
a pending persistent node classifies as `FailClosedForbidden` — *not published*
— rather than as a handle with a missing identity.

⚠ **One thing adoption had to do that I did not anticipate.** The emitted
allocator bumps the *published header's* live counts; the Rust-side counts still
describe the region as it was at publication. Re-publishing without absorbing
them **truncates exactly the nodes being adopted** — the referent vanishes from
under its own new identity. `absorb_published_counts` is part of adoption for
that reason, and the control caught it: the consumer read `ERR_BOUNDS` where it
expected a slot.

## The mutations

| # | mutation | reddens | collateral |
|---|---|---|---|
| **M38** | the emitted escape gate stops requiring adoption | the adoption round trip | **none — 1** |
| **M39** | adoption mints again instead of reusing an interned identity | the convergence differential | **none — 1** |
| **M40** | adoption installs no identity | both adoption controls | 2 |
| **M41** | a pending persistent node classifies as a published handle | the `AC-10` sweep | **none — 1** |

⚠ **Honest residual, stated rather than discovered:** `read_ground` decodes
`Int`, `Bytes`, `String`, `Constructor` and `Record`. `Closure`, `HostResult` and
`BorrowedOpaque` have no canonical store image, so adoption **refuses** them with
an exact status. That is a conservative reject, not a silent admission — but it
does mean an emitted-constructed persistent `Closure` cannot be published, and
`PersistentClosure` is a live represented arm of the disposition. **That is a
real gap in the represented population**, and it belongs on the record rather
than inside a green.

---

# ⛔ THE CYCLE QUESTION, ANSWERED — and it was a live defect

The ruling on the `Closure` gap says: *"If closure cycles are constructible,
stop and route rather than recursing accidentally."* I answered that before
building, and the answer is worse than it sounds.

## Measured: cycles ARE constructible, and the shipped adoption recursed on them

`ken_boundary_store_field_local` refuses only a **persistent parent with an
invocation-owned child**. So emitted code can allocate two persistent nodes and
write each as the other's child — and **both writes return `OK`**, through every
guard: bounds, tag, frozen prefix, escape.

⛔ **This is not a closure question. It was a live defect in the ground adoption
I shipped at `fe7d8a08`**, which recursed over `Constructor`/`Record` children
with no cycle guard. `b2v_ac10_a_constructible_node_cycle_is_refused_not_recursed`
builds the cycle through the emitted interface, asserts both writes are admitted,
and asserts adoption returns an exact status.

⭐ **M42 is the sharpest mutation on this node.** Removing the guard does not
redden the control — it **stack-overflows and aborts the test binary**. That is
precisely the failure the guard prevents, demonstrated rather than argued.

⚠ The positive control cost me a round: my acyclic fixture used arbitrary
constructor ids, and adoption refused it — correctly, because a node naming an id
the store never interned has no canonical image. Interning first keeps the
control about the cycle rather than about symbols.

## Also fixed: adoption was retagging canonicalized children

`adopt_node` rewrote a canonicalized child as `PersistentGround`. For a nested
`PersistentClosure` that **silently changes what the child is** — the exact
retagging the ruling forbids. The child's tag is now preserved; it is the
child's, not the parent's to choose.

## ⛔ WHAT IS NOT IN THIS FOLD — the `Closure` canonical-image layer

**Unstarted, and I am not shipping a partial one.** The ruling requires a closed
canonical-image layer over `Value::Closure { code_id, captured }`, with `code_id`
derived from B2O/B2R callable-unit identity in an **artifact-bound** namespace.
What I established while scoping it, which the next fold can start from:

- `Value::Closure { code_id: u64, captured: Vec<Value> }` **already exists**
  (`values.rs:74`), so the image layer returns `Value`, not
  `RuntimeGroundValue` — that is the "do not force `Closure` through
  `read_ground`" instruction, and it also fixes tag preservation for free,
  because each child's image kind follows its own node class.
- ⛔ **`StaticOriginId` is a bare `u32` ordinal** (`semantic_ir.rs:26`) — exactly
  the "collides across artifacts" case the ruling excludes. It must be combined
  with `RuntimeArtifactIdentity { package_identity, core_semantic_hash,
  artifact_hash }` (`artifact_validation.rs:50`), which means the store needs an
  **artifact binding** it does not currently have, and adoption must fail closed
  while unbound.
- Adoption would intern `Value` directly rather than through
  `persist(RuntimeGroundValue)`, so a `persist_image` seam is needed.

⚠ `HostResult` and `BorrowedOpaque` stay invocation-owned and keep rejecting, per
the ruling — that part is unchanged and correct.

---

# AC → discharging control

Required by the frame's second amendment (`origin/main` = `fdda953f`): one row
per AC, with evidence. ⛔ **An AC with no control reads exactly
`NO CONTROL — open residual`** — there is no cell here for "covered by review",
because a taxonomy with nowhere to put the honest answer records it as covered.

| AC | discharging control | evidence |
|---|---|---|
| **AC-1** closed + type-enforced | `b2v_the_tag_set_is_closed_in_both_directions`; `b2v_emitted_code_admits_exactly_the_closed_tag_set` | Rust sweep + **256-byte emitted sweep** through `class` and `escape_check`; admitted count `== ALL.len()`, rejected `== 256 - ALL.len()`. A new tag is a compile error at `from_bits`'s closed `match` and at `ALL` |
| **AC-1** region bands | `b2v_the_region_thresholds_agree_with_referent_owner` | the CLIF's numeric bands classify every tag exactly as `referent_owner()`; both bands asserted non-empty |
| **AC-2** no value-specialization | **the compiler** + `b2v_the_immediate_handle_choice_tracks_magnitude_only` | `boundary_value` imports no `NativeSeedEnvironment` and no environment vector — passing one **does not compile**. Magnitude cases test `MAX`, `MAX±1`, `MIN`, `MIN±1` |
| **AC-3** five static policies | `b2v_ac3_every_variant_carries_exactly_one_of_the_five_static_policies` | all 21 variant **tags** swept — total by construction, because `boundary_disposition` takes `LoweredVariant` and has no value to sample. `StaticEncodingPolicy` makes "one of five" a fact about the type rather than a reading of an `Option`. Non-degenerate pair `Int` (declares spill) vs `Bool` (does not); every policy asserted inhabited and not universal. Causal: **M32** (a declared spill mapped to immediate-only) and **M33** (`Int` loses its spill) |
| **AC-3** exhaustive, no wildcard | `b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm` | arm-*head* enumeration (every `=>` line starts `Lowered::` or `\|`), 21 variants named, `Constructor`/`HostResult` checked **positionally** outside the fail-closed block, single-dispatch assertion. ⚠ first form was **defeated** by a binding catch-all |
| **AC-4** **content** | `b2v_a_separately_compiled_consumer_reads_a_spilled_int_by_content`; `b2v_emitted_code_constructs_equal_length_bytes_and_strings_by_content`; `b2v_the_two_string_producers_agree_byte_for_byte` | a spilled `Int` is a `NativeIntV1` pair decoded by `ken_native_int_resolve_local`; `Bytes`/`String` are **built and read** byte-by-byte from the region's data span, by emitted code, with the class a run-time argument. Every case is **equal length** or **asserted to spill**, and the results are asserted mutually distinct. ⚠ The prior form was **defeated** — its `String` handles were Rust-materialized, so `M14` was green on `ea8d9824` |
| **AC-4** **construct** | `b2v_emitted_code_constructs_a_nonconstant_constructor_and_a_consumer_reads_it`; `…constructs_both_host_result_arms`; `…constructs_a_record_readable_by_name`; `b2v_construction_fails_closed_at_each_ceiling` | a separately compiled **producer** mints each live class from `alloc` + `store_*`; one compiled body, three runtime heads; every ceiling and closed-set refusal asserted at its **exact** status |
| **AC-4** **discriminate** | `b2v_emitted_code_selects_the_host_result_arm_at_runtime`; `…constructs_both_host_result_arms` | both arms, runtime discriminant, same compiled body |
| **AC-4** **project** | `b2v_emitted_code_projects_a_non_constant_constructor_field`; `…projects_a_nested_aggregate` | separately compiled consumer; no step runs in Rust |
| **AC-5** M1/M2/M3 | recorded in the mutation table above and in the prior candidate's rows | each names **which** detector fired and its `left`/`right` |
| **AC-5** M4b/M5/M6/M7/M8 | the new mutation table | **M5, M6, M7 redden exactly one control each** — the detector is named by the redden itself |
| **AC-5** M9–M13b | the second block's mutation table | ⚠ **M13's green is reported as a no-op**, not dropped: the emission `assert!` is a latch, so only **M13b** (re-wiring the setter) proves it |
| **AC-5** M14/M15/M16 | the QA-block mutation table | **M14 is QA's own mutation** — green on `ea8d9824`, red here on unchanged production bytes. **M16 is labelled test-side**, so it is evidence about the control's non-vacuity only |
| **AC-6** owner distinguishable | `b2v_referent_owner_distinguishes_persistent_from_borrowed` | `BoundaryReferentOwner` is a **distinct type** from `AbiStorageOwner`; the pair is non-degenerate (`left: 1, right: 2`) |
| **AC-6** persistent identity | `b2v_a_constructed_persistent_word_survives_the_invocation_arena`; `b2v_the_frozen_prefix_refuses_emitted_mutation`; `b2v_equal_values_share_one_persistent_referent` | the arena is **dropped** and a second invocation resolves the same word; orphan-arena positive control returns `ERR_BOUNDS`; one slot ⇒ one word, byte-identical across invocations |
| **AC-7** escape, exact error | `b2v_borrowed_ingress_fails_closed_on_escape_with_an_exact_error`; `b2v_a_persistent_node_refuses_an_invocation_owned_child` | exact `ERR_ESCAPE`; malformed ⇒ `ERR_TAG`, not `ERR_ESCAPE`; the construction-time invariant the Θ(1) check rests on is itself pinned, with both mirrors as positive controls |
| **AC-8** INERT | `correspondence_adds_no_emitted_unit_to_the_production_census`; `the_backend_production_surface_inventory_is_closed`; `px8i_…identical_local_helper_clif` | all three **unchanged and re-run**; `lowering/mod.rs` census stays `0`/`0`/`0`. ⚠ **Helper delta re-baselined: 13 → 19 → 25 → 27 → 28** (`seal_int`) — `alloc`, `store_tag_id`, `store_scalar`, `store_field`, `store_name`. Stated here before the verdict, not discovered in it |
| **AC-9** Θ(1) per module | `b2v_helper_population_does_not_grow_with_the_value_population`; `b2v_the_helper_inventory_is_closed_and_named` | two module emissions compared, and the value population varied ×1024 with the helper count fixed; the inventory pin reads the **module's actually declared** `ken_boundary_*` symbols against the permitted set. ⚠ first form was **defeated** — it only read the list |
| **AC-1** tag × class relation | `b2v_the_tag_class_relation_is_closed_over_the_whole_product` | all 81 `tag × class` pairs through the emitted allocator, expectations from `boundary_relation_admits`; admitted **and** rejected counts asserted; the CLIF's mask re-checked against the table per pair |
| **AC-6** writer/reader partition | `b2v_emitted_code_cannot_assign_store_identity` | every helper is a declared reader **or** a declared writer and the union must equal `BOUNDARY_LOCAL_HELPERS`. ⚠ The prior form discovered writers by a `ken_boundary_store_` **prefix**, so `seal_int` — which writes `NODE_INT_SEALED` — was invisible to it: a forbidden-needle defect wearing an allowed-inventory costume |
| **AC-6** no forged identity | `b2v_emitted_code_cannot_assign_store_identity`; `b2v_a_persistent_int_refuses_an_invocation_scoped_big` | `store_slot` **removed**; the allowed-writer inventory is pinned; every writer is exercised on a fresh node and the slot still reads `NULL_SLOT`, with the written fields checked as the positive control. ⚠ The emission latch is proved by **M13b**, not by the predicate |
| **AC-1** tag *reachability* | **`NO CONTROL — open residual`** — ⛔ **promoted into `AC-10`'s scope by the RECUT** | a tag no disposition can produce is caught by review, not by CI. The marker sweep now pins the tag set that admits `Int`, which is **one axis** of reachability and not the sweep over the admitted domain `AC-10` asks for |
| **AC-4** wide `Int` at a persistent boundary | `b2v_a_wide_persistent_int_materializes_and_reads_back_by_content`; `b2v_emitted_code_constructs_a_wide_persistent_int_that_outlives_the_arena`; `b2v_the_magnitude_marker_relation_is_closed_over_owner_and_marker` | **the residual is CLOSED, not carried.** The magnitude lives in the region's own limb table; the oracle is `RuntimeIntV1::canonical_sign_and_limbs`, not the emitted answer re-read. The producer is emitted CLIF and the read happens after the arena is **dropped**. The `(owner, marker)` product is swept with both counts asserted |
| **AC-1**/**AC-2** emitted immediate construction | `b2v_emitted_immediate_construction_refuses_what_it_cannot_represent` | every immediate tag × nine boundary-straddling payloads, expectations from `BOUNDARY_IMMEDIATE_DOMAIN`; the **round-trip** is the positive control, so the pin is about truncation and not about a status code; `ERR_SHAPE` and `ERR_BOUNDS` asserted as *distinct* refusals. ⚠ The prior form was **absent** — the only magnitude control exercised Rust materialization |
| **AC-5** M17b/M18/M19/M20/M21/M22 | the third block's mutation table | ⚠ **M17's first form is reported as uninformative**: it reddened 142 tests by making the graph type-invalid, so it measured the CLIF **verifier**, not the range check |
| **AC-5** M23/M24/M25/M26/M27 | the fourth block's mutation table | each reddens **exactly one** control; **M23 is the shipped wrapping check restored verbatim**, so its redden is the discriminator rather than an argument about one. ⚠ The restore check printed `NOT RESTORED` on every run — `git diff --quiet` asks about the *worktree*, and the fold was uncommitted; re-run against the committed baseline it reports byte-identical |
| **AC-1** layout closure | `b2v_the_layout_inventory_is_the_sole_authority` | ⛔ **Derivation, not constant-vs-constant** — the bound clause excludes the latter and my first repair was it. `NodeField`/`RegionHeaderField` are the sole authority; offsets are `position × 8`, extents are `ALL.len() × 8`, and `publish`/`push_node` place every word through a `match` with **no `_` arm**, so a new field does not build. The control measures a **published** header, asserts `offset + 8 <= extent` (the clause's width clause), and pins each emitted-side constant to its own field. Causal on the two axes that can still drift: **M29** (published word count) and **M30** (emitted offset); the other two are derived and have no independent value to drift to |
| **AC-1**/**AC-4** canonical emitted magnitude | `b2v_emitted_wide_int_construction_refuses_a_noncanonical_magnitude` | six magnitude shapes, one per canonicity clause, each differing from an admitted row in one component; the **unsealed-read** arm is the seal's own positive control, so a producer ignoring the status still cannot publish. ⚠ The prior control used one arbitrary nonzero seed and reached no boundary |
| **AC-1** non-wrapping span | `b2v_a_wrapped_limb_span_fails_closed` | fault-injected directly, because **no production path can build a malformed span** and a control that cannot construct the violating input is not evidence about the guard. The Rust oracle is asserted to refuse the same span |
| **AC-6** persistent *content-addressing* | **`NO CONTROL — open residual`** — ⛔ **promoted into `AC-10`'s scope by the RECUT** | an emitted-constructed node carries `NULL_SLOT`. The **limit** is pinned (the survival control asserts it); the property is not delivered. Identity minting is the store's alone, so closing this is a lifecycle decision, not a control I can add |
| **AC-10**/**AC-6** store adoption | `b2v_ac10_emitted_construction_publishes_only_through_store_adoption`; `b2v_ac10_adoption_converges_equal_values_and_never_aliases_unequal`; `b2v_ac10_adoption_fails_closed_before_publication` | emitted construct → seal → **store adopt** → a separately compiled consumer recovers the **real non-null identity** and the content after the producer's arena is gone; equal/unequal differential proving canonical reuse and non-aliasing; the emitted escape gate refuses a pending word; emitted `NODE_SLOT` assignment remains impossible while the store path is positively exercised. Causal: **M38/M39/M40/M41**. ⚠ Adoption refuses `Closure`/`HostResult`/`BorrowedOpaque` — a recorded gap, not a green |
| **AC-10** total classified-domain closure | `b2v_ac10_every_boundary_input_receives_one_policy_entailed_outcome`; `b2v_ac10_the_magnitude_boundary_is_a_real_emitted_partition`; `b2v_ac3_every_variant_carries_exactly_one_of_the_five_static_policies` | the sealed wildcard-free disposition closes the variant layer; four closed finite partitions with total projections close the value layer; the sweep runs the whole 21×2×3×2 product and asserts each outcome is **permitted by its policy**, that all four outcomes are inhabited, that a policy's outcome varies only in the discriminators it declares, and that every handle outcome carries class/owner/identity/lifetime. Causal: **M34/M35/M36** on the classifier, **M37** through the emitted producer. ⚠ Identity is discharged as a *classification* — see the flagged fork |
| **AC-10** *(superseded row)* | **`NO CONTROL — open residual`** | ⛔ **Not in this fold, and said so rather than implied.** The recut is a review ref that has not bound and the Architect stated it adds no constraint to the fold in flight. The two blocked defects are faces of the predicate and are closed as such; the structural closure that makes further faces unreachable is the next fold's deliverable |

---

# THE `Closure` CANONICAL IMAGE + THE CYCLE/DEPTH/SEAL CONTRACT

Both Architect rulings, on a fresh descendant of `9b254fb9`. This section is
written against the folded frame `wp/steward-scale-gate-nodes = 02272b62`
(blob `9b45c213`), read from the file.

## 1. The canonical-image layer replaces the two-hop ground decoder

`read_ground` produced a `RuntimeGroundValue` and `store_image` turned that into
a `Value`. Two hops, and the intermediate type **has no closure arm** — which is
why `Closure` could only ever be a conservative reject there.

`canonical_image` is one exhaustive, wildcard-free `match` over `BoundaryClass`
producing `Value` directly. ⭐ **A new class does not compile** until it is given
an image, which is strictly stronger than any test over the classes that exist
today. `Closure` yields `Value::Closure { code_id, captured }` — the normative
image the ruling names — and `persist_image` interns it.

⛔ **`Closure` is not forced through `RuntimeGroundValue`, and that was the
instruction.** Adding a closure arm to that type would have been a second value
taxonomy for one call site.

## 2. `code_id` is artifact-bound, and unbound adoption FAILS CLOSED

The node carries the **local origin ordinal**; the store binds it. Emitted code
never computes a `code_id` — it holds no artifact identity, and `B2F` dispatch
stays inert.

⛔ **The store has no artifact binding by default, so that is not a hypothetical
state — it is the initial one.** Minting from the bare ordinal while unbound is
exactly the cross-artifact collision Ruling B excludes, so `Closure` adoption
returns `BOUNDARY_ERR_UNBOUND` until `bind_artifact` is called. Ground adoption
is untouched: a ground value's identity is its content.

## 3. Adoption is three ordered phases, and the walk is iterative

| phase | what it does | why it is separate |
|---|---|---|
| **seal** | `ARENA_SEALED` in the published header; emitted writers refused | every later phase reads a snapshot |
| **validate** | iterative tri-colour worklist over the complete reachable graph | a fault must be found before any identity exists |
| **canonicalize** | postorder: image, intern, mint or reuse, repoint children | children are canonical before their parent is read |

⭐ **Tri-colour rather than a visited set, and the DAG is what forces it.** A
second edge into a node is *malformed* when that node is still on the stack
(grey) and *legal sharing* when it is finished (black). A "have I seen this?"
set collapses those into one answer and would have to reject every shared child
to stay safe on the cycle. `M45` and `M46` are the two directions of that.

⛔ **`BOUNDARY_ERR_CYCLE` is a distinct status from `BOUNDARY_ERR_SHAPE`**, so
the cycle control can say which finding it caught. A shared status would have
left that unattributable.

## 4. The seal is one definition covering every writer

`seal_guard` sits in `mutable_guard` — which all ten word-taking mutators
already call — plus `define_alloc`, which takes no word and so needs its own.
Between them that is the whole of `EMITTED_WRITERS`.

⛔ **Copying the check into eleven bodies would be the hand-maintained matrix
`RECUT 2` retires.** The control drives a probe table asserted equal to the
production writer partition, so a new writer with no seal probe is red.

⚠ **`&mut self` is not the proof, and the frame says so.** Emitted code holds
the raw region base it was published and never consults the borrow checker; the
seal has to live where the mutators actually read, which is the header.

## The mutations

| mutation | what it breaks | reddens | tests |
|---|---|---|---|
| **M43** | `mutable_guard` loses its seal check | the writer-totality control, naming `ken_boundary_seal_int_local` | 1 |
| **M44** | `define_alloc` loses its seal check | the same control, naming `ken_boundary_alloc_local` | 1 |
| **M45** | grey treated as unvisited (tri-colour → visited set) | both cycle controls **and** the no-partial-identity control | 3 |
| **M46** | black treated as grey (any repeated edge rejected) | the shared-DAG half only | 1 |
| **M47** | `code_id` drops the artifact fields | the cross-artifact collision control | 1 |
| **M48** | `code_id` drops the length prefix | **none — and that is correct, see below** | 0 |
| **M49** | canonicalization rewrites a child's tag as `PersistentGround` | the **deduped**-capture retagging control | 1 |
| **M50** | unbound store falls back to the bare ordinal | the artifact-unbound fail-closed control | 1 |
| **M51** | `child_images` sorts, collapsing capture order | the capture-order non-aliasing control | 1 |
| **M52** | `adopt` drops the seal precondition | the refuses-unsealed control | 1 |

Every mutation was restored with `git checkout --` and verified byte-identical
by `git diff --quiet` **against the committed baseline**, so the check is
meaningful rather than reporting the fold's own dirt.

### ⭐ M43 and M44 redden the same test, and the attribution is checked

Two mutations reddening one control is exactly the case where *"a mutation that
reddens does not confirm which detector caught it"* bites. The failure messages
were captured:

```
M43 -> AC-6: ken_boundary_seal_int_local must be refused once the store owns …
M44 -> AC-6: ken_boundary_alloc_local  must be refused once the store owns …
```

Different writers, so the totality is real: `alloc` genuinely is a second path
and is not covered by `mutable_guard`.

### ⚠ M48 did not redden, and my DOC was the defect

I wrote that the length prefix stops `("ab", …)` and `("a", …)` colliding.
**That is false as the function stands.** `package_identity` is the only
variable-length field and every other one is fixed-width, so the total length
already determines where the string ends — the encoding is injective with or
without the prefix.

⛔ **The mutation is correct and the claim was wrong.** The prefix is kept as
future-proofing against a *second* variable-length field, the doc now says only
that, and the assertion that was labelled as exercising it has been relabelled
a distinctness check. A control named for a property it does not test is the
overclaim this corpus keeps paying for.

### ⚠ M49 did not redden either — my control never reached the mutation site

Canonicalization only rewrites a child word when that child dedups onto a
**different** node (`if target != child.payload()`). My nested-closure control
captured two **distinct** closures, so neither deduped, the branch never ran,
and hard-coding the rewritten tag back to `PersistentGround` — the historical
defect, restored verbatim — left it green.

⛔ **That is a pin that never exercises the violating mechanism.** The added
control captures two **structurally equal, separately constructed** closures, so
the second must canonicalize onto the first and the rewrite fires. It carries
its own non-vacuity assertions: the two captures must end up naming one node,
and the second's word must genuinely have changed. `M49` then reddens it.

⚠ The sibling test is kept, because it pins a different thing — that *distinct*
nested closures survive without being merged — but it is no longer the evidence
for tag preservation, and its doc says which control is.

## ⛔ DEPTH — ***THIS SECTION WAS WRONG. Corrected in place, 2026-07-26.***

⛔ **The text that stood here claimed AC-10's depth clause was NOT discharged**,
that the end-to-end bound was ~2500, set by a recursive
`canonical::encode_canonical` plus `Value`'s derived `Clone`/`Drop`, and that
closing it was not a `B2V`-sized change. ⛔ **Every clause of that was false on
the bytes it was attached to.** It is rewritten rather than annotated, because an
appended correction leaves the false version in the position a reader obeys.

⭐ **Runtime QA found it** (`evt_7czz8h2r717z7`) by raising this control's `DEPTH`
from 2000 to 3000 and watching it pass — a one-constant probe against a claim
the candidate presented as a measured residual.

### What is actually true, measured on these bytes

| mechanism | measured | bound |
|---|---|---|
| the **former** recursive adoption, restored verbatim, 8 MiB | died between 800 and 1600 | host stack |
| **this** iterative walk | 3000, 10000, **30000** all adopt | allocation and time (~142 s at 30000), never the host stack |
| `canonical::encode_canonical` | **iterative** — work stack is a heap `Vec` | O(1) host stack in depth |
| encoder + derived `Clone` + **drop glue** | `value_depth_totality` covers all three out of process at depth **131_072**, 1 MiB stated stack | closed by `RT-VALUE-TOTALITY-P1` |

⚠ And the figure the old text cited for the *pre-change* recursive encoder was
itself low by roughly 4x: `P1`'s own bisection puts the landed pre-change
mechanisms at **9032** / **10074** / **65486** at 8 MiB, not 2000–3000.

⇒ ✅ **AC-10's depth clause IS discharged.** The walk is iterative with a heap
frontier, depth is never reclassified as malformed (asserted directly), and the
`Value` side is closed by a landed sibling WP at depth 131_072. The remaining
bound is allocation and time — an ordinary resource boundary, which is the same
language `P1` uses about the encoder — and **not** the host stack.

The deep instance is kept **executable** as `#[ignore]`d
`b2v_ac10_a_deep_acyclic_chain_adopts_at_thirty_thousand` rather than recorded as
prose here, because a measurement that lives only in a document cannot fail.

### ⛔ How the false claim survived — the transferable part

**The measurement was inherited across a re-anchor, not re-derived.** It was
taken on a **pre-P1** base. The branch was then re-anchored onto a base
*containing* `RT-VALUE-TOTALITY-P1` — the WP that made the encoder iterative —
and the number came along unre-measured.

⭐ **`P1` was on this WP's explicit do-not-touch list, and "not mine to change"
was read as "not relevant to re-check."** That is the entire error, and it is a
cheap one to repeat: a standing constraint that keeps you *out* of a subsystem
says nothing about whether that subsystem still bounds your claims. The
re-anchor was the moment to re-measure every inherited number, and the
re-anchor's own evidence went no further than proving `crates/` was
byte-identical — which is a statement about **my** files, not about the premises
they rest on.

⚠ **The residual was persuasive precisely because it was honest.** It named a
mechanism, gave a measured interval, declined to narrow itself into a green, and
routed the fix as out-of-scope. Every one of those is a virtue, and together they
made it read as settled — so nobody re-derived it, including me, twice, across a
re-anchor and a compaction.

## What this fold does NOT close

- **`ImmediateExitStatus` / `ImmediateBoundedNat` / `ImmediateStructuralNat` as
  children of a persistent aggregate.** They are constructible through
  `store_field`, and `Value` has no arm meaning "exit status" or "bounded Nat".
  The landed reader refused them too; picking a `Value` arm here would invent
  semantics rather than encode them, so `child_image` refuses with an exact
  status. ⚠ **Raised as a question, not folded into an AC** — whether those tags
  are admitted in persistent child position is a representation decision.
- **The `canonical.rs` closure-arity cap.** `encode_canonical` writes
  `captured.len().min(65535)`, so two closures differing only above 65535
  captures would encode alike. Pre-existing, not reachable within any region
  reservation used here, and changing the canonical encoding is a contract
  decision. Recorded, not touched.

## AC → discharging control — rows added by this fold

| AC | discharging control | evidence |
|---|---|---|
| **AC-6** `Closure` adoption | `b2v_ac6_an_emitted_closure_adopts_with_artifact_scoped_identity` | emitted closure with ordered captures → seal → adopt → **producer arena dropped** → a separately compiled consumer reads tag `PersistentClosure` on the word, class `Closure` on the node, and a **non-null** slot; the store's own `slot → bytes → Value` decode yields `Value::Closure` with `boundary_code_id(identity, origin)` and both captures **in order** |
| **AC-6** closure identity | `b2v_ac6_closure_identity_converges_and_never_aliases` | equal code identity + equal ordered captures converge on one slot; a changed code identity, a changed capture **value**, and a changed capture **order** each fail to alias. ⭐ The order pair differs in *nothing but order* (`[11,22]` vs `[22,11]`), so a set-valued canonical form reddens it. Causal: **M51** |
| **AC-6** artifact namespace | `b2v_ac6_equal_ordinals_in_two_artifacts_do_not_collide` | the same ordinal in two artifacts is two identities; the ordinal still discriminates **within** one artifact; same artifact + same ordinal is one identity. Causal: **M47**. ⚠ One assertion in it was relabelled after **M48** — see above |
| **AC-6** unbound fail-closed | `b2v_ac6_closure_adoption_fails_closed_while_artifact_unbound` | exact `BOUNDARY_ERR_UNBOUND`; positive control is the **identical graph** adopting once bound. Causal: **M50** |
| **AC-6** no retagging | `b2v_ac6_a_deduped_closure_capture_keeps_its_tag_through_the_rewrite`; `b2v_ac6_a_nested_closure_capture_adopts_without_retagging` | the first is the one that **reaches the rewrite** (equal captures ⇒ dedup ⇒ repoint), with non-vacuity asserted both ways; the second pins that distinct nested closures are not merged. Causal: **M49** on the first only — reported, because on the second it was green for the wrong reason |
| **AC-6** sealed handoff | `b2v_ac6_every_emitted_writer_is_refused_after_the_sealed_handoff`; `b2v_ac6_adoption_refuses_an_unsealed_region` | the probe table is asserted **equal to `EMITTED_WRITERS`**, so a new writer without a probe is red; each writer returns a non-`SEALED` status before the seal and `BOUNDARY_ERR_SEALED` after, and the whole region is byte-identical across the refused writes. Adoption refuses an unsealed region, with the same graph adopting once sealed. Causal: **M43**/**M44**/**M52**, attribution captured |
| **AC-6** validation precedes minting | `b2v_ac6_a_refused_graph_installs_no_identity_at_all` | a refused cyclic graph leaves **every** node at `NULL_SLOT`; the acyclic twin mints every node, so the emptiness is a refusal and not an adoption that does nothing. Causal: **M45** |
| **AC-6** closure decode | `b2v_ac6_closure_canonical_decode_round_trips_and_refuses_malformed` | encode→decode is the identity on a **nested** closure; **every** truncation of those bytes is refused; `Array` is encodable and still refused, so the decoder was widened for `Closure` and nothing else |
| **AC-6** invocation-owned classes | `b2v_ac6_invocation_owned_classes_are_never_persisted`; `b2v_ac6_an_invocation_owned_capture_rejects_before_publication` | `HostResult`/`BorrowedOpaque` have no persistent adoption boundary; an invocation-owned capture is refused at construction, with a persistent capture into the **same slot** as the positive control. ⚠ Neither arm is narrowed or reclassified — what is refused is *persistence* |
| **AC-10** cycles vs sharing | `b2v_ac10_a_multi_node_cycle_is_refused_while_a_shared_dag_adopts`; `b2v_ac10_a_constructible_node_cycle_is_refused_not_recursed` | a three-node cycle is refused **deterministically** — twice, from every entry point on the ring — while a shared-child DAG of the same shape adopts and the shared child resolves to **one** canonical node. Causal: **M45** and **M46**, which are the two directions of the same discriminator |
| **AC-10** depth | `b2v_ac10_a_deep_acyclic_chain_adopts_without_walk_recursion` (depth 3000, every run) + `#[ignore]`d `..._at_thirty_thousand` | adopts, is store-minted, and is asserted **never** to return `BOUNDARY_ERR_CYCLE`. ✅ **Closed** — the walk is iterative with a heap frontier and the `Value` side is closed by `RT-VALUE-TOTALITY-P1` at depth 131_072. The former "~2500 encoder ceiling" residual was **false**; corrected above |
| **AC-6** persistent content-addressing *(was `NO CONTROL — open residual`)* | the `AC-10`/`AC-6` adoption rows above, plus the `Closure` rows | ⭐ **Now closed for every persistent class the disposition admits.** The row above at *"an emitted-constructed node carries `NULL_SLOT`"* recorded the state before adoption existed; adoption mints for ground values and, with this fold, for closures. It is retained above as the record of what was true then |

---

# `RECUT 2` fold — the phase-closure artifact

`RECUT 2` retires the per-cell `AC`→control map as a *sufficient* proof shape:
the map stays required, and stops being the proof. What replaces it is one
mechanically closed artifact over the finite structural partition, spanning

```text
authority -> producer -> validator -> canonicalizer/adopter -> publisher -> consumer
```

## What is compiler-closed, and what is not

⛔ **Stated as a split rather than as one claim, because the two halves have
very different strength.**

| half | mechanism | strength |
|---|---|---|
| **completeness** — every required phase is bound | `PhaseClosure` has six mandatory fields (no `Option`, no `Default`); `LifecyclePhase::index` is a wildcard-free match; `BoundaryOutcome::phase_closure` is wildcard-free | **compile error** — a row with a hole, a seventh phase, or a new outcome does not build |
| **derivation** — the required set is not a per-row choice | `BoundaryOutcome::requires` derives it from the outcome's *class*; `StructurallyAbsent` is never selected by a row | **compile + control** — the sweep reddens on either mismatch direction |
| **identity** — the bound anchor *is* the production item | `derived_witness` for 5 anchors; `CONTROL_CLOSED` names a causal control for the 3 that need a JIT | ⛔ **control-closed only. See `M-E`.** |

## The mutations

Each applied at its natural production site, run, then restored
byte-identically and verified with `git diff --quiet` (⚠ `--stat` always exits
`0` and is not an emptiness test).

| id | mutation | pin it must redden | result |
|---|---|---|---|
| **M-A** | the `StoreMinted` row drops `canonicalizer_adopter` to `StructurallyAbsent` | `recut2_every_admitted_row_closes_every_required_phase` | ✅ **RED** — this is blocks `#5`/`#6` reproduced as a one-line edit |
| **M-B** | `requires` says an `ImmediateWord` needs the adopter | same | ✅ **RED** (the required-but-absent direction, from the other side) |
| **M-C** | `LifecyclePhase::ALL` repeats a phase instead of listing all six | `recut2_the_phase_inventory_is_bound_to_the_type` | ✅ **RED** |
| **M-D** | a JIT-only anchor loses its row in `CONTROL_CLOSED` | `recut2_every_anchor_is_closed_by_a_witness_or_a_named_control` | ✅ **RED** |
| **M-E** | `derived_witness` deletes the production call and returns the literal `Some(1)` | `recut2_derived_witnesses_come_from_the_production_authority` | ⛔ **GREEN — THE EVASION WINS** |
| **M-F** | an invocation handle claims to require adoption | `recut2_only_the_store_minted_handle_requires_the_whole_lifecycle` | ✅ **RED** |
| **M-G** | the normalization authority drifts (accepts a leading zero) **while** the witness stays hardcoded | `recut2_derived_witnesses_come_from_the_production_authority` | ✅ **RED** — the drift a frozen literal cannot see |

## ⛔ `M-E` won, and the repair does not fully close it

**Reported rather than quietly patched.** Replacing the witness's production
call with the constant it currently returns leaves the pin green, because a
hardcoded value and a live call are indistinguishable **while they agree**.

The pin originally compared the witness against a frozen `Some(1)` — two
hand-maintained constants agreeing, which is the `AC-1` defect in miniature. The
repair computes the expected side **from the authority**:

```rust
let normalization_rejects_leading_zero =
    !boundary_int_magnitude_is_canonical(0, &[1, 0]);
assert_eq!(anchor.derived_witness(), Some(i64::from(normalization_rejects_leading_zero)));
```

⚠ **Measured, both before and after the repair:** `M-E` is **still green**. The
repair does **not** make a hardcoded witness detectable today, and claiming
otherwise would be the overclaim this table exists to prevent. What it does buy
is `M-G`: once the authority's behaviour moves, the hardcoded witness diverges
from it and the pin reddens. So the honest statement is **drift-closed, not
identity-closed**, and identity remains control-closed via `CONTROL_CLOSED`.

## ⛔ What this artifact does NOT close — the finding

**The classification layer has no production consumer, and the compiler says
so.** On the lib (non-`cfg(test)`) build, `rustc` reports every one of
`BoundaryDisposition`, `StaticEncodingPolicy`, `BoundaryInput`,
`BoundaryOutcome`, `MagnitudePartition`, `ReachabilityPartition`,
`AdoptionPartition`, `HandleIdentity`, `LoweredVariant`, `permitted_by` and
`policy` as **never used**. The emitted producer, by contrast, **is** a
production consumer — `lowering/core.rs:81`, non-test.

⇒ The static policy and the value partition are Rust oracles governing an
emitted path that never consults them. `RECUT 2` names exactly this: *"a
declaration, classifier row, Rust oracle, or residual label with no production
consumer does not discharge the predicate."*

⛔ **This artifact sits in that same layer and does not by itself discharge the
predicate.** It closes completeness; it adds no production consumer. Whether
wiring the classifier into the already-production emitter is in scope for `B2V`
or barred by `D6` inertness is an open scope question routed to the Architect,
not a judgment taken here.

## `D6` — predicted before measuring, then measured

⚠ Prediction stated in `evt_387scrzz83p0b` **before** running any census. The
artifact was deliberately placed **inside `lowering/mod.rs`** rather than in a
new module, to avoid the `B2R` registration-driven fan-out where registering one
file changed the input to every pin iterating `BACKEND_PRODUCTION_SOURCES`.

| quantity | predicted | measured |
|---|---|---|
| `BACKEND_PRODUCTION_SOURCES` entries | unchanged | **13** — unchanged |
| declared-module list | unmoved (no new `mod`) | unchanged; `correspondence_adds_no_emitted_unit_to_the_production_census` green |
| `lowering/mod.rs` census row | `0` definitions / `0` declarations | unchanged — a `match` and some types declare and define nothing |
| `LOCAL_HELPER_COUNT` | `6` | **6** |
| `BOUNDARY_LOCAL_HELPERS` | unchanged | **28**, file untouched by the commit |

✅ **No row moved, so there is nothing to re-baseline**, and the prediction was
not adjusted to fit the measurement.

## Suite

`scripts/ken-cargo test -p ken-runtime` (targeted, from the seat's worktree):
**431 lib + 12 integration passed, 0 failed.** The five new controls are the
delta from 426.

⚠ **One correction to an earlier report.** The first gate run was piped through
`tail -60`, so the grep that produced *"compiler warnings/errors: 0"* could only
see the last sixty lines and no compiler output at all. Re-measured on the full
buffer: **28 lib + 7 lib-test warnings**, all pre-existing dead-code and
unused-mut notices. The test counts were unaffected. ⭐ Those unread warnings
are what surfaced the finding above.

---

# The authority-to-emitter edge — Architect ruling `evt_7nkbf495pg54h`

The phase-closure artifact above closed **completeness** and left the finding
that the classification layer had **no production consumer**. The Architect
ruled the wiring **in `B2V` scope and required**: `RECUT 2` governs whether the
fixed helper artifact is *generated from* the representation authority, while
`D6` governs whether it is *inert at the semantic call graph*. Production
codegen consumption is not `B2F` activation.

## What was actually hand-maintained

Seven `class_guard` call sites in `boundary_value_clif.rs` each carried a
literal class list beside the helper body:

```rust
class_guard(&mut b, node, &[BoundaryClass::Int]);                        // x5
class_guard(&mut b, node, &[BoundaryClass::Bytes, BoundaryClass::String]); // x2
```

⛔ That is *"another hand-maintained table beside the helper bodies"*, which the
ruling names as not counting. All seven now read from the plan.

## The seam

```text
lowering/core.rs   BoundaryEmissionPlan::derive()      <- the authority
        |                                                (BoundaryInput -> outcome
        |                                                 -> class, x storage_shape)
        v
emit_boundary_value_local_graph(module, native_int, &plan)
        v
define_store_int_limbs / _limb / _tag / seal_int / bytes_len / byte_access / int_part
        v
class_guard(.., plan.int_magnitude_classes() | plan.byte_span_classes())
```

`BoundaryEmissionPlan` is data-only and `pub(crate)`; its **derivation** lives in
`cranelift_backend::lowering` because `BoundaryInput` is private to that module.
⭐ **That privacy is the mechanism, not an accident:** the emitter *cannot*
restate the authority, because it cannot see it.

⚠ **Measured before rewiring:** the derived sets are exactly `[Int]` and
`[Bytes, String]` — precisely what the literals said. So the change is
behaviour-preserving today and its whole value is that the edge is now causal.

## Causal evidence — the ruling's bar, not a "plan is passed" check

| id | mutation / probe | result |
|---|---|---|
| **M-H** | the emitter ignores the plan: five guards revert to `&[BoundaryClass::Int]` | ✅ **RED** — `recut2_the_emitted_helper_graph_changes_when_the_authority_changes` |
| **perturbed plan** | same emitter, plan's int-magnitude set changed to `[Record]` | ✅ emitted CLIF **differs**, and differs *in the class constant the plan supplies* |
| **same plan twice** | two captures under the identical plan | ✅ **equal** — so the inequality above is attributable, not noise |
| **derivation** | recompute both sets in the test from the classifier and `storage_shape` | ✅ `recut2_the_plan_is_derived_from_the_partition_not_restated` |

⛔ **A `capture_..._with_plan` injection point was added specifically so this
could be causal.** Without it the only available evidence would have been "the
plan is passed", which is the `let _ = plan` the ruling excludes.

⚠ **Restore discipline, and a mistake worth recording.** The first `M-H` run was
performed while the wiring was still **uncommitted**, and `git checkout -- <file>`
restored the file to `HEAD`, discarding the entire wiring along with the
mutation. Nothing was lost permanently — it was re-applied from the same script
— but the rule is the one this repo already carries: **commit the real change
before any mutation-proof reset.** `M-H` was then re-proved against the
committed tree, and the tree verified clean with `git diff --quiet`.

## `D6` — re-measured after the wiring, still unchanged

| quantity | predicted | measured after wiring |
|---|---|---|
| `BACKEND_PRODUCTION_SOURCES` | unchanged | **13** |
| `LOCAL_HELPER_COUNT` | `6` | **6** |
| `BOUNDARY_LOCAL_HELPERS` | unchanged | **28** |
| new module / new helper | none | none |

✅ The plan changes helper **body contents** only — the fixed Θ(1) helper set,
the semantic generated-function population, the calls and the ownership topology
are untouched, which is the inertness `D6` protects and `B2F` depends on.

**Suite:** `scripts/ken-cargo test -p ken-runtime` — **433 lib + 12 integration,
0 failed.**

## What remains open

⛔ **This closes the edge for the class-legality axis of the helper bodies.** It
does **not** claim every representation authority is consumed: `tag`, `owner`
and `identity` legality in the emitted bodies were not part of this fold, and
the `RECUT 2` predicate names them alongside class. They are the next increment,
not a residual being waived.

## Next increment — the tag / owner / identity axis, located

⭐ **Recorded so the next fold does not repeat the discovery.** The class axis
was closed by finding the *hand-maintained thing* (seven literal class lists)
and replacing it with a plan lookup. The same search has been run for the
remaining axes; these are the targets.

**Tag legality — four hand-picked range endpoints in `boundary_value_clif.rs`:**

| const | line | what it hand-encodes |
|---|---|---|
| `FIRST_HANDLE_TAG` | `:452` | where the handle tags begin |
| `LAST_PERSISTENT_TAG` | `:459` | where the persistent tags end |
| `LAST_TAG` | `:461` | the closed tag set's upper bound |
| `FIRST_INVOCATION_TAG` | `:1270` | where the invocation tags begin |

⚠ **These are the `AC-1` defect's shape:** each is a second authority derived by
hand from `BoundaryTag`'s declaration order, and a range check against them is
*two hand-maintained constants agreeing*. They should come from the plan as an
**admitted tag set** derived from the partition's `HandleWord` outcomes — the
same sweep the class sets already use, which already carries `tag` alongside
`class`. ⛔ An ordering-dependent range is also fragile in a way a set is not:
reordering the enum silently changes what is admitted.

**Owner legality — `boundary_value_clif.rs` `:1495`/`:1498` (the owner constants
the helpers emit), `:1699`, and `:2330`–`:2338` (the per-owner marker masks via
`boundary_int_marker_mask`).**

**Identity** is already partly structural — `HandleIdentity` is computed by
`BoundaryInput::handle_identity` from the owner — so the likely finding is that
identity needs no separate wiring once owner is derived. ⚠ **That is a
prediction, not a result:** it must be measured the same way the class sets
were, by checking whether the emitted bodies contain an identity decision the
authority does not supply.

**The method that worked, to reuse:** derive the plan set, compare it against
the literals *before* rewiring (they matched exactly for class, which is what
established the change was behaviour-preserving), rewire, then prove causality
with a perturbed plan plus a same-plan control, and prove an emitter that
ignores the plan reddens.

---

# The tag / owner / identity fold — `184ec6f9`, `4b064718`, `6fa674f2`

⭐ **Read the mutation table first.** Three of the five mutations run against
the first commit's evidence went **green**, and all three were the exact shape
`RULING R3` says must redden. The fold is not the interesting part; the two
rounds of repair it took to make the evidence real are.

## What the authority now supplies, and what was deleted

`BoundaryEmissionPlan` gained a `BoundaryTagAdmission` derived from the same
`BoundaryInput::all() → outcome()` sweep the class sets already used:

| derived | from | replaces |
|---|---|---|
| `admitted()` | every `ImmediateWord`/`HandleWord` tag | `tag <= LAST_TAG` ×4 |
| `immediate()` | `ImmediateWord` tags | `tag < FIRST_HANDLE_TAG` ×2 |
| `handle()` | `HandleWord` tags | `tag >= FIRST_HANDLE_TAG` ×2 |
| `owner_bands()` | `HandleWord` grouped by `owner` | `tag <= LAST_PERSISTENT_TAG` ×3, `tag >= FIRST_INVOCATION_TAG` ×2 |

All four constants are **deleted**, not left beside the new mechanism. The
emitted code tests set membership (`tag_in_set`, a disjunction) instead of an
ordinal band, so the property the old pin had to defend — *the closed tag set
stays grouped by referent owner* — is no longer a property anything depends on.

⛔ **An ordinal range is strictly weaker than a derived set**, and the weakness
is silent: reorder `BoundaryTag` and every constant stays well-formed while the
admitted region changes underneath them.

## The located inventory was incomplete — a fifth site, and a sixth still open

The `ab11a3d2` inventory named four constants. It missed two things.

**`:1194`, found and closed in this fold.** `define_escape_check` compared
`tag >= BoundaryTag::InvocationBorrowed as i64` **inline**. Identical defect,
identical repair — and it was missed for a structural reason worth naming:
⚠ **the inventory was built by grepping for the constants, so a site that
spells the same band without one is invisible to it.** The sweep that found it
was `grep BoundaryTag::` over the whole emitter, which enumerates *uses of the
authority* rather than *names the defect gave itself*.

**`:715`, found and NOT closed — routed, not silently absorbed.**
`define_class` decides an immediate's class with
`is_bool ? BoundaryClass::Bool : BoundaryClass::Int`. That is a hand-maintained
immediate-tag → class map beside the helper bodies, which is `R3`'s third
excluded discharge. ⛔ **It cannot be derived today**, because
`BoundaryOutcome::ImmediateWord { tag }` carries **no class** — and
`BOUNDARY_TAG_CLASS_RELATION` says so on purpose: *"Immediate tags are absent by
construction — they have no node, so they have no class."* Closing it means
extending the authority's shape, which is the Architect's call and not a
spelling choice. It is stated here as open.

## Behaviour preservation, measured before rewiring

The method from the class axis, reapplied: derive the sets, compare them to the
literals **before** touching the emitter.

```
immediate  = {0,1,2,3,4}  == tag <  FIRST_HANDLE_TAG
handle     = {5,6,7,8}    == tag >= FIRST_HANDLE_TAG
persistent = {5,6}        == FIRST_HANDLE_TAG ..= LAST_PERSISTENT_TAG
invocation = {7,8}        == FIRST_INVOCATION_TAG ..= LAST_TAG
admitted   = {0..8}       == tag <= LAST_TAG
```

Exact match on every set, so the rewiring changes no admitted word — and the
256-byte emitted-interface sweep `b2v_emitted_code_admits_exactly_the_closed_tag_set`
still passes, which is the behavioural confirmation rather than the argument.

⭐ **One live divergence was closed as a side effect.** The old marker-mask
`select` asked *"is the owner the store"* and gave every other answer the
invocation arena's mask — so a node recording `NoReferent` was handed the
invocation markers, while the Rust twin `boundary_int_marker_admits` admits only
the owner-agnostic ones for it. Two implementations of one relation disagreeing
on an arm neither reaches today. The fold over `BoundaryReferentOwner::ALL`
makes them agree.

## ⛔ The mutation table, and the two rounds of repair

Round 1, against `184ec6f9`'s evidence:

| # | mutation | expected | **measured** |
|---|---|---|---|
| `M-T1` | `define_resolve`'s validity test hardcoded `tag <= 8` | red | ⛔ **GREEN** |
| `M-T2` | region selection's band test hardcoded `tag <= 6` | red | ⛔ **GREEN** |
| `M-T3` | the node's recorded owner hardcoded | red | red (8 tests) |
| `M-T4` | `derive()` restating the immediate set | red | red (4 tests) |
| `M-T5` | `escape_check`'s invocation band hardcoded `tag >= 7` | red | ⛔ **GREEN** |

⭐ **Three defeats sharing a structure is a granularity fault, not three
missing cases.** `recut2_the_emitted_helper_graph_changes_when_the_tag_sets_change`
compares the **entire** captured CLIF. Four sites consume the admitted set;
disconnect one and the other three still move the aggregate, so the difference
the test demands is still there and it stays green. **The pin's granularity was
the graph. The property is per-site.**

The repair is behavioural and **total over the tag dimension** rather than a
hand-listed set of sites. `compile_probe_with_plan` compiles the helpers against
a caller-supplied plan, so a helper can be asked what it **answers**:

- `b2v_every_emitted_tag_admission_test_is_the_plans` — sweep every admitted
  tag, remove it from the plan, require each probed helper to go from its real
  status to `ERR_TAG`. A hardcoded constant cannot follow.
- `b2v_every_emitted_owner_band_test_is_the_plans` — sweep every handle tag,
  move it to the other band, require each probed helper's answer to change.

Both are two-sided: the real plan's answer must **not already** be `ERR_TAG`, or
the perturbation could not change it and the pin would pass vacuously.

Round 2, against `4b064718`:

| # | expected | **measured** | caught by |
|---|---|---|---|
| `M-T1` | red | red | `b2v_every_emitted_tag_admission_test_is_the_plans` |
| `M-T2` | red | red | `b2v_every_emitted_owner_band_test_is_the_plans` |
| `M-T5` | red | ⛔ **STILL GREEN** | — |

⚠ **`M-T5` survived the repair too, and the reason is one level down from the
first.** The band pin drove only the `class` probe, and `class` never reaches
`escape_check`'s gate — so the site was uncovered by the very test named for
covering every band decision. Coarse granularity blinded round 1; an incomplete
**probe set** blinded round 2. ⛔ *A probe that never exercises the mechanism is
not evidence about it, whatever the test is called.*

Round 3, against `6fa674f2` (band sweep drives both probes):

| # | expected | **measured** | caught by |
|---|---|---|---|
| `M-T1` | red | red | tag-admission pin |
| `M-T2` | red | red | owner-band pin |
| `M-T3` | red | red | 8 behavioural tests |
| `M-T4` | red | red | derivation pin + 3 behavioural |
| `M-T5` | red | red | owner-band pin |

Every mutation restored with `cp`, never `git checkout -- <path>`, and each
restore verified with `git diff --quiet`.

⛔ **The residual is named in the test's own doc comment, not left implied by
its name.** `store_field`'s child-tag check and `make_immediate`'s immediate-set
check are not reached by these probes; for those two the evidence is the
whole-graph pin plus review. A test called *"every emitted tag test"* that
silently meant *"every one I could reach"* is the overclaim this WP keeps paying
for.

## Identity — the prediction, discharged by measurement

⚠ The `ab11a3d2` prediction, **labelled as one**: identity needs no separate
wiring, because `HandleIdentity` is computed by `BoundaryInput::handle_identity`
from the owner alone.

**MEASURED and TRUE.** The pin
`recut2_identity_is_a_function_of_owner_and_needs_no_second_wiring`
sweeps every admitted handle outcome and finds each owner yields exactly one
identity, with non-vacuity checked in both directions (≥2 owners publish, and
≥2 distinct identities occur — a constant identity would satisfy the first
assertion alone).

⛔ **THE GAP, stated rather than closed:** this shows emitted code cannot mint a
*store* identity — every `alloc`ed node is written `NULL_SLOT`, which this ABI
reads as *no store identity*. It does **not** show no future helper could. The
residual is review-enforced, with `escape_check`'s adoption gate as the runtime
mechanism, and it is tested separately.

## A pin was RETIRED rather than left passing

`b2v_the_region_thresholds_agree_with_referent_owner` is renamed to
`b2v_the_plan_owner_bands_agree_with_referent_owner` and re-asserted against the
bands, with two clauses **deleted**: the threshold-based classification, and the
assertion that the two owner bands are numerically **contiguous**. Both existed
only because a threshold cannot separate bands that interleave.

⚠ **Keeping a still-green assertion about a property no mechanism rests on is
not free** — it reads to the next author as a constraint they must preserve, and
it would have made a legitimate non-contiguous tag set look like a regression.

The replacement adds the clause the per-band sweep could not see on its own: the
bands' **union** must be the admitted handle set, because a handle tag in *no*
band would resolve nowhere and every per-band assertion would still pass.

## `D6` — predicted before measuring, not adjusted

| row | predicted | measured |
|---|---|---|
| `BOUNDARY_LOCAL_HELPERS` | 28 | **28** |
| `declare(...)` in the emitter | 28 | **28** |
| `BACKEND_PRODUCTION_SOURCES` | 13 | **13** |
| `LOCAL_HELPER_COUNT` | 6 | **6** |
| `lowering/mod.rs` census | 0/0/0 | **0/0/0** |
| non-test emitter call sites | 1 (`lowering/core.rs:87`) | **1** |

Helper **bodies** changed; the helper **population** did not. No new module, no
new generated semantic function, no semantic-body call to a boundary helper, no
second emitter.

Dead-code oracle: **18 lib warnings before and after**, same set — this fold
introduced no new consumer-less item, and none of the items it consumed were on
that list to begin with (they were consumed by the class fold at `720f301c`).

---

# The class axis, re-opened by measurement — `7d1b307b`, `1c0c1fea`

⭐ **This section exists because I ran the tag axis's winning mutation against
the class axis the Architect had already confirmed.** It won there too.

## The measurement that re-opened a confirmed axis

`class_guard(&mut b, node, plan.int_magnitude_classes())` appears at **five**
sites. One was disconnected — the literal `&[BoundaryClass::Int]` it used to be
— with the other four left consuming the plan:

```
test result: ok. 439 passed; 0 failed
```

⛔ **An emitter that ignores the plan at one site did not redden**, which is
`R3`'s bar verbatim. Identical cause to the tag axis: the whole-graph
differential cannot see one defector while four consumers still move the
aggregate.

⚠ **This does not say `720f301c` was wrong.** The wiring is real, the literals
are gone, the derivation is the partition's, the behaviour is preserved. It says
the **evidence** established *some* consumption, not per-site consumption — and
the Architect withdrew the "closed" statement on exactly that basis
(`evt_51xk9sxqdtzgt`, point 3), while confirming the wiring stands.

⭐ **The transferable part: a confirmed deliverable is not a reason to skip the
mutation.** The confirmation was accurate about the code it read. What it could
not tell me was whether the pin behind it had the strength both of us were
treating it as having — and the only thing that answers that is running the
mutation, which cost one command.

## Two mechanisms, because neither covers the surface alone

| | covers | limit |
|---|---|---|
| `b2v_every_emitted_class_guard_is_the_plans` | behavioural, `define_int_part`'s three readers | probe shapes: 5-param and 3-param helpers do not fit |
| `b2v_every_class_guard_call_site_takes_its_set_from_the_plan` | source scan, **all seven** call sites | a helper laundering a literal into plan-shaped text |

The scan pins the **allowed form** — every argument must come from `plan` — so a
guard spelled any other way reddens, including one nobody imagined. An
undetermined parse **fails**, and it has a positive control on the site count,
because a scan that matched nothing passes for any reason at all.

⚠ **The scan reads its own source and matched its own needle literal on the
first run.** It was caught by the undetermined-parse branch firing — which is
precisely the failure that branch exists for, working as intended on its author.

| # | mutation | expected | measured | caught by |
|---|---|---|---|---|
| `M-C1` | `define_int_part`'s guard → literal (probe-reachable) | red | red | **both** pins |
| `M-C2` | `define_store_int_tag`'s guard → literal (unreachable) | red | red | the scan |

## The immediate word's class — the site neither earlier fold could see

`define_class` answered `is_bool ? BoundaryClass::Bool : BoundaryClass::Int`.

⭐ **Why it escaped twice, which is the finding.** It was invisible to the tag
inventory because **it names no threshold constant to grep for**, and invisible
to the class fold because **that fold only ever looked at `NODE_CLASS`**. Two
searches, each complete against its own notion of the surface, and the site was
in neither. ⛔ *A sweep is bounded by the notion of "the surface" it was built
from, and that notion is rarely written down anywhere it can be checked.*

Ruled in scope by the Architect (`evt_51xk9sxqdtzgt`). The carrier is
`BoundaryTag::immediate_value_class` — total, wildcard-free, the same shape as
`referent_owner`.
`ImmediateWord` carries it, `derive()` sweeps it, the emitter folds over the
relation with a **fail-closed innermost value**: an immediate the authority
gives no class returns `ERR_CLASS` rather than defaulting to `Int`.

⛔ **Kept separate from `BOUNDARY_TAG_CLASS_RELATION`, and the distinction is
named at all four places a reader can land** — the authority method, the plan
accessor, the outcome variant, and the emitted site. That relation governs
`NODE_CLASS` legality and correctly excludes immediate tags because an immediate
has no node; this answers what the uniform `class` helper reports for an
immediate *word*. Merging them would invent a fictional immediate node class and
make the node-legality relation admit tags it must keep refusing.

Behaviour-preserving, measured before rewiring: the derived relation agrees with
the literal rule on all five immediate tags.

Evidence is per-entry and three-way — the real plan must report the real class
(the baseline, or the perturbations measure against the wrong thing), a **remap**
must change the answer, and a **drop** must fail closed rather than default:

| # | mutation | expected | measured | caught by |
|---|---|---|---|---|
| `M-I1` | the fold ignores the plan and re-derives `is_bool ? Bool : Int` | red | red | `b2v_the_emitted_immediate_class_is_the_plans` |

The derivation pin now sweeps this relation too, so the doc comment claiming it
does is **executable** rather than a `///` line nobody runs.

## `D6` — unchanged again, checked after each fold

`BOUNDARY_LOCAL_HELPERS` **28**, `declare(...)` **28**,
`BACKEND_PRODUCTION_SOURCES` **13**, dead-code oracle **18 lib warnings, same
set**. `define_class` gained a parameter and a fold; it did not gain a helper.
