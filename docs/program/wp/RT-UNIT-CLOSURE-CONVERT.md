# RT-UNIT-CLOSURE-CONVERT — activate function-unit closure conversion

**A predeclared unit's lowering environment is built from its declared
`Parameter`/`Capture` slot run alone, and it then lowers a retained nested body
whose free de Bruijn references exceed that run. The body is reachable and the
unit does not carry the lexical values the body names. This node makes the
free variables of a retained body into declared typed capture slots, supplied
by exact caller operands per call edge and reconstructed at unit entry.**

**Owner:** Team Runtime. **Size:** TBD — `D1` is the sizing instrument; see
section 3.
**Node:** `docs/program/issues/RT-UNIT-CLOSURE-CONVERT.md`.
**Risk:** medium — the mechanism is ruled and the contract is already merged;
the open variable is how much of it is inert versus absent.

**Authority:** Architect disposition `evt_56jh63qntwtfe` (2026-08-05),
classifying the moved `D4a` reds as a frame/substrate boundary rather than a
bounded `D3b` repair and leaving the checkpoint-versus-node call to the
Steward. Steward scope recut `evt_7he9qv8wbv1yq`.

---

## 0. The standing perishability clause

**Treat every anchor in this frame as perishable. If a fixed input turns out
false against the landed code, say so and escalate — do not quietly build
around it.**

This frame has an unusually strong reason to say so. Its fixed inputs are
measured at `origin/main`, but the *population* that exercises them does not
exist on `main` at all — it exists only on the unmerged
`RT-CONTSRC-PRODUCER-LOCAL` branch. The two halves are measured at different
commits by construction, and the substrate half will have moved by the time
this node runs. `RT-FNSPLIT-B2R`'s own anchors section already warns that every
anchor in that chain has moved at least once. **Re-derive; do not read the
numbers.**

---

## 1. Base and fixed inputs

### The two bases, and why there are two

| what | base | why |
|---|---|---|
| the merged closure-conversion substrate | `origin/main` `5e36e193` | `RT-FNSPLIT-B2R` (PR #967) and its siblings are merged; the contract is on `main` today |
| the failing population | preserved `bc371f13`, branch `wp/RT-DECL-CLOSURE-PORT-typed-units` | the five reds only reach this boundary once `D3b`'s arms admit producer-local continuation inputs |

**The execution base is neither of these.** This node `depends_on`
`RT-CONTSRC-PRODUCER-LOCAL`, which is unmerged and still has `D3c`, a re-cut
`D3b`, `D4b` and its candidate ahead of it. **Measure the fixed inputs at the
actual base at release time and record what moved.** Naming a base is section
3's first obligation, not this section's claim.

### Substrate blobs at `5e36e193`

| path | blob |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `fbbb575e052a53656d0b29a263fb3d929e5976e6` |
| `.../planning/static_transition/semantic_ir.rs` | `c5e0c9318c93a00c2320ac4dd27ba157f5c1a59a` |
| `.../planning/static_transition/abi.rs` | `23b9f5d778bf98fbb2907cf087bf06da30d82e7d` |

### The gap, at `origin/main` `5e36e193`

`define_unit_body` (`units.rs:1512`) builds the body's lowering environment
from exactly the `Parameter | Capture` slot run and nothing else
(`units.rs:1666` onward):

```rust
let mut env = Vec::new();
for (slot, offset) in unit.slots.iter().zip(&unit.offsets) {
    if matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture) {
```

Measured symptom on the population base:

```
Var: no runtime binding for index 2   env_len=2  defining=Predeclared(PredeclaredFunctionId(3))
```

`D4a`'s own shifted fixture reaches the identical boundary at `index=3`. **One
uniform gap, not five.**

### What `B2R` left, as measured — the sizing-critical half

`RT-FNSPLIT-B2R` is `merged` and its scope section *"Inert only — the
already-ruled scaffold escape"* landed the closure-conversion **contract** as
production code while banning any executable edge: declarative ABI, layout and
ownership types, descriptor construction and validators are production; zero
new callable target unit, call edge, dispatch edge, callback, flag or alternate
entry; and no second live body-emission authority.

Three measurements at `5e36e193` that a reader must not take on trust:

1. **Counts and a cross-path reconcile exist.** `declared_arity`
   (`abi.rs:1038`) derives a unit's parameter and capture counts from the
   defining occurrence's own declaration and rejects when
   `layout.slots.len != seed.capture_slots` — written by different code paths,
   so it is a real detector rather than a restatement.
2. **A dense-ordinal validator exists on the continuation-unit path**
   (`units.rs:747` onward): the projected continuation inputs must cover the
   `Capture` slot run, and must be dense in ordinal order. This is the
   reject-on-missing/extra shape, already production.
3. **`CaptureSlot` is ordinal-only.** At `semantic_ir.rs:736` it is
   `struct CaptureSlot { ordinal: u32 }`. **The merged substrate carries
   counts, ordinals and layouts — not free-variable identities.** Identities do
   appear elsewhere in the planner's own vocabulary; `abi.rs:1903` records that
   captures are *"capture child origins for a lexical closure, `CaptureSymbol`
   atoms for a seed closure."*

Point 3 is the frame's single most load-bearing measurement and it is exactly
what `D1` must confirm or refute at the real base. If identities are reachable
from the descriptor, this node is small. If the ordinal is genuinely the only
thing a slot can say, then giving the obligation *"this slot holds free
variable X of the retained body"* a checked home is itself part of the work.

---

## 2. The expressibility audit, run and recorded

Per `steward/frame-authoring.md` §(b-triple-prime): for each obligation the
mechanism must carry, name the in-shape checkable home where it is written.

| obligation | candidate home at `5e36e193` | status |
|---|---|---|
| this unit declares N captures | `seed.capture_slots`, `layout.slots.len` | has a home, and a cross-path reconcile |
| the supplied run is dense and complete | `units.rs:747` density/coverage checks | has a home on the continuation path; **unverified for predeclared function units** |
| slot *i* holds free variable *X* | `CaptureSlot { ordinal }` | **no home measured** — see `D1` |
| this caller passes the operand the callee's slot *i* names | not measured | **open** |
| the retained body's `Var(k)` resolves to slot *i* | not measured | **open** |

The three open rows are why `D1` is an inventory rather than a formality. **An
obligation with no in-shape home is an expressibility gap, and the honest
outcomes are extend the shape or descope — never a comment, never a
caller-fabricated value.** That last is not hypothetical here: it is precisely
what the four forbidden repairs below do.

---

## 3. Deliverables

### `D1` — the inventory, and it is the sizing instrument. HARD STOP after it.

**Do not size this node, and do not start `D2`, before `D1` is handed back.**

1. **Name the base.** Record the exact SHA this node executes from, the branch,
   and the blob of each of the three substrate files. State which of this
   frame's section-1 measurements survived and which moved.
2. **Inventory the five mechanism elements** the Architect named. For each,
   classify as **production**, **test-only**, or **absent**, with `file:line`:
   - planner-issued exact free-variable identities for a retained body;
   - declared typed capture slots in the unit descriptor;
   - the exact caller operands at each call edge;
   - reconstruction from those slots at unit entry;
   - caller/callee equality checked before emission.
3. **For each element classified production, state what makes it inert** —
   which executable edge is missing, not merely that no caller exists today.
   *"Nothing calls it"* is not the same finding as *"nothing emits it."*
4. **Answer the `CaptureSlot` identity question** against the real base: can a
   declared slot name the free variable it holds, in the descriptor's own
   checked vocabulary? If not, name where that obligation would have to live.
5. **Check the predeclared path against the continuation path.** The density
   and coverage validators at `units.rs:747` are on the continuation-unit path.
   State whether the predeclared function-unit path has an equivalent, or
   whether admitting captures there needs one built.

**Deliverable form:** a table plus the base record, posted to the ring and
committed to the node. **Steward re-sizes and re-cuts `D2` onward on this
result** — the deliverables below are provisional and are explicitly subject to
that re-cut.

### `D2` — the population, characterized at the base (provisional)

Enumerate which programs reach `define_unit_body` with a retained body whose
free de Bruijn depth exceeds the declared run. **Enumerate against the base; do
not inherit "five" from this frame.** The count is a measurement, the
*uniformity* is the claim to test: one gap or several.

Report the full vector per site, not the first cause — a probe that stops at
the first declining member reports a property of the population that the
population does not have.

### `D3`–`D5` (provisional, re-cut on `D1`)

- **`D3`** — planner: exact free-variable identities per retained body, issued
  into the unit descriptor's declared capture slots.
- **`D4`** — caller side: the exact operands supplied per call edge.
- **`D5`** — callee side: reconstruction at unit entry, replacing the
  `Parameter | Capture`-only environment walk, with caller/callee equality
  checked **before** emission.

---

## 4. Acceptance criteria

**AC-1 (`D1`).** The inventory exists, classifies all five elements with
`file:line` against a named base, and answers the `CaptureSlot` identity
question. *Control:* a reviewer can open each cited line and see the claimed
classification. An element recorded as production with no cited line fails.

**AC-2.** The five (or however many `D2` measures) failing sites resolve their
`Var` references through declared capture slots. *Control:* each site's
`Var(k)` resolves to a slot the unit descriptor declares, and the descriptor's
declared count equals the caller's supplied count at every call edge into it.

**AC-3 — the equality is checked, not assumed.** Caller/callee agreement is
verified before emission, and the check fires. *Control:* per call edge, a
mutation that supplies one operand too few, one too many, and one of the right
arity but the wrong identity. **All three, enumerated per edge — not "each
edge" as a quantifier the reader resolves.** Record the result per edge. The
wrong-identity mutation is the one that matters: arity mutations may be caught
by a bounds check that proves nothing about identity.

**AC-4 — the mutation must not be applied downstream of its own check.** Each
mutation is injected at or before the point the check reads, never on an
already-verified value. A mutation applied after the check slips past the check
it was meant to exercise and proves only that the value changed.

**AC-5 — no fabricated binding.** For each of the four forbidden repairs in
section 5, a control that would fail if it had been used. *Control:* the
wrong-identity mutation of AC-3 is the positive control for padding and for the
ambient-tail copy; name the control for the `Var` shift and for the
continuation-input reuse separately, or state plainly that they are guarded by
review rather than by CI. **Do not leave a residual arm unrecorded.**

**AC-6 — no regression.** The base's red rows are unchanged except for the
sites `D2` enumerated, which move to passing. Workspace-green, `--locked` and
conformance are **CI's**, never a local `--workspace` run
(`agent/COORDINATION.md §12`).

**AC-7 — `B2R`'s ban survives or is explicitly retired.** This node adds an
executable edge, which is what `B2R` banned. State which clause of that ban is
being lifted, by whose authority, and confirm that no *second* live
body-emission authority is created. *Control:* production still retains exactly
one root `FunctionBuilder` and one `define_function`.

---

## 5. Banned scope

**The four forbidden repairs** (Architect, `evt_56jh63qntwtfe`). Each
fabricates a binding and violates the existing no-implicit-tail ABI law:

| forbidden | what it fakes |
|---|---|
| pad the environment vector | a value that was never passed |
| shift the `Var` | a different variable than the source names |
| copy an ambient caller tail | an implicit suffix the ABI law forbids |
| reuse continuation-call inputs as implicit unit captures | conflates the continuation contract with the unit's |

**There is no relax option.** That is why this is a node and not a waiver.

Also banned: numeric equality, constant offset, reverse search or fallback
bridging the entry-ABI and lexical-position domains — the same law
`RT-CONTSRC-PRODUCER-LOCAL` `D3c` is measuring. No alternate lowerer, no
permanent side map, no direct construction of a descriptor a planner did not
issue.

**Do not touch `RT-CONTSRC-PRODUCER-LOCAL`'s in-flight work.** `bc371f13` is
preserved exactly and no unit-frame edit is authorized until that node's
`D3c` result lands and its `D3b` is re-cut.

---

## 6. Sequencing and contention

**Depends on:** `RT-CONTSRC-PRODUCER-LOCAL` — this node builds on the branch
that admits the population.

**Gates, recorded as prose and deliberately not as an edge:**
`RT-CONTSRC-PRODUCER-LOCAL`'s candidate cannot close until this node lands,
because its remaining red rows fail here. **An edge both ways is a cycle
`gen-progress.sh` cannot resolve.** Same pattern as
`RT-CONTSRC-PRODUCER-LOCAL` against `RT-DECL-CLOSURE-PORT`: both active, one
branch, sequenced by the frame.

**Contention:** one branch, `wp/RT-DECL-CLOSURE-PORT-typed-units`, one team.
No other ring touches `crates/ken-runtime/src/cranelift_backend/`. The doc
track runs concurrently and is contention-free against it.

**Critical path cost, stated plainly:** this adds a node ahead of every one of
the seven `RecursiveDescent` retirement nodes, all of which funnel through
`RT-DECL-CLOSURE-PORT`. It was cut out of `RT-CONTSRC-PRODUCER-LOCAL` rather
than deepening a node already four checkpoints past its own recut. The operator
holds the campaign sizing question separately; this frame does not pre-empt it.

---

## 7. Hard stop

Stop and hand back, without repairing, if:

- `D1` finds any of the five elements **absent** rather than inert — that
  changes the node's size and the cut is the Steward's;
- the base's substrate has moved such that a section-1 measurement is false;
- resolving a `Var` requires any of the four forbidden repairs;
- the entry-ABI versus lexical-position question resurfaces here — it belongs
  to `RT-CONTSRC-PRODUCER-LOCAL` `D3c` and must not be answered twice;
- lifting `B2R`'s inert-only ban would create a second live body-emission
  authority.
