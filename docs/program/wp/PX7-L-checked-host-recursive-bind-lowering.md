# PX7-L — checked-host lowering for computational recursive `ITree.bind` (compiler prerequisite)

- **ID:** PX7-L · **Owner:** Team Runtime · **Size:** L · **Risk:** High
- **Objective:** Add principled checked-host native lowering for the
  already-kernel-checked generic `ITree.bind` when its `Vis` operation is a
  **closed, runtime-selected** host operation and its recursive induction
  hypothesis is **computational** (the recursively bound continuation), so that an
  honest bracket over an arbitrary `HostIO` body — a delayed body whose operations
  are **not statically visible at the definition site** — compiles to a sound
  linked-native artifact. This is a **general** compiler capability
  (resource-independent); PX7-F is its first consumer.
- **Depends on:** the landed kernel-checked `ITree.bind` semantics and the current
  checked-host lowering / erasure / native-IR pipeline (`ken-elaborator`
  erasure + `checked_core`, `ken-runtime` `RuntimeExpr`/cranelift/native
  differential). Branches off `origin/main @ 30bc5dfd`. **Does NOT depend on
  PX7-F** — it is PX7-F's prerequisite.
- **Feeds:** unblocks **PX7-F's deferred public linked-native AC2/AC3 evidence**
  (Architect-ruled `evt_7aynkstwtkg3n`); and any future higher-order host
  sequencing (brackets, pipelines, combinators over an abstract `HostIO` body).
- **Gate:** no new gate. **Route = Architect §14 only** (compiler-lowering
  soundness / native-differential correctness). **No CV** — no `spec/`,
  `conformance/`, Ward schema, kernel, ABI-wire, or PX7 bracket-semantic change.

## The defect (Architect-grounded, `evt_7aynkstwtkg3n`)

The two observed reds are **one general compiler defect**, not two PX7-F defects:

1. **No sound representation for a closed, runtime-selected host op.**
   `lower_checked_host_computation` recognizes `Vis` **only** when
   `decode_checked_host_operation` can see a checked constructor spine and convert
   it immediately to a **static `HostOpV1`**; `RuntimeExpr::Effect` likewise stores
   a static `HostOpV1`. But in the `Vis` branch of generic `ITree.bind` the
   operation is presented as the **method variable at de Bruijn index 2**
   (`host_coproduct_shape` fails: op is `Variable { de_bruijn_index: 2 }`), so the
   deforester has no lowering for it.
2. **The blanket "recursive IH = erased" rule is false for a computational
   eliminator.** `BranchBinderRemap::enter_match` erases every recursive
   induction-hypothesis binder, but `bind`'s `ih` is **computational** — it is the
   recursively bound continuation, and the `RightNotHeld` control really reaches it
   (`erased_induction_hypothesis_reached_runtime`).

## Fixed inputs — DO NOT REOPEN (Architect-ruled `evt_7aynkstwtkg3n`;
## `ARCHITECT-STATE.md @ 7db63fea`)

- **This is option (a): add principled checked-host lowering.** Do **NOT** add a
  `withResource` intrinsic; do **NOT** change the public bracket shape; do **NOT**
  rewrite the surface to avoid delayed-body sequencing. Any honest bracket over an
  arbitrary `HostIO` body must sequence a computation whose operations are not
  statically visible at the bracket definition site — a new spelling would only
  hide the same missing capability or weaken the settled contract.
- **Do NOT recognize `withResource` by symbol** and reimplement acquire/body/settle
  in Rust — that duplicates the checked Ken definition's early-release, body-error,
  controlled-trap, and cleanup-failure semantics at a **second trust boundary**.
- **Keep `RuntimeExpr::Effect` statically tagged if possible.** The preferred
  lowering is an **exhaustive, identity-checked dispatch over the closed admitted
  operation coproduct**, producing the existing static `HostOpV1` effect in each
  branch. An **untyped/raw dynamic operation lane is FORBIDDEN.** Unknown or
  wrong-family identities remain **fail-closed**.
- **Support the already-kernel-checked generic `ITree.bind` semantics**, including
  **computational recursive hypotheses** and **delayed bodies with ordinary
  lexical captures.** The recursive IH is the continuation and may legitimately be
  reached at runtime — the lowering must represent it, not erase it.
- **Preserve the exact PX7-F red state and its independently green surface,
  interpreter, ABI, and T-emitter work.** No part of the experimental
  lexical-capture change (which merely *exposed* the next boundary) is approved by
  this WP; it is **evidence, not an accepted implementation.**
- **Static direct-`Vis` programs stay byte/observation compatible** — the existing
  static-constructor lowering path is unchanged for programs that already have a
  visible constructor spine. This WP **adds** a lowering for the runtime-selected
  case; it does not replace the static one.

## Mandated deliverable outline — each section ends in a concrete choice

1. **Sound lowering for a runtime-selected closed host op.** In
   `lower_checked_host_computation` (and its `decode_checked_host_operation`
   helper), when the `Vis` operation is the bound method variable rather than a
   static constructor spine, lower it as an **exhaustive identity-checked dispatch
   over the closed admitted-operation coproduct**: one branch per admitted
   operation identity, each producing the existing **static `HostOpV1`**
   `RuntimeExpr::Effect`. No raw/dynamic op value ever flows to codegen. Unknown
   identity / wrong family / malformed coproduct → **fail closed** (the existing
   error path), never a silent or untyped fallthrough.
2. **Computational recursive IH representation.** Change
   `BranchBinderRemap::enter_match` (or the site enforcing "recursive IH = erased")
   so that for a **computational eliminator** (`bind`'s `Vis(op, cont, ih)`) the
   recursive induction hypothesis — the recursively bound continuation — is
   **preserved and lowered**, not erased. Purely-logical/erased hypotheses keep
   their existing erasure. The discriminator between the two is
   structural/kind-based, not a name check.
3. **De Bruijn / capture correctness.** The delayed body's ordinary lexical
   captures and the de Bruijn remapping across the new dispatch branches must be
   correct end-to-end (the earlier `implicit_closure_capture` lane is superseded by
   a principled representation, not a patch). Closure-capture order and de Bruijn
   remap get their **own opposite controls** (§AC5).
4. **Interpreter↔native agreement.** The generic-`bind`-over-`HostIO`-body program
   observed through the interpreter and through the linked-native artifact must
   produce **identical observations** (the `native_execution_differential` harness).
5. **Fail-closed + static-compat preservation.** Unknown operation identity, wrong
   family, capability denial, and malformed coproduct controls remain fail-closed;
   static direct-`Vis` programs remain byte/observation compatible.

## Acceptance criteria (testable) — the Architect's required precursor discriminators

- **AC1 — resource-independent generic-`bind` fixture, interp↔native agree.** A
  checked Ken fixture with **no resource content** uses generic `ITree.bind` over a
  **delayed, lexically-capturing `HostIO` body** with **at least two different
  admitted operation constructors** and **more than one `Vis` step**. Interpreter
  and linked-native observations **agree** exactly. (This is the positive proof the
  capability exists, independent of PX7-F.)
- **AC2 — static-only-decode mutation fails at `host_coproduct_shape`.** A mutation
  that restores **static-constructor-only** operation decoding **must fail** the
  AC1 fixture at `host_coproduct_shape` (proves the runtime-selected-op lowering is
  the load-bearing net, not incidental).
- **AC3 — erased-computational-IH mutation fails.** A mutation that **erases the
  computational recursive hypothesis** **must fail** a **multi-step** fixture at
  `erased_induction_hypothesis_reached_runtime` **or** produce a discriminating
  wrong observation (proves the IH-preservation is load-bearing).
- **AC4 — static direct-`Vis` compatibility + fail-closed controls.** Static
  direct-`Vis` programs remain **byte/observation compatible** (no regression on
  the existing static path); **unknown operation identity, wrong family, capability
  denial, and malformed coproduct** controls remain **fail-closed** (assert the
  specific rejection, not merely `is_err`).
- **AC5 — closure-capture / de Bruijn opposite controls.** Closure-capture order
  and de Bruijn remapping have their **own opposite controls** (a mutation that
  mis-orders captures or mis-remaps indices is caught). The parked PX7-F
  lexical-capture WIP is **evidence, not an accepted implementation** — it is not
  imported wholesale.
- **AC6 — scope boundary held; no-regression = CI-green.** **No** resource-table,
  ABI, Ward schema, kernel, or PX7 bracket-semantic change is in this WP's diff
  (`git grep` / diff review). The full-workspace + `--locked` + conformance gates
  are **green in CI** (never a local `--workspace` run; targeted `-p ken-runtime`
  / `-p ken-elaborator` / `-p ken-cli --test <name>` locally only).

## Do-not-reopen guards

- Do NOT add a `withResource` (or any bracket) **intrinsic**, and do NOT recognize
  it by symbol — the checked Ken definition is the single trust boundary.
- Do NOT change the public surface / bracket shape / delayed-body sequencing to
  dodge the missing capability.
- Do NOT introduce an **untyped/raw dynamic host-operation lane** — the lowering is
  an exhaustive identity-checked dispatch producing static `HostOpV1`; unknown/
  wrong-family identities fail closed.
- Do NOT weaken any fail-closed control, and do NOT make static direct-`Vis`
  programs observably different.
- Do NOT touch `ResourceTableV1` / ABI wire / Ward schema / kernel / PX7 bracket
  semantics — this is a **lowering** precursor only.
- Do NOT import the parked PX7-F lexical-capture experiment as the implementation;
  build the principled representation and treat the WIP as evidence.

## Grounding anchors (re-ground on `origin/main @ 30bc5dfd` before building)

- `crates/ken-elaborator/src/erasure.rs` — `BranchBinderRemap` / `enter_match`
  (the "recursive IH = erased" rule to correct for computational eliminators);
  `implicit_closure_capture`.
- `crates/ken-elaborator/src/checked_core.rs` — checked-core representation of
  `Vis(op, cont, ih)`.
- The checked-host lowering path — `lower_checked_host_computation` /
  `decode_checked_host_operation` (the static-constructor-spine recognizer to
  extend for the runtime-selected op).
- `crates/ken-runtime/src/ir.rs` — `RuntimeExpr` / `RuntimeExpr::Effect`
  (keep statically tagged `HostOpV1`).
- `crates/ken-runtime/src/cranelift_backend.rs` — native codegen consumer.
- `crates/ken-runtime/src/native_execution_differential.rs` — the interp↔native
  differential harness (AC1/AC3 evidence).
- The exact PX7-F reproducer (parked on `wp/px7f-system-resource-bracket`):
  `scripts/ken-cargo test -p ken-cli --test px7f_resource_native \
   linked_public_escape_is_exact_closed -- --exact --nocapture` — do NOT depend on
  PX7-F sources; author a **resource-independent** fixture for AC1.

## Diff scope / review route

- **Touches:** `crates/ken-elaborator/**` (erasure/checked-core lowering),
  `crates/ken-runtime/**` (IR / cranelift / native differential), and the
  new/added lowering-precursor tests. **No** `spec/`, `conformance/`, Ward, kernel,
  ABI-wire, or PX7 substrate change.
- **Route:** **Architect §14 only** (compiler-lowering soundness; TCB-relevant
  because it changes native lowering + erasure). **No CV.** Gates: standard
  no-regression (**CI-green**, per `COORDINATION §12`; never local `--workspace`).
- **One branch, one Decision:** `wp/px7-l-checked-host-recursive-bind-lowering`
  off `origin/main @ 30bc5dfd`.
