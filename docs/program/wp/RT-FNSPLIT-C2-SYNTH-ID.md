# `RT-FNSPLIT-C2-SYNTH-ID` — closed identity for synthesized constructor roles

**`constructor_symbol_identity(origin)` is the one typed constructor-identity
authority, and it is occurrence-keyed. A compiler-synthesized effect payload has
no source occurrence. So the last two producer arms `B2F` cannot build are not
lookup bugs — there is nothing lawful to ask.** This node supplies the missing
identity source as a **closed, typed, unforgeable role capability** owned by the
same semantic plane, together with the `DynamicConstructor` producer that
consumes it.

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/RT-FNSPLIT-C2-SYNTH-ID`. **Size:** M.
**Risk:** medium — it adds a planner capability, which is the category this chain
has spent nine hard-stops keeping narrow.

**Status:** Steward frame, shovel-ready, released.
⭐ **On the Linux ABI I critical path.** It blocks `B2F`'s `D9`, and `B2F` gates
`RT-SCALE-B` plus the operator's `n=3..7` scaling gate.

---

## 0. ⛔ READ FIRST — WHY THIS IS A NODE AND NOT A `D9` REPAIR

**Architect ruling `evt_xf4znbnb6vz9`, verbatim on the point that decides it:**

> *"This is a frame/prerequisite defect, not a local `D9` lookup repair."*

⇒ The capability is a **planner addition**, and `B2F`'s standing no-widening
boundary forbids the implementer from creating one locally. ⛔ **So this is not
`D9` work postponed; it is work `D9` is not permitted to contain.** ⭐ Same shape
as the two prerequisites already cut ahead of `B2F` on its own hard-stops —
`B2V` (`evt_28cnmxf6ncghn`, #10) and `C1` (`evt_7ay6s5s79awz8`, #11).

⚠ **And `B2F`'s "the preparatory escape is SPENT" clause is not violated by
this.** That clause governs inert scaffold *inside* `B2F`'s atomic boundary and
says a deliverable wanting its own preparatory merge is **a hard-stop to raise**.
⭐ **It was raised, and this node is the ruling's answer** — which is the clause
working, not being bypassed.

---

## 1. Fixed inputs

Measured at `origin/main = 06097ebd`. ⛔ Re-derive on your own base; yours wins.

| path | blob |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition/semantic_ir.rs` | `a80da199ae3183a9815c6fd7557806bd2fd5a160` |
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `6e36074dadb7b68a544f249904221714a19c8b8f` |
| `crates/ken-runtime/src/native_process_entrypoint.rs` | `8891d2c35aa7219f35cab5271cb305de018d1848` |
| `crates/ken-runtime/src/cranelift_backend/lowering/mod.rs` | `4387aab50dacb6c9e517134e82eb32030d933606` |
| `crates/ken-runtime/src/boundary_value_clif.rs` | `47610c6de93ea7296eacb72abb761fd726deb630` |

---

## 2. The measurement — ⭐ production already states the defect

The two refusals in `lowering/mod.rs` say why they refuse, and the second one is
this node's whole justification:

- `:1071` — *"the carrier producer does not yet emit a `HostResult` handle: its
  two payload words are selected by a runtime discriminant"*
- `:1076` — *"the carrier producer does not yet emit a dynamic constructor: its
  alternatives are selected at runtime, so **one occurrence origin does not name
  one constructor identity**"*

⭐ **That last clause is the prerequisite, written in production source.** ⛔ Do
not treat it as a TODO to route around; it is a correct statement of an absent
authority.

The surrounding facts, measured:

| fact | where |
|---|---|
| `constructor_symbol_identity(origin)` — the one typed authority | `semantic_ir.rs:1159`, re-exported `static_transition.rs:1181` |
| `ConstructorIdentity(pub(super) DenseRange)` — **unmintable outside the plane**, by field visibility | `semantic_ir.rs:45` |
| `tag_abi_word` — the identity's ABI projection | `semantic_ir.rs:106` |
| `fn intern(&mut self, bytes) -> DenseRange` — the spelling authority, **private** | `semantic_ir.rs:490` |
| `NativeProcessSymbols` — the role carrier | `native_process_entrypoint.rs:25` |
| `host_success` / `host_payload` — the emitted-helper route | `boundary_value_clif.rs:156`, `:158` |

⭐⭐ **`intern` being private is why no arbitrary string-to-identity lookup exists
today, and `ConstructorIdentity`'s `pub(super)` field is why an identity cannot
be minted outside the plane.** ⇒ **Both properties are load-bearing and this node
must preserve them.** ⛔ The capability goes *through* that authority, never
beside it.

---

## 3. ⭐ Steward-discharged — the Architect ruled the shape, not just the gap

`evt_xf4znbnb6vz9`. ⛔ **Settled inputs; do not re-litigate.**

### 3a. `HostResult`'s wrapper representation does NOT change

⛔ **Do not move `HostResult` to `FailClosedForbidden`.** ⛔ **Do not add
`ok_constructor` / `err_constructor` fields to the node.** The existing ABI
already answers the wrapper question: an `InvocationHostResult` word is the
nominal `Result` wrapper and its **runtime success bit** selects field 0 or 1.

⇒ A carried consumer stays on the **`HostResult`-specific emitted-helper route**:
compare source cases against the artifact's existing
`NativeProcessSymbols.result_ok` / `result_err` roles **at compile time** → call
`host_success` **at runtime** to select the case → call `host_payload` to project
the selected **payload word** into that case's environment.

⛔ **It must not** re-wrap the word as `Lowered::Constructor`, recover a
compile-time `HostResult` template, or expect `Result`-constructor identities in
the two child words.

### 3b. The producer obstruction is separate, and it is the real work

**Each of the two fields is a payload value that independently follows its own
`boundary_disposition`.** ⇒ A synthesized `Lowered::Constructor` or
`DynamicConstructor` *inside* either payload still needs **its own constructor
identity in its own carrier word**, and ⛔ **the `HostResult` wrapper can neither
supply nor erase that identity.**

### 3c. ⛔ Four rejected identity sources — all are a second derivation

⛔ the parent effect origin (it may hold a **different atom** and is not a proxy)
· ⛔ `RuntimeSymbol` spelling · ⛔ `intern_symbol` · ⛔ a hash, a dense ordinal, or
a scan for some unrelated equal-spelling occurrence.

⭐ **Why this list matters more than the mechanism:** each one *works* on a
one-fixture demo. The reason to refuse them is that they install a **second
identity derivation** beside the plane's, which is the defect this program has
spent nodes removing.

### 3d. The required shape — a closed, typed, unforgeable role capability

1. **Plan construction inventories the exact closed set** of
   `NativeProcessSymbols` roles effect lowering may synthesize, **including every
   dynamic alternative.**
2. Those spellings are interned **by the existing semantic material authority**.
3. Lowering can ask **only by an unforgeable closed role**, receiving the
   existing **unmintable** `ConstructorIdentity` / `tag_abi_word`.
4. ⛔ **No arbitrary string-to-identity lookup, no second interner, and no
   exposure of the plane or its name arena.**
5. **Duplicate role spellings converge** through the same authority, while
   **distinct roles cannot alias.**
6. **A synthesized role absent from the closed inventory REJECTS before unit
   emission.**

> ### ⛔⛔ `io_errors: Vec<RuntimeSymbol>` — THE INVENTORY IS NOT A FIELD COUNT
>
> `NativeProcessSymbols` is mostly fixed fields, **but `io_errors` is a `Vec`**
> (`native_process_entrypoint.rs`). ⇒ ⛔ **"Inventory the closed set" is not
> satisfied by walking the struct's named fields** — a `Vec` role carries an
> arbitrary number of alternatives at plan-construction time. ⭐ **This is exactly
> the "every dynamic alternative" the ruling requires**, and it is the part a
> field-by-field reading silently drops. ⛔ Derive the inventory; ⛔ do not
> transcribe a count into a test.

---

## 4. ⛔ Banned shapes

- ⛔ **Do not widen the planner surface beyond this capability.** One closed
  role-keyed accessor returning an existing unmintable identity. ⛔ No second
  derivation, ⛔ no plane exposure, ⛔ no name-arena access.
- ⛔ **Do not make `intern` public** and ⛔ do not add a sibling interner.
- ⛔ **Do not change `HostResult`'s representation** (§3a).
- ⛔ **Do not deliver the identity capability without the `DynamicConstructor`
  producer.** ⭐ Ruled explicitly: *"otherwise the new identity answer would exist
  while no emitted runtime-discriminant route consumes it."* ⚠ **A representation
  node must name who eliminates it** — an identity nobody consumes is the inert
  half-node this chain already rejected twice.
- ⛔ **Do not add a source-text oracle.** Tests assert **behaviour**
  (operator standing policy). ⚠ See `§6`'s tripwire note for the existing one.
- ⛔ **No `--workspace` run.** Targeted only (`COORDINATION §12`).

---

## 5. Deliverables

- **`D1`** — the closed role inventory, built during **plan construction**,
  covering every fixed role **and every dynamic alternative** effect lowering may
  synthesize.
- **`D2`** — the **role-keyed accessor**: lowering asks by an unforgeable closed
  role and receives the existing `ConstructorIdentity` / `tag_abi_word`.
  ⛔ Unmintable outside the plane, as today.
- **`D3`** — **rejection before unit emission** for any synthesized role absent
  from `D1`'s inventory.
- **`D4`** — the **`DynamicConstructor` producer**, consuming `D2`, replacing the
  `mod.rs:1076` refusal.
- **`D5`** — the **`HostResult` producer** on the §3a helper route, replacing the
  `mod.rs:1071` refusal, with each payload independently following its own
  `boundary_disposition`.
- **`D6`** — a written statement of **which populations `D1` covers and which it
  does not**, as a partition with its discriminator. ⛔ An example list is not a
  partition.

---

## 6. Acceptance criteria

- **`AC-1`** — **the identity is unmintable and the plane is unexposed.**
  **Control:** `ConstructorIdentity`'s field stays `pub(super)` and `intern`
  stays private; show that lowering's only route is `D2`'s role-keyed accessor.
  ⛔ A test asserting the *visibility keyword* discharges nothing — ⚠ a census at
  one layer is not a finding about the exported surface. Demonstrate it by the
  **absence of any other reachable route**, and say how you established that.

- **`AC-2`** ⭐ **(load-bearing)** — **distinct roles cannot alias, and duplicate
  spellings converge.** **Control:** two roles with distinct spellings yield
  distinct identities; two roles spelled identically yield **the same** identity.
  ⛔ **Both halves, or the AC is satisfied by a function that returns a constant.**

- **`AC-3`** — **an unlisted synthesized role rejects BEFORE unit emission.**
  **Control:** a refusal observed at the pre-emission seam, ⛔ **not** a later
  failure. ⚠ Use the three-valued epoch discipline `B2F` `AC-11` already
  established: `None` = never reached the seam is **not** a zero.

- **`AC-4`** — **`D4` and `D5` emit, and the identity they emit is the one a
  consumer reads.** **Control, and it is behavioural:** one compiled effect body
  driven to **both success and error at runtime**; a **separately generated**
  consumer selects the exact case through `host_success`, receives the exact
  payload through `host_payload`, then observes **nested constructor
  identity/fields** through ordinary carrier helpers.

- **`AC-5`** ⭐ **(the mutation set is the deliverable here)** — each of these
  reds **at its own production/consumption site**, ⛔ not merely somewhere:
  using the **parent origin** · swapping **Ok/Err** roles or fields · swapping
  the **success-to-case** association · **aliasing two synthesized roles** ·
  **omitting one dynamic alternative** · reading a **compile-time template**.
  ⚠ **A mutation that reddens does not confirm which detector caught it** — name
  the failing assertion per row.

- **`AC-6`** — **the payload population is not the one reported fixture.**
  Effect lowering synthesizes nested `Constructor`, `DynamicConstructor`,
  `ResourceToken` and `ResponseBytes` graphs across the `HostResult` arms.
  **Control:** `D6`'s partition, with at least one fixture per covered class.
  ⛔ **A one-fixture shortcut leaves the admitted variant partial** — ruled.

- **`AC-7`** — targeted green. **Control:** name the exact
  `scripts/ken-cargo test -p <crate>` invocations and their pass counts.

> ### ⚠ THE PIN YOU WILL BREAK FIRST — re-baseline it deliberately
>
> `lowering/core/tests/control.rs` asserts the **source text** of
> `constructor_symbol_identity` (`:3507`), `ConstructorIdentity` (`:4499`) and
> `tag_abi_word` (`:4501`). ⇒ **Any signature change reddens that census.**
> ⭐ It is a **TRIPWIRE, not a failure** — re-baseline it with the reason
> recorded, exactly as `B2F` does. ⛔ Do not extend it, and ⛔ do not add a new
> source-text oracle: ⚠ it cannot distinguish a `cfg(test)` probe from production
> surface, because it reads text.

---

## 7. Contention

`planning/static_transition/semantic_ir.rs` and `lowering/mod.rs` — ⚠ **`B2F` is
live on `lowering/` right now.** ⛔ **Re-measure at pickup and coordinate the
`lowering/mod.rs` edit with the `B2F` build turn holder**; do not open a
concurrent edit on that file. ⭐ The `planning/` half is contention-free today.

---

## 8. Hard stop

⛔ Route to the Steward if:

- the closed inventory cannot be built at plan construction without reading
  something the plane does not own; **or**
- `D3`'s pre-emission rejection cannot be placed before unit emission without
  widening the surface further; **or**
- the real payload population turns out to include a class `D2`'s role keying
  cannot name — ⭐ **say which class and why, rather than adding a second
  derivation for it.**
