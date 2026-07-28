# `RT-FNSPLIT-B2F` `S4`/`AC-11` — the timing instrument was broken, and the
# walk works

**Author:** `runtime-implementer` · **Branch:**
`wp/RT-FNSPLIT-B2F-functionization-live` · **Ruling:** Architect on exact
`779ddc8b`, frame blob `aa798ca9` — keep clause 3, take route **(a)**, repair
the measurement **before** touching origins.

---

## 1. ⛔ The finding I reported at `779ddc8b` was wrong, and here is why

I posted that the pre-emission producer walk **catches neither hole**, on the
strength of a measured `holeA = 1`, `holeB = 1`. ⛔ **That number was stale
recorder state.** The instrument that produced it could not distinguish the two
outcomes it existed to separate:

```rust
let sentinel = RuntimeExpr::Value(RuntimeValue::Bool(true));
ac11_compiles(&sentinel).expect("sentinel compiles");   // forces the cell to 1
assert!(ac11_compiles(expr).is_err());
b2f_last_unit_emission().0                              // reads back … 1
```

⭐ **No pre-emission refusal path writes `B2F_UNIT_EMISSION`.** So:

| the world | what the cell holds afterwards |
|---|---|
| refused **before** `declare_unit_bundle` (the wanted `0`) | the sentinel's `1` — nothing overwrote it |
| declared one unit, **then** refused late (the feared `1`) | `1` |

⇒ ⛔ **Identical readings.** The measurement was uninterpretable in *both*
directions, and I drew a confident negative conclusion from it.

⚠ **The in-source comment made it worse, not better.** It read:

> *"Poison the recorder first, so a compile that never reaches
> `declare_unit_bundle` cannot be confused with one that declared none."*

⛔ **That is exactly inverted.** Forcing the cell to a nonzero value is what
*creates* the confusion; the comment asserted the property the mechanism
destroyed. ⭐ **A stated rationale is not a check** — it was the most confident
sentence in the block and the only false one.

---

## 2. ⭐ Why the timing mattered enough to be worth repairing

The Steward's discriminator, which inverted the whole question:

> ⛔ **Does the late `Err(Unsupported)` refusal live in a code path `S7`
> DELETES?**

It does. The late refusal is `lower_expr`'s `RuntimeExpr::ImportedDeclarationRef`
arm — the recursive-descent inliner that **`D6`/`S7` removes** as the old
whole-configuration emission authority.

⇒ ⭐ **A refusal performed by the authority being retired is not a property of
the surviving boundary.** "It is rejected either way" is true today and becomes
false at `S7` — silently, with **no test reddening at the moment the hole
opens**, because a control that asks `is_err()` would still pass. ⛔ Re-scoping
clause 3 would have written a scheduled hole.

---

## 3. ✅ The repair — an attempt epoch, stamped at the seam

Three outcomes, all distinguishable, replacing one number that meant nothing:

| reading | meaning |
|---|---|
| `None` | ⚠ the compile never reached the emission seam. **Not** a zero |
| `Some(0)` | ✅ reached it, refused **before** any unit was declared — clause 3 |
| `Some(n > 0)` | ⛔ `n` units already declared when the refusal came |

⛔ **The stamp is written in `core.rs` immediately before
`validate_emitted_transfers_are_representable`, never inside
`declare_unit_bundle`.** Stamping inside the bundle would make `Some(0)`
unreachable by construction: the only way to observe the epoch would be to
declare a unit, which is precisely the event whose *absence* is being measured.

⚠ `b2f_open_compile_attempt` deliberately does **not** clear the counts. Clearing
there would hide a compile that never reached the seam behind a plausible
`(0, 0)` — the same confusion, one layer out.

---

## 4. ⭐ The differential the ruling required

```
tip   c6444fa5   ·   suite ken-runtime 497 + 26 + 14 passed, 0 failed
```

| fixture | walk **enabled** | walk **gated off** |
|---|---|---|
| Hole A — `If { true, imported, imported }` capture | ✅ `Some(0)` | ⛔ `Some(2)` — RED |
| Hole B — `LexicalClosure { captures: [], body: imported }` | ✅ `Some(0)` | ⛔ `Some(2)` — RED |
| wrapped intra-module | ✅ accepted | accepted |
| bare-body intra-module | ✅ accepted | accepted |

⇒ ⭐ **The pre-emission producer walk refuses both named holes, before a unit is
declared.** `AC-11` clause 3 is discharged **for these two shapes**. ⛔ No origin
resolution was touched — per the ruling, step 4 was never reached.

The gate-off mutation discards the verdict rather than deleting the call
(`let _ = …`), which isolates the **authority** from the computation: the walk
still runs, and only its power to refuse is removed.

### ⚠ Hole B needed its own measurement

⛔ **The first gated run reddened on Hole A and panicked, leaving Hole B
unmeasured.** A mutation that reddens the first row of a test says nothing about
the second — the assertion order short-circuits exactly the evidence you are
collecting. ⇒ Hole B was measured under the same production mutation with Hole
A's row neutralized, and reported `Some(2)` independently. Both mutations
restored, `git diff --quiet` exit 0 from a clean `HEAD`.

### ⭐ The instrument has its own positive control

⛔ **`Some(0)` is also what a stamp that fires beside a counter that never
increments would report.** So a successful compile runs in its own stamped
attempt and is required to report `Some(n > 0)`. Without that row the two
rejection rows are satisfiable by a dead counter.

---

## 5. ⭐ The sentinel is retired, and its replacement is a different promise class

The `1`-valued assertion was labelled a **transition sentinel**. ⛔ It was not
one: it pinned **recorder state**, not refusal timing, so the event it claimed
to announce could not have reddened it.

Its replacement is a **durable invariant**: *the refusal comes from the
pre-emission side of the boundary.* ⭐ **`D6`/`S7`'s removal of `lower_expr`'s
late arm must leave it green** — that is the whole reason to assert it now
rather than after the deletion. The test is renamed
`an_unrepresentable_transfer_is_refused_before_any_unit_is_declared`.

---

## 6. ⛔ NOT CLAIMED — `AC-11` remains OPEN

Stated as a partition with its discriminator, not as examples.

1. ⛔ **Clause 1 is discharged for `Capture` and `Result` only.** The
   discriminator is *does the transfer kind have a nonempty emitted population
   today?* — `Parameter` transfers are **empty** until `S5` supplies call sites.
   ⚠ A vacuous population is not a passing one; it must join the same proof when
   `S5` fills it.
2. ⛔ **A `Match`/`ComputationalMatch` arm is not traced.** `producers_of`
   passes through `If` branches and `Let` bodies only; case bodies derive
   through `case_body_occurrence`, not `child_occurrence`, and their positional
   layout in `plane.child_origins` is unestablished. ⇒ An import reaching a slot
   through a match arm is covered by **neither** the walk nor any control here.
3. ⛔ **The two holes are the *named* ones, not the whole shape class.** What is
   shown is that the walk sees through `If` pass-through and reaches a bare
   closure body — not that no wrapper exists that defeats it. `Match` above is
   one known such wrapper.
4. ⛔ **`AC-3`'s four width invariants are still undischarged**; only the seed
   material's alignment is checked.

⇒ ⛔ **Do not total these populations as discharged.** `AC-11` closes when the
`Match` residual is handled and `S5`'s `Parameter` population joins the proof.
