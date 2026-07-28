# `RT-FNSPLIT-C3-ACTIVATION` — the opaque activation owner

**Owner: Runtime. Size: L. Blocks: `RT-FNSPLIT-B2F`.**
**Origin: Architect corrected ruling `evt_2yjg12pyqqjdv`, bound to exact
`7ce4198f580829e166f751f62e5d206831143e9f`.**

---

## §0 — why this is a node, and ⛔ not `B2F`-local wiring

The arena ruling `evt_1m082dp6xf0mw` closed with *"not another prerequisite
node. Fold it into `S6`/`D6`'s atomic reland."* ⭐ **That sentence rested on two
premises, and measurement falsified both:**

1. *"the runtime/store activation owner"* — ⛔ **there is no such thing in
   production.** Every `reserve` / `reserve_persistent` **call** site is under
   `#[cfg(test)]` or in a rig.
2. *"the existing store/plan capacity authority grants storage"* — ⛔ **that
   phrase has no referent.** The four numbers are caller-supplied parameters.

⇒ Both are **retracted by the corrected ruling**, and with them the scope
answer. ⭐ **This is the `C2-SYNTH-ID` shape again:** the fix is a capability
`B2F`'s no-widening boundary forbids the implementer from creating — here a
crate-type change, a new C ABI, and a new deployment-policy input. ⛔ Not `S6`
postponed; work `S6` is not permitted to contain.

> ### ⛔⛔ THE WITHDRAWN FAIL-CLOSED CLAUSE — do not reintroduce it
> The prior ruling said *"a launcher missing a non-null, fully published
> boundary arena does not call generated code."* ⚠ Combined with the rlib-only
> measurement, ⇒ **every linked process executable stops running.** ⭐ **That is
> a working→broken regression wearing a guard's clothes.**
> ⛔ **It must not land in any form.** The replacement is a **configuration
> refusal before packaging or activation** — ⛔ never a linked executable that
> starts and then silently omits generated execution.

---

## §1 — fixed inputs, measured at `origin/main = 7faa91cd`

⛔ **Re-measure before you start.** These are the objects the ruling reasons
about; if a blob moved, the reasoning is re-checked, not assumed.

| path | blob |
|---|---|
| `crates/ken-runtime/Cargo.toml` | `90e56adaf55ff37ade68ce45965b785f66430801` |
| `crates/ken-host/Cargo.toml` | `e05a78e8afa3b329c83b0ea26e319b9e8afd083d` |
| `crates/ken-runtime/src/object_linker_packaging.rs` | `9e285cfb4db5aa74f0ad17e3ba9d11f86bf374c5` |
| `crates/ken-runtime/src/boundary_value.rs` | `49242144a1c5886e5b6638ae7308439b92d80108` |
| `crates/ken-runtime/src/boundary_value_clif.rs` | `47610c6de93ea7296eacb72abb761fd726deb630` |
| `crates/ken-runtime/src/native_process_entrypoint.rs` | `8891d2c35aa7219f35cab5271cb305de018d1848` |

⚠ **`GeneratedActivationServicesV1` is NOT on `main`.** It is landed on
`wp/RT-FNSPLIT-B2F-functionization-live` at **`7ce4198f`** — `#[repr(C)]`, two
distinct typed fields, offsets derived from a closed field inventory, three
behavioural controls. ⭐ **It stays as landed and this node consumes it**; ⛔ do
not re-derive or re-lay-out it here.

---

## §2 — the measurement this node exists to repair

| fact | where |
|---|---|
| `ken-runtime` has **no `[lib]` section at all** ⇒ **rlib only** | `crates/ken-runtime/Cargo.toml` |
| ⭐ `ken-host` **already** carries `crate-type = ["rlib", "staticlib"]` | `crates/ken-host/Cargo.toml:11-12` |
| the starter's link inputs are the emitted object + the generated C stub + `ken_host_staticlib()` | `object_linker_packaging.rs:648`, `:800` |
| that resolver matches `libken_host.a` | `:1103`, `:1120` |
| public entry symbol `ken_nc23_entrypoint` | `:303` |
| ⛔ C **duplicates** `struct KenNativeIntArenaV1` — twice | `:1641`, `:1755` |
| ⛔ C **owns** the invocation struct `KenNativeInvocationV1`, whose fourth field is `struct KenNativeIntArenaV1 *native_int_arena` | `:1770` |
| ⛔⛔ **C CONSTRUCTS the arena itself** — `struct KenNativeIntArenaV1 native_int_arena = {0};` then `.native_int_arena = &native_int_arena` | `:1943`, `:1948` |
| `reserve(nodes, words, data, limbs)` — four **caller-supplied** numbers | `boundary_value.rs:1401` (def), `:1699`, `:1764` |
| `reserve_persistent(...)` — the other four | `:1959` |
| ⛔ the numbers are **parameters** threaded from the rig, not derived | `boundary_value_clif.rs:2859`, `:2870`, `:2871`, consumed `:2881` |
| the store lifecycle verbs already exist | `boundary_value.rs:1642`/`:1704`/`:1779` `publish`, `:1933` `seal_persistent`, `:1968` `publish_persistent`, `:2013` `adopt` |
| the JIT-side process entry | `native_process_entrypoint.rs:250` |

⭐⭐ **The two rows that decide the design:** `ken-host` already being
`rlib + staticlib` means `D1` **copies a landed pattern rather than inventing
one**; and C at `:1943`/`:1948` already **constructing** an arena is the
concrete precedent the ruling says to **subsume, ⛔ not repeat**.

> ### ⭐ THE LIFECYCLE VERBS EXIST — THE OWNER DOES NOT
> ⛔ **Do not read the `publish`/`seal`/`adopt` row as "mostly done".** Those are
> **definitions with no production caller**. ⇒ This node supplies the **caller
> and its lifetime**, and that is the whole difficulty: ⚠ ordering, ownership
> and teardown are the parts a definition cannot express.

---

## §3 — the ruling, transcribed (⛔ not paraphrased)

### 3a — one Rust authority, exported as a static library

- `ken-runtime` becomes **`rlib + staticlib`**.
- The linked starter uses **`libken_runtime.a`** as its runtime-support archive
  and ⛔ **does not also link `libken_host.a` as a second copy**. The runtime
  staticlib already owns the direction `ken-runtime → ken-host`.
- `ken-runtime` exposes a **small C ABI over an opaque activation handle**:
  **construct · obtain the generated-entry argument/services view · finish ·
  destroy.** ⛔ C stores **only** an opaque pointer and status values.
- The Rust handle owns the per-invocation `NativeIntArenaV1`, `BoundaryArenaV1`
  and `GeneratedActivationServicesV1`, and owns or borrows the
  `BoundaryValueStore` for at least the lifetime of every adopted result.
  ⭐ **Rust alone performs reserve, bind, publish, seal, adopt and teardown.**
- ⛔ The existing C copies of `KenNativeIntArenaV1` are **subsumed** by this
  owner — **precedent to remove, ⛔ not precedent to repeat.**

⛔ **A new lower crate is NOT required** — that would split the existing runtime
value authority merely to make it linkable. ⛔ **Construction does not move into
`ken-host`** — that would make the OS-policy layer own Ken runtime value/store
semantics and invert the present layering.

### 3b — the arena is invocation-owned, ⛔ never artifact-static

The artifact may carry **immutable profile and identity metadata**. ⛔ It may
**not** carry the mutable arena itself.

⚠ **Why the attractive option is wrong:** an artifact-static arena is shared by
repeated, concurrent and re-entrant activations, and its published table
pointers and counts are **mutable and invocation-specific**. ⇒ Two executions
**alias storage**. ⛔ **`D3` seed material is therefore not an arena allocation
mechanism** — ⭐ this explicitly closes the fork the implementer rated strongest.

### 3c — capacity authority

C3 introduces a versioned **`BoundaryResourceProfileV1`**, supplied by the
**deployment/package caller**, with separate named limits for **invocation** and
**persistent** boundary storage:

- **nodes** · **words** · **data bytes** · **native-`Int` limbs**

⇒ **Eight quantities — exactly what the two existing reserve operations
consume.** ⭐ They are **deployment resource policy**, ⛔ not compiler semantics
and ⛔ not an emitter-derived formula. The emitter **may validate and carry** the
selected profile; ⛔ it **may not invent, widen, or silently default** it.

- For an object-linked artifact the selected profile is recorded in **package
  metadata/provenance** and **included in the package identity**. ⭐ The
  generated stub **may embed those already-authorized numbers; ⛔ it is not
  their authority.**
- JIT execution receives **the same typed profile** from its caller.

⚠ **Why a profile is required rather than optional:** the current fixed
published regions ⛔ **cannot honestly represent an unspecified unbounded
profile**, because post-publication growth can **move their tables**. ⇒ Absence
is a **configuration refusal before packaging or activation**. Exhaustion
**names the exact resource and scope** and returns the existing loud
`CapacityExhausted`-class outcome ⛔ **before corruption or substitution**.

### 3d — the root/adapter seam

Keep **one** public object entry argument, and split public adaptation from the
generated internal ABI:

```text
public starter entry: (opaque_activation_ptr) -> i64
internal root/unit:   (frame_ptr, services_ptr) -> i64
```

C3 establishes the opaque activation owner and the **one-argument public adapter
seam on both** the JIT and object-linked launch paths. The adapter obtains the
root frame/services view **from the Rust owner**. ⇒ ⭐ **`B2F` may then change
the internal root and units atomically to the two-parameter convention without
changing or duplicating the public C representation.**

⛔ **A JIT-only signature half-step remains forbidden** — `STARTER_ENTRY_SYMBOL`
(`:303`) compiles through the same `sig`, so the two paths move together or not
at all.

---

## §4 — banned shapes

⛔ Reintroducing the withdrawn fail-closed clause in any wording that lets a
**linked executable start and then not call generated code**.
⛔ An **artifact-static** or otherwise process-lifetime arena.
⛔ A **second** copy of the arena, services, native-`Int` or activation layout in
generated C — including keeping the two that exist at `:1641` and `:1755`.
⛔ Moving boundary-value construction into **`ken-host`**.
⛔ A **new lower crate** carved out to make the authority linkable.
⛔ Linking **both** `libken_runtime.a` and `libken_host.a` into one starter.
⛔ An **emitter-derived**, defaulted, or widened capacity.
⛔ Changing the **internal** root/unit signature here — ⭐ that is `B2F`'s `S6`,
and this node exists precisely so it can be done atomically later.
⛔ A **polymorphic `arena`** field, two positional raw parameters, or an
emitter-selected pointer — the two typed fields stand.

---

## §5 — deliverables

**`D1` — `ken-runtime` becomes `rlib + staticlib`.** ⭐ Copy the landed
`ken-host/Cargo.toml:11-12` pattern. ⚠ Verify the archive actually exports the
C ABI symbols; a `crate-type` line alone is a build-system claim, not a link
one.

**`D2` — the opaque activation handle and its C ABI.** Construct · view ·
finish · destroy. ⛔ Opaque pointer and status values on the C side only.

**`D3` — the handle owns the per-invocation objects and the lifecycle.**
`NativeIntArenaV1`, `BoundaryArenaV1`, `GeneratedActivationServicesV1`, and
ownership-or-borrow of the `BoundaryValueStore` across every adopted result.
Reserve · bind (`ARENA_PERSISTENT`, `ARENA_NATIVE_INT`) · publish · seal ·
adopt · tear down, ⭐ in the landed order, ⛔ with no post-publication
reservation or materialization.

**`D4` — `BoundaryResourceProfileV1`.** Versioned, eight named limits, supplied
by the deployment/package caller. ⛔ No default, no widening, no emitter
derivation.

**`D5` — profile provenance for object-linked artifacts.** Recorded in package
metadata and **included in the package identity**; the stub may embed the
authorized numbers. ⭐ JIT takes the same typed profile from its caller.

**`D6` — the one-argument public adapter seam on BOTH launch paths.**
`(opaque_activation_ptr) -> i64` publicly; the adapter obtains the root
frame/services view from the owner. ⛔ The internal convention does not change
in this node.

**`D7` — remove the subsumed C duplicates** at `:1641` and `:1755`, and the
C-side construction at `:1943`/`:1948`. ⭐ `KenNativeInvocationV1` (`:1770`) is
the seam that has to stop carrying an arena pointer C owns.

---

## §6 — acceptance criteria

⭐ **The ruling states its own floor: *"not discharged by layout tests alone."*
Every AC below is behavioural or a link-level fact.**

**`AC-1` — a real linked starter reaches the Rust constructor and calls
generated code with a **published** boundary arena.** ⛔ Not a unit test of the
handle: an actual packaged executable, run.

**`AC-2` — two activations get distinct mutable arena state.** Simultaneous or
sequential; they share **only** explicitly store-owned persistent state.
⚠ **This is the control that would have caught the artifact-static fork**, so it
must fail if storage is aliased — ⛔ a fixture that runs one activation twice
and checks it did not crash is not this.

**`AC-3` — three substitutions RED.** (a) the old host-only link in place of the
runtime staticlib · (b) bypassing publish · (c) passing the native arena as the
boundary arena. ⭐ **(c) is Finding 8's exact defect** and must red here, ⛔ not
merely be absent.

**`AC-4` — each of the eight profile limits governs its named region and
resource, and at-limit-plus-one fails loudly naming that exact scope.** ⛔ Eight
separate cases. ⚠ A single "capacity exhausted" assertion shared across limits
cannot tell which limit fired, ⇒ it is one control claiming to be eight.

**`AC-5` — C contains no private copy of the activation, services or arena
layouts.** ⭐ The oracle is a **build/link fact plus the removal of `:1641` and
`:1755`**, ⛔ not a grep asserted in prose — and per operator test policy, ⛔ not
a source-line census test either.

**`AC-6` — the existing native-only linked and JIT positives still pass after
the adapter migration.** ⭐ **This is the anti-regression AC and the reason the
withdrawn clause is banned.** ⛔ A green run of *new* tests does not discharge
it; the *pre-existing* positives are the population.

**`AC-7` — absence of a profile is a refusal BEFORE packaging or activation.**
⛔ The failure must be reachable only at configuration time. ⚠ A starter that
links, runs, and then declines to execute generated code is the banned shape,
⭐ **so this AC's control must distinguish refusal-to-package from
refusal-at-run** — those are different observations and only one is permitted.

> ### ⚠ On `AC-3` and `AC-5` — a mutation you DESIGNED is not an evasion you
> ATTEMPTED
> Each substitution above is written as a **production** mutation. ⛔ A control
> that reds only against a rig-side edit has measured the rig. ⭐ And for
> `AC-5`, note the honest residual shape: removing two known copies does not
> prove a third was never added — ⇒ **state that as a partition with its
> discriminator**, ⛔ never as *"C has no copies"*.

---

## §7 — contention

⚠ **`object_linker_packaging.rs` is the hot file** and is also where the C stub
text lives. `boundary_value.rs` and `boundary_value_clif.rs` are touched by
`B2F`'s producer work — ⭐ **but `B2F` is held at `7ce4198f`**, so the contention
is with **landed** work, not a live turn.

⛔ `RT-FNSPLIT-C2-SYNTH-ID` is independent of this node — different files, no
shared deliverable. ⭐ **Runtime holds both; the ordering between them is the
leader's call.**

---

## §8 — the hard stop

⛔ **`B2F` does not resume `S6`/`D6` until this node is durable.** The landed
`GeneratedActivationServicesV1` record may stay; ⛔ **no per-function binder, no
shared-root signature change, no reland** before then.

⭐ The implementer may continue the already-independent `AC-2`/`AC-3`/`AC-8`
work on `B2F` in the meantime — ⚠ **that is not a licence to start `S6`
groundwork under another name.**
